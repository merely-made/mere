#[path = "C:/Users/mark_/Code/repos/mere/crates/cambium/backstep/src/lib.rs"]
mod original;
use original::History;

#[test]
fn saving_during_a_run_keeps_later_changed_content_clean_and_skips_saved_state() {
    let mut history: History<u32, &str> = History::new().with_window(400);
    history.record(0, Some("name"), 1000); // current document becomes 1
    history.mark_saved(); // document 1 was successfully persisted
    assert!(!history.is_dirty());
    history.record(1, Some("name"), 1100); // current document becomes 2
    assert!(!history.is_dirty(), "current implementation falsely reads clean");
    assert_eq!(history.undo(2), Some(0), "current undo skips persisted document 1");
}

#[test]
fn reducing_a_populated_history_cap_does_not_enforce_the_new_depth() {
    let mut history: History<u32> = History::new().with_cap(0);
    for before in 0..5 { history.record(before, None, 0); }
    let mut history = history.with_cap(1);
    history.record(5, None, 0);
    let mut steps = 0;
    while history.undo(9).is_some() { steps += 1; }
    assert_eq!(steps, 5, "one oldest entry was removed, leaving five under cap 1");
}
