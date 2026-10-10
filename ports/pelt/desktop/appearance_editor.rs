// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The application-owned workshop window. Pelt supplies its existing render
//! core and routes window IDs; the shared host owns every native lifecycle,
//! caption, input, accessibility and presentation operation inside this window.

use cambium::{Key, KeyEvent, NamedKey};
use cambium_genet_winit_host::{
    AppCtx, CaptionLabels, CloseDisposition, HostHooks, HostOptions, WindowFrame, WinitHost,
};
use genet_winit_host::RenderCore;
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};
use tabard::theme::choice::ThemeChoice;
use tabard_workshop::{
    WorkshopState, WorkshopView,
    native_host::{WorkshopLogic as Logic, native_hooks},
    workshop_stylesheet,
};
use taproot::ProbeSnapshot;
use winit::event_loop::ActiveEventLoop;

type Context<'a> = AppCtx<'a, WorkshopState, Logic, WorkshopView>;
pub(crate) use tabard_workshop::native_host::{
    WorkshopHost as EditorHost, native_init as initialize,
};

pub(crate) struct EditorWindow {
    pub host: EditorHost,
    pub completion: Rc<Cell<Option<bool>>>,
}

pub(crate) fn hooks() -> HostHooks<WorkshopState, Logic, WorkshopView> {
    native_hooks(|artifact| {
        let extension = if artifact.format == tabard_workshop::ExportFormat::Css {
            "css"
        } else {
            "json"
        };
        cambium_genet_winit_host::choose_save_path(
            "Export theme",
            &artifact.suggested_name,
            &[extension],
        )
    })
}

