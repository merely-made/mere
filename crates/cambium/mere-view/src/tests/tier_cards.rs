/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

#[test]
fn the_tier_puts_sessions_and_archives_on_the_canvas_one_level_up() {
    let (dom, mut runner) = runner(Host::new(tier_model(), 900, 600));
    let root = runner.root();
    {
        let dom = dom.borrow();
        let host = runner.state();
        let view = tier_view(host);
        assert_eq!(attr(&dom, root, "data-sessions"), Some("tier"));
        assert_eq!(
            view.graph_size(),
            (900, 600 - 40),
            "the tier is on the graph's own canvas, which keeps the whole body"
        );
        assert_eq!(view.overmap_height(), 0, "no band above the canvas");
        let tier = view.tier_height() as f32;
        assert!(tier > 0.0);

        // Sessions and archives are canvas nodes in the tier; the graph's own
        // nodes are laid out below it.
        let swatch = view.swatch();
        for (node, (id, (_, y))) in swatch.graph.nodes.iter().zip(swatch.projected_positions()) {
            let in_tier = !matches!(node.kind, MereNodeKind::Graph(_));
            if in_tier {
                assert!(y < tier, "{id} sits in the tier");
            } else {
                assert!(y > tier, "{id} sits below the tier");
            }
        }
        assert_eq!(
            swatch
                .graph
                .nodes
                .iter()
                .filter(|node| node.id.starts_with(SESSION_NODE_PREFIX))
                .count(),
            3
        );

        // The shown session is marked, in words and as the current node.
        let shown = find(&dom, root, "data-key", &session_node_key(session_id(1)))
            .expect("a session is a canvas node");
        assert_eq!(
            attr(&dom, shown, "aria-label"),
            Some("Session Morning reading, shown here")
        );
        assert_eq!(attr(&dom, shown, "aria-current"), Some("true"));
        assert_eq!(attr(&dom, shown, "data-attached"), Some("true"));
        assert_eq!(attr(&dom, shown, "aria-expanded"), Some("false"));
        let trashed = find(&dom, root, "data-key", &session_node_key(session_id(3))).unwrap();
        assert_eq!(
            attr(&dom, trashed, "aria-label"),
            Some("Session Old draft, in the trash")
        );

        // An archive is a node joined to the session it was saved from; one
        // no live session holds says it is in the archive.
        let saved = find(&dom, root, "data-key", &archive_node_key("saved")).unwrap();
        assert_eq!(attr(&dom, saved, "aria-haspopup"), Some("dialog"));
        assert_eq!(attr(&dom, saved, "data-tier"), Some("archive"));
        let route = find(&dom, root, "data-relation-id", "saved-from:saved")
            .expect("the archive's route to its session");
        assert_eq!(attr(&dom, route, "data-kind"), Some(SAVED_FROM));
        let composed = find(&dom, root, "data-key", &archive_node_key("composed")).unwrap();
        assert_eq!(
            attr(&dom, composed, "aria-label"),
            Some("Codicil blake3:0c0de, in the archive")
        );
        assert!(find(&dom, root, "data-relation-id", "saved-from:composed").is_none());

        // No column, no band, no card until one is opened.
        assert_eq!(count(&dom, root, "data-container", "session"), 0);
        assert!(find(&dom, root, "class", "mere-view-overmap").is_none());
    }

    // New session is the tier's own control; a graph node is still the host's.
    let mint = find(&dom.borrow(), root, "data-request", "mint").unwrap();
    runner.dispatch_click(mint, PointerClick::at((1., 1.)));
    let alpha = find(&dom.borrow(), root, "data-key", "a").unwrap();
    runner.dispatch_click(alpha, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().requests,
        [MereViewRequest::Mint, MereViewRequest::Activate("a".into())]
    );
}

