// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Windowless production layout and transactional controls for the shared skin.
use cambium_genet_winit_host::Harness;
use layout_dom_api::LayoutDom;
use tabard::theme::registry::Mode;
use tabard_workshop::{WorkshopState, WorkshopView, workshop_stylesheet, workshop_view};

type Host = Harness<WorkshopState, fn(&WorkshopState) -> WorkshopView, WorkshopView>;

fn host() -> Host {
    Harness::new(
        workshop_stylesheet(),
        WorkshopState::in_memory(),
        workshop_view as fn(&WorkshopState) -> WorkshopView,
    )
}

#[test]
fn wide_and_narrow_frames_keep_real_theme_identity_and_mode_controls() {
    let mut host = host();
    let id = host.state().draft_theme().id.clone();
    for width in [1280., 520.] {
        for mode in [Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark] {
            host.update(|s| s.set_mode(mode));
            host.layout_at(width, 1000.);
            let root = host.with_dom(|dom| {
                let roots = dom.all_with_class(dom.document(), "instrument-workspace");
                assert_eq!(roots.len(), 1);
                let root = roots[0];
                assert!(
                    dom.attributes(root)
                        .any(|a| a.name.local.as_ref() == "data-theme-definition" && a.value == id)
                );
                assert_eq!(dom.all_with_class(root, "workspace-pane-header").len(), 6);
                root
            });
            let (_, _, w, h) = host.painted_rect(root).expect("real laid-out frame");
            assert!((w - width).abs() < 1. && h > 0.);
            assert_eq!(host.state().draft_theme().id, id);
        }
    }
}

#[test]
fn instrument_frame_does_not_apply_or_commit_the_draft() {
    let mut host = host();
    host.layout_at(1280., 1000.);
    let before = host.state().registry().active_theme();
    let original = host.state().draft_theme().name.clone();
    host.update(|s| {
        *s.text_field_mut("name").unwrap() = cambium::TextInput::new("Instrument draft");
        s.sync_controls();
    });
    assert!(host.state().has_changes());
    assert_eq!(host.state().registry().active_theme(), before);
    assert!(host.state().saved_choice().is_err());
    let dirty = host.with_dom(|dom| {
        dom.all_with_class(dom.document(), "workspace-pane-state")
            .into_iter()
            .any(|n| {
                dom.dom_children(n)
                    .filter_map(|n| dom.text(n))
                    .collect::<String>()
                    == "Unsaved changes"
            })
    });
    assert!(dirty);
    host.update(WorkshopState::discard);
    assert_eq!(host.state().draft_theme().name, original);
    assert_eq!(host.state().registry().active_theme(), before);
}
