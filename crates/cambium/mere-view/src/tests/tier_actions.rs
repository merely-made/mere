/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

#[test]
fn every_node_has_a_kind_the_host_colours_by() {
    let host = Host::new(tier_model(), 900, 600);
    let swatch = tier_view(&host).swatch();
    let kind = |key: &str| {
        swatch
            .graph
            .nodes
            .iter()
            .find(|node| node.id == key)
            .map(|node| node.kind)
    };
    assert_eq!(kind("b"), Some(MereNodeKind::Graph(NodeState::Unavailable)));
    assert_eq!(
        kind(&session_node_key(session_id(1))),
        Some(MereNodeKind::Session {
            attached: true,
            trashed: false
        })
    );
    assert_eq!(
        kind(&archive_node_key("saved")),
        Some(MereNodeKind::Archive { held: true })
    );
    assert_eq!(
        kind(&archive_node_key("composed")),
        Some(MereNodeKind::Archive { held: false })
    );
    assert_eq!(kind(SHELF_NODE_KEY), Some(MereNodeKind::Shelf));
    // A palette that knows only graph states still reads a tier node.
    assert_eq!(
        MereNodeKind::Session {
            attached: true,
            trashed: false
        }
        .state(),
        NodeState::Open
    );
    // The palette sees each kind, and every target says its kind.
    let seen = RefCell::new(Vec::new());
    tier_view(&host).paint_leaf(|kind| {
        seen.borrow_mut().push(kind.token());
        sprigging::ColorF {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        }
    });
    for token in ["graph", "session", "archive", "shelf"] {
        assert!(seen.borrow().contains(&token), "the palette sees {token}");
    }
    let (dom, runner) = runner(host);
    let dom = dom.borrow();
    let shelf = find(&dom, runner.root(), "data-key", SHELF_NODE_KEY).unwrap();
    assert_eq!(attr(&dom, shelf, "data-node-kind"), Some("shelf"));
    let alpha = find(&dom, runner.root(), "data-key", "a").unwrap();
    assert_eq!(attr(&dom, alpha, "data-node-kind"), Some("graph"));
}

#[test]
fn an_open_card_grows_the_tier_and_the_graph_is_laid_out_again_below_it() {
    let (dom, mut runner) = runner(Host::new(tier_model(), 900, 600));
    let root = runner.root();
    let (rest, laid_before) = {
        let host = runner.state();
        (tier_view(host).tier_height(), host.view.position("a"))
    };
    let morning = find(
        &dom.borrow(),
        root,
        "data-key",
        &session_node_key(session_id(1)),
    )
    .unwrap();
    runner.dispatch_click(morning, PointerClick::at((1., 1.)));
    let host = runner.state();
    let view = tier_view(host);
    let open = view.tier_height();
    assert!(
        open > rest,
        "the tier grows to hold the card: {rest} to {open}"
    );
    assert_ne!(
        host.view.position("a"),
        laid_before,
        "the graph is laid out again in the smaller area"
    );
    let swatch = view.swatch();
    let card = swatch
        .projected_node_footprint(&session_node_key(session_id(1)))
        .unwrap();
    assert!(
        card.top + card.height <= open as f32,
        "the card sits in the tier"
    );
    for (node, (id, (_, y))) in swatch.graph.nodes.iter().zip(swatch.projected_positions()) {
        if matches!(node.kind, MereNodeKind::Graph(_)) {
            assert!(y > open as f32, "{id} sits below the grown tier");
        } else {
            assert!(y < open as f32, "{id} sits in the grown tier");
        }
    }
    // Closing it lets the tier rest again.
    runner.update(|host| {
        host.view.apply(MereViewEvent::Expand(None));
        host.view.lay_out(&host.model, host.width, host.height);
    });
    assert_eq!(tier_view(runner.state()).tier_height(), rest);
    assert_eq!(runner.state().view.position("a"), laid_before);
}

