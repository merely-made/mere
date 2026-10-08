// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Acceptance receipts for the shipped retained workshop surface. These use
//! the desktop host's actual pointer, keyboard, and accessibility routing.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use accesskit::Role;
use cambium::TextCommand;
use cambium_genet_winit_host::Harness;
use genet_scripted_dom::NodeId;
use layout_dom_api::LayoutDom;
use tabard::theme::registry::ThemeSource;
use tabard_workshop::{WORKSHOP_CSS, WorkshopState, WorkshopView, workshop_view};
use taproot::Selector;
use winit::keyboard::NamedKey;

type Host = Harness<WorkshopState, fn(&WorkshopState) -> WorkshopView, WorkshopView>;

fn host(state: WorkshopState) -> Host {
    let mut host = Harness::new(
        WORKSHOP_CSS,
        state,
        workshop_view as fn(&WorkshopState) -> WorkshopView,
    );
    host.layout_at(1280.0, 960.0);
    host
}

fn action(name: &str) -> Selector {
    Selector::role("button").with_attr("data-action", name)
}

fn mode(name: &str) -> Selector {
    let label = match name {
        "light" => "Light",
        "dark" => "Dark",
        "hc_light" => "High contrast light",
        "hc_dark" => "High contrast dark",
        _ => panic!("unknown workshop mode {name}"),
    };
    Selector::role("button")
        .with_attr("data-mode", name)
        .containing(label)
}

fn hue() -> Selector {
    Selector::role("slider").containing("Primary hue")
}

#[track_caller]
fn click(host: &mut Host, selector: &Selector) {
    assert!(
        host.click_on(selector),
        "control does not paint: {selector:?}"
    );
}

fn selected_node(host: &Host, selector: &Selector) -> NodeId {
    host.with_dom(|dom| {
        let nodes = taproot::matching(dom, selector);
        assert_eq!(
            nodes.len(),
            1,
            "control identity must be unique: {selector:?}"
        );
        nodes[0]
    })
}

fn attribute(host: &Host, node: NodeId, name: &str) -> String {
    host.with_dom(|dom| {
        dom.attributes(node)
            .find(|attr| attr.name.local.as_ref() == name)
            .unwrap_or_else(|| panic!("node {node:?} needs {name}"))
            .value
            .to_owned()
    })
}

fn specimen(host: &Host, id: &str) -> NodeId {
    host.with_dom(|dom| {
        fn visit(dom: &genet_scripted_dom::ScriptedDom, node: NodeId, id: &str) -> Option<NodeId> {
            if dom
                .attributes(node)
                .any(|attr| attr.name.local.as_ref() == "id" && attr.value == id)
            {
                return Some(node);
            }
            dom.dom_children(node)
                .find_map(|child| visit(dom, child, id))
        }
        visit(dom, dom.document(), id).unwrap_or_else(|| panic!("missing specimen {id}"))
    })
}

/// Read a declaration that the retained specimen actually gives the renderer,
/// and require a nonempty laid-out box. A data attribute alone proves neither.
fn specimen_color(host: &Host, id: &str, property: &str) -> String {
    let node = specimen(host, id);
    let (_, _, width, height) = host.painted_rect(node).expect("specimen must paint");
    assert!(width > 0.0 && height > 0.0, "{id} has an empty painted box");
    attribute(host, node, "style")
        .split(';')
        .filter_map(|declaration| declaration.split_once(':'))
        .find(|(name, _)| name.trim() == property)
        .unwrap_or_else(|| panic!("{id} needs painted {property}"))
        .1
        .trim()
        .to_owned()
}

/// Reach a control entirely through native Tab traversal, without injecting
/// state or setting focus directly through the runner.
fn tab_to(host: &mut Host, selector: &Selector) {
    for _ in 0..100 {
        if host.focus() == Some(selected_node(host, selector)) {
            return;
        }
        host.tab(true);
    }
    panic!("native Tab cannot reach {selector:?}");
}

struct LibraryDir(PathBuf);

impl LibraryDir {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "tabard-surface-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&path).expect("create isolated library directory");
        Self(path)
    }

    fn library(&self) -> PathBuf {
        self.0.join("themes.json")
    }
}

impl Drop for LibraryDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn pointer_seed_edit_updates_the_specimen_and_history_without_activating_it() {
    let mut host = host(WorkshopState::in_memory());
    let original = host.state().draft_theme().clone();
    let active = host.state().registry().active_theme();
    let before = specimen_color(&host, "chrome-accent", "background");

    click(&mut host, &hue());

