// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Ruling AC7's windowless mount probe (app composition brief, section 12).
//!
//! A forest mount's boundary is a shadow root: a `RetainedMount` owner sits
//! inside an open shadow root, and the guest's sheet is a `<style>` element in
//! that root. This module asks whether Genet's cascade, hit testing and
//! accessibility projection hold that boundary over a real Cambium runner,
//! with no window.
//!
//! The probe runs its own style pipeline, not `OwnedLayout::new`. `OwnedLayout`
//! builds its `StyleSet` with `StyleSet::cambium(sheets)`, which gives every
//! sheet `owner_node: None`, so every sheet is document-scope and the `<style>`
//! elements a DOM carries are never read. Genet's tree-scope boundary keys on
//! the owner node, so the production host path cannot scope a guest sheet yet.
//! Here the sheets come from `ResolvedDocumentResources::discover` and
//! `StyleSet::cambium_resources`, as Genet's own shadow tests build them.
//! Layout, hit test and projection then follow `OwnedLayout`'s calls.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cambium::{
    AnyView, GenetAppRunner, GenetCtx, GenetElement, RetainedMount, button, el, html_qual,
};
use document_session_api::DocumentA11yRole;
use genet_document_resources::ResolvedDocumentResources;
use genet_livery::{
    Device, InteractionStates, StyleSet, TextSystem, ViewportSizes, hit_test_with_scroll,
    layout_with_text_system, resolve_styles,
};
use genet_render::{A11yStyleQueries, document_a11y_projection_with_style};
use layout_dom_api::{
    LayoutDom as _, LayoutDomMut as _, LocalName, Namespace, ShadowRootInit, ShadowRootMode,
};

use crate::{NodeId, ScriptedDom, WindowDom};

const WIDTH: f32 = 400.0;
const HEIGHT: f32 = 300.0;

const GREEN: &str = "rgb(0, 128, 0)";
const BLUE: &str = "rgb(0, 0, 255)";

/// The document's sheet. `#host` sets the colour the guest must inherit.
/// `button, .note` is the rule that must not cross into the guest.
const DOCUMENT_SHEET: &str = "#host { display:block; width:300px; height:100px; \
     color: rgb(0, 0, 255); } \
     button, .note { font-style: italic; }";

/// The guest's sheet. It must style the guest and nothing outside it.
const GUEST_SHEET: &str = "button { display:block; width:120px; height:30px; \
     color: rgb(0, 128, 0); font-weight: bold; }";

type Child = Box<dyn AnyView<(), (), GenetCtx, GenetElement>>;
type Logic = fn(&()) -> Child;

fn guest_view(_: &()) -> Child {
    Box::new(el(
        "section",
        (
            button("Guest action", |_: &mut (), _| {}),
            el("span", "Guest note".to_string()).attr("class", "note"),
        ),
    ))
}

/// Where the mount sits.
#[derive(Clone, Copy, Debug)]
enum Boundary {
    /// The brief's probe shape: the owner is a child of a separate host's
    /// shadow root, and the runner mounts at the owner.
    ShadowRoot,
    /// AC7(a)'s wording: the owner is the shadow host, in the host's light
    /// DOM, and the runner mounts at its shadow root.
    OwnerIsHost,
    /// Control: the owner is the host's child, as `RetainedMount` is today.
    PlainDiv,
}

/// Where the guest's `<style>` element goes.
#[derive(Clone, Copy, Debug)]
enum GuestSheet {
    /// Beside the owner, in the same tree as the mount.
    WithMount,
    /// Control: in the document, outside the shadow root.
    InDocument,
}

struct World {
    dom: Rc<RefCell<ScriptedDom>>,
    // Held so the owner container and the runner outlive the analysis.
    mount: RetainedMount,
    host: NodeId,
    _runner: GenetAppRunner<(), Logic, Child>,
    guest_button: NodeId,
    guest_note: NodeId,
    outside_button: NodeId,
    outside_note: NodeId,
}

fn id_of(dom: &ScriptedDom, node: NodeId) -> Option<&str> {
    dom.attribute(node, &Namespace::default(), &LocalName::from("id"))
}

fn find_id(dom: &ScriptedDom, node: NodeId, wanted: &str) -> Option<NodeId> {
    if id_of(dom, node) == Some(wanted) {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find_id(dom, child, wanted))
}

fn style_element(dom: &mut ScriptedDom, css: &str) -> NodeId {
    let style = dom.create_element(html_qual("style"));
    let text = dom.create_text(css);
    dom.append_child(style, text);
    style
}