#[test]
fn a_session_node_opens_in_place_into_its_card_one_at_a_time() {
    let model = tier_model();
    let before = model.clone();
    let (dom, mut runner) = runner(Host::new(model, 900, 600));
    let root = runner.root();
    let morning = session_node_key(session_id(1));
    let node = find(&dom.borrow(), root, "data-key", &morning).unwrap();
    runner.dispatch_click(node, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().view.expanded,
        Some(TierCard::Session(session_id(1)))
    );
    assert!(
        runner.state().requests.is_empty(),
        "opening a card asks nothing of the host"
    );
    {
        let dom = dom.borrow();
        let host = runner.state();
        let swatch = tier_view(host).swatch();
        let card = find(&dom, root, "aria-label", "Session Morning reading").expect("the card");
        assert_eq!(attr(&dom, card, "data-container"), Some("session"));
        assert!(text(&dom, card).contains("Shown here"));
        let node = find(&dom, root, "data-key", &morning).unwrap();
        assert_eq!(attr(&dom, node, "aria-expanded"), Some("true"));

        // The card is the node's footprint: it sits where the canvas says,
        // and the archive's route ends at its border, not its centre.
        let region = swatch
            .projected_node_footprint(&morning)
            .expect("footprint");
        let style = attr(&dom, card, "style").unwrap();
        assert!(
            style.contains(&format!("left:{}px", region.left)),
            "{style}"
        );
        assert!(style.contains(&format!("top:{}px", region.top)), "{style}");
        let (_, route) = swatch
            .projected_relations()
            .into_iter()
            .find(|(relation, _)| relation.id == "saved-from:saved")
            .unwrap();
        let end = *route.last().unwrap();
        let on_edge = |value: f32, edge: f32| (value - edge).abs() < 0.5;
        assert!(
            on_edge(end.1, region.top)
                || on_edge(end.1, region.top + region.height)
                || on_edge(end.0, region.left)
                || on_edge(end.0, region.left + region.width),
            "the route ends at the card's border: {end:?} {region:?}"
        );
        let (_, saved) = swatch
            .projected_positions()
            .into_iter()
            .find(|(id, _)| **id == archive_node_key("saved"))
            .unwrap();
        assert!(
            saved.1 > region.top + region.height || saved.1 < region.top,
            "the archive node moves clear of the card"
        );
        // The card holds the session's archives as chips.
        assert!(find(&dom, card, "data-archive-chip", "saved").is_some());
    }

    // The card's controls are the session's requests.
    for label in ["Save Morning reading to archive", "Fork Morning reading"] {
        let control = find(&dom.borrow(), root, "aria-label", label).unwrap();
        runner.dispatch_click(control, PointerClick::at((1., 1.)));
    }
    assert_eq!(
        runner.state().requests,
        [
            MereViewRequest::Archive(ArchiveRequest::Save(session_id(1))),
            MereViewRequest::Session(session_id(1), SessionStep::Fork),
        ]
    );

    // Opening another session's node moves the one card there.
    let evening = find(
        &dom.borrow(),
        root,
        "data-key",
        &session_node_key(session_id(2)),
    )
    .unwrap();
    runner.dispatch_click(evening, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().view.expanded,
        Some(TierCard::Session(session_id(2)))
    );
    assert_eq!(count(&dom.borrow(), root, "data-container", "session"), 1);
    assert!(find(&dom.borrow(), root, "aria-label", "Session Evening reading").is_some());

    // Close returns focus to the session's node; so does Escape.
    let close = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Close session Evening reading",
    )
    .unwrap();
    assert_eq!(
        runner.focus(),
        Some(close),
        "the card takes focus as it opens"
    );
    runner.dispatch_click(close, PointerClick::at((1., 1.)));
    assert_eq!(runner.state().view.expanded, None);
    let evening = find(
        &dom.borrow(),
        root,
        "data-key",
        &session_node_key(session_id(2)),
    )
    .unwrap();
    assert_eq!(runner.focus(), Some(evening));
    runner.dispatch_click(evening, PointerClick::at((1., 1.)));
    let close = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Close session Evening reading",
    )
    .unwrap();
    runner.set_focus(Some(close));
    runner.dispatch_key(escape());
    assert_eq!(runner.state().view.expanded, None);
    assert_eq!(runner.state().model, before, "a card is presentation only");
}

