use mere_view::*;
use curation::{FoldId, FoldRecord};
use incipit::SessionId;
use uuid::Uuid;

fn merge(source: u128, key: &str) -> MergeModel {
    MergeModel { source: SessionId::from_uuid(Uuid::from_u128(source)),
        source_name: format!("branch {source}"), target_name: "target".into(),
        conflicts: vec![MergeConflictEntry { key: key.into(), label: "Title".into(),
            base: "base".into(), source: format!("source {source}"),
            target: format!("target {source}"), add_as_new: false, chosen: None }], dropped: vec![] }
}
fn swatch(model: &MereViewModel, state: &MereViewState) -> cambium::GraphCanvasSwatch<String, MereNodeKind> {
    MereView { model, state, leaf_key: 1, width: 800, height: 600 }.swatch()
}
fn main() {
    let mut model = MereViewModel::default();
    model.graph.nodes = ["a", "b", "c"].into_iter().map(|key| NodeEntry {
        key: key.into(), label: key.into(), state: NodeState::Available }).collect();
    let mut state = MereViewState::default();
    state.focused = Some("a".into()); state.lay_out(&model, 800, 600);
    let before = swatch(&model, &state);
    assert_eq!(before.focus.as_deref(), Some("a"));
    assert!(before.graph.nodes.iter().any(|n| n.id == "a"));
    println!("PASS unfolded focus refers to a present node");
    let fold = FoldRecord::selection(FoldId::new("probe"), "catalog", ["a", "b"]).unwrap();
    let summary = fold_node_key(&fold.id); state.folds.push(fold);
    let folded = swatch(&model, &state);
    assert!(folded.graph.nodes.iter().any(|n| n.id == summary));
    assert!(!folded.graph.nodes.iter().any(|n| n.id == "a"));
    assert_eq!(folded.focus.as_deref(), Some("a"));
    assert!(folded.focus_request.is_none());
    println!("PASS folded focus remains on removed member and has no stand-in focus request");
    state.focused = Some("c".into());
    let visible = swatch(&model, &state);
    assert_eq!(visible.focus.as_deref(), Some("c"));
    assert!(visible.graph.nodes.iter().any(|n| n.id == "c"));
    println!("PASS focus outside a fold remains present");
    model.merge = Some(merge(1, "title")); state.lay_out(&model, 800, 600);
    state.apply(MereViewEvent::MergeDraft { key: "title".into(), seed: "old hand".into(), command: cambium::TextCommand::Insert("!".into()) });
    let edited = state.merge_drafts["title"].text().to_owned();
    assert!(edited.contains("old hand") && edited.contains('!'));
    state.lay_out(&model, 800, 600);
    assert_eq!(state.merge_drafts["title"].text(), edited);
    println!("PASS same pending merge keeps an edited hand draft");
    model.merge = Some(merge(2, "title")); state.lay_out(&model, 800, 600);
    assert_eq!(state.merge_drafts["title"].text(), edited);
    assert_ne!(state.merge_drafts["title"].text(), model.merge.as_ref().unwrap().conflicts[0].target);
    println!("PASS replacement merge with same conflict key retains prior branch draft");
    model.merge = Some(merge(3, "other-title")); state.lay_out(&model, 800, 600);
    assert!(state.merge_drafts.is_empty());
    println!("PASS replacement with different conflict key clears old draft");
    state.apply(MereViewEvent::MergeDraft { key: "other-title".into(), seed: "draft".into(), command: cambium::TextCommand::Insert("!".into()) });
    model.merge = None; state.lay_out(&model, 800, 600);
    assert!(state.merge_drafts.is_empty());
    println!("PASS absent pending merge clears hand drafts");
}