#[test]
fn past_a_few_archives_a_session_shows_one_more_node_that_opens_its_card() {
    let mut model = tier_model();
    let archive = model.archive.as_mut().unwrap();
    for n in 0..4 {
        archive.entries.push(ArchiveEntry {
            key: format!("evening-{n}"),
            name: format!("Codicil blake3:e{n}"),
            detail: "Read-only archive".into(),
            can_open: true,
            session: Some(session_id(2)),
        });
    }
    let (dom, mut runner) = runner(Host::new(model, 900, 600));
    let root = runner.root();
    let more_key = more_node_key(TierCard::Session(session_id(2)));
    {
        let dom = dom.borrow();
        let more = find(&dom, root, "data-key", &more_key).expect("one +N node");
        assert_eq!(
            attr(&dom, more, "aria-label"),
            Some("+4 archives of Session Evening reading")
        );
        assert_eq!(attr(&dom, more, "data-count"), Some("4"));
        assert!(
            find(&dom, root, "data-key", &archive_node_key("evening-0")).is_none(),
            "the four collapse into it"
        );
        assert!(
            find(
                &dom,
                root,
                "data-relation-id",
                "saved-from:more:00000000-0000-0000-0000-000000000002"
            )
            .is_some()
        );
        // Morning's single archive still shows as its own node.
        assert!(find(&dom, root, "data-key", &archive_node_key("saved")).is_some());
    }
    assert!(DEFAULT_MAX_ARCHIVE_NODES < 4);
    let more = find(&dom.borrow(), root, "data-key", &more_key).unwrap();
    runner.dispatch_click(more, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().view.expanded,
        Some(TierCard::Session(session_id(2)))
    );
    // A +N node only opens: a second press keeps the card open.
    let more = find(&dom.borrow(), root, "data-key", &more_key).unwrap();
    runner.dispatch_click(more, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().view.expanded,
        Some(TierCard::Session(session_id(2)))
    );
    // The card's chips stand in for the collapsed nodes: one pins its popover
    // beside the chip, and Escape hands focus back to the chip.
    let chip = find(&dom.borrow(), root, "data-archive-chip", "evening-2").unwrap();
    runner.dispatch_click(chip, PointerClick::at((1., 1.)));
    let open = {
        let dom = dom.borrow();
        let dialog = find(&dom, root, "role", "dialog").expect("the chip pins a popover");
        assert_eq!(attr(&dom, dialog, "aria-label"), Some("Codicil blake3:e2"));
        find(&dom, dialog, "data-step", "open-archive").unwrap()
    };
    runner.set_focus(Some(open));
    runner.dispatch_key(escape());
    let chip = find(&dom.borrow(), root, "data-archive-chip", "evening-2").unwrap();
    assert_eq!(runner.focus(), Some(chip));
}

#[test]
fn the_host_sets_how_many_archives_show_before_they_collapse() {
    let mut model = tier_model();
    let archive = model.archive.as_mut().unwrap();
    for n in 0..4 {
        archive.entries.push(ArchiveEntry {
            key: format!("evening-{n}"),
            name: format!("Codicil blake3:e{n}"),
            detail: "Read-only archive".into(),
            can_open: true,
            session: Some(session_id(2)),
        });
    }
    let nodes = |model: &MereViewModel| {
        let host = Host::new(model.clone(), 900, 600);
        tier_view(&host)
            .swatch()
            .graph
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>()
    };
    let more = more_node_key(TierCard::Session(session_id(2)));
    assert!(
        nodes(&model).contains(&more),
        "four pass the default of three"
    );
    model.max_archive_nodes = Some(4);
    let shown = nodes(&model);
    assert!(!shown.contains(&more), "the host lets four show");
    assert!(shown.contains(&archive_node_key("evening-3")));
    model.max_archive_nodes = Some(0);
    assert!(
        nodes(&model).contains(&more_node_key(TierCard::Session(session_id(1)))),
        "even one collapses at zero"
    );
}

#[test]
fn apply_and_lay_out_lays_the_graph_out_again_for_an_open_card() {
    let model = tier_model();
    let mut state = MereViewState::default();
    state.lay_out(&model, 900, 600);
    let before = state.position("a");
    state.apply_and_lay_out(
        MereViewEvent::Expand(Some(TierCard::Session(session_id(1)))),
        &model,
        900,
        600,
    );
    assert_eq!(state.expanded, Some(TierCard::Session(session_id(1))));
    assert_ne!(
        state.position("a"),
        before,
        "laid out again below the grown tier"
    );
}

#[test]
fn tier_labels_show_the_name_and_keep_clear_of_the_tier_controls() {
    let host = Host::new(tier_model(), 900, 600);
    let swatch = tier_view(&host).swatch();
    let visible = |key: &str| {
        swatch
            .visible_labels
            .iter()
            .find(|(id, _)| id == key)
            .map(|(_, text)| text.as_str())
    };
    let morning = session_node_key(session_id(1));
    assert_eq!(visible(&morning), Some("Morning reading"));
    assert_eq!(
        visible(&archive_node_key("composed")),
        Some("Codicil blake3:0c0de")
    );
    assert_eq!(visible(SHELF_NODE_KEY), Some("Archive"));
    let node = swatch
        .graph
        .nodes
        .iter()
        .find(|node| node.id == morning)
        .unwrap();
    assert_eq!(
        node.label, "Session Morning reading, shown here",
        "the accessible name keeps the words"
    );
    // The shelf sits in the last slot, against the tier's controls: its
    // codicils' labels go left; the first session's stays right.
    let placement = |key: &str| {
        swatch
            .label_placements
            .iter()
            .find(|(id, _)| id == key)
            .map(|(_, placement)| *placement)
    };
    assert_eq!(
        placement(&archive_node_key("forgotten")),
        Some(cambium::GraphLabelPlacement::Left)
    );
    assert_eq!(placement(&morning), None);
}

