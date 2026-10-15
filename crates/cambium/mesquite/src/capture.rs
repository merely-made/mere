// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Platform readback, leaving capture scheduling and completion in the lane.
use cambium_rootstock::{CaptureFn, CaptureFrame, Frame, PresentedFrame, StampedCaptureFn};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

/// Polled once per presented frame until pixels or an error arrive.
pub type Readback = Box<dyn FnMut() -> Option<Result<Frame, String>>>;

pub struct StampedFrame {
    pub identity: crate::CaptureRequest,
    pub presentation: PresentedFrame,
    pub pixels: Frame,
}
pub type StampedReadback = Box<dyn FnMut() -> Option<Result<StampedFrame, String>>>;

/// A host's readback mechanism. Event-loop implementations must not block.
pub trait CaptureBackend {
    fn arm(&mut self, capture: &mut Option<CaptureFn>) -> Readback;
    /// Pairing is opt-in. An older backend must refuse it rather than invent
    /// the identity of pixels obtained through its unstamped path.
    fn arm_stamped(
        &mut self,
        _capture: &mut Option<StampedCaptureFn>,
        _identity: crate::CaptureRequest,
    ) -> Result<StampedReadback, String> {
        Err("capture backend does not support presented-state pairing".into())
    }
    /// Whether named scenario captures receive filesystem paths. Browser
    /// products publish pixels through `Product::inspect` instead.
    fn writes_files(&self) -> bool {
        true
    }
    /// Maximum presented frames to wait for an asynchronous readback.
    fn patience(&self) -> u64 {
        8
    }
}

pub(crate) struct NativeCapture;
impl CaptureBackend for NativeCapture {
    fn arm_stamped(
        &mut self,
        capture: &mut Option<StampedCaptureFn>,
        identity: crate::CaptureRequest,
    ) -> Result<StampedReadback, String> {
        let slot = Rc::new(RefCell::new(None));
        let sink = slot.clone();
        *capture = Some(Box::new(move |result| {
            let frame = match result {
                Ok(frame) => frame,
                Err(refusal) => {
                    *sink.borrow_mut() = Some(Err(refusal.to_string()));
                    return;
                },
            };
            *sink.borrow_mut() = Some(Ok(stamp_readback(frame, identity, Instant::now())));
        }));
        let mut started: Option<StampedReadback> = None;
        Ok(Box::new(move || {
            if started.is_none() {
                match slot.borrow_mut().take()? {
                    Ok(frame) => started = Some(frame),
                    Err(error) => return Some(Err(error)),
                }
            }
            let result = started.as_mut()?();
            if result.is_some() {
                started = None;
            }
            result
        }))
    }
    fn arm(&mut self, capture: &mut Option<CaptureFn>) -> Readback {
        let slot = Rc::new(RefCell::new(None));
        let sink = slot.clone();
        *capture = Some(Box::new(move |result| {
            let frame = match result {
                Ok(frame) => frame,
                Err(refusal) => {
                    *sink.borrow_mut() = Some(Err(refusal.to_string()));
                    return;
                },
            };
            *sink.borrow_mut() = Some(Ok(native_readback(frame, Instant::now())));
        }));
        let mut started: Option<Readback> = None;
        Box::new(move || {
            if started.is_none() {
                match slot.borrow_mut().take()? {
                    Ok(frame) => started = Some(frame),
                    Err(error) => return Some(Err(error)),
                }
            }
            let result = started.as_mut()?();
            if result.is_some() {
                started = None;
            }
            result
        })
    }
}

fn native_readback(mut pending: CaptureFrame, armed: Instant) -> Readback {
    Box::new(move || {
        poll_native(armed, Instant::now(), || {
            pending.poll().map(|result| {
                result
                    .map(|(_, pixels)| pixels)
                    .map_err(|error| error.to_string())
            })
        })
    })
}

// The job returns its original presentation beside the original queued pixels.
fn stamp_readback(
    mut pending: CaptureFrame,
    identity: crate::CaptureRequest,
    armed: Instant,
) -> StampedReadback {
    Box::new(move || {
        poll_native(armed, Instant::now(), || {
            pending.poll().map(|result| {
                result
                    .map(|(presentation, pixels)| StampedFrame {
                        identity: identity.clone(),
                        presentation,
                        pixels,
                    })
                    .map_err(|error| error.to_string())
            })
        })
    })
}

// A deadline applies to GPU completion, independently of presentation counts.
// Check it before servicing the device so a timed-out capture is cancelled and
// can never produce late pixels attributed to a newer presentation.
fn poll_native<T>(
    started: Instant,
    now: Instant,
    poll: impl FnOnce() -> Option<Result<T, String>>,
) -> Option<Result<T, String>> {
    if now.saturating_duration_since(started) >= Duration::from_secs(5) {
        return Some(Err("native frame readback timed out after 5000 ms".into()));
    }
    poll()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_native_readback_yields_until_completion_and_preserves_errors() {
        let start = Instant::now();
        assert!(poll_native::<Frame>(start, start, || None).is_none());
        let later = start + Duration::from_millis(10);
        let pixels = poll_native(start, later, || {
            Some(Ok(Frame {
                width: 1,
                height: 1,
                rgba: vec![18, 52, 86, 255],
            }))
        })
        .unwrap()
        .unwrap();
        assert_eq!(pixels.rgba, [18, 52, 86, 255]);
        assert!(
            matches!(poll_native::<Frame>(start, later, || Some(Err("map failed".into()))),
            Some(Err(error)) if error == "map failed")
        );
    }

    #[test]
    fn delayed_native_pixels_keep_the_original_request_and_presentation() {
        let identity = crate::CaptureRequest {
            run: "native-original-run".into(),
            request: 7,
        };
        let presentation = PresentedFrame {
            context_generation: cambium_rootstock::ContextGeneration(1),
            host: 3,
            sequence: 11,
            width: 1,
            height: 1,
            layout_scale: 2.0,
        };
        struct Delayed {
            calls: u8,
            stamp: PresentedFrame,
        }
        impl cambium_rootstock::PendingPresentationCapture for Delayed {
            fn poll(
                &mut self,
            ) -> Option<Result<(PresentedFrame, Frame), cambium_rootstock::CaptureRefusal>>
            {
                self.calls += 1;
                (self.calls == 2).then(|| {
                    Ok((
                        self.stamp,
                        Frame {
                            width: 1,
                            height: 1,
                            rgba: vec![18, 52, 86, 255],
                        },
                    ))
                })
            }
        }
        let pixels: CaptureFrame = Box::new(Delayed {
            calls: 0,
            stamp: presentation,
        });
        let mut stamped = stamp_readback(pixels, identity.clone(), Instant::now());
        assert!(
            stamped().is_none(),
            "a pending copy must not invent a stamp"
        );
        // A different window/frame can present before the original map lands.
        let later_presentation = PresentedFrame {
            context_generation: cambium_rootstock::ContextGeneration(1),
            host: 9,
            sequence: 22,
            ..presentation
        };
        let captured = stamped().unwrap().unwrap();
        assert_eq!(captured.identity, identity);
        assert_eq!(captured.presentation, presentation);
        assert_ne!(captured.presentation, later_presentation);
        assert_eq!(captured.pixels.rgba, [18, 52, 86, 255]);
    }

    #[test]
    fn expired_native_readback_refuses_late_pixels_without_polling_gpu() {
        let start = Instant::now();
        let result = poll_native::<Frame>(start, start + Duration::from_secs(5), || {
            panic!("an expired capture must not service the GPU or accept late pixels")
        });
        assert!(matches!(result, Some(Err(error)) if error.contains("5000 ms")));
    }
}
