// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Pending jobs belong to an appearance, even while its host is borrowed.
use crate::{
    CaptureFrame, CaptureRefusal, ContextGeneration, Frame, PendingPresentationCapture,
    PresentationBackend, PresentedFrame,
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

struct Job {
    pending: RefCell<Option<CaptureFrame>>,
    refusal: RefCell<Option<CaptureRefusal>>,
}
impl Job {
    fn cancel(&self, refusal: CaptureRefusal) {
        if self.refusal.borrow().is_none() {
            *self.refusal.borrow_mut() = Some(refusal);
        }
        // A reentrant close can invalidate a job being polled. The wrapper
        // checks the independent refusal after polling and releases it there.
        if let Ok(mut pending) = self.pending.try_borrow_mut() {
            pending.take();
        }
    }
}
struct State {
    valid: Cell<bool>,
    disposed: Cell<bool>,
    context: Cell<Option<(PresentationBackend, ContextGeneration)>>,
    refusal: RefCell<Option<CaptureRefusal>>,
    jobs: RefCell<Vec<Weak<Job>>>,
}
#[derive(Clone)]
pub struct CaptureScope(Rc<State>);
impl CaptureScope {
    pub fn invalidate(&self, refusal: CaptureRefusal) {
        self.0.valid.set(false);
        *self.0.refusal.borrow_mut() = Some(refusal.clone());
        self.0.jobs.borrow_mut().retain(|weak| {
            if let Some(job) = weak.upgrade() {
                job.cancel(refusal.clone());
                true
            } else {
                false
            }
        });
    }
    /// Terminal and idempotent. This requires no Host RefCell loan.
    pub fn dispose(&self) {
        if self.0.disposed.replace(true) {
            return;
        }
        let (backend, generation) = self
            .0
            .context
            .get()
            .unwrap_or((PresentationBackend::Wgpu, ContextGeneration(0)));
        self.invalidate(CaptureRefusal::PresentationError {
            backend,
            generation,
            source_key: None,
            error: "appearance disposed".into(),
        });
    }
}

pub struct CaptureOwner {
    scope: CaptureScope,
}
impl Default for CaptureOwner {
    fn default() -> Self {
        Self {
            scope: CaptureScope(Rc::new(State {
                valid: Cell::new(true),
                disposed: Cell::new(false),
                context: Cell::new(None),
                refusal: RefCell::new(None),
                jobs: RefCell::new(Vec::new()),
            })),
        }
    }
}
impl Drop for CaptureOwner {
    fn drop(&mut self) {
        self.scope.dispose();
    }
}
impl CaptureOwner {
    pub fn is_disposed(&self) -> bool {
        self.scope.0.disposed.get()
    }
    pub fn scope(&self) -> CaptureScope {
        self.scope.clone()
    }
    pub fn resume(
        &self,
        backend: PresentationBackend,
        generation: ContextGeneration,
    ) -> Result<(), CaptureRefusal> {
        if self.scope.0.disposed.get() {
            return Err(self.refusal(backend, generation));
        }
        if let Some((old_backend, old_generation)) = self.scope.0.context.get()
            && (old_backend, old_generation) != (backend, generation)
        {
            self.scope.invalidate(CaptureRefusal::ContextLost {
                backend: old_backend,
                generation: old_generation,
            });
        }
        self.scope.0.context.set(Some((backend, generation)));
        self.scope.0.valid.set(true);
        self.scope.0.refusal.borrow_mut().take();
        Ok(())
    }
    fn refusal(
        &self,
        backend: PresentationBackend,
        generation: ContextGeneration,
    ) -> CaptureRefusal {
        self.scope
            .0
            .refusal
            .borrow()
            .clone()
            .unwrap_or(CaptureRefusal::ContextLost {
                backend,
                generation,
            })
    }
    /// Check appearance liveness before allocating or copying graphics data.
    pub fn begin(
        &self,
        backend: PresentationBackend,
        stamp: PresentedFrame,
        allocate: impl FnOnce() -> Result<CaptureFrame, CaptureRefusal>,
    ) -> Result<CaptureFrame, CaptureRefusal> {
        if self.scope.0.context.get().is_none() {
            self.resume(backend, stamp.context_generation)?;
        }
        if !self.scope.0.valid.get()
            || self.scope.0.disposed.get()
            || self.scope.0.context.get() != Some((backend, stamp.context_generation))
        {
            return Err(self.refusal(backend, stamp.context_generation));
        }
        self.wrap(backend, stamp, allocate()?)
    }
    pub fn wrap(
        &self,
        backend: PresentationBackend,
        stamp: PresentedFrame,
        pending: CaptureFrame,
    ) -> Result<CaptureFrame, CaptureRefusal> {
        if self.scope.0.context.get().is_none() {
            self.resume(backend, stamp.context_generation)?;
        }
        if !self.scope.0.valid.get()
            || self.scope.0.disposed.get()
            || self.scope.0.context.get() != Some((backend, stamp.context_generation))
        {
            return Err(self.refusal(backend, stamp.context_generation));
        }
        let job = Rc::new(Job {
            pending: RefCell::new(Some(pending)),
            refusal: RefCell::new(None),
        });
        let mut jobs = self.scope.0.jobs.borrow_mut();
        jobs.retain(|job| job.strong_count() > 0);
        jobs.push(Rc::downgrade(&job));
        Ok(Box::new(ScopedCapture {
            job,
            stamp,
            backend,
            finished: false,
        }))
    }
}
struct ScopedCapture {
    job: Rc<Job>,
    stamp: PresentedFrame,
    backend: PresentationBackend,
    finished: bool,
}
impl PendingPresentationCapture for ScopedCapture {
    fn poll(&mut self) -> Option<Result<(PresentedFrame, Frame), CaptureRefusal>> {
        if self.finished {
            return None;
        }
        let refused = { self.job.refusal.borrow().clone() };
        let result = if let Some(refusal) = refused {
            Some(Err(refusal))
        } else {
            self.job
                .pending
                .borrow_mut()
                .as_mut()
                .and_then(|job| job.poll())
        };
        let refused = { self.job.refusal.borrow().clone() };
        let result = if let Some(refusal) = refused {
            Some(Err(refusal))
        } else {
            result
        };
        if result.is_some() {
            self.finished = true;
            self.job.pending.borrow_mut().take();
        }
        result.map(|result| {
            result.and_then(|(stamp, pixels)| {
                let bytes = (stamp.width as usize)
                    .checked_mul(stamp.height as usize)
                    .and_then(|n| n.checked_mul(4));
                if stamp != self.stamp
                    || (pixels.width, pixels.height) != (stamp.width, stamp.height)
                    || bytes != Some(pixels.rgba.len())
                {
                    return Err(CaptureRefusal::PresentationError {
                        backend: self.backend,
                        generation: self.stamp.context_generation,
                        source_key: None,
                        error: "capture returned foreign identity or pixel extent".into(),
                    });
                }
                Ok((stamp, pixels))
            })
        })
    }
}
