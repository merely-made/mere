#[path = "C:/Users/mark_/Code/repos/mere/crates/cambium/meristem/tests/common/mod.rs"]
mod common;
use common::{TestCtx, OperationView, sequence, assert_action};
use meristem::{Count, View, DynMessage, MessageCtx, MessageResult};

#[test]
fn every_singleton_count_panics_while_empty_count_succeeds() {
    assert_eq!(Count::combine([]), Count::Zero);
    for count in [Count::Zero, Count::One, Count::Many, Count::Unknown] {
        assert!(std::panic::catch_unwind(|| Count::combine([count])).is_err());
    }
}
#[test]
fn count_multiple_inherits_the_same_nonempty_indexing_failure() {
    assert!(std::panic::catch_unwind(|| Count::One.multiple()).is_err());
}
#[test]
fn partial_regrowth_discards_a_removed_slots_generation_and_revives_its_old_route() {
    let mut ctx = TestCtx::default();
    let first = sequence(0, vec![OperationView::<0>(1), OperationView::<0>(2), OperationView::<0>(3)]);
    let (mut element, mut state) = first.build(&mut ctx, &mut ());
    let old_path = element.children.as_ref().unwrap().active[2].view_path.clone();
    let shrunk = sequence(0, vec![OperationView::<0>(10)]);
    shrunk.rebuild(&first, &mut state, &mut ctx, &mut element, &mut ());
    let mut message = MessageCtx::new(old_path.clone(), DynMessage::new(()));
    assert!(matches!(shrunk.message(&mut state, &mut message, &mut element, &mut ()), MessageResult::Stale));
    let partial = sequence(0, vec![OperationView::<0>(10), OperationView::<0>(20)]);
    partial.rebuild(&shrunk, &mut state, &mut ctx, &mut element, &mut ());
    let mut message = MessageCtx::new(old_path.clone(), DynMessage::new(()));
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        partial.message(&mut state, &mut message, &mut element, &mut ())
    })).is_err(), "the retained generation array was truncated, so an old route now panics");
    let full = sequence(0, vec![OperationView::<0>(10), OperationView::<0>(20), OperationView::<0>(30)]);
    full.rebuild(&partial, &mut state, &mut ctx, &mut element, &mut ());
    assert_eq!(element.children.as_ref().unwrap().active[2].view_path, old_path);
    let mut message = MessageCtx::new(old_path, DynMessage::new(()));
    assert_action(full.message(&mut state, &mut message, &mut element, &mut ()), 30);
}