    let edited = host.state().draft_theme().clone();
    assert_ne!(edited.seeds.primary, original.seeds.primary);
    let after = specimen_color(&host, "chrome-accent", "background");
    assert_ne!(after, before, "the displayed accent must follow the seed");
    assert_eq!(host.state().registry().active_theme(), active);
    assert!(host.state().registry().theme_def(&edited.id).is_none());

    click(&mut host, &action("undo"));
    assert_eq!(host.state().draft_theme(), &original);
    assert_eq!(specimen_color(&host, "chrome-accent", "background"), before);

    click(&mut host, &action("redo"));
    assert_eq!(host.state().draft_theme(), &edited);
    assert_eq!(specimen_color(&host, "chrome-accent", "background"), after);

    click(&mut host, &action("discard"));
    assert_eq!(host.state().draft_theme(), &original);
    assert_eq!(specimen_color(&host, "chrome-accent", "background"), before);
    assert_eq!(host.state().registry().active_theme(), active);
}

#[test]
fn all_four_mode_controls_change_the_rendered_specimens() {
    let mut host = host(WorkshopState::in_memory());
    let original = host.state().draft_theme().clone();
    let active = host.state().registry().active_theme();
    let mut backgrounds = Vec::new();
    let mut syntax_backgrounds = Vec::new();

    for key in ["light", "dark", "hc_light", "hc_dark"] {
        click(&mut host, &mode(key));
        assert_eq!(host.state().mode_key(), key);
        backgrounds.push(specimen_color(&host, "reader-specimen", "background"));
        syntax_backgrounds.push(specimen_color(&host, "syntax-specimen", "background"));
        for id in [
            "chrome-specimen",
            "reader-specimen",
            "syntax-specimen",
            "graph-specimen",
        ] {
            let (_, _, width, height) = host.painted_rect(specimen(&host, id)).unwrap();
            assert!(
                width > 0.0 && height > 0.0,
                "{id} must remain laid out in {key}"
            );
        }
    }

    assert_ne!(
        backgrounds[0], backgrounds[1],
        "light and dark change colors"
    );
    assert_ne!(
        backgrounds[2], backgrounds[3],
        "both high contrast schemes render"
    );
    assert_ne!(
        backgrounds[0], backgrounds[2],
        "high contrast light changes the palette"
    );
    assert_ne!(
        backgrounds[1], backgrounds[3],
        "high contrast dark changes the palette"
    );
    assert_ne!(
        syntax_backgrounds[0], syntax_backgrounds[1],
        "syntax follows the light and dark schemes"
    );
    assert_ne!(
        syntax_backgrounds[2], syntax_backgrounds[3],
        "syntax follows both high contrast schemes"
    );
    assert_ne!(
        syntax_backgrounds[0], syntax_backgrounds[2],
        "high contrast light must change the actual syntax surface"
    );
    assert_ne!(
        syntax_backgrounds[1], syntax_backgrounds[3],
        "high contrast dark must change the actual syntax surface"
    );
    assert_eq!(
        host.state().draft_theme(),
        &original,
        "preview mode is not an authored edit"
    );
    assert_eq!(host.state().registry().active_theme(), active);
}

