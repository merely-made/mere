// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cambium_rootstock::{
    ContextGeneration, PresentationBackend, PresentationChoice, PresentationLifetime,
    PresentationState, backend_attempts,
};

#[test]
fn forced_backend_never_attempts_another_route() {
    assert_eq!(
        backend_attempts(PresentationChoice::WebGpu),
        &[PresentationBackend::Wgpu]
    );
    assert_eq!(
        backend_attempts(PresentationChoice::WebGl2),
        &[PresentationBackend::WebGl2]
    );
    assert_eq!(
        backend_attempts(PresentationChoice::Auto),
        &[PresentationBackend::Wgpu, PresentationBackend::WebGl2]
    );
}

#[test]
fn loss_refuses_old_completions_before_restoration_starts() {
    let old = ContextGeneration(7);
    let mut life = PresentationLifetime::new(old);
    assert_eq!(life.state(), PresentationState::Ready);
    assert!(life.accepts(old));
    life.lose();
    assert_eq!(life.state(), PresentationState::Lost);
    assert!(!life.accepts(old));
    life.lose();
    assert_eq!(life.generation(), old);
    life.begin_restore().unwrap();
    assert_eq!(life.state(), PresentationState::Restoring);
    assert!(!life.accepts(old));
    let fresh = life.finish_restore().unwrap();
    assert_eq!(fresh, ContextGeneration(8));
    assert_eq!(life.state(), PresentationState::Ready);
    assert!(!life.accepts(old));
    assert!(life.accepts(fresh));
    assert!(life.finish_restore().is_err());
    assert_eq!(life.generation(), fresh);
}

#[test]
fn loss_during_rebuild_requires_a_new_restoration_attempt() {
    let mut life = PresentationLifetime::new(ContextGeneration(17));
    life.lose();
    life.begin_restore().unwrap();
    life.lose();
    assert_eq!(life.state(), PresentationState::Lost);
    assert!(life.finish_restore().is_err());
    assert_eq!(life.generation(), ContextGeneration(17));
    life.begin_restore().unwrap();
    assert_eq!(life.finish_restore().unwrap(), ContextGeneration(18));
}

#[test]
fn healthy_context_cannot_be_restored_without_loss() {
    let mut life = PresentationLifetime::new(ContextGeneration(4));
    assert!(life.begin_restore().is_err());
    assert!(life.finish_restore().is_err());
    assert!(life.accepts(ContextGeneration(4)));
}

#[test]
fn duplicate_begin_restore_does_not_restart_generation() {
    let mut life = PresentationLifetime::new(ContextGeneration(9));
    life.lose();
    life.begin_restore().unwrap();
    assert!(life.begin_restore().is_err());
    assert_eq!(life.finish_restore().unwrap(), ContextGeneration(10));
}

#[test]
fn generation_overflow_stays_unavailable_and_never_reuses_an_identity() {
    let mut life = PresentationLifetime::new(ContextGeneration(u64::MAX));
    life.lose();
    life.begin_restore().unwrap();
    assert!(life.finish_restore().is_err());
    assert!(!life.accepts(ContextGeneration(u64::MAX)));
    assert!(!life.accepts(ContextGeneration(0)));
    assert_eq!(life.generation(), ContextGeneration(u64::MAX));
}

#[test]
fn disposal_is_terminal_even_after_late_loss_notifications() {
    let mut life = PresentationLifetime::new(ContextGeneration(3));
    life.dispose();
    life.lose();
    life.dispose();
    assert_eq!(life.state(), PresentationState::Disposed);
    assert!(!life.accepts(ContextGeneration(3)));
    assert!(life.begin_restore().is_err());
    assert!(life.finish_restore().is_err());
}
