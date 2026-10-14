// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The bounded local saved-graph workflow on the retained tree.
use super::*;
use cambium::{FileEvent, SelectState, TextInput};
use graphshell::local_edit::{NodeMetadata, sync_canvas_metadata_from_graph};
use graphshell::local_intake::{file_metadata, sync_canvas_intake};
use graphshell::resident_host::{BrowserPersistencePhase, ResidentHostError};
use graphshell::web_resident::{
    RefreshOutcome, WebPersistenceWatch, WebRefreshCompletion, WebResidentApp, WebWorkbench,
};
use mere::canvas::Role;
use uuid::Uuid;

type App = WebResidentApp;
#[derive(Clone, Copy)]
enum CompletedEdit {
    Metadata,
    Intake,
}
type Completion = (App, CompletedEdit, Result<NodeMetadata, String>);

pub(super) struct SavedProduct {
    app: Option<App>,
    progress: Option<WebPersistenceWatch>,
    progress_revision: u64,
    published_coverage: String,
    controls_done: Rc<RefCell<Option<(App, Result<(), String>)>>>,
    // Automatic checks own only their typed read request. Draft fields and the
    // admitted App stay here until an explicit operation takes ownership.
    read_done: Rc<RefCell<Option<WebRefreshCompletion>>>,
    read_pending: bool,
    read_graph: Option<Graph>,
    read_error: Option<String>,
    inspection: Option<graphshell::web_resident::GraphAddress>,
    policy: SelectState,
    projection: SelectState,
    refresh: SelectState,
    stratum: SelectState,
    settings_fields: [TextInput; 8],

    completed: Rc<RefCell<Option<Completion>>>,
    pub(super) selected: Option<Uuid>,
    pub(super) title: TextInput,
    pub(super) tags: TextInput,
    pub(super) address: String,
    pub(super) detail_open: bool,
    /// The picked item's own role: index 0 is "as recipe" (no override),
    /// then each role (F48).
    pub(super) role: SelectState,
    pub(super) saving: bool,
    /// Save feedback for the detail editor; empty until a save.
    pub(super) status: String,
    /// The store's open and persistence state, shown in Graph tools.
    pub(super) storage: String,
    pub(super) save_state: &'static str,
    pub(super) session: String,
    pub(super) reopened: bool,
    intake_open: bool,
    intake_address: TextInput,
    intake_title: TextInput,
    file_requested: bool,
    /// A created but unacknowledged member. Retrying stores this same object.
    pending_intake: Option<Uuid>,
}

/// The existing fixture and generated comparison routes remain unchanged.
/// `?app=local` is the opt-in migration route onto the existing browser store.
pub(super) async fn open() -> Result<Option<SavedProduct>, String> {
    let search = super::super::window()?
        .location()
        .search()
        .map_err(|_| "cannot read page options")?;
    let params =
        web_sys::UrlSearchParams::new_with_str(&search).map_err(|_| "invalid page options")?;
    if params.get("app").as_deref() != Some("local") {
        return Ok(None);
    }
    if web_graphs::requested().is_some() {
        return Err("app=local cannot also select a generated graph".into());
    }
    let root = super::super::document()?
        .get_element_by_id("graphshell")
        .ok_or("missing tree root")?;
    let opening = super::super::opening_notice(&root)?;
    let mut app = WebResidentApp::open(SelectedPersonaRef {
        persona: FIXTURE_PERSONA_ADDRESS.into(),
        profile: "profile:graphshell-h3".into(),
    })
    .await
    .map_err(|error| error.to_string())?;
    let reopened = app.host.was_reopened();
    app.finish_startup(now_secs())
        .await
        .map_err(|error| error.to_string())?;
    opening.remove();
    app.mount_local().map_err(|error| error.to_string())?;
    let persistence = super::super::resolve_storage_persistence().await;
    let storage = graphshell::browser_storage::status_line(
        if reopened {
            "IndexedDB reopened"
        } else {
            "IndexedDB seeded"
        },
        &persistence,
    );
    let session = app.host.session_id().0.to_string();
    let settings = app.host.resident_settings().clone();
    let choices = app.host.projection_choices();
    let projection_index = settings
        .projection
        .as_ref()
        .and_then(|id| choices.iter().position(|(key, _)| key == id))
        .map_or(0, |i| i + 1);
    let (partial, hops) = match settings.policy {
        graphshell::web_resident::ResidencyPolicy::Full => (false, 0),
        graphshell::web_resident::ResidencyPolicy::Neighborhood { hops } => (true, hops),
    };
    let minimum = match settings.refresh {
        graphshell::resident_host::ResidentRefreshMode::Manual => 1000,
        graphshell::resident_host::ResidentRefreshMode::HostDriven { minimum_check_ms } => {
            minimum_check_ms
        },
    };
    Ok(Some(SavedProduct {
        controls_done: Rc::new(RefCell::new(None)),
        read_done: Rc::new(RefCell::new(None)),
        read_pending: false,
        read_graph: None,
        read_error: None,
        inspection: None,
        policy: SelectState::new(usize::from(partial)).with_label("Display"),
        projection: SelectState::new(projection_index).with_label("Authored projection"),
        refresh: SelectState::new(usize::from(matches!(
            settings.refresh,
            graphshell::resident_host::ResidentRefreshMode::HostDriven { .. }
        )))
        .with_label("Refresh mode"),
        stratum: SelectState::new(0).with_label("Inspect stratum"),
        settings_fields: [
            TextInput::new(hops.to_string()),
            TextInput::new(minimum.to_string()),
            TextInput::new(settings.publication_batch_ms.to_string()),
            TextInput::new(""),
            TextInput::new(""),
            TextInput::new(""),
            TextInput::new("{}"),
            TextInput::new(""),
        ],
        app: Some(app),
        progress: None,
        progress_revision: 0,
        published_coverage: String::new(),
        completed: Rc::new(RefCell::new(None)),
        selected: None,
        title: TextInput::new(""),
        tags: TextInput::new(""),
        address: String::new(),
        detail_open: false,
        role: SelectState::new(0).with_label("Item role"),
        saving: false,
        status: String::new(),
        storage,
        save_state: "idle",
        session,
        reopened,
        intake_open: false,
        intake_address: TextInput::new(""),
        intake_title: TextInput::new(""),
        file_requested: false,
        pending_intake: None,
    }))
}