#[test]
fn save_and_reopen_keep_authored_data_and_protect_built_ins() {
    let directory = LibraryDir::new();
    let path = directory.library();
    let mut host = host(WorkshopState::load(&path).expect("open new library"));
    let built_ins: Vec<_> = host
        .state()
        .registry()
        .list()
        .into_iter()
        .filter(|theme| theme.source == ThemeSource::BuiltIn)
        .cloned()
        .collect();
    let active = host.state().registry().active_theme();
    let name = Selector::role("textbox").containing("Theme name");

    click(&mut host, &name);
    host.key_named(NamedKey::End);
    host.key_injected(" receipt");
    assert!(
        host.state().draft_theme().name.ends_with(" receipt"),
        "native text reaches the name field"
    );
    click(&mut host, &hue());
    let authored = host.state().draft_theme().clone();
    assert_eq!(authored.source, ThemeSource::User);
    assert!(!built_ins.iter().any(|theme| theme.id == authored.id));

    click(&mut host, &action("save"));
    assert!(
        !host.state().is_dirty(),
        "a successful save establishes the baseline"
    );
    assert!(path.is_file(), "saving must produce a real library file");
    assert_eq!(
        host.state().registry().theme_def(&authored.id),
        Some(&authored)
    );
    assert_eq!(host.state().registry().active_theme(), active);
    for built_in in &built_ins {
        assert_eq!(
            host.state().registry().theme_def(&built_in.id),
            Some(built_in)
        );
    }

    let reopened = WorkshopState::load(&path).expect("reopen the saved library");
    assert_eq!(reopened.registry().theme_def(&authored.id), Some(&authored));
    assert_eq!(reopened.draft_theme(), &authored);

    // A reload cannot erase unsaved work. The explicit discard makes the
    // subsequent reload safe and leaves the saved theme as the editing basis.
    tab_to(&mut host, &hue());
    host.key_named(NamedKey::ArrowRight);
    let unsaved = host.state().draft_theme().clone();
    assert_ne!(unsaved, authored);
    click(&mut host, &action("reopen"));
    assert_eq!(host.state().draft_theme(), &unsaved);
    assert!(host.state().is_dirty());
    click(&mut host, &action("discard"));
    click(&mut host, &action("reopen"));
    assert_eq!(host.state().draft_theme(), &authored);
    assert!(!host.state().is_dirty());

    click(&mut host, &action("new-copy"));
    let copy = host.state().draft_theme().clone();
    assert_ne!(copy.id, authored.id, "a new copy needs its own identity");
    assert_eq!(copy.source, ThemeSource::User);
    assert_eq!(copy.seeds, authored.seeds);
    assert!(host.state().registry().theme_def(&copy.id).is_none());
    assert_eq!(
        host.state().registry().theme_def(&authored.id),
        Some(&authored)
    );
    click(&mut host, &action("save"));
    let reopened = WorkshopState::load(&path).expect("reopen both saved user themes");
    assert_eq!(reopened.registry().theme_def(&copy.id), Some(&copy));
    assert_eq!(reopened.registry().theme_def(&authored.id), Some(&authored));
}

#[test]
fn a_failed_save_keeps_the_draft_and_does_not_register_a_saved_theme() {
    let directory = LibraryDir::new();
    let path = directory.library();
    let mut host = host(WorkshopState::load(&path).expect("open new library"));
    click(&mut host, &hue());
    let authored = host.state().draft_theme().clone();
    let active = host.state().registry().active_theme();
    let before_status = host.state().status().to_owned();
    // Make the destination unwritable after opening it. This deterministic
    // filesystem failure works even when the test process can bypass mode bits.
    std::fs::create_dir(&path).expect("block the library destination");

    click(&mut host, &action("save"));

    assert_eq!(host.state().draft_theme(), &authored);
    assert!(
        host.state().is_dirty(),
        "failed writes must leave save pending"
    );
    assert!(host.state().registry().theme_def(&authored.id).is_none());
    assert_eq!(host.state().registry().active_theme(), active);
    assert_ne!(
        host.state().status(),
        before_status,
        "surface must report the failed write"
    );
    assert!(
        path.is_dir(),
        "a failed save must preserve the existing destination"
    );
}

#[test]
fn locked_harmony_explains_dependent_hues_and_keeps_other_channels_editable() {
    let mut host = host(WorkshopState::in_memory());
    click(
        &mut host,
        &Selector::role("button").with_attr("data-harmony", "triadic"),
    );
    click(
        &mut host,
        &Selector::role("button").with_attr("data-seed", "secondary"),
    );

    assert!(host.with_dom(|dom| {
        taproot::matching(dom, &Selector::role("slider").containing("Secondary hue")).is_empty()
    }));
    assert!(host.with_surfaces(|surfaces| {
        taproot::text_present(surfaces, "Hue follows Primary in this harmony")
    }));
    let secondary = host.state().draft_theme().seeds.secondary;
    let primary = host.state().draft_theme().seeds.primary;
    let saturation = Selector::role("slider").containing("Secondary saturation");
    tab_to(&mut host, &saturation);
    host.key_named(NamedKey::ArrowRight);
    assert_ne!(host.state().draft_theme().seeds.secondary, secondary);
    assert_eq!(host.state().draft_theme().seeds.primary, primary);
    assert!(
        host.resolve(&Selector::role("slider").containing("Secondary lightness"))
            .is_some()
    );

    click(
        &mut host,
        &Selector::role("button").with_attr("data-harmony", "custom"),
    );
    let independent_hue = Selector::role("slider").containing("Secondary hue");
    assert!(host.resolve(&independent_hue).is_some());
    assert!(!host.with_surfaces(|surfaces| {
        taproot::text_present(surfaces, "Hue follows Primary in this harmony")
    }));
    let secondary = host.state().draft_theme().seeds.secondary;
    tab_to(&mut host, &independent_hue);
    host.key_named(NamedKey::ArrowRight);
    assert_ne!(host.state().draft_theme().seeds.secondary, secondary);
}

