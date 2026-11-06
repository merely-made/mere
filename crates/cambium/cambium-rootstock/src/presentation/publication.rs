// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Facts frozen with a picture, independent of the model at completion time.
use crate::{CaptureFn, CaptureRefusal, PaintCaptureFn, PresentedFrame, StampedCaptureFn};

/// An owned observation of a prepared frame. Source replacement can revoke it
/// while ordinary model changes are free to coalesce behind that frame.
pub struct PreparedPresentationObserver {
    publish: Box<dyn FnOnce(PresentedFrame)>,
    live: Box<dyn Fn() -> bool>,
}
impl PreparedPresentationObserver {
    pub fn new(publish: impl FnOnce(PresentedFrame) + 'static) -> Self {
        Self {
            publish: Box::new(publish),
            live: Box::new(|| true),
        }
    }
    pub fn while_valid(mut self, is_live: impl Fn() -> bool + 'static) -> Self {
        let previous = self.live;
        self.live = Box::new(move || previous() && is_live());
        self
    }
    pub fn is_live(&self) -> bool {
        (self.live)()
    }
    /// Hosts check every guard before queue presentation, then publish facts.
    pub fn publish(self, frame: PresentedFrame) -> bool {
        if !self.is_live() {
            return false;
        }
        (self.publish)(frame);
        true
    }
}

/// Requests belong to one prepared picture. New requests remain in the Host
/// for its next preparation rather than being relabelled as this older image.
#[derive(Default)]
pub(crate) struct PreparedCaptures {
    pub capture: Option<CaptureFn>,
    pub stamped: Option<StampedCaptureFn>,
    pub paint: Option<PaintCaptureFn>,
}
impl PreparedCaptures {
    pub(crate) fn refuse(&mut self, reason: CaptureRefusal) {
        for callback in [self.capture.take(), self.stamped.take()]
            .into_iter()
            .flatten()
        {
            callback(Err(reason.clone()));
        }
        if let Some(callback) = self.paint.take() {
            callback(Err(reason));
        }
    }
}
impl Drop for PreparedCaptures {
    fn drop(&mut self) {
        self.refuse(CaptureRefusal::PresentationError {
            backend: super::PresentationBackend::Wgpu,
            generation: super::ContextGeneration(0),
            source_key: None,
            error: "prepared picture retired before presentation".into(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };
    fn stamp() -> PresentedFrame {
        PresentedFrame {
            host: 1,
            sequence: 4,
            width: 20,
            height: 30,
            layout_scale: 1.,
            context_generation: super::super::ContextGeneration(2),
        }
    }
    #[test]
    fn observation_owns_prepared_facts_and_replacement_revokes_it() {
        let model = Rc::new(Cell::new(7));
        let accepted = Rc::new(Cell::new(0));
        let lifetime = Rc::new(Cell::new(true));
        let prepare = || {
            let fact = model.get();
            let accepted = accepted.clone();
            let lifetime = lifetime.clone();
            PreparedPresentationObserver::new(move |_| accepted.set(fact))
                .while_valid(move || lifetime.get())
        };
        let first = prepare();
        model.set(9);
        assert_eq!(accepted.get(), 0);
        assert!(first.publish(stamp()));
        assert_eq!(accepted.get(), 7);
        let second = prepare();
        lifetime.set(false);
        assert!(!second.is_live());
        assert!(!second.publish(stamp()));
        assert_eq!(accepted.get(), 7);
    }
    #[test]
    fn cancellation_refuses_each_bound_request_once() {
        let outcomes = Rc::new(RefCell::new(Vec::new()));
        let mut captures = PreparedCaptures::default();
        let out = outcomes.clone();
        captures.capture = Some(Box::new(move |r| out.borrow_mut().push(r.is_err())));
        let out = outcomes.clone();
        captures.stamped = Some(Box::new(move |r| out.borrow_mut().push(r.is_err())));
        let out = outcomes.clone();
        captures.paint = Some(Box::new(move |r| out.borrow_mut().push(r.is_err())));
        captures.refuse(CaptureRefusal::HostDisposed);
        drop(captures);
        assert_eq!(&*outcomes.borrow(), &[true, true, true]);
    }
}
