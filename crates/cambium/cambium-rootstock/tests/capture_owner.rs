// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use cambium_rootstock::{
    CaptureOwner, CaptureRefusal, ContextGeneration, Frame, PendingPresentationCapture,
    PresentationBackend, PresentedFrame,
};
use std::{cell::Cell, rc::Rc};

struct Pending {
    dropped: Rc<Cell<bool>>,
    stamp: PresentedFrame,
}
impl Drop for Pending {
    fn drop(&mut self) {
        self.dropped.set(true);
    }
}
impl PendingPresentationCapture for Pending {
    fn poll(&mut self) -> Option<Result<(PresentedFrame, Frame), CaptureRefusal>> {
        Some(Ok((
            self.stamp,
            Frame {
                width: 1,
                height: 1,
                rgba: vec![18, 52, 86, 255],
            },
        )))
    }
}
fn stamp(generation: u64) -> PresentedFrame {
    PresentedFrame {
        host: 3,
        sequence: 11,
        width: 1,
        height: 1,
        layout_scale: 1.,
        context_generation: ContextGeneration(generation),
    }
}

#[test]
fn dropping_owner_releases_pending_resources_and_refuses_late_pixels() {
    let owner = CaptureOwner::default();
    let dropped = Rc::new(Cell::new(false));
    let mut capture = owner
        .wrap(
            PresentationBackend::WebGl2,
            stamp(1),
            Box::new(Pending {
                dropped: dropped.clone(),
                stamp: stamp(1),
            }),
        )
        .unwrap();
    drop(owner);
    assert!(
        dropped.get(),
        "GPU resources must release before a client polls"
    );
    assert!(capture.poll().unwrap().is_err());
}
#[test]
fn restored_owner_never_revives_a_cancelled_capture() {
    let owner = CaptureOwner::default();
    let dropped = Rc::new(Cell::new(false));
    let mut capture = owner
        .wrap(
            PresentationBackend::WebGl2,
            stamp(1),
            Box::new(Pending {
                dropped: dropped.clone(),
                stamp: stamp(1),
            }),
        )
        .unwrap();
    owner.scope().invalidate(CaptureRefusal::ContextLost {
        backend: PresentationBackend::WebGl2,
        generation: ContextGeneration(1),
    });
    assert!(dropped.get());
    owner
        .resume(PresentationBackend::WebGl2, ContextGeneration(2))
        .unwrap();
    assert!(matches!(
        capture.poll(),
        Some(Err(CaptureRefusal::ContextLost {
            generation: ContextGeneration(1),
            ..
        }))
    ));
}
#[test]
fn foreign_completion_stamp_is_refused_and_resources_release() {
    let owner = CaptureOwner::default();
    let dropped = Rc::new(Cell::new(false));
    let mut capture = owner
        .wrap(
            PresentationBackend::WebGl2,
            stamp(1),
            Box::new(Pending {
                dropped: dropped.clone(),
                stamp: stamp(2),
            }),
        )
        .unwrap();
    assert!(capture.poll().unwrap().is_err());
    assert!(dropped.get());
}

#[test]
fn disposal_during_poll_refuses_completion_and_releases_the_active_job() {
    struct CloseDuringPoll {
        scope: cambium_rootstock::CaptureScope,
        dropped: Rc<Cell<bool>>,
    }
    impl Drop for CloseDuringPoll {
        fn drop(&mut self) {
            self.dropped.set(true);
        }
    }
    impl PendingPresentationCapture for CloseDuringPoll {
        fn poll(&mut self) -> Option<Result<(PresentedFrame, Frame), CaptureRefusal>> {
            self.scope.dispose();
            Some(Ok((
                stamp(1),
                Frame {
                    width: 1,
                    height: 1,
                    rgba: vec![18, 52, 86, 255],
                },
            )))
        }
    }
    let owner = CaptureOwner::default();
    let dropped = Rc::new(Cell::new(false));
    let mut capture = owner
        .wrap(
            PresentationBackend::WebGl2,
            stamp(1),
            Box::new(CloseDuringPoll {
                scope: owner.scope(),
                dropped: dropped.clone(),
            }),
        )
        .unwrap();
    assert!(capture.poll().unwrap().is_err());
    assert!(dropped.get());
    assert!(
        owner
            .resume(PresentationBackend::WebGl2, ContextGeneration(2))
            .is_err()
    );
}
#[test]
fn disposed_scope_refuses_before_allocating_readback() {
    let owner = CaptureOwner::default();
    let allocated = Rc::new(Cell::new(false));
    owner
        .resume(PresentationBackend::WebGl2, ContextGeneration(1))
        .unwrap();
    owner.scope().dispose();
    let probe = allocated.clone();
    let result = owner.begin(PresentationBackend::WebGl2, stamp(1), move || {
        probe.set(true);
        Ok(Box::new(Pending {
            dropped: Rc::new(Cell::new(false)),
            stamp: stamp(1),
        }) as cambium_rootstock::CaptureFrame)
    });
    assert!(result.is_err());
    assert!(!allocated.get());
}