fn now_secs() -> u64 {
    (js_sys::Date::now() / 1_000.0) as u64
}

#[derive(Clone, Copy)]
enum TextField {
    Title,
    Tags,
    IntakeAddress,
    IntakeTitle,
    Settings(usize),
}

impl TextField {
    fn get(self, product: &SavedProduct) -> &TextInput {
        match self {
            Self::Title => &product.title,
            Self::Tags => &product.tags,
            Self::IntakeAddress => &product.intake_address,
            Self::IntakeTitle => &product.intake_title,
            Self::Settings(index) => &product.settings_fields[index],
        }
    }
    fn get_mut(self, product: &mut SavedProduct) -> &mut TextInput {
        match self {
            Self::Title => &mut product.title,
            Self::Tags => &mut product.tags,
            Self::IntakeAddress => &mut product.intake_address,
            Self::IntakeTitle => &mut product.intake_title,
            Self::Settings(index) => &mut product.settings_fields[index],
        }
    }
}

/// Map the retained fields to the host's caret, IME and selection path.
pub(super) fn focused_text(
    runner: &cambium_rootstock::Runner<TreePage, Logic, Child>,
) -> Option<cambium_rootstock::FocusedTextSlot<TreePage>> {
    let product = runner.state().product.as_ref()?;
    if product.saving || product.pending_intake.is_some() {
        return None;
    }
    let node = runner.focus()?;
    let dom = runner.dom();
    let dom = dom.borrow();
    let field = [
        ("Title", TextField::Title, product.detail_open),
        ("Tags", TextField::Tags, product.detail_open),
        ("Hops", TextField::Settings(0), true),
        ("Minimum check interval (ms)", TextField::Settings(1), true),
        (
            "Publication batch interval (ms)",
            TextField::Settings(2),
            true,
        ),
        (
            "Exact roster (comma-separated UUIDs)",
            TextField::Settings(3),
            true,
        ),
        ("Inspect stable ID", TextField::Settings(4), true),
        ("Facet ID", TextField::Settings(5), true),
        ("Facet JSON", TextField::Settings(6), true),
        ("Cites target UUID", TextField::Settings(7), true),
        ("Address", TextField::IntakeAddress, product.intake_open),
        (
            "New object title",
            TextField::IntakeTitle,
            product.intake_open,
        ),
    ]
    .into_iter()
    .find_map(|(label, field, visible)| {
        (visible
            && taproot::matching(&dom, &Selector::role("textbox").containing(label))
                .contains(&node))
        .then_some(field)
    })?;
    Some(cambium_rootstock::FocusedTextSlot {
        node,
        get: Box::new(move |page: &TreePage| {
            let product = page.product.as_ref().expect("focused local detail");
            field.get(product)
        }),
        get_mut: Box::new(move |page: &mut TreePage| {
            let product = page.product.as_mut().expect("focused local detail");
            field.get_mut(product)
        }),
    })
}