fn build(boundary: Boundary, guest_sheet: GuestSheet) -> World {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let root = dom.borrow().document();
    dom.borrow_mut().set_inner_html(
        root,
        &format!(
            "<style>{DOCUMENT_SHEET}</style>\
             <button id=\"outside-button\">Outside action</button>\
             <span id=\"outside-note\" class=\"note\">Outside note</span>\
             <div id=\"host\"></div>"
        ),
    );
    let host = find_id(&dom.borrow(), root, "host").expect("host");
    let mount = RetainedMount::new(dom.clone());
    let attach = |dom: &Rc<RefCell<ScriptedDom>>, element: NodeId| {
        dom.borrow_mut()
            .attach_shadow(
                element,
                ShadowRootInit {
                    mode: ShadowRootMode::Open,
                    ..Default::default()
                },
            )
            .expect("an open shadow root attaches to a div")
    };
    // `container` holds the guest's `<style>`; `mount_at` is where the runner
    // builds. `host_child` is the light-DOM child of `#host`, when there is one.
    let (container, mount_at) = match boundary {
        Boundary::ShadowRoot => {
            let root = attach(&dom, host);
            dom.borrow_mut().insert_before(root, mount.node(), None);
            (root, mount.node())
        },
        Boundary::OwnerIsHost => {
            dom.borrow_mut().insert_before(host, mount.node(), None);
            let root = attach(&dom, mount.node());
            (root, root)
        },
        Boundary::PlainDiv => {
            dom.borrow_mut().insert_before(host, mount.node(), None);
            (host, mount.node())
        },
    };
    {
        let mut document = dom.borrow_mut();
        let style = style_element(&mut document, GUEST_SHEET);
        match guest_sheet {
            // Before the guest's content, as a `<style>` precedes it in a page.
            GuestSheet::WithMount => {
                let first = document.dom_children(container).next();
                document.insert_before(container, style, first);
            },
            GuestSheet::InDocument => document.append_child(root, style),
        }
    }
    let runner: GenetAppRunner<(), Logic, Child> =
        GenetAppRunner::new_at(dom.clone(), mount_at, guest_view as Logic, ());
    let (guest_button, guest_note) = {
        let document = dom.borrow();
        let section = runner.root();
        let mut children = document.dom_children(section);
        (
            children.next().expect("guest button"),
            children.next().expect("guest note"),
        )
    };
    let (outside_button, outside_note) = {
        let document = dom.borrow();
        (
            find_id(&document, root, "outside-button").expect("outside button"),
            find_id(&document, root, "outside-note").expect("outside note"),
        )
    };
    World {
        dom,
        mount,
        host,
        _runner: runner,
        guest_button,
        guest_note,
        outside_button,
        outside_note,
    }
}

/// What each done-condition found, with the values behind it.
#[derive(Debug)]
struct Report {
    /// The guest's rule applies inside.
    inside: bool,
    /// The document's rule does not cross in.
    no_cross_in: bool,
    /// Inheritance still crosses in.
    inheritance: bool,
    /// The guest's rule does not leak out.
    no_leak_out: bool,
    /// A hit test inside the guest's rectangle returns the guest's node.
    hit: bool,
    /// The guest's button is in the AccessKit-bound projection.
    a11y: bool,
    values: String,
}

fn analyse(world: &World) -> Report {
    let dom = world.dom.borrow();
    let window = WindowDom::document(&dom);
    let resources = ResolvedDocumentResources::discover(&window, None).stylesheets;
    let style_set = StyleSet::cambium_resources(&resources);
    let device = Device::screen(WIDTH, HEIGHT);
    let styles = resolve_styles(&window, &style_set, &device, &InteractionStates::default());
    let mut text = TextSystem::new();
    let images = HashMap::new();
    let (laid_out, fragments) = layout_with_text_system(
        &window,
        &styles,
        WIDTH,
        HEIGHT,
        ViewportSizes::uniform(WIDTH, HEIGHT),
        &mut text,
        &images,
    )
    .expect("the probe's layout resolves");

    let style =
        |node: NodeId, property: &str| styles.computed_style(node, property).unwrap_or_default();
    let button = world.guest_button;
    let note = world.guest_note;

    let hit = fragments.get(button).is_some_and(|rect| {
        let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
        hit_test_with_scroll(&window, &laid_out, &fragments, &HashMap::new(), x, y) == Some(button)
    });

    let projection = document_a11y_projection_with_style(
        &window,
        &fragments,
        None,
        0,
        None,
        &A11yStyleQueries {
            generated: &|node| genet_livery::rendered_generated_text(&window, &laid_out, node),
            rendered: &|node| genet_livery::rendered_visible(&window, &laid_out, node),
        },
    );
    let guest_id = window.opaque_id(button);
    let a11y = projection
        .nodes()
        .iter()
        .any(|node| node.role == DocumentA11yRole::Button && node.id.get() == guest_id);

    let values = format!(
        "guest button: color={} weight={} style={} | guest note: color={} style={} | \
         outside button: color={} weight={} style={} | outside note: style={} | \
         hit={:?} projected buttons={:?}",
        style(button, "color"),
        style(button, "font-weight"),
        style(button, "font-style"),
        style(note, "color"),
        style(note, "font-style"),
        style(world.outside_button, "color"),
        style(world.outside_button, "font-weight"),
        style(world.outside_button, "font-style"),
        style(world.outside_note, "font-style"),
        fragments.get(button).map(|r| (r.x, r.y, r.width, r.height)),
        projection
            .nodes()
            .iter()
            .filter(|node| node.role == DocumentA11yRole::Button)
            .map(|node| node.name.clone().unwrap_or_default())
            .collect::<Vec<_>>(),
    );

    // The document's own rule must demonstrably work, or "does not cross in"
    // proves nothing.
    assert_eq!(
        style(world.outside_note, "font-style"),
        "italic",
        "the document sheet does not style the document: {values}"
    );

    Report {
        inside: style(button, "color") == GREEN,
        no_cross_in: style(button, "font-style") == "normal"
            && style(note, "font-style") == "normal",
        inheritance: style(note, "color") == BLUE,
        no_leak_out: style(world.outside_button, "color") != GREEN,
        hit,
        a11y,
        values,
    }
}