impl EditorWindow {
    pub fn open(
        event_loop: &ActiveEventLoop,
        core: Arc<RenderCore>,
        library: PathBuf,
        appearance_store: Option<PathBuf>,
        choice: &ThemeChoice,
        wake: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<Self, String> {
        let mut state = WorkshopState::load(library).map_err(|error| error.to_string())?;
        protect_application_files(&mut state, appearance_store);
        // Resolve against the editor's freshly read library, including visible
        // fallback for missing identities/modes. Application selection remains
        // unchanged until the host's explicit saved-choice controls apply it.
        let resolved = tabard::resolve_theme_choice(state.registry(), choice)
            .map_err(|error| error.to_string())?;
        state.edit_definition(&resolved.theme, resolved.resolved.theme_mode)?;
        let completion = Rc::new(Cell::new(None));
        let lane = mesquite::LaneConfig::from_env("PELT_APPEARANCE")
            .map(|config| {
                mesquite::Lane::from_config(
                    config,
                    EditorLane {
                        completion: completion.clone(),
                        sheet: workshop_stylesheet(),
                    },
                    cambium_genet_winit_host::read_file,
                )
                .map(|lane| lane.with_frame_limit(Some(1800)))
            })
            .transpose()?;
        let lane = Rc::new(RefCell::new(lane));
        let close_lane = lane.clone();
        let product_hooks = hooks();
        let mut close = product_hooks.close_request;
        let hooks = HostHooks {
            after_frame: Box::new(move |ctx| {
                if let Some(lane) = lane.borrow_mut().as_mut() {
                    lane.after_frame(ctx);
                }
            }),
            close_request: Box::new(move |ctx, request| {
                if let Some(lane) = close_lane.borrow_mut().as_mut()
                    && !lane.finished()
                {
                    lane.request_close();
                    return CloseDisposition::KeepVisible;
                }
                close(ctx, request)
            }),
            ..product_hooks
        };
        let mut host = WinitHost::with_shared_render_core(
            HostOptions {
                title: "Pelt — Edit themes".into(),
                initial_logical_size: (1180.0, 800.0),
                window_frame: WindowFrame::App,
                maximize_control_label: CaptionLabels::default().maximize,
                ..Default::default()
            },
            move |_, commands, _| initialize(state, commands),
            hooks,
            core,
            wake,
        );
        host.open(event_loop)?;
        Ok(Self { host, completion })
    }
}

fn protect_application_files(state: &mut WorkshopState, appearance_store: Option<PathBuf>) {
    state.set_protected_export_paths(appearance_store.into_iter().collect());
}

fn focus_name(ctx: &Context<'_>) -> String {
    let Some(node) = ctx.runner.focus() else {
        return "none".into();
    };
    let dom = ctx.runner.dom();
    let dom = dom.borrow();
    for name in [
        "data-action",
        "data-mode",
        "data-field",
        "data-seed",
        "aria-label",
    ] {
        if let Some(value) = dom.attribute(node, &Namespace::from(""), &LocalName::from(name)) {
            return format!("{name}={value}");
        }
    }
    "other".into()
}

struct EditorLane {
    completion: Rc<Cell<Option<bool>>>,
    sheet: String,
}
impl mesquite::Product for EditorLane {
    type State = WorkshopState;
    type Logic = Logic;
    type View = WorkshopView;
    const KIND: &'static str = "pelt-appearance";
    const SURFACE: &'static str = "app";
    const LOG_PREFIX: &'static str = "pelt-appearance";
    fn sheet(&self) -> &str {
        &self.sheet
    }
    fn snapshot(&self, ctx: &Context<'_>, _: usize, _: f32) -> ProbeSnapshot {
        let state = ctx.runner.state();
        ProbeSnapshot::default()
            .with_field("name", state.draft_theme().name.clone())
            .with_field("mode", state.mode_key())
            .with_field("dirty", state.is_dirty().to_string())
            .with_field("theme", state.draft_theme().id.clone())
            .with_field("status", state.status())
            .with_field("focus", focus_name(ctx))
            .with_field("saved", state.saved_choice().is_ok().to_string())
    }
    fn busy_mut(&mut self, _: &mut Context<'_>, pending: bool) -> Option<bool> {
        Some(pending)
    }
    fn app_step(
        &mut self,
        ctx: &mut Context<'_>,
        _: mesquite::Checkpoints<'_>,
        line: &str,
    ) -> Result<(), String> {
        if let Some(text) = line.strip_prefix("text ") {
            ctx.runner
                .dispatch_key(KeyEvent::new(Key::Character(text.into())));
            ctx.runner.update(WorkshopState::sync_controls);
            return Ok(());
        }
        if let Some(target) = line.strip_prefix("key tab-until ") {
            for _ in 0..150 {
                if focus_name(ctx) == target.trim() {
                    return Ok(());
                }
                ctx.runner
                    .dispatch_key(KeyEvent::new(Key::Named(NamedKey::Tab)));
            }
            return Err(format!("Tab never reached {target}"));
        }
        Err(format!("unknown scenario verb: {line}"))
    }
    fn complete(&mut self, _: &mut Context<'_>, outcome: &taproot::Outcome) -> Result<(), String> {
        self.completion.set(Some(outcome.ok));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cambium_genet_winit_host::{CloseRequest, Harness};
    use taproot::Selector;

    #[test]
    fn editor_exports_preserve_the_application_choice_bytes_and_allow_other_destinations() {
        let temporary = tempfile::tempdir().unwrap();
        let selection = temporary.path().join("appearance.json");
        let original = "{\"theme_id\":\"theme:default\",\"theme_mode\":\"dark\"}";
        std::fs::write(&selection, original).unwrap();
        let mut state = WorkshopState::load(temporary.path().join("themes.json")).unwrap();
        protect_application_files(&mut state, Some(selection.clone()));
        std::fs::create_dir(temporary.path().join("nested")).unwrap();
        let mut destinations = vec![
            selection.clone(),
            temporary.path().join("nested/../appearance.json"),
        ];
        #[cfg(unix)]
        {
            let alias = temporary.path().join("appearance-alias.json");
            std::os::unix::fs::symlink(&selection, &alias).unwrap();
            destinations.push(alias);
        }
        for destination in destinations {
            state.request_export();
            let artifact = state.take_export().unwrap();
            state.complete_export(artifact, Some(destination));
            assert!(state.status().contains("protected application files"));
            assert!(state.replacement_path().is_none());
            assert_eq!(std::fs::read_to_string(&selection).unwrap(), original);
        }
        let exported = temporary.path().join("theme.json");
        state.request_export();
        let artifact = state.take_export().unwrap();
        state.complete_export(artifact, Some(exported.clone()));
        assert!(
            tabard::portable::parse_theme_json(&std::fs::read_to_string(&exported).unwrap())
                .is_ok()
        );
        std::fs::write(&exported, "previous export").unwrap();
        state.request_export();
        let artifact = state.take_export().unwrap();
        state.complete_export(artifact, Some(exported.clone()));
        assert!(state.replacement_path().is_some());
        state.replace_export();
        assert!(
            tabard::portable::parse_theme_json(&std::fs::read_to_string(exported).unwrap()).is_ok()
        );
        assert_eq!(std::fs::read_to_string(selection).unwrap(), original);
    }

    #[test]
    fn production_editor_text_and_close_guard_preserve_the_unsaved_draft() {
        let temporary = tempfile::tempdir().unwrap();
        let library = temporary.path().join("themes.json");
        let state = WorkshopState::load(&library).unwrap();
        let mut host = Harness::with_command_init(
            |commands| initialize(state, commands),
            hooks(),
            HostOptions::default(),
        );
        host.layout_at(1180.0, 800.0);
        assert!(host.click_on(&Selector::role("textbox").with_attr("data-field", "name")));
        host.key_injected(" Pelt author");
        assert!(host.state().draft_theme().name.contains("Pelt author"));
        host.request_close(CloseRequest::Native);
        host.relayout();
        assert!(!host.close_requested());
        assert!(host.state().close_requested());
        assert!(host.click_on(&Selector::role("button").with_attr("data-action", "cancel-close")));
        assert!(!host.state().close_requested());
        assert!(!host.close_requested());
        assert!(host.state().draft_theme().name.contains("Pelt author"));
        assert!(!library.exists());
        host.request_close(CloseRequest::Command);
        host.relayout();
        assert!(host.click_on(&Selector::role("button").with_attr("data-action", "save-close")));
        assert!(host.close_requested());
        assert!(library.exists());
        assert!(host.state().saved_choice().is_ok());
    }
}