#[test]
fn an_archive_node_peeks_on_hover_and_pins_a_popover_on_click() {
    use cambium::HoverPhase;
    let (dom, mut runner) = runner(Host::new(tier_model(), 900, 600));
    let root = runner.root();
    let saved_key = archive_node_key("saved");
    let saved = find(&dom.borrow(), root, "data-key", &saved_key).unwrap();

    runner.dispatch_hover(saved, hover(HoverPhase::Enter));
    {
        let dom = dom.borrow();
        let peek = find(&dom, root, "role", "tooltip").expect("hover peeks");
        assert!(text(&dom, peek).contains("Saved from Morning reading"));
        assert!(
            find(&dom, peek, "data-step", "open-archive").is_none(),
            "a peek is words, not controls"
        );
    }
    runner.dispatch_hover(saved, hover(HoverPhase::Leave));
    assert!(find(&dom.borrow(), root, "role", "tooltip").is_none());

    // A click pins the controls, and the first takes focus.
    runner.dispatch_click(saved, PointerClick::at((1., 1.)));
    let open = {
        let dom = dom.borrow();
        let dialog = find(&dom, root, "role", "dialog").expect("a click pins");
        assert_eq!(
            attr(&dom, dialog, "aria-label"),
            Some("Codicil blake3:f612a")
        );
        assert!(find(&dom, dialog, "data-request", "compose-archives").is_some());
        let saved = find(&dom, root, "data-key", &saved_key).unwrap();
        assert_eq!(attr(&dom, saved, "aria-expanded"), Some("true"));
        find(
            &dom,
            dialog,
            "aria-label",
            "Open Codicil blake3:f612a as session",
        )
        .unwrap()
    };
    assert_eq!(runner.focus(), Some(open));
    // Hovering another node leaves a pinned popover where it is.
    let composed = find(
        &dom.borrow(),
        root,
        "data-key",
        &archive_node_key("composed"),
    )
    .unwrap();
    runner.dispatch_hover(composed, hover(HoverPhase::Enter));
    assert!(find(&dom.borrow(), root, "role", "dialog").is_some());
    assert!(find(&dom.borrow(), root, "role", "tooltip").is_none());
    runner.dispatch_hover(composed, hover(HoverPhase::Leave));

    let select = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Select Codicil blake3:f612a for composition",
    )
    .unwrap();
    runner.dispatch_click(select, PointerClick::at((1., 1.)));
    // Escape dismisses and hands focus back to the archive's node.
    runner.set_focus(Some(open));
    runner.dispatch_key(escape());
    assert!(find(&dom.borrow(), root, "role", "dialog").is_none());
    let saved = find(&dom.borrow(), root, "data-key", &saved_key).unwrap();
    assert_eq!(runner.focus(), Some(saved));

    // Open as session asks for the session and closes the popover, so the
    // next click reaches the canvas rather than the dismiss layer.
    runner.dispatch_click(saved, PointerClick::at((1., 1.)));
    let open = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Open Codicil blake3:f612a as session",
    )
    .unwrap();
    runner.dispatch_click(open, PointerClick::at((1., 1.)));
    assert!(find(&dom.borrow(), root, "role", "dialog").is_none());
    assert_eq!(runner.state().view.archive_popover, None);

    // A second archive's popover composes the two; an outside click closes it.
    let composed = find(
        &dom.borrow(),
        root,
        "data-key",
        &archive_node_key("composed"),
    )
    .unwrap();
    runner.dispatch_click(composed, PointerClick::at((1., 1.)));
    let select = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Select Codicil blake3:0c0de for composition",
    )
    .unwrap();
    runner.dispatch_click(select, PointerClick::at((1., 1.)));
    let compose = find(&dom.borrow(), root, "data-request", "compose-archives").unwrap();
    runner.dispatch_click(compose, PointerClick::at((1., 1.)));
    let outside = find(
        &dom.borrow(),
        root,
        "class",
        "overlay-surface-dismiss-layer",
    )
    .unwrap();
    runner.dispatch_click(outside, PointerClick::at((1., 1.)));
    assert!(find(&dom.borrow(), root, "role", "dialog").is_none());
    assert_eq!(
        runner.state().requests,
        [
            MereViewRequest::Archive(ArchiveRequest::Open("saved".into())),
            MereViewRequest::Archive(ArchiveRequest::Compose(vec![
                "saved".into(),
                "composed".into()
            ])),
        ]
    );

    // A card's archive chip pins the same popover.
    let morning = find(
        &dom.borrow(),
        root,
        "data-key",
        &session_node_key(session_id(1)),
    )
    .unwrap();
    runner.dispatch_click(morning, PointerClick::at((1., 1.)));
    let chip = find(&dom.borrow(), root, "data-archive-chip", "saved").unwrap();
    runner.dispatch_click(chip, PointerClick::at((1., 1.)));
    assert!(find(&dom.borrow(), root, "role", "dialog").is_some());
}