impl SavedProduct {
    pub(super) fn workbench(&self) -> Option<WebWorkbench> {
        Some(self.app.as_ref()?.workbench())
    }
    fn observe_owned(&mut self, app: &mut App) {
        self.published_coverage = app.host.coverage_text();
        let watch = app.host.observe_persistence();
        self.progress_revision = watch.snapshot().revision;
        self.progress = Some(watch);
    }
    /// Retire only a real completed read, including an invalidated load. Never
    /// clear the host's single-flight slot merely to make a submit available.
    fn retire_read(&mut self) {
        let completion = self.read_done.borrow_mut().take();
        let Some(completion) = completion else {
            return;
        };
        self.read_pending = false;
        let Some(app) = self.app.as_mut() else {
            // Submit serialization keeps the App present for every such load.
            unreachable!("automatic read lost its admitted App");
        };
        if super::super::web_timing::page_hidden() {
            app.host.detach_resident_refresh();
        }
        match app.host.install_resident_refresh(completion) {
            Ok(outcome) => {
                self.read_error = None;
                if matches!(
                    outcome,
                    RefreshOutcome::TruthChanged | RefreshOutcome::ObservationOnly
                ) {
                    self.read_graph = Some(app.host.presentation_graph().clone());
                }
            },
            Err(ResidentHostError::StaleCompletion) => {},
            Err(error) => self.read_error = Some(format!("Graph check failed: {error}")),
        }
        // A read cannot replace the last foreground result, reset a draft, or
        // acknowledge an unrelated owned write. Its errors have their own line.
    }
    fn submit_available(&mut self) -> bool {
        self.retire_read();
        if self.read_pending {
            self.status = "Action not applied during graph check; try again".into();
            return false;
        }
        true
    }
    fn choose_file(&mut self) {
        if !self.submit_available() || self.saving || self.pending_intake.is_some() {
            return;
        }
        self.file_requested = true;
    }
    fn control(&mut self, action: &str) {
        if !self.submit_available() {
            return;
        }
        if self.saving {
            // The running owner alone completes publication. A Save here is a
            // coalesced notice, never another writer or a replay of the edit.
            if action == "save"
                && self
                    .progress
                    .as_ref()
                    .is_some_and(WebPersistenceWatch::request_save)
            {
                self.status = "Stored; presentation pending · Save requested".into();
            }
            return;
        }
        let Some(mut app) = self.app.take() else {
            return;
        };
        let parsed = (|| -> Result<_, String> {
            use graphshell::resident_host::{PaneRoots, ResidentRefreshMode, ResidentSettings};
            use graphshell::web_resident::{GraphAddress, ResidencyPolicy, ViewKey};
            let demand = if action == "demand" {
                let id = Uuid::parse_str(self.settings_fields[4].text().trim())
                    .map_err(|e| format!("invalid stable ID: {e}"))?;
                Some(if self.stratum.selected == 1 {
                    GraphAddress::Resource(id)
                } else {
                    GraphAddress::Surface(id)
                })
            } else {
                None
            };
            if action == "undo" {
                app.host.undo_now("graphshell").map_err(|e| e.to_string())?;
            }
            if action == "redo" {
                app.host.redo_now("graphshell").map_err(|e| e.to_string())?;
            }
            if action == "facet" {
                let id = self.selected.ok_or("Select a Surface first")?;
                let value: serde_json::Value = serde_json::from_str(self.settings_fields[6].text())
                    .map_err(|e| format!("invalid facet JSON: {e}"))?;
                app.host
                    .set_product_facet(
                        id,
                        self.settings_fields[5].text().trim(),
                        &value.to_string(),
                    )
                    .map_err(|e| e.to_string())?;
            }
            if action == "relation" {
                let from = self.selected.ok_or("Select a Surface first")?;
                let to = Uuid::parse_str(self.settings_fields[7].text().trim())
                    .map_err(|e| format!("invalid relation target ID: {e}"))?;
                app.host
                    .assert_product_relation(from, to, graphshell::product::EditableRelation::Cites)
                    .map_err(|e| e.to_string())?;
            }
            if action == "pin" {
                let id = self.selected.ok_or("Select a Surface first")?;
                let key = app
                    .host
                    .presentation_graph()
                    .get_node_key_by_id(id)
                    .ok_or("Surface not loaded")?;
                let pinned = app
                    .host
                    .presentation_graph()
                    .node_is_pinned(key)
                    .unwrap_or(false);
                app.host
                    .set_product_pinned(id, !pinned)
                    .map_err(|e| e.to_string())?;
            }
            if action == "settings" {
                if app.host.resident_busy() {
                    return Err("Save or refresh the retained operation first".into());
                }
                let hops = self.settings_fields[0]
                    .text()
                    .parse::<u16>()
                    .map_err(|_| "invalid hop count")?;
                let minimum = self.settings_fields[1]
                    .text()
                    .parse::<u64>()
                    .map_err(|_| "invalid minimum interval")?;
                let batch = self.settings_fields[2]
                    .text()
                    .parse::<u64>()
                    .map_err(|_| "invalid publication interval")?;
                let roster = self.settings_fields[3]
                    .text()
                    .split(',')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| Uuid::parse_str(s.trim()))
                    .collect::<Result<std::collections::BTreeSet<_>, _>>()
                    .map_err(|e| format!("invalid roster: {e}"))?;
                let projection = self.projection.selected.checked_sub(1).and_then(|i| {
                    app.host
                        .projection_choices()
                        .get(i)
                        .map(|(id, _)| id.clone())
                });
                app.host.set_resident_pane(
                    ViewKey::new("graphshell", "tree-controls").map_err(|e| e.to_string())?,
                    PaneRoots {
                        roster,
                        focus: None,
                    },
                );
                app.host
                    .set_resident_settings(ResidentSettings {
                        policy: if self.policy.selected == 0 {
                            ResidencyPolicy::Full
                        } else {
                            ResidencyPolicy::Neighborhood { hops }
                        },
                        projection,
                        refresh: if self.refresh.selected == 0 {
                            ResidentRefreshMode::Manual
                        } else {
                            ResidentRefreshMode::HostDriven {
                                minimum_check_ms: minimum,
                            }
                        },
                        publication_batch_ms: batch,
                    })
                    .map_err(|e| format!("Pane context updated; settings refused: {e}"))?;
            }
            Ok(demand)
        })();
        let demand = match parsed {
            Ok(value) => value,
            Err(error) => {
                self.status = error;
                self.app = Some(app);
                return;
            },
        };
        self.inspection = demand;
        self.saving = true;
        let save = matches!(
            action,
            "save" | "settings" | "pin" | "facet" | "relation" | "undo" | "redo"
        );
        if save {
            app.host.suspend_resident_refresh(false);
        }
        self.observe_owned(&mut app);
        let completed = self.controls_done.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = if let Some(address) = demand {
                app.demand(address).await
            } else if save {
                app.host
                    .persist(now_secs())
                    .await
                    .map_err(|e| e.to_string())
            } else {
                app.host.refresh().await.map_err(|e| e.to_string())
            };
            if super::super::web_timing::page_hidden() {
                app.host.detach_resident_refresh();
            }
            *completed.borrow_mut() = Some((app, result));
        });
    }
    pub(super) fn visibility(&mut self, hidden: bool) {
        if self.saving {
            return;
        } // the owned async operation keeps its lease
        if self.read_pending {
            if let Some(app) = self.app.as_mut() {
                if hidden {
                    app.host.detach_resident_refresh();
                } else {
                    app.host.attach_resident_refresh();
                }
            }
            return; // native completion still owns the single-flight retirement
        }
        if let Some(app) = self.app.as_mut() {
            if hidden && (app.host.has_unstored() || app.host.pending_publication()) {
                app.host.suspend_resident_refresh(false);
                self.control("save"); // explicit detach flush bypasses idle cadence
            } else if hidden {
                app.host.detach_resident_refresh();
            } else {
                app.host.attach_resident_refresh();
            }
        }
    }
    pub(super) fn needs_poll(&self) -> bool {
        !self.saving
            || self.ready()
            || self
                .progress
                .as_ref()
                .is_some_and(|watch| watch.snapshot().revision != self.progress_revision)
    }
    pub(super) fn ready(&self) -> bool {
        self.completed.borrow().is_some()
            || self.controls_done.borrow().is_some()
            || self.read_done.borrow().is_some()
    }
    pub(super) fn selection_locked(&self) -> bool {
        self.saving || self.file_requested || self.pending_intake.is_some()
    }
    pub(super) fn graph(&self) -> Graph {
        self.app
            .as_ref()
            .expect("app present before a save")
            .host
            .graph()
            .clone()
    }

    /// Inspect the current source owner, never a cached Canvas or form draft.
    pub(super) fn inspect_access(
        &self,
        member: Uuid,
    ) -> Result<graphshell::access_inspection::AccessInspection, String> {
        let app = self
            .app
            .as_ref()
            .ok_or("Source inspection will resume when the current save finishes")?;
        graphshell::access_inspection::inspect_access(app.host.graph(), member)
    }

    /// Selected file facts from the acknowledged local owner, for receipts.
    pub(super) fn saved_file(&self) -> Option<serde_json::Value> {
        if self.saving || self.pending_intake.is_some() {
            return None;
        }
        let app = self.app.as_ref()?;
        let (_, node) = app.host.graph().get_node_by_id(self.selected?)?;
        let content = app
            .host
            .facet_value(node.url(), graphshell::product::CONTENT_FACET)?;
        let local = app
            .host
            .facet_value(node.url(), graphshell::product::LOCAL_FILE_FACET)?;
        Some(serde_json::json!({ "content": content, "local": local }))
    }

    pub(super) fn select(&mut self, selected: Option<Uuid>) {
        if self.selection_locked() || self.selected == selected {
            return;
        }
        self.selected = selected;
        self.detail_open = false;
        self.save_state = "idle";
        let data =
            selected.and_then(|member| self.app.as_ref().and_then(|app| app.metadata(member).ok()));
        if let Some(data) = data {
            self.title = TextInput::new(data.title);
            self.tags = TextInput::new(data.tags.join(", "));
            self.address = data.address;
        } else {
            self.selected = None;
            self.title = TextInput::new("");
            self.tags = TextInput::new("");
            self.address.clear();
        }
        if let Some(app) = &mut self.app {
            app.host.set_foreground_focus(
                self.selected
                    .map(graphshell::web_resident::GraphAddress::Surface),
            );
        }
    }

    /// Open the detail editor, its item role read from the canvas.
    pub(super) fn open_detail(&mut self, canvas: &Canvas) {
        // Keep the open_file view's registered target alive until the host
        // finishes reading the chosen bytes and dispatches its answer.
        if self.selection_locked() {
            return;
        }
        self.intake_open = false;
        self.detail_open = true;
        self.role.selected =
            item_role_index(self.selected.and_then(|member| canvas.member_role(member)));
    }

    pub(super) fn save(&mut self) {
        if !self.submit_available() {
            return;
        }
        if self.saving || self.file_requested {
            return;
        }
        if self.pending_intake.is_some() {
            self.status = "Retry intake before saving other changes".into();
            return;
        }
        let Some(member) = self.selected else {
            self.status = "Select an object before saving".into();
            return;
        };
        let Some(mut app) = self.app.take() else {
            return;
        };
        let title = self.title.text().to_string();
        let tags = self.tags.text().to_string();
        self.saving = true;
        self.save_state = "saving";
        self.status = "Saving changes…".into();
        self.observe_owned(&mut app);
        let completed = self.completed.clone();
        // The task owns the app. No RefCell borrow crosses the IndexedDB await.
        wasm_bindgen_futures::spawn_local(async move {
            let result = app.save_metadata(member, &title, &tags, now_secs()).await;
            *completed.borrow_mut() = Some((app, CompletedEdit::Metadata, result));
        });
    }

    fn add_address(&mut self) {
        if !self.submit_available() {
            return;
        }
        if self.saving || self.pending_intake.is_some() || self.file_requested {
            return;
        }
        let Some(app) = self.app.as_mut() else {
            return;
        };
        match app
            .host
            .create_address(self.intake_address.text(), self.intake_title.text())
            .map_err(|e| e.to_string())
        {
            Ok(member) => self.store_intake(member),
            Err(error) => {
                self.status = error;
                self.save_state = "error";
            },
        }
    }

    fn receive_file(&mut self, event: FileEvent) {
        // Automatic reads cannot start while the chooser owns this callback.
        if !self.submit_available() {
            return;
        }
        self.file_requested = false;
        if self.saving || self.pending_intake.is_some() {
            return;
        }
        let Some(file) = event.files.into_iter().next() else {
            self.status = "File choice canceled".into();
            return;
        };
        let Some(app) = self.app.as_mut() else {
            return;
        };
        match file_metadata(
            &file.name,
            file.media_type.as_deref(),
            file.last_modified_ms,
            &file.bytes,
        )
        .and_then(|metadata| {
            app.host
                .create_file_metadata(metadata)
                .map_err(|e| e.to_string())
        }) {
            Ok(member) => self.store_intake(member),
            Err(error) => {
                self.status = format!("File intake refused · {error}");
                self.save_state = "error";
            },
        }
    }

    fn store_intake(&mut self, member: Uuid) {
        if !self.submit_available() {
            return;
        }
        if self.saving {
            return;
        }
        let Some(mut app) = self.app.take() else {
            return;
        };
        self.pending_intake = Some(member);
        self.saving = true;
        self.save_state = "saving";
        self.status = "Saving new object…".into();
        self.observe_owned(&mut app);
        let completed = self.completed.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = app.persist_intake(member, now_secs()).await;
            *completed.borrow_mut() = Some((app, CompletedEdit::Intake, result));
        });
    }

    pub(super) fn poll(&mut self, canvas: &mut Canvas) -> bool {
        self.retire_read();
        if let Some(graph) = self.read_graph.take() {
            let selected = canvas.selected_members();
            canvas.set_graph(graph);
            canvas.set_selected_members(&selected);
            return true; // outer pump reinstalls the current Forme view
        }
        if let Some(watch) = &self.progress {
            let snapshot = watch.snapshot();
            if snapshot.revision != self.progress_revision {
                self.progress_revision = snapshot.revision;
                self.status = if snapshot.save_requested {
                    format!("{} · Save requested", snapshot.phase.label())
                } else {
                    snapshot.phase.label().into()
                };
            }
        }
        let control = self.controls_done.borrow_mut().take();
        if let Some((app, result)) = control {
            self.status = match result {
                Ok(()) => self
                    .inspection
                    .take()
                    .map(|address| app.host.observation_text(address))
                    .unwrap_or_else(|| app.host.persistence_label().into()),
                Err(error) => format!("{}: {error}", app.host.persistence_label()),
            };
            let selected = canvas.selected_members();
            canvas.set_graph(app.host.presentation_graph().clone());
            canvas.set_selected_members(&selected);
            self.app = Some(app);
            self.saving = false;
            self.progress = None;
            return true;
        }
        if !self.saving
            && !self.read_pending
            && !self.file_requested
            && self.pending_intake.is_none()
        {
            // A check must never capture dirty or retained graph work whose
            // hidden flush needs the existing independently running Save future.
            // All graph submits are refused during the read; Forme saves only
            // its separate workspace key and cannot create such an obligation.
            if let Some(app) = self.app.as_mut()
                && !app.host.has_unstored()
                && !app.host.pending_publication()
                && !app.host.resident_busy()
            {
                app.host
                    .suspend_resident_refresh(super::super::web_timing::page_hidden());
                app.host
                    .tick_resident_refresh(super::super::web_timing::now_ms());
                match app.host.prepare_resident_refresh() {
                    Ok(Some(request)) => {
                        self.read_pending = true;
                        let completed = Rc::downgrade(&self.read_done);
                        wasm_bindgen_futures::spawn_local(async move {
                            let result = request.load().await;
                            if let Some(completed) = completed.upgrade() {
                                *completed.borrow_mut() = Some(result);
                            }
                            // Dropping the presentation is not native cancellation:
                            // the request and backend supervisor drain normally.
                        });
                    },
                    Ok(None) => {},
                    Err(error) => self.read_error = Some(format!("Graph check failed: {error}")),
                }
            }
        }
        let Some((app, edit, result)) = self.completed.borrow_mut().take() else {
            return false;
        };
        self.app = Some(app);
        self.saving = false;
        self.progress = None;
        match result {
            Ok(saved) => {
                let refresh = match edit {
                    CompletedEdit::Metadata => sync_canvas_metadata_from_graph(
                        canvas,
                        self.app.as_ref().expect("completed app").host.graph(),
                        &saved,
                    ),
                    CompletedEdit::Intake => {
                        let refresh = sync_canvas_intake(
                            canvas,
                            self.app.as_ref().expect("completed app").host.graph(),
                            saved.member,
                        );
                        self.pending_intake = None;
                        if refresh.is_ok() {
                            self.select(Some(saved.member));
                            self.open_detail(canvas);
                            self.intake_address = TextInput::new("");
                            self.intake_title = TextInput::new("");
                        }
                        refresh
                    },
                };
                if matches!(edit, CompletedEdit::Metadata) || refresh.is_ok() {
                    self.title = TextInput::new(saved.title);
                    self.tags = TextInput::new(saved.tags.join(", "));
                }
                self.status = match refresh {
                    Ok(()) => match edit {
                        CompletedEdit::Metadata => "Changes saved".into(),
                        CompletedEdit::Intake => "Object added and saved".into(),
                    },
                    Err(error) => match edit {
                        CompletedEdit::Metadata => {
                            format!("Changes saved · canvas refresh failed: {error}")
                        },
                        CompletedEdit::Intake => format!(
                            "Object saved · canvas refresh failed: {error} · reload to restore the local view"
                        ),
                    },
                };
                self.save_state = "saved";
            },
            Err(error) => {
                self.status = match edit {
                    CompletedEdit::Metadata => {
                        format!("Save failed · {error} · changes remain in memory")
                    },
                    CompletedEdit::Intake => format!(
                        "Intake save failed · {error} · new object remains in memory; retry intake to save it"
                    ),
                };
                self.save_state = "error";
            },
        }
        true
    }
}

