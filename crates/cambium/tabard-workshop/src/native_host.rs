// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Native workshop binding for apps that retain their own event loop and
//! existing renderer. The shared host owns previews, focused text, exports,
//! and the workshop's dirty-close decision. Apps own window routing, selection
//! persistence, and application exit policy.

use crate::{
    ExportArtifact, GRAPH_LEAF_KEY, PreviewScene, READER_LEAF_KEY, ReaderSpecimen,
    STYLESHEET_LEAF_KEY, StylesheetSpecimen, WorkshopState, WorkshopView, workshop_stylesheet,
    workshop_view_with_captions,
};
use cambium_genet_winit_host::{
    CaptionLabels, CloseDisposition, FocusedTextSlot, HostHooks, Init, SceneProducer, WinitHost,
    inert_hooks, platform_caption_controls,
};
use cambium_rootstock::{ProducerRole, ProducerSemantics};
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

pub type WorkshopLogic = Box<dyn FnMut(&WorkshopState) -> WorkshopView>;
pub type WorkshopHost = WinitHost<WorkshopState, WorkshopLogic, WorkshopView>;

fn scene_producer<T: PreviewScene>(source: Rc<RefCell<T>>, key: u64) -> SceneProducer<T> {
    SceneProducer::new(source, key, T::frame, T::revision, |source| {
        Some(ProducerSemantics {
            role: Some(ProducerRole::Image),
            name: Some(source.accessible_name().into()),
            children: Vec::new(),
        })
    })
}

/// Retained preview producers shared by standalone and embedded workshops.
/// Call `register` on every frame with the current workshop value. The binding
/// owns producer identity while the workshop owns preview source replacement.
/// Leaf keys are reserved by Tabard within the caller's host scope.
pub struct PreviewBindings {
    raster_base: u64,
    reader: Option<Rc<RefCell<SceneProducer<ReaderSpecimen>>>>,
    stylesheet: Option<Rc<RefCell<SceneProducer<StylesheetSpecimen>>>>,
}

// A render core can serve several independent workshop windows. Their
// retained raster outputs need distinct identities even when leaf keys and
// initial revision numbers match. Allocation happens once per binding.
static NEXT_RASTER_BASE: AtomicU64 = AtomicU64::new(0x7461_6200_0000_0000);

impl Default for PreviewBindings {
    fn default() -> Self {
        let raster_base = NEXT_RASTER_BASE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |key| {
                key.checked_add(2)
            })
            .expect("Tabard preview raster identity exhausted");
        Self {
            raster_base,
            reader: None,
            stylesheet: None,
        }
    }
}

impl PreviewBindings {
    pub fn register(
        &mut self,
        state: &WorkshopState,
        leaves: &mut sprigging::LeafRegistry<u64>,
        producers: &mut cambium_rootstock::ProducerRegistry,
    ) {
        leaves.insert(GRAPH_LEAF_KEY, Box::new(state.graph_leaf()));
        let reader_key = self.raster_base;
        let stylesheet_key = self.raster_base + 1;
        let source = state.reader_preview();
        let producer = self.reader.get_or_insert_with(|| {
            Rc::new(RefCell::new(scene_producer(source.clone(), reader_key)))
        });
        producer.borrow_mut().set_source(source);
        if !producers.contains(READER_LEAF_KEY) {
            producers
                .register(READER_LEAF_KEY, producer.clone(), &[])
                .expect("reserved Tabard reader key");
        }
        let source = state.stylesheet_preview();
        let producer = self.stylesheet.get_or_insert_with(|| {
            Rc::new(RefCell::new(scene_producer(source.clone(), stylesheet_key)))
        });
        producer.borrow_mut().set_source(source);
        if !producers.contains(STYLESHEET_LEAF_KEY) {
            producers
                .register(STYLESHEET_LEAF_KEY, producer.clone(), &[])
                .expect("reserved Tabard stylesheet key");
        }
    }
}

/// Synchronize committed controls and process an explicit export using the
/// host's chooser. Returns the workshop's own exit request; the parent decides
/// whether that closes a tool window, an embedded pane, or the application.
pub fn sync_and_export(
    state: &mut WorkshopState,
    choose_export: &mut impl FnMut(&ExportArtifact) -> Option<PathBuf>,
) -> bool {
    state.sync_controls();
    if let Some(artifact) = state.take_export() {
        let destination = choose_export(&artifact);
        state.complete_export(artifact, destination);
    }
    state.exit_requested()
}