#[test]
fn a_card_or_popover_closes_when_its_session_or_archive_is_gone() {
    let mut host = Host::new(tier_model(), 900, 600);
    host.view
        .apply(MereViewEvent::Expand(Some(TierCard::Session(session_id(
            2,
        )))));
    host.view.apply(MereViewEvent::ArchivePopover(
        "composed".into(),
        cambium::DetailPopoverEvent::TogglePinned,
    ));
    assert!(host.view.archive_popover.is_some());
    host.model
        .sessions
        .retain(|session| session.id != session_id(2));
    if let Some(archive) = &mut host.model.archive {
        archive.entries.retain(|entry| entry.key != "composed");
    }
    host.view.lay_out(&host.model, 900, 600);
    assert_eq!(host.view.expanded, None);
    assert_eq!(host.view.archive_popover, None);

    // Without sessions, archives or minting there is no tier at all.
    let mut bare = tier_model();
    bare.sessions.clear();
    bare.archive = None;
    bare.can_mint = false;
    let host = Host::new(bare, 900, 600);
    assert_eq!(tier_view(&host).tier_height(), 0);
    assert_eq!(tier_view(&host).graph_size(), (900, 560));
}

#[test]
fn loose_archives_hang_off_the_shelf_which_opens_into_its_card() {
    let (dom, mut runner) = runner(Host::new(tier_model(), 900, 600));
    let root = runner.root();
    {
        let dom = dom.borrow();
        let shelf = find(&dom, root, "data-key", SHELF_NODE_KEY).expect("a shelf node");
        assert_eq!(attr(&dom, shelf, "aria-label"), Some("Archive"));
        for loose in ["composed", "forgotten"] {
            let route = find(
                &dom,
                root,
                "data-relation-id",
                &format!("{ON_SHELF}:{loose}"),
            )
            .expect("a loose archive joins the shelf");
            assert_eq!(attr(&dom, route, "data-kind"), Some(ON_SHELF));
        }
        assert!(find(&dom, root, "data-relation-id", "on-shelf:saved").is_none());
        // New session and Compose are the tier's own controls.
        let mut controls = Vec::new();
        find_all(&dom, root, "class", &mut controls);
        let tier_controls = controls
            .into_iter()
            .find(|node| attr(&dom, *node, "class") == Some("mere-view-tier-controls"))
            .expect("the tier's controls");
        assert!(find(&dom, tier_controls, "data-request", "mint").is_some());
        assert!(find(&dom, tier_controls, "data-request", "compose-archives").is_some());
    }
    let shelf = find(&dom.borrow(), root, "data-key", SHELF_NODE_KEY).unwrap();
    runner.dispatch_click(shelf, PointerClick::at((1., 1.)));
    assert_eq!(runner.state().view.expanded, Some(TierCard::Shelf));
    let dom = dom.borrow();
    let card = find(&dom, root, "data-container", "shelf").expect("the shelf's card");
    assert_eq!(attr(&dom, card, "aria-label"), Some("Archive"));
    assert!(find(&dom, card, "data-request", "compose-archives").is_some());
    assert!(find(&dom, card, "data-archive-chip", "composed").is_some());
    assert!(find(&dom, card, "data-archive-chip", "forgotten").is_some());
    assert!(find(&dom, card, "data-archive-chip", "saved").is_none());
}
