// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Immediate framebuffer copy, followed by nonblocking fence polling.
use cambium_rootstock::{
    CaptureFrame, CaptureRefusal, Frame, PendingPresentationCapture, PresentationBackend,
    PresentedFrame,
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{WebGl2RenderingContext as Gl, WebGlBuffer, WebGlFramebuffer, WebGlSync};

pub(crate) struct Buffers {
    gl: Gl,
    pbo: WebGlBuffer,
    fence: WebGlSync,
    bytes: usize,
    usage: Rc<Cell<usize>>,
}
impl Drop for Buffers {
    fn drop(&mut self) {
        self.gl.delete_sync(Some(&self.fence));
        self.gl.delete_buffer(Some(&self.pbo));
        self.usage.set(self.usage.get().saturating_sub(self.bytes));
    }
}
pub(crate) type PendingBuffers = Weak<RefCell<Option<Buffers>>>;
pub(crate) fn cancel(jobs: &mut Vec<PendingBuffers>) {
    for job in jobs.drain(..) {
        if let Some(job) = job.upgrade() {
            job.borrow_mut().take();
        }
    }
}
fn refusal(gl: &Gl, stamp: PresentedFrame, message: impl ToString) -> CaptureRefusal {
    if gl.is_context_lost() {
        CaptureRefusal::ContextLost {
            backend: PresentationBackend::WebGl2,
            generation: stamp.context_generation,
        }
    } else {
        CaptureRefusal::PresentationError {
            backend: PresentationBackend::WebGl2,
            generation: stamp.context_generation,
            source_key: None,
            error: message.to_string(),
        }
    }
}
fn optional<T: JsCast>(value: JsValue) -> Option<T> {
    if value.is_null() {
        None
    } else {
        Some(value.unchecked_into())
    }
}
struct PackState {
    gl: Gl,
    buffer: Option<WebGlBuffer>,
    framebuffer: Option<WebGlFramebuffer>,
    read_buffer: u32,
    values: [i32; 4],
}
impl PackState {
    fn save(gl: &Gl) -> Result<Self, String> {
        let get = |key| {
            gl.get_parameter(key)
                .map_err(|e| format!("GL pack state: {e:?}"))
        };
        let number = |key| {
            get(key)?
                .as_f64()
                .map(|value| value as i32)
                .ok_or_else(|| "GL pack state is not numeric".to_owned())
        };
        Ok(Self {
            gl: gl.clone(),
            buffer: optional(get(Gl::PIXEL_PACK_BUFFER_BINDING)?),
            framebuffer: optional(get(Gl::READ_FRAMEBUFFER_BINDING)?),
            read_buffer: number(Gl::READ_BUFFER)? as u32,
            values: [
                number(Gl::PACK_ALIGNMENT)?,
                number(Gl::PACK_ROW_LENGTH)?,
                number(Gl::PACK_SKIP_ROWS)?,
                number(Gl::PACK_SKIP_PIXELS)?,
            ],
        })
    }
}
impl Drop for PackState {
    fn drop(&mut self) {
        self.gl
            .bind_framebuffer(Gl::READ_FRAMEBUFFER, self.framebuffer.as_ref());
        self.gl.read_buffer(self.read_buffer);
        self.gl
            .bind_buffer(Gl::PIXEL_PACK_BUFFER, self.buffer.as_ref());
        for (key, value) in [
            Gl::PACK_ALIGNMENT,
            Gl::PACK_ROW_LENGTH,
            Gl::PACK_SKIP_ROWS,
            Gl::PACK_SKIP_PIXELS,
        ]
        .into_iter()
        .zip(self.values)
        {
            self.gl.pixel_storei(key, value);
        }
    }
}
pub(crate) fn start(
    gl: &Gl,
    stamp: PresentedFrame,
    usage: Rc<Cell<usize>>,
    limit: usize,
) -> Result<(CaptureFrame, PendingBuffers), CaptureRefusal> {
    if gl.is_context_lost() {
        return Err(refusal(gl, stamp, "context lost"));
    }
    let bytes = (stamp.width as usize)
        .checked_mul(stamp.height as usize)
        .and_then(|n| n.checked_mul(4))
        .filter(|n| {
            *n > 0
                && *n <= i32::MAX as usize
                && usage
                    .get()
                    .checked_add(*n)
                    .is_some_and(|total| total <= limit)
        })
        .ok_or_else(|| refusal(gl, stamp, "capture exceeds configured readback byte budget"))?;
    let error = gl.get_error();
    if error != Gl::NO_ERROR {
        return Err(refusal(
            gl,
            stamp,
            format!("graphics error before capture: {error:#x}"),
        ));
    }
    let _saved = PackState::save(gl).map_err(|e| refusal(gl, stamp, e))?;
    let pbo = gl
        .create_buffer()
        .ok_or_else(|| refusal(gl, stamp, "could not allocate capture buffer"))?;
    gl.bind_buffer(Gl::PIXEL_PACK_BUFFER, Some(&pbo));
    gl.buffer_data_with_i32(Gl::PIXEL_PACK_BUFFER, bytes as i32, Gl::STREAM_READ);
    if gl
        .get_buffer_parameter(Gl::PIXEL_PACK_BUFFER, Gl::BUFFER_SIZE)
        .as_f64()
        != Some(bytes as f64)
    {
        gl.delete_buffer(Some(&pbo));
        return Err(refusal(gl, stamp, "capture buffer allocation refused"));
    }
    gl.bind_framebuffer(Gl::READ_FRAMEBUFFER, None);
    gl.read_buffer(Gl::BACK);
    gl.pixel_storei(Gl::PACK_ALIGNMENT, 1);
    gl.pixel_storei(Gl::PACK_ROW_LENGTH, 0);
    gl.pixel_storei(Gl::PACK_SKIP_ROWS, 0);
    gl.pixel_storei(Gl::PACK_SKIP_PIXELS, 0);
    if let Err(error) = gl.read_pixels_with_i32(
        0,
        0,
        stamp.width as i32,
        stamp.height as i32,
        Gl::RGBA,
        Gl::UNSIGNED_BYTE,
        0,
    ) {
        gl.delete_buffer(Some(&pbo));
        return Err(refusal(gl, stamp, format!("capture copy: {error:?}")));
    }
    let error = gl.get_error();
    if error != Gl::NO_ERROR {
        gl.delete_buffer(Some(&pbo));
        return Err(refusal(
            gl,
            stamp,
            format!("capture copy failed: {error:#x}"),
        ));
    }
    let Some(fence) = gl.fence_sync(Gl::SYNC_GPU_COMMANDS_COMPLETE, 0) else {
        gl.delete_buffer(Some(&pbo));
        return Err(refusal(gl, stamp, "capture fence refused"));
    };
    gl.flush();
    let error = gl.get_error();
    if error != Gl::NO_ERROR {
        gl.delete_sync(Some(&fence));
        gl.delete_buffer(Some(&pbo));
        return Err(refusal(
            gl,
            stamp,
            format!("capture fence submission failed: {error:#x}"),
        ));
    }
    usage.set(usage.get() + bytes);
    let buffers = Rc::new(RefCell::new(Some(Buffers {
        gl: gl.clone(),
        pbo,
        fence,
        bytes,
        usage,
    })));
    let weak = Rc::downgrade(&buffers);
    Ok((
        Box::new(Readback {
            gl: gl.clone(),
            stamp,
            buffers,
            finished: false,
        }),
        weak,
    ))
}
struct Readback {
    gl: Gl,
    stamp: PresentedFrame,
    buffers: Rc<RefCell<Option<Buffers>>>,
    finished: bool,
}
impl PendingPresentationCapture for Readback {
    fn poll(&mut self) -> Option<Result<(PresentedFrame, Frame), CaptureRefusal>> {
        if self.finished {
            return None;
        }
        if self.gl.is_context_lost() || self.buffers.borrow().is_none() {
            self.finished = true;
            self.buffers.borrow_mut().take();
            return Some(Err(CaptureRefusal::ContextLost {
                backend: PresentationBackend::WebGl2,
                generation: self.stamp.context_generation,
            }));
        }
        let status = {
            let buffers = self.buffers.borrow();
            self.gl
                .client_wait_sync_with_u32(&buffers.as_ref().unwrap().fence, 0, 0)
        };
        if status == Gl::TIMEOUT_EXPIRED {
            return None;
        }
        self.finished = true;
        if status != Gl::ALREADY_SIGNALED && status != Gl::CONDITION_SATISFIED {
            self.buffers.borrow_mut().take();
            return Some(Err(refusal(&self.gl, self.stamp, "capture fence failed")));
        }
        let buffers = self.buffers.borrow_mut().take().unwrap();
        let saved = self
            .gl
            .get_parameter(Gl::PIXEL_PACK_BUFFER_BINDING)
            .ok()
            .and_then(optional::<WebGlBuffer>);
        self.gl
            .bind_buffer(Gl::PIXEL_PACK_BUFFER, Some(&buffers.pbo));
        let mut rgba = vec![0; buffers.bytes];
        self.gl
            .get_buffer_sub_data_with_i32_and_u8_array(Gl::PIXEL_PACK_BUFFER, 0, &mut rgba);
        self.gl.bind_buffer(Gl::PIXEL_PACK_BUFFER, saved.as_ref());
        let error = self.gl.get_error();
        if error != Gl::NO_ERROR {
            return Some(Err(refusal(
                &self.gl,
                self.stamp,
                format!("capture buffer read failed: {error:#x}"),
            )));
        }
        if self.gl.is_context_lost() {
            return Some(Err(refusal(
                &self.gl,
                self.stamp,
                "context lost during capture",
            )));
        }
        let stride = self.stamp.width as usize * 4;
        for y in 0..self.stamp.height as usize / 2 {
            let opposite = self.stamp.height as usize - 1 - y;
            let (before, after) = rgba.split_at_mut(opposite * stride);
            before[y * stride..(y + 1) * stride].swap_with_slice(&mut after[..stride]);
        }
        Some(Ok((
            self.stamp,
            Frame {
                width: self.stamp.width,
                height: self.stamp.height,
                rgba,
            },
        )))
    }
}