fn assert_scoped(world: &World, label: &str) {
    assert!(
        world.dom.borrow().has_shadow_trees(),
        "the host has no shadow root"
    );
    let report = analyse(world);
    eprintln!("AC7 probe, {label}: {report:#?}");
    assert!(report.inside, "the guest's rule must apply inside");
    assert!(
        report.no_cross_in,
        "the document's rule must not cross in: {}",
        report.values
    );
    assert!(
        report.inheritance,
        "inheritance must still cross in: {}",
        report.values
    );
    assert!(
        report.no_leak_out,
        "the guest's rule must not leak out: {}",
        report.values
    );
    assert!(
        report.hit,
        "a hit inside the guest's rectangle must return the guest's node: {}",
        report.values
    );
    assert!(
        report.a11y,
        "the guest's button must reach the projection: {}",
        report.values
    );
}

/// The brief's probe shape: a host element with an open shadow root holding the
/// `<style>` and the `RetainedMount` owner.
#[test]
fn a_shadow_root_mount_scopes_the_guest_and_stays_hittable_and_accessible() {
    assert_scoped(
        &build(Boundary::ShadowRoot, GuestSheet::WithMount),
        "owner inside a host's shadow root",
    );
}

/// AC7(a)'s wording: the owner itself is the shadow host and the runner mounts
/// at its shadow root.
#[test]
fn an_owner_that_is_the_shadow_host_scopes_the_guest_the_same_way() {
    assert_scoped(
        &build(Boundary::OwnerIsHost, GuestSheet::WithMount),
        "owner is the shadow host",
    );
}

/// `RetainedMount::withdraw_from` finds its source by walking `parent()`. A
/// shadow root has no parent, so the walk from inside a shadow tree stops at
/// the root and never reaches the host. The shape where the owner is the host
/// keeps the owner in the light DOM, where the walk still works.
#[test]
fn withdrawal_walks_parents_so_it_needs_the_owner_outside_the_shadow_root() {
    let inside = build(Boundary::ShadowRoot, GuestSheet::WithMount);
    let withdrew_from_inside = inside.mount.withdraw_from(inside.host);
    let host_shape = build(Boundary::OwnerIsHost, GuestSheet::WithMount);
    let withdrew_owner_is_host = host_shape.mount.withdraw_from(host_shape.host);
    eprintln!(
        "AC7 probe, withdraw_from(host): owner inside shadow root = {withdrew_from_inside},          owner is the host = {withdrew_owner_is_host}"
    );
    assert!(
        !withdrew_from_inside,
        "withdrawal reached across a shadow root, which has no parent"
    );
    assert!(
        withdrew_owner_is_host,
        "withdrawal must still work when the owner is the host"
    );
}

/// Control: with a plain `div` as the boundary, the host's rule reaches the
/// guest. The guest's sheet is document-scope there too, so it also leaks out.
#[test]
fn control_a_plain_div_mount_lets_the_document_rule_in() {
    let world = build(Boundary::PlainDiv, GuestSheet::WithMount);
    let report = analyse(&world);
    eprintln!("AC7 probe, plain-div control: {report:#?}");
    assert!(
        !report.no_cross_in,
        "a plain div must let the document's rule in: {}",
        report.values
    );
    assert!(
        !report.no_leak_out,
        "a plain div must let the guest's rule out: {}",
        report.values
    );
}

/// Control: the guest's `<style>` placed outside the shadow root belongs to
/// the document scope. It leaks out, and cannot reach in.
#[test]
fn control_a_guest_style_outside_the_shadow_root_leaks_out() {
    let world = build(Boundary::ShadowRoot, GuestSheet::InDocument);
    let report = analyse(&world);
    eprintln!("AC7 probe, style-outside control: {report:#?}");
    assert!(
        !report.no_leak_out,
        "a guest sheet outside the shadow root must leak out: {}",
        report.values
    );
    assert!(
        !report.inside,
        "a document-scope guest sheet cannot cross into the shadow root: {}",
        report.values
    );
}