/// The item role select's index for an override (0 is "as recipe").
fn item_role_index(role: Option<Role>) -> usize {
    role.and_then(|role| Role::ALL.iter().position(|r| *r == role))
        .map_or(0, |index| index + 1)
}

/// Set or clear the picked item's own role (F48). It is view state, saved
/// with the scene, not graph truth.
fn apply_item_role(page: &mut TreePage) {
    let Some(product) = &mut page.product else {
        return;
    };
    let Some(member) = product.selected else {
        product.status = "Select an object first".into();
        return;
    };
    let role = product
        .role
        .selected
        .checked_sub(1)
        .and_then(|i| Role::ALL.get(i).copied());
    let set = page
        .shared
        .canvas
        .borrow_mut()
        .set_member_role(member, role);
    product.status = match role {
        _ if !set => "Item role refused: the item does not permit it".into(),
        Some(role) => format!("Item role set to {}", role.id()),
        None => "Item role follows the recipe".into(),
    };
    page.shared.dirty.set(true);
}

fn intake_controls(page: &TreePage) -> Child {
    use cambium::{FileFilter, button, el, lens, open_file, text_field_typed};
    let product = page.product.as_ref().expect("local intake");
    let mut children: Vec<Child> = vec![Box::new(
        button("Add an object", |page: &mut TreePage, _| {
            if let Some(product) = &mut page.product
                && !product.saving
                && !product.file_requested
            {
                product.intake_open = !product.intake_open;
                if product.intake_open {
                    product.detail_open = false;
                }
            }
        })
        .attr(
            "aria-expanded",
            if product.intake_open { "true" } else { "false" },
        ),
    )];
    if product.intake_open {
        let mut fields: Vec<Child> = Vec::new();
        if product.saving {
            fields.push(Box::new(el("p", "Saving new object…")));
        } else if let Some(member) = product.pending_intake {
            fields.push(Box::new(button(
                "Retry intake",
                move |page: &mut TreePage, _| {
                    if let Some(product) = &mut page.product {
                        product.store_intake(member);
                    }
                },
            )));
        } else {
            fields.push(Box::new(el(
                "label",
                (
                    "Address",
                    lens(
                        |input: &mut TextInput| {
                            text_field_typed(input).attr("aria-label", "Address")
                        },
                        |page: &mut TreePage| {
                            &mut page.product.as_mut().expect("local intake").intake_address
                        },
                    ),
                ),
            )));
            fields.push(Box::new(el(
                "label",
                (
                    "New object title",
                    lens(
                        |input: &mut TextInput| {
                            text_field_typed(input).attr("aria-label", "New object title")
                        },
                        |page: &mut TreePage| {
                            &mut page.product.as_mut().expect("local intake").intake_title
                        },
                    ),
                ),
            )));
            fields.push(Box::new(button(
                cambium::catalogue::label(cambium::catalogue::ids::NODE_NEW)
                    .expect("shared New node label"),
                |page: &mut TreePage, _| {
                    if let Some(product) = &mut page.product {
                        product.add_address();
                    }
                },
            )));
            fields.push(Box::new(open_file(
                button("Choose a file", |page: &mut TreePage, _| {
                    if let Some(product) = &mut page.product {
                        product.choose_file();
                    }
                }),
                product.file_requested,
                FileFilter::default(),
                |page: &mut TreePage, event: FileEvent| {
                    if let Some(product) = &mut page.product {
                        product.receive_file(event);
                    }
                },
            )));
        }
        children.push(Box::new(
            el("section", fields)
                .attr("class", "tree-detail tree-intake")
                .attr("aria-label", "Add an object"),
        ));
    }
    Box::new(el("div", children))
}