/// Find an actual workshop-controlled field for a native focused-text adapter.
/// Parent-state hosts use the returned key in their own projection closures.
pub fn focused_field(
    dom: &cambium::DomHandle,
    node: genet_scripted_dom::NodeId,
    state: &WorkshopState,
) -> Option<String> {
    let field = dom
        .borrow()
        .attribute(node, &Namespace::from(""), &LocalName::from("data-field"))?
        .to_string();
    state.text_field(&field)?;
    Some(field)
}

/// Bind the workshop to a native host. A host-supplied save dialog is called
/// only for explicit export effects; cancellation returns `None`.
pub fn native_hooks(
    mut choose_export: impl FnMut(&ExportArtifact) -> Option<PathBuf> + 'static,
) -> HostHooks<WorkshopState, WorkshopLogic, WorkshopView> {
    let mut previews = PreviewBindings::default();
    HostHooks {
        frame: Box::new(move |ctx| {
            previews.register(ctx.runner.state(), ctx.leaves, ctx.producers);
            false
        }),
        after_dispatch: Box::new(move |ctx| {
            let mut exit = false;
            ctx.runner
                .update(|state| exit = sync_and_export(state, &mut choose_export));
            if exit {
                *ctx.close = true;
            }
        }),
        close_request: Box::new(|ctx, _| {
            let mut allow = false;
            ctx.runner.update(|state| allow = state.request_close());
            if allow {
                CloseDisposition::Exit
            } else {
                CloseDisposition::KeepVisible
            }
        }),
        focused_text: Box::new(|runner| {
            let node = runner.focus()?;
            let field = focused_field(&runner.dom(), node, runner.state())?;
            let mutable = field.clone();
            Some(FocusedTextSlot {
                node,
                get: Box::new(move |state| state.text_field(&field).expect("registered field")),
                get_mut: Box::new(move |state| {
                    state.text_field_mut(&mutable).expect("registered field")
                }),
            })
        }),
        ..inert_hooks()
    }
}

