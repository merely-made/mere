// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    A11yAdapterState as Adapter, A11yPublication as Publication, A11yPublicationOutcome as Outcome,
    A11yPublicationRefusal as Refusal, A11yPublicationSkip as Skip, A11ySync,
};
use std::cell::Cell;

struct Publisher(Rc<Cell<Publication>>, Rc<RefCell<Vec<A11yRequest>>>);
impl Accessibility for Publisher {
    fn adapter_state(&self) -> Adapter {
        self.0.get().adapter
    }

    fn sync(
        &mut self,
        _: &crate::A11yOwner,
        _: &WindowDom<'_>,
        _: &OwnedLayout,
        _: &sprigging::LeafRegistry<u64>,
        _: &ProducerRegistry,
        _: Option<u64>,
        _: f64,
    ) -> A11ySync {
        A11ySync {
            publication: self.0.get(),
            requests: self.1.take(),
        }
    }
}

#[test]
fn publication_observer_tracks_refusal_retry_skips_and_disposal_without_dispatch_claims() {
    let mut host = counter_host();
    let publication = Rc::new(Cell::new(Publication {
        adapter: Adapter::Installed,
        outcome: Outcome::Submitted {
            nodes: 3,
            contribution_issues: 1,
        },
    }));
    let pending = Rc::new(RefCell::new(Vec::new()));
    host.s.a11y = Some(Box::new(Publisher(publication.clone(), pending.clone())));
    let samples = Rc::new(RefCell::new(Vec::new()));
    let sink = samples.clone();
    host.s.a11y_publication_observer = Some(Box::new(move |value| sink.borrow_mut().push(value)));
    assert_eq!(host.a11y_publication().sequence, 0);
    assert!(host.sync_a11y().is_empty());
    let first = host.a11y_publication();
    assert_eq!(first.publication, publication.get());
    let refused = Publication::refused(Adapter::Installed, Refusal::InvalidTree);
    publication.set(refused);
    // Dispatch is independently admitted, even when today's publication fails.
    let button = host.s.runner.as_ref().unwrap().focusables()[0];
    pending.borrow_mut().push(A11yRequest::node(
        crate::A11yAction::SetValue(f64::NAN),
        button,
    ));
    assert_eq!(
        host.sync_a11y(),
        [crate::A11yDispatch::Rejected(
            crate::A11yRejection::InvalidValue
        )]
    );
    assert_eq!(host.a11y_publication().publication, refused);
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 0);
    publication.set(first.publication);
    host.sync_a11y();
    assert_eq!(host.a11y_publication().publication, first.publication);
    publication.set(Publication {
        adapter: Adapter::Detached,
        ..first.publication
    });
    host.s.layout = None;
    host.sync_a11y();
    assert_eq!(
        host.a11y_publication().publication.outcome,
        Outcome::NotAttempted {
            reason: Skip::NoLayout
        }
    );
    assert_eq!(
        host.a11y_publication().publication.adapter,
        Adapter::Detached,
        "skips read current attachment, not an older installed report"
    );
    host.s.runner = None;
    host.sync_a11y();
    assert_eq!(
        host.a11y_publication().publication.outcome,
        Outcome::NotAttempted {
            reason: Skip::NoRunner
        }
    );
    host.s.a11y = None;
    host.sync_a11y();
    assert_eq!(
        host.a11y_publication().publication,
        Publication::skipped(Adapter::NotConfigured, Skip::NoAdapter)
    );
    let before_read = host.a11y_publication();
    for _ in 0..3 {
        assert_eq!(host.a11y_publication(), before_read);
    }
    assert_eq!(samples.borrow().len(), 6);
    drop(host.take_disposal());
    assert_eq!(
        host.a11y_publication().publication.outcome,
        Outcome::NotAttempted {
            reason: Skip::OwnerEnded
        }
    );
    let last = host.a11y_publication();
    drop(host.take_disposal());
    assert_eq!(host.a11y_publication(), last, "disposal is idempotent");
    let samples = samples.borrow();
    assert_eq!(samples.len(), 7);
    for (i, sample) in samples.iter().enumerate() {
        assert_eq!(sample.host, first.host);
        assert_ne!(sample.host, 0);
        assert_eq!(sample.sequence, i as u64 + 1);
    }
    assert_ne!(counter_host().a11y_publication().host, first.host);
}

#[test]
fn disposal_publication_observer_runs_after_the_host_borrow_is_released() {
    let host = Rc::new(RefCell::new(counter_host()));
    let weak = Rc::downgrade(&host);
    let called = Rc::new(Cell::new(false));
    let observed = called.clone();
    host.borrow_mut().s.a11y_publication_observer = Some(Box::new(move |snapshot| {
        let host = weak.upgrade().unwrap();
        let loan = host
            .try_borrow_mut()
            .expect("disposal notification must permit host reentry");
        assert_eq!(loan.a11y_publication(), snapshot);
        assert_eq!(
            snapshot.publication.outcome,
            Outcome::NotAttempted {
                reason: Skip::OwnerEnded
            }
        );
        observed.set(true);
    }));
    let disposal = host.borrow_mut().take_disposal();
    assert!(!called.get());
    drop(disposal);
    assert!(called.get());
}
