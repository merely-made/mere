// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::{
    A11yDispatch as D, A11yRejection as R, ProducerChildLifetime, ProducerChildToken,
    TextureProducer,
};
use std::cell::Cell;

struct Board {
    life: Rc<RefCell<Option<ProducerChildLifetime>>>,
    cached: Option<ProducerChildToken>,
    reads: Rc<Cell<usize>>,
    replace_at: Rc<Cell<usize>>,
    calls: Rc<Cell<usize>>,
}
impl TextureProducer for Board {
    fn render(&mut self, _: &crate::ProducerContext<'_>) -> Option<crate::ProducedTexture> {
        None
    }
    fn semantics(&self) -> Option<crate::ProducerSemantics> {
        LifetimeBoard {
            calls: Rc::new(Cell::new(0)),
            revision: Rc::new(Cell::new(0)),
        }
        .semantics()
    }
    fn child_lifetime(&self, _: u64) -> Option<ProducerChildToken> {
        let token = self.cached.clone().or_else(|| {
            self.life
                .borrow()
                .as_ref()
                .map(ProducerChildLifetime::token)
        });
        let n = self.reads.get() + 1;
        self.reads.set(n);
        if n == self.replace_at.get() {
            *self.life.borrow_mut() = Some(ProducerChildLifetime::new());
        }
        token
    }
    fn act(&mut self, _: u64, _: &str) -> bool {
        self.calls.set(self.calls.get() + 1);
        true
    }
}
fn bind(host: &Host<u32, CounterLogic, CounterChild>) -> A11yRequest {
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    lifetime_produced().bind_publication(
        &crate::A11yOwner::new(&host.s.a11y_owner),
        &WindowDom::new(&dom.borrow(), runner.mount()),
        &host.s.shared.leaves,
        &host.s.shared.producers,
    )
}
#[test]
fn expired_cached_child_token_refuses_raw_bound_queued_and_publication_paths() {
    let mut host = counter_host();
    let life = ProducerChildLifetime::new();
    let token = life.token();
    assert!(token.is_live());
    drop(life);
    assert!(!token.is_live());
    assert!(!ProducerChildToken::matches_current(
        Some(&token),
        Some(&token)
    ));
    host.s
        .shared
        .producers
        .register(
            7,
            Board {
                life: Rc::new(RefCell::new(None)),
                cached: Some(token),
                reads: Rc::new(Cell::new(0)),
                replace_at: Rc::new(Cell::new(usize::MAX)),
                calls: Rc::new(Cell::new(0)),
            },
            &[],
        )
        .unwrap();
    let bound = bind(&host);
    assert_eq!(
        host.apply_a11y_requests(&[bound, lifetime_produced()]),
        [D::Rejected(R::StaleTarget), D::Rejected(R::StaleTarget)]
    );
    let queued = defer_lifetime_request(&mut host, lifetime_produced());
    host.drain_pointer();
    assert_eq!(queued.result(), Some(D::Rejected(R::StaleTarget)));
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    let dom = dom.borrow();
    let view = WindowDom::new(&dom, runner.mount());
    let p = crate::project_semantics(
        &view,
        host.s.layout.as_ref().unwrap(),
        &host.s.shared.leaves,
        &host.s.shared.producers,
        None,
    );
    assert!(matches!(
        p.publication_bindings(
            &crate::A11yOwner::new(&host.s.a11y_owner),
            &view,
            &host.s.shared.leaves,
            &host.s.shared.producers
        ),
        Err(crate::PublicationIssue::ContributorEnded(_))
    ));
}
#[test]
fn activation_rechecks_after_the_last_admission_callback_for_raw_and_bound_requests() {
    for raw in [false, true] {
        let mut host = counter_host();
        let calls = Rc::new(Cell::new(0));
        let reads = Rc::new(Cell::new(0));
        let replace_at = Rc::new(Cell::new(usize::MAX));
        let life = Rc::new(RefCell::new(Some(ProducerChildLifetime::new())));
        host.s
            .shared
            .producers
            .register(
                7,
                Board {
                    life: life.clone(),
                    cached: None,
                    reads: reads.clone(),
                    replace_at: replace_at.clone(),
                    calls: calls.clone(),
                },
                &[],
            )
            .unwrap();
        let request = if raw {
            lifetime_produced()
        } else {
            bind(&host)
        };
        let old = life.borrow().clone(); // Retaining the old life cannot authorize the replacement.
        replace_at.set(reads.get() + if raw { 3 } else { 2 });
        let hooks = Rc::new(Cell::new(0));
        let observed = hooks.clone();
        host.hooks.after_dispatch = Box::new(move |_| observed.set(observed.get() + 1));
        assert_eq!(
            host.apply_a11y_requests(&[request]),
            [D::Rejected(R::StaleTarget)]
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(hooks.get(), 0);
        assert_eq!(
            host.apply_a11y_requests(&[lifetime_produced()]),
            [D::Dispatched]
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(hooks.get(), 1);
        drop(old);
    }
}
#[test]
fn rc_producer_holds_one_mutable_borrow_for_checked_validation_and_dispatch() {
    struct Owned {
        life: ProducerChildLifetime,
        alias: Rc<RefCell<Option<std::rc::Weak<RefCell<Owned>>>>>,
    }
    impl TextureProducer for Owned {
        fn render(&mut self, _: &crate::ProducerContext<'_>) -> Option<crate::ProducedTexture> {
            None
        }
        fn child_lifetime(&self, _: u64) -> Option<ProducerChildToken> {
            let alias = self.alias.borrow().as_ref().unwrap().upgrade().unwrap();
            assert!(alias.try_borrow_mut().is_err());
            Some(self.life.token())
        }
        fn act(&mut self, _: u64, _: &str) -> bool {
            let alias = self.alias.borrow().as_ref().unwrap().upgrade().unwrap();
            assert!(alias.try_borrow_mut().is_err());
            true
        }
    }
    let alias = Rc::new(RefCell::new(None));
    let life = ProducerChildLifetime::new();
    let token = life.token();
    let mut shared = Rc::new(RefCell::new(Owned {
        life,
        alias: alias.clone(),
    }));
    *alias.borrow_mut() = Some(Rc::downgrade(&shared));
    assert_eq!(
        shared.act_checked(40, "pin", Some(&token)),
        crate::ProducerActionDispatch::Dispatched
    );
    let old = std::mem::replace(&mut shared.borrow_mut().life, ProducerChildLifetime::new());
    assert_eq!(
        shared.act_checked(40, "pin", Some(&token)),
        crate::ProducerActionDispatch::StaleTarget
    );
    drop(old);
}