pub(super) fn controls(page: &TreePage) -> Child {
    use cambium::{DetailRow, DetailSection, button, detail_panel, el, lens, text_field_typed};
    let Some(product) = &page.product else {
        return Box::new(el("div", ()));
    };
    let mut children: Vec<Child> = Vec::new();
    if !product.status.is_empty() {
        children.push(Box::new(
            el("p", product.status.clone()).attr("role", "status"),
        ));
    }
    children.push(resident_controls(page));
    children.push(intake_controls(page));
    children.push(super::inspection::open_button(page));
    if product.selected.is_some() {
        children.push(Box::new(button(
            if product.detail_open {
                "Close details"
            } else {
                "Open details"
            },
            |page: &mut TreePage, _| {
                let canvas = page.shared.canvas.borrow();
                if let Some(product) = &mut page.product {
                    if product.detail_open {
                        product.detail_open = false;
                    } else {
                        product.open_detail(&canvas);
                    }
                }
            },
        )));
    }
    if product.detail_open {
        let mut fields: Vec<Child> = vec![Box::new(detail_panel(&[DetailSection::new(
            "Object details",
            vec![DetailRow::new("Address", &product.address)],
        )]))];
        fields.push(Box::new(el("p", "Tags in a partial view may be Not loaded. Inspect the related Resource or choose Full before replacing hidden tags.")));
        if product.saving {
            fields.push(Box::new(el("p", "Saving changes…")));
        } else {
            fields.push(Box::new(el(
                "label",
                (
                    "Title",
                    lens(
                        |input: &mut TextInput| text_field_typed(input).attr("aria-label", "Title"),
                        |page: &mut TreePage| {
                            &mut page.product.as_mut().expect("local detail").title
                        },
                    ),
                ),
            )));
            fields.push(Box::new(el(
                "label",
                (
                    "Tags",
                    lens(
                        |input: &mut TextInput| text_field_typed(input).attr("aria-label", "Tags"),
                        |page: &mut TreePage| {
                            &mut page.product.as_mut().expect("local detail").tags
                        },
                    ),
                ),
            )));
            fields.push(Box::new(button(
                "Save changes",
                |page: &mut TreePage, _| {
                    if let Some(product) = &mut page.product {
                        product.save();
                    }
                },
            )));
            let options: Vec<&'static str> = std::iter::once("As recipe")
                .chain(Role::ALL.iter().map(|role| role.label()))
                .collect();
            fields.push(Box::new(el(
                "label",
                (
                    "Item role",
                    lens(
                        move |state: &mut SelectState| cambium::select(state, &options),
                        |page: &mut TreePage| {
                            &mut page.product.as_mut().expect("local detail").role
                        },
                    ),
                ),
            )));
            fields.push(Box::new(button(
                "Apply item role",
                |page: &mut TreePage, _| apply_item_role(page),
            )));
        }
        children.push(Box::new(
            el("section", fields)
                .attr("class", "tree-detail")
                .attr("aria-label", "Object details"),
        ));
    }
    Box::new(
        el("aside", children)
            .attr("class", "tree-product")
            .attr("aria-label", "Saved graph"),
    )
}