#[test]
fn the_sheet_pins_the_rows_a_card_and_a_popover_are_measured_by() {
    use crate::tier::{BUTTON_HEIGHT, CARD_BORDER, CARD_HEAD, ROW_GAP};
    use crate::view::{CONTAINER_PAD, LINE};
    let sheet = MERE_VIEW_CSS;
    for rule in [
        format!(
            ".mere-view-card {{ position: absolute; z-index: 3; border-width: {CARD_BORDER}px; padding: {CONTAINER_PAD}px;"
        ),
        format!(
            ".mere-view-card-head {{ display: flex; flex-wrap: nowrap; align-items: center; gap: {ROW_GAP}px; height: {CARD_HEAD}px;"
        ),
        // Each pinned button's line is its height less its 1px borders, so its
        // text sits in the middle rather than at the top.
        format!(
            ".mere-view-card .mere-view-session-steps > button {{ height: {BUTTON_HEIGHT}px; line-height: {}px;",
            BUTTON_HEIGHT - 2
        ),
        ".mere-view-card .mere-view-card-close { height: 28px; line-height: 26px;".to_string(),
        ".mere-view-archive-chip { height: 24px; line-height: 22px;".to_string(),
        format!(
            ".mere-view-card .mere-view-session-steps {{ gap: {ROW_GAP}px; padding: 0 0 {ROW_GAP}px 0;"
        ),
        format!(
            ".mere-view-popover-layer .mere-view-archive-steps > button {{ height: {BUTTON_HEIGHT}px; line-height: {}px;",
            BUTTON_HEIGHT - 2
        ),
    ] {
        assert!(sheet.contains(&rule), "the sheet pins {rule}");
    }
    // Morning's card: border and padding, head, state, Fork and Trash then
    // Save (two rows of controls), and its one codicil.
    let host = Host::new(tier_model(), 900, 600);
    let view = tier_view(&host);
    let expected = 2 * (CARD_BORDER + CONTAINER_PAD)
        + CARD_HEAD
        + LINE
        + (ROW_GAP + 2 * BUTTON_HEIGHT + ROW_GAP)
        + 28;
    assert_eq!(
        crate::tier::card_height(view.model, TierCard::Session(session_id(1))),
        expected
    );
}

#[test]
fn branch_sits_beside_fork_and_merge_into_source_on_a_branch() {
    let mut model = model();
    model.sessions[1].steps = vec![
        SessionStep::Switch,
        SessionStep::Fork,
        SessionStep::Branch,
        SessionStep::Merge,
        SessionStep::Trash,
    ];
    let (dom, mut runner) = runner(Host::new(model, 900, 600));
    let root = runner.root();
    {
        let dom = dom.borrow();
        let evening = find(&dom, root, "data-session", &Uuid::from_u128(2).to_string()).expect("2");
        let mut steps = Vec::new();
        find_all(&dom, evening, "data-step", &mut steps);
        let tokens: Vec<_> = steps.iter().map(|&s| attr(&dom, s, "data-step")).collect();
        assert_eq!(
            tokens,
            [
                Some("switch"),
                Some("fork"),
                Some("branch"),
                Some("merge"),
                Some("trash")
            ]
        );
    }
    for label in [
        "Branch Evening reading",
        "Merge into source Evening reading",
    ] {
        let step = find(&dom.borrow(), root, "aria-label", label).expect(label);
        runner.dispatch_click(step, PointerClick::at((1.0, 1.0)));
    }
    assert_eq!(
        runner.state().requests,
        [
            MereViewRequest::Session(session_id(2), SessionStep::Branch),
            MereViewRequest::Session(session_id(2), SessionStep::Merge),
        ]
    );
}