/// Initialize the shared workshop frame with the native host's caption
/// controls. Window creation and render-core selection stay with the caller.
pub fn native_init(
    state: WorkshopState,
    commands: &cambium_genet_winit_host::WindowCommands,
) -> Init<WorkshopState, WorkshopLogic> {
    let commands = commands.clone();
    Init {
        state,
        logic: Box::new(move |state| {
            workshop_view_with_captions(
                state,
                platform_caption_controls(&commands, &CaptionLabels::default()),
            )
        }),
        sheet: workshop_stylesheet(),
        fonts: Vec::new(),
        images: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cambium_genet_winit_host::{CloseRequest, Harness, WindowCommands};
    use tabard::theme::registry::{Mode, THEME_ID_DEFAULT, ThemeRegistry};
    use taproot::Selector;

    fn harness(
        state: WorkshopState,
        choose: impl FnMut(&ExportArtifact) -> Option<PathBuf> + 'static,
    ) -> Harness<WorkshopState, WorkshopLogic, WorkshopView> {
        let mut host = Harness::with_hooks(
            native_init(state, &WindowCommands::new()),
            native_hooks(choose),
        );
        host.layout_at(1280.0, 1100.0);
        host
    }

    #[test]
    fn preview_bindings_retain_identity_and_rebind_reopened_workshop_sources() {
        let mut bindings = PreviewBindings::default();
        let independent = PreviewBindings::default();
        assert_ne!(bindings.raster_base, independent.raster_base);
        assert_ne!(
            bindings.raster_base + 1,
            independent.raster_base,
            "sibling workshop scenes cannot alias the shared render cache"
        );
        let mut leaves = sprigging::LeafRegistry::new();
        let mut producers = cambium_rootstock::ProducerRegistry::new();
        let first = WorkshopState::in_memory();
        bindings.register(&first, &mut leaves, &mut producers);
        let reader = bindings.reader.clone().unwrap();
        let stylesheet = bindings.stylesheet.clone().unwrap();
        assert_eq!(
            producers.semantics(READER_LEAF_KEY).unwrap().role,
            Some(ProducerRole::Image)
        );
        assert_eq!(
            producers.semantics(STYLESHEET_LEAF_KEY).unwrap().role,
            Some(ProducerRole::Image)
        );
        let mut reopened = WorkshopState::in_memory();
        reopened.set_mode(Mode::HcLight);
        assert!(!Rc::ptr_eq(
            &first.reader_preview(),
            &reopened.reader_preview()
        ));
        bindings.register(&reopened, &mut leaves, &mut producers);
        assert!(Rc::ptr_eq(&reader, bindings.reader.as_ref().unwrap()));
        assert!(Rc::ptr_eq(
            &stylesheet,
            bindings.stylesheet.as_ref().unwrap()
        ));
        assert!(
            !reader.borrow_mut().set_source(reopened.reader_preview()),
            "already bound to reopened source"
        );
        assert!(
            !stylesheet
                .borrow_mut()
                .set_source(reopened.stylesheet_preview()),
            "already bound to reopened source"
        );
        assert_eq!(
            producers
                .semantics(READER_LEAF_KEY)
                .unwrap()
                .name
                .as_deref(),
            Some(reopened.reader_preview().borrow().accessible_name())
        );
        assert!(producers.remove(READER_LEAF_KEY));
        bindings.register(&reopened, &mut leaves, &mut producers);
        assert!(
            producers.contains(READER_LEAF_KEY),
            "retired registrations can be installed again"
        );
    }

    #[test]
    fn native_hooks_deliver_controlled_text_to_the_actual_workshop_field() {
        let mut host = harness(WorkshopState::in_memory(), |_| {
            panic!("text editing must not open an export chooser")
        });
        let selector = Selector::role("textbox").with_attr("data-field", "name");
        let node = host.with_dom(|dom| taproot::matching(dom, &selector)[0]);
        assert_eq!(
            focused_field(&host.runner().dom(), node, host.state()).as_deref(),
            Some("name")
        );
        assert!(host.click_on(&selector));
        host.key_injected("Ω");
        assert!(
            host.state()
                .text_field("name")
                .unwrap()
                .text()
                .contains('Ω')
        );
        let export = host.with_dom(|dom| {
            taproot::matching(
                dom,
                &Selector::role("button").with_attr("data-action", "export"),
            )[0]
        });
        assert!(focused_field(&host.runner().dom(), export, host.state()).is_none());
    }

    #[test]
    fn native_export_effect_uses_the_host_destination_once_and_keeps_the_snapshot() {
        let temporary = tempfile::tempdir().unwrap();
        let destination = temporary.path().join("snapshot.theme.json");
        let chosen = destination.clone();
        let calls = Rc::new(std::cell::Cell::new(0));
        let observed = calls.clone();
        let mut host = harness(WorkshopState::in_memory(), move |_| {
            observed.set(observed.get() + 1);
            Some(chosen.clone())
        });
        assert_eq!(calls.get(), 0);
        let original = host.state().draft_theme().clone();
        assert!(host.click_on(&Selector::role("button").with_attr("data-action", "export")));
        assert_eq!(calls.get(), 1);
        let exported =
            tabard::portable::parse_theme_json(&std::fs::read_to_string(&destination).unwrap())
                .unwrap();
        assert_eq!(exported, original);
        host.key_named(winit::keyboard::NamedKey::Tab);
        assert_eq!(
            calls.get(),
            1,
            "subsequent dispatches cannot repeat a consumed export"
        );
    }

    #[test]
    fn native_close_uses_the_shared_dirty_save_discard_cancel_policy() {
        let temporary = tempfile::tempdir().unwrap();
        let library = temporary.path().join("themes.json");
        let mut state = WorkshopState::load(&library).unwrap();
        state
            .edit_definition(
                ThemeRegistry::default()
                    .theme_def(THEME_ID_DEFAULT)
                    .unwrap(),
                None,
            )
            .unwrap();
        let mut host = harness(state, |_| None);
        host.request_close(CloseRequest::Native);
        assert!(!host.close_requested());
        assert!(host.state().close_requested());
        host.relayout();
        assert!(host.click_on(&Selector::role("button").with_attr("data-action", "cancel-close")));
        assert!(!host.close_requested());
        assert!(!host.state().close_requested());
        assert!(!library.exists());
        host.request_close(CloseRequest::Native);
        host.relayout();
        assert!(host.click_on(&Selector::role("button").with_attr("data-action", "save-close")));
        assert!(host.close_requested());
        assert!(host.state().saved_choice().is_ok());
        assert_eq!(
            tabard::library::ThemeLibraryStore::load(library)
                .unwrap()
                .themes(),
            &[host.state().draft_theme().clone()]
        );
    }
}