#[test]
fn pending_native_text_is_synchronized_before_controls_replace_or_guard_the_draft() {
    for control in ["seed", "theme", "new-copy", "reopen"] {
        let mut host = host(WorkshopState::in_memory());
        let original = host.state().draft_theme().clone();
        let active_id = host.state().registry().active_theme().resolved_id;
        // A native text adapter edits the public focused-text model before
        // the next retained action. The action owes that pending commit the
        // same preservation as text routed directly by the view.
        host.update(|state| {
            let text = state.text_field_mut("name").expect("embedding name slot");
            text.apply(TextCommand::SelectAll);
            text.apply(TextCommand::CommitComposition("Pending native name".into()));
        });
        assert_eq!(host.state().draft_theme(), &original);
        let selector = match control {
            "seed" => Selector::role("button").with_attr("data-seed", "secondary"),
            "theme" => Selector::role("button").with_attr("data-theme-id", active_id),
            "new-copy" | "reopen" => action(control),
            _ => unreachable!(),
        };

        click(&mut host, &selector);

        assert_eq!(
            host.state().draft_theme().name,
            "Pending native name",
            "{control} lost the text commit"
        );
        assert_eq!(
            host.state().draft_theme().id,
            original.id,
            "{control} replaced a draft with pending text"
        );
        assert!(host.state().has_changes());
        if control != "seed" {
            assert!(
                host.state().status().contains("Save or discard"),
                "{control} must explain its pending-edit guard"
            );
        }
    }
}

#[test]
fn native_keyboard_reaches_edits_and_can_undo_them() {
    let mut host = host(WorkshopState::in_memory());
    let original = host.state().draft_theme().clone();
    tab_to(&mut host, &hue());
    host.key_named(NamedKey::ArrowRight);
    assert_ne!(
        host.state().draft_theme().seeds.primary,
        original.seeds.primary
    );
    let edited = host.state().draft_theme().clone();

    tab_to(&mut host, &action("undo"));
    host.key_named(NamedKey::Enter);
    assert_eq!(host.state().draft_theme(), &original);
    tab_to(&mut host, &action("redo"));
    host.key_named(NamedKey::Space);
    assert_eq!(host.state().draft_theme(), &edited);

    tab_to(&mut host, &mode("dark"));
    host.key_named(NamedKey::Enter);
    assert_eq!(host.state().mode_key(), "dark");
}

#[test]
fn native_accessibility_announces_named_controls_and_their_hit_boxes() {
    let mut host = host(WorkshopState::in_memory());
    let (tree, map) = host.a11y_tree();
    for (name, role, selector) in [
        (
            "Theme name",
            Role::TextInput,
            Selector::role("textbox").containing("Theme name"),
        ),
        ("Primary hue", Role::Slider, hue()),
        (
            "Primary saturation",
            Role::Slider,
            Selector::role("slider").containing("Primary saturation"),
        ),
        (
            "Primary lightness",
            Role::Slider,
            Selector::role("slider").containing("Primary lightness"),
        ),
        ("Save theme", Role::Button, action("save")),
        ("New copy", Role::Button, action("new-copy")),
        ("Light", Role::Button, mode("light")),
        ("Dark", Role::Button, mode("dark")),
        ("High contrast light", Role::Button, mode("hc_light")),
        ("High contrast dark", Role::Button, mode("hc_dark")),
    ] {
        // Labels and text runs may share the control's name. Match the actual
        // interactive DOM target, then require its projected role and name.
        let expected_dom = selected_node(&host, &selector);
        let (id, control) = tree
            .nodes
            .iter()
            .find(|(id, _)| map.get(id) == Some(&expected_dom))
            .unwrap_or_else(|| panic!("missing announced control {name}"));
        assert_eq!(control.label(), Some(name), "announced name for {name}");
        assert_eq!(control.role(), role, "announced role for {name}");
        let bounds = control
            .bounds()
            .unwrap_or_else(|| panic!("{name} needs announced bounds"));
        let node = *map
            .get(id)
            .expect("an announced control must route back to the DOM");
        let painted = host
            .painted_rect(node)
            .expect("announced control has a hit box");
        assert_eq!(
            (
                bounds.x0 as f32,
                bounds.y0 as f32,
                (bounds.x1 - bounds.x0) as f32,
                (bounds.y1 - bounds.y0) as f32
            ),
            painted
        );
        assert!(
            painted.2 > 0.0 && painted.3 > 0.0,
            "{name} must have usable geometry"
        );
    }
}