#[test]
fn a_pending_merge_lists_each_conflict_with_its_choices_then_apply() {
    let (dom, mut runner) = runner(Host::new(merge_model(), 900, 600));
    let root = runner.root();
    {
        let dom = dom.borrow();
        let card = find(
            &dom,
            root,
            "aria-label",
            "Pending merge: Evening reading into Morning reading",
        )
        .expect("merge card");
        assert_eq!(attr(&dom, card, "data-resolved"), Some("false"));
        let title = find(&dom, card, "data-key", "title-1").expect("conflict");
        let words = text(&dom, title);
        for want in [
            "Title of One",
            "Base: One",
            "Source: Branch",
            "Target: Target",
        ] {
            assert!(words.contains(want), "{words}");
        }
        assert_eq!(count(&dom, title, "data-choice", "add-as-new"), 0);
        let vanished = find(&dom, card, "data-key", "node-2").expect("conflict");
        assert!(text(&dom, vanished).contains("Target: absent"));
        assert_eq!(attr(&dom, vanished, "data-chosen"), Some("target"));
        assert!(find(&dom, vanished, "aria-label", "Add Two as new").is_some());
        let target = find(&dom, vanished, "aria-label", "Keep the target's Two").unwrap();
        assert_eq!(attr(&dom, target, "aria-pressed"), Some("true"));
        let apply = find(&dom, card, "aria-label", "Apply merge").expect("apply");
        assert_eq!(attr(&dom, apply, "aria-disabled"), Some("true"));
    }
    let apply = find(&dom.borrow(), root, "aria-label", "Apply merge").unwrap();
    runner.dispatch_click(apply, PointerClick::at((1.0, 1.0)));
    assert!(runner.state().requests.is_empty(), "nothing to apply yet");
    for label in [
        "Take the source's Title of One",
        "Add Two as new",
        "Cancel merge",
    ] {
        let control = find(&dom.borrow(), root, "aria-label", label).expect(label);
        runner.dispatch_click(control, PointerClick::at((1.0, 1.0)));
    }
    assert_eq!(
        runner.state().requests,
        [
            MereViewRequest::Merge(MergeRequest::Choose("title-1".into(), MergeOption::Source)),
            MereViewRequest::Merge(MergeRequest::Choose("node-2".into(), MergeOption::AddAsNew)),
            MereViewRequest::Merge(MergeRequest::Cancel),
        ]
    );

    let mut resolved = merge_model();
    resolved.merge.as_mut().unwrap().conflicts[0].chosen = Some(MergeOption::Source);
    let (dom, mut resolved) = self::runner(Host::new(resolved, 900, 600));
    let root = resolved.root();
    let apply = find(&dom.borrow(), root, "aria-label", "Apply merge").unwrap();
    assert_eq!(attr(&dom.borrow(), apply, "aria-disabled"), None);
    resolved.dispatch_click(apply, PointerClick::at((1.0, 1.0)));
    assert_eq!(
        resolved.state().requests,
        [MereViewRequest::Merge(MergeRequest::Apply)]
    );
}

#[test]
fn a_hand_value_is_typed_in_place_and_reported_by_edit_by_hand() {
    let (dom, mut runner) = runner(Host::new(merge_model(), 900, 600));
    let root = runner.root();
    let field = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Hand value for Title of One",
    )
    .expect("hand field");
    assert_eq!(
        attr(&dom.borrow(), field, "data-cambium-text-value"),
        Some("Target"),
        "the draft starts from the target's value"
    );
    runner.set_focus(Some(field));
    for key in [
        cambium::Key::Named(cambium::NamedKey::Backspace),
        cambium::Key::Character("!".into()),
    ] {
        runner.dispatch_key(cambium::KeyEvent::new(key));
    }
    assert_eq!(runner.state().view.merge_drafts["title-1"].text(), "Targe!");
    let hand = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Use the hand value for Title of One",
    )
    .expect("edit by hand");
    runner.dispatch_click(hand, PointerClick::at((1.0, 1.0)));
    assert_eq!(
        runner.state().requests,
        [MereViewRequest::Merge(MergeRequest::Choose(
            "title-1".into(),
            MergeOption::Hand("Targe!".into())
        ))]
    );

    // A draft goes with its merge.
    let mut view = runner.state().view.clone();
    let mut model = merge_model();
    model.merge = None;
    view.lay_out(&model, 900, 600);
    assert!(view.merge_drafts.is_empty());
}

#[test]
fn a_replanned_merge_names_the_choices_it_dropped() {
    let (dom, runner) = runner(Host::new(merge_model(), 900, 600));
    assert!(
        find(
            &dom.borrow(),
            runner.root(),
            "aria-label",
            "Dropped choices"
        )
        .is_none(),
        "nothing dropped, nothing shown"
    );
    let mut model = merge_model();
    model.merge.as_mut().unwrap().dropped =
        vec!["Title of Two: Target (no longer in conflict)".into()];
    let (dom, runner) = self::runner(Host::new(model, 900, 600));
    let dom = dom.borrow();
    let dropped = find(&dom, runner.root(), "aria-label", "Dropped choices").expect("list");
    assert_eq!(attr(&dom, dropped, "role"), Some("status"));
    assert_eq!(
        text(&dom, dropped),
        "Title of Two: Target (no longer in conflict)"
    );
}