fn resident_controls(page: &TreePage) -> Child {
    use cambium::{button, el, lens, text_field_typed};
    let product = page.product.as_ref().unwrap();
    let mut fields: Vec<Child> = vec![Box::new(el("h3", "Graph residency"))];
    if let Some(app) = &product.app {
        fields.push(Box::new(
            el("p", app.host.persistence_label()).attr("role", "status"),
        ));
        fields.push(Box::new(el("pre", app.host.coverage_text())));
    } else if let Some(watch) = &product.progress {
        fields.push(Box::new(
            el("p", watch.snapshot().phase.label()).attr("role", "status"),
        ));
        fields.push(Box::new(el("pre", product.published_coverage.clone())));
    }
    // Keep this node present when idle so a check does not replace/reindex
    // focused fields merely by inserting or removing a status element.
    fields.push(Box::new(
        el(
            "p",
            if product.read_pending {
                "Checking stored graph…".to_owned()
            } else {
                product.read_error.clone().unwrap_or_default()
            },
        )
        .attr("role", "status"),
    ));
    if product.saving {
        fields.push(Box::new(el(
            "p",
            "Operation in progress; the published view remains available.",
        )));
        if product.progress.as_ref().is_some_and(|watch| {
            watch.snapshot().phase == BrowserPersistencePhase::PublicationPending
        }) {
            fields.push(Box::new(button("Save", |page: &mut TreePage, _| {
                if let Some(product) = &mut page.product {
                    product.control("save");
                }
            })));
        }
    } else {
        for (label, which, options) in [
            (
                "Display",
                0,
                vec!["Full".to_owned(), "Neighborhood".to_owned()],
            ),
            (
                "Authored projection",
                1,
                std::iter::once("Choose a projection".to_owned())
                    .chain(product.app.as_ref().into_iter().flat_map(|app| {
                        app.host
                            .projection_choices()
                            .into_iter()
                            .map(|(_, label)| label)
                    }))
                    .collect(),
            ),
            (
                "Refresh mode",
                2,
                vec!["Manual".to_owned(), "Host-driven".to_owned()],
            ),
            (
                "Inspect stratum",
                3,
                vec!["Surface".to_owned(), "Resource".to_owned()],
            ),
        ] {
            fields.push(Box::new(el(
                "label",
                (
                    label,
                    lens(
                        move |state: &mut SelectState| {
                            let labels: Vec<&str> = options.iter().map(String::as_str).collect();
                            cambium::select(state, &labels)
                        },
                        move |page: &mut TreePage| {
                            let product = page.product.as_mut().unwrap();
                            match which {
                                0 => &mut product.policy,
                                1 => &mut product.projection,
                                2 => &mut product.refresh,
                                _ => &mut product.stratum,
                            }
                        },
                    ),
                ),
            )));
        }
        for (index, label) in [
            "Hops",
            "Minimum check interval (ms)",
            "Publication batch interval (ms)",
            "Exact roster (comma-separated UUIDs)",
            "Inspect stable ID",
            "Facet ID",
            "Facet JSON",
            "Cites target UUID",
        ]
        .into_iter()
        .enumerate()
        {
            fields.push(Box::new(el(
                "label",
                (
                    label,
                    lens(
                        move |input: &mut TextInput| {
                            text_field_typed(input).attr("aria-label", label)
                        },
                        move |page: &mut TreePage| {
                            &mut page.product.as_mut().unwrap().settings_fields[index]
                        },
                    ),
                ),
            )));
        }
        for (label, action) in [
            ("Apply graph settings", "settings"),
            ("Save", "save"),
            ("Refresh", "refresh"),
            ("Inspect / Demand", "demand"),
            ("Toggle selected pin", "pin"),
            ("Apply selected facet", "facet"),
            ("Assert Cites relation", "relation"),
            ("Undo my latest change", "undo"),
            ("Redo", "redo"),
        ] {
            if matches!(
                action,
                "settings" | "pin" | "facet" | "relation" | "undo" | "redo"
            ) && product
                .app
                .as_ref()
                .is_some_and(|app| app.host.resident_busy())
            {
                fields.push(Box::new(el(
                    "span",
                    format!("{label}: Save or Refresh first"),
                )));
                continue;
            }
            fields.push(Box::new(button(label, move |page: &mut TreePage, _| {
                if let Some(product) = &mut page.product {
                    product.control(action);
                }
            })));
        }
    }
    Box::new(el("section", fields).attr("aria-label", "Graph residency"))
}
