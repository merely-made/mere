// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Event-driven shared mere panel over one admitted endpoint. SessionDriver
//! owns wire correlation and resource validation; this adapter sequences the
//! sessions, graph and optional archive readings and never owns a store.
use crate::{
    route as mere_route,
    wire::{MERE_ARCHIVE, MERE_GRAPH, MERE_SESSIONS},
};
use chirograph::{
    CapabilityProfile, ContentHash, EndpointDescriptor, ProjectionSession, ProjectionSnapshot,
    ProtocolVersion,
};
use graphshell_client::core::Outcome;
use graphshell_client::driver::Advance;
use graphshell_client::{MountedScene, ResolvedContent, SessionDriver};
use mere_view::{
    HostAction, MereViewModel, MereViewRequest, MergeOption, MergeRequest, StatusNote, ViewStatus,
};
use sceno::InstanceId;
use std::collections::{BTreeMap, HashMap, VecDeque};
#[derive(Clone)]
enum Task {
    Mount(usize),
    Resolve(ProjectionSession, InstanceId, ContentHash),
    Invoke(MereViewRequest),
    RecordAction {
        key: String,
        target: graphshell_client::ActionDraftTarget,
        action: chirograph::AdvertisedAction,
        command: hest::Invocation,
    },
    Media(ProjectionSession, InstanceId),
    /// A pending merge card's material in the sessions projection.
    MergeMedia(ProjectionSession, InstanceId),
}
pub struct MereRouteSession {
    /// The admitted application this panel reads as. The owner discloses
    /// only that application's own parallel view state.
    application: String,
    driver: SessionDriver,
    indices: Vec<usize>,
    sessions: ProjectionSession,
    graph: ProjectionSession,
    archive: Option<ProjectionSession>,
    tasks: VecDeque<Task>,
    current: Option<Task>,
    cards: HashMap<ContentHash, Vec<u8>>,
    outbox: Vec<String>,
    model: MereViewModel,
    notice: Option<String>,
    closed: bool,
    primary: Option<MountedScene>,
    saved_view: pandect::ViewIntent,
    read_only: bool,
    material: Option<crate::wire::SessionGraphV1>,
    records: HashMap<String, chirograph::PortableCardV1>,
    inspected: Option<String>,
    transient_layout: Option<String>,
    /// The pending merge the owner discloses, and the choices made on it here
    /// so far, by conflict key. Choices stay with this panel until Apply.
    merge: Option<crate::wire::PendingMergeV1>,
    merge_choices: BTreeMap<String, MergeOption>,
}
impl MereRouteSession {
    /// A panel over the route `descriptor` describes, read as `application`:
    /// the id the host was admitted under. Its saved view is the
    /// application's `<application>.mere` view.
    pub fn from_descriptor(
        descriptor: EndpointDescriptor,
        profile: CapabilityProfile,
        application: &str,
    ) -> Result<Self, String> {
        let find = |name: &str| -> Result<Option<usize>, String> {
            let matching: Vec<_> = descriptor
                .projections
                .iter()
                .enumerate()
                .filter(|(_, offer)| offer.request.session.0 == name)
                .map(|(index, _)| index)
                .collect();
            if matching.len() > 1 {
                return Err(format!(
                    "The route advertises duplicate {name} projections."
                ));
            }
            Ok(matching.first().copied())
        };
        let sessions_index =
            find(MERE_SESSIONS)?.ok_or("The route offers no sessions projection.")?;
        let graph_index = find(MERE_GRAPH)?.ok_or("The route offers no graph projection.")?;
        let archive_index = find(MERE_ARCHIVE)?;
        let sessions = descriptor.projections[sessions_index]
            .request
            .session
            .clone();
        let graph = descriptor.projections[graph_index].request.session.clone();
        let archive =
            archive_index.map(|index| descriptor.projections[index].request.session.clone());
        let indices = [Some(sessions_index), Some(graph_index), archive_index]
            .into_iter()
            .flatten()
            .collect();
        let mut result = Self {
            application: application.to_owned(),
            driver: SessionDriver::from_descriptor(descriptor, profile),
            indices,
            sessions,
            graph,
            archive,
            tasks: VecDeque::new(),
            current: None,
            cards: HashMap::new(),
            outbox: Vec::new(),
            model: MereViewModel {
                title: "Shared mere".into(),
                hide_graph: true,
                status: ViewStatus::Building(StatusNote {
                    message: "Reading the resident's mere.".into(),
                    action: None,
                }),
                ..Default::default()
            },
            notice: None,
            closed: false,
            primary: None,
            saved_view: Default::default(),
            read_only: false,
            material: None,
            records: HashMap::new(),
            inspected: None,
            transient_layout: None,
            merge: None,
            merge_choices: BTreeMap::new(),
        };
        result.refresh()?;
        Ok(result)
    }
    /// The application's view of the mere that this panel keeps.
    pub fn view_name(&self) -> String {
        format!("{}.mere", self.application)
    }
    pub fn show_notice(&mut self, reason: String) {
        self.model.notice = Some(reason);
    }
    pub fn model(&self) -> &MereViewModel {
        &self.model
    }
    pub fn saved_view(&self) -> &pandect::ViewIntent {
        &self.saved_view
    }
    pub fn records(&self) -> &HashMap<String, chirograph::PortableCardV1> {
        &self.records
    }
    pub fn read_only(&self) -> bool {
        self.read_only
    }
    pub fn view_saved(&self) -> bool {
        let Some(key) = self.focus_key() else {
            return false;
        };
        let address = self
            .records
            .get(&key)
            .and_then(|card| card.values.iter().find(|value| value.label == "Address"));
        !self.in_flight()
            && address.is_some_and(|address| self.saved_view.focus.as_ref() == Some(&address.value))
            && self
                .saved_view
                .strategy
                .as_deref()
                .unwrap_or("spectral.default")
                == self.model.layout
    }
    pub fn focus_instance(&self) -> Option<String> {
        let key = self.focus_key()?;
        let primary = self.primary()?;
        primary
            .scene
            .active_items_in_order()
            .into_iter()
            .find(|(_, item)| {
                primary.scene.tables.sources[item.source.0 as usize]
                    .as_ref()
                    .is_some_and(|source| source.id == key)
            })
            .map(|(instance, _)| instance.0.to_string())
    }
    pub fn focus_key(&self) -> Option<String> {
        if let Some(key) = &self.inspected {
            return self.records.contains_key(key).then(|| key.clone());
        }
        let snapshot = self.snapshot(&self.graph).ok()?;
        let focus = self.saved_view.focus.as_ref()?;
        mere_route::node_cards(&snapshot, &self.cards)
            .into_iter()
            .find(|(_, card)| {
                card.values
                    .iter()
                    .any(|value| value.label == "Address" && &value.value == focus)
            })
            .map(|(key, _)| key)
    }

    /// Current actions on one exact disclosed graph record. A record key is
    /// not an endpoint item ID; repeated appearances do not select a first
    /// match. This uses the already admitted route, never source-name lookup.
    pub fn record_actions(
        &self,
        key: &str,
    ) -> Result<
        (
            graphshell_client::ActionDraftTarget,
            Vec<chirograph::AdvertisedAction>,
        ),
        String,
    > {
        if self.closed || self.in_flight() || !self.records.contains_key(key) {
            return Err("The record's owner actions are unavailable.".into());
        }
        let core = self.driver.core().ok_or("The route is closed.")?;
        let mounted = core
            .client()
            .mounted(&self.graph)
            .ok_or("The graph is not mounted.")?;
        let targets: Vec<_> = mounted
            .scene
            .active_items_in_order()
            .into_iter()
            .filter(|(_, item)| {
                mounted.scene.tables.sources[item.source.0 as usize]
                    .as_ref()
                    .is_some_and(|source| {
                        source.adapter == cartography::MERE_GRAPH_ADAPTER && source.id == key
                    })
            })
            .map(|(instance, _)| instance)
            .collect();
        let [instance] = targets.as_slice() else {
            return Err(
                "The owner has not supplied one exact action target for this record.".into(),
            );
        };
        let target = graphshell_client::ActionDraftTarget {
            session: self.graph.clone(),
            target: *instance,
            observed_epoch: mounted.scene.epoch,
            observed_revision: mounted.scene.revision,
        };
        Ok((target, core.current_actions(&self.graph, *instance)?))
    }

    /// A selected-record command uses the existing owner contract and exact
    /// observed target. The owner still authorizes and enacts it.
    pub fn invoke_record_command(
        &mut self,
        key: String,
        target: graphshell_client::ActionDraftTarget,
        action: chirograph::AdvertisedAction,
        command: hest::Invocation,
    ) -> Result<(), String> {
        if self.focus_key().as_deref() != Some(&key) || self.record_actions(&key)?.0 != target {
            return Err("The selected record changed; reopen its owner action.".into());
        }
        self.notice = None;
        self.tasks.push_back(Task::RecordAction {
            key,
            target,
            action,
            command,
        });
        self.advance()
    }
    pub fn primary(&self) -> Option<&MountedScene> {
        if self.closed {
            return None;
        }
        self.primary.as_ref()
    }
    pub fn in_flight(&self) -> bool {
        self.current.is_some() || !self.tasks.is_empty() || self.driver.is_awaiting()
    }
    /// A mounted panel may be idle or purged while its carrier is reconnecting.
    /// Scenario waits must also hold through the transport's real admission,
    /// before the replacement projection driver has any requests to await.
    pub fn in_flight_with_transport(&self, transport: &graphshell_client::RemoteSession) -> bool {
        self.in_flight() || transport.in_flight()
    }
    pub fn status(&self) -> &str {
        if self.closed {
            "unavailable"
        } else if self.in_flight() {
            "reading shared mere"
        } else {
            "open"
        }
    }
    pub fn take_outgoing(&mut self) -> Vec<String> {
        std::mem::take(&mut self.outbox)
    }
    pub fn request(&mut self, request: MereViewRequest) -> Result<(), String> {
        if self.closed {
            return Err("The resident mere is unavailable; reconnect.".into());
        }
        if self.in_flight() {
            return Err("Wait for the resident's current operation.".into());
        }
        if matches!(&request,MereViewRequest::Host(key) if key=="refresh") {
            return self.refresh();
        }
        // A choice stays with this panel until Apply sends them all.
        if let MereViewRequest::Merge(MergeRequest::Choose(key, option)) = &request {
            let known = self.merge.as_ref().is_some_and(|merge| {
                merge
                    .conflicts
                    .iter()
                    .any(|conflict| &mere_route::conflict_key(&conflict.part) == key)
            });
            if !known {
                return Err("That conflict is no longer pending.".into());
            }
            self.merge_choices.insert(key.clone(), option.clone());
            mere_route::fill_merge(&mut self.model, self.merge.as_ref(), &self.merge_choices);
            return Ok(());
        }
        self.notice = None;
        match &request {
            MereViewRequest::Activate(key) => {
                if !self.records.contains_key(key) {
                    return Err("The record is no longer disclosed.".into());
                }
                self.inspected = Some(key.clone());
            },
            MereViewRequest::Layout(key) => {
                if !self.model.layouts.iter().any(|(id, _)| id == key) {
                    return Err("That layout is not offered.".into());
                }
                self.transient_layout = Some(key.clone());
            },
            _ => {},
        }
        if self.read_only
            && matches!(
                &request,
                MereViewRequest::Activate(_) | MereViewRequest::Layout(_)
            )
        {
            self.notice = Some("Viewing a read-only archive; this view is not saved.".into());
            return self.advance();
        }
        self.tasks.push_back(Task::Invoke(request));
        self.advance()
    }
    pub fn on_line(&mut self, line: &str) {
        if self.closed {
            return;
        }
        match self.driver.on_line(line) {
            Ok(advance) => {
                if let Err(reason) = self.accept(advance).and_then(|()| self.advance()) {
                    self.fail(reason);
                }
            },
            Err(reason) => self.fail(reason),
        }
    }
    pub fn fail(&mut self, reason: String) {
        self.closed = true;
        self.tasks.clear();
        self.current = None;
        self.cards.clear();
        self.outbox.clear();
        self.primary = None;
        self.saved_view = Default::default();
        self.read_only = false;
        self.material = None;
        self.records.clear();
        self.inspected = None;
        self.transient_layout = None;
        self.merge = None;
        self.merge_choices.clear();
        self.driver.disconnect();
        if let Some(core) = self.driver.core_mut() {
            core.purge();
        }
        self.model = MereViewModel {
            title: "Shared mere".into(),
            hide_graph: true,
            status: ViewStatus::Unavailable(StatusNote {
                message: reason,
                action: None,
            }),
            ..Default::default()
        };
    }
    fn snapshot(&self, session: &ProjectionSession) -> Result<ProjectionSnapshot, String> {
        let mounted = self
            .driver
            .core()
            .and_then(|core| core.client().mounted(session))
            .ok_or("The resident projection is not mounted.")?;
        Ok(ProjectionSnapshot {
            version: ProtocolVersion { major: 1, minor: 0 },
            session: session.clone(),
            scene: mounted.scene.clone(),
            presentation: mounted.presentation.clone(),
            cache_policy: mounted.cache_policy.clone(),
        })
    }
    fn refresh(&mut self) -> Result<(), String> {
        self.cards.clear();
        self.merge = None;
        self.tasks
            .extend(self.indices.iter().copied().map(Task::Mount));
        self.advance()
    }
    fn advance(&mut self) -> Result<(), String> {
        loop {
            if self.current.is_some() {
                return Ok(());
            }
            let Some(task) = self.tasks.pop_front() else {
                let sessions = self.snapshot(&self.sessions)?;
                let graph = self.snapshot(&self.graph)?;
                let mut primary = self
                    .driver
                    .core()
                    .and_then(|core| core.client().mounted(&self.graph))
                    .cloned()
                    .ok_or("The graph is not mounted.")?;
                let hidden: Vec<_> = primary
                    .scene
                    .active_items_in_order()
                    .into_iter()
                    .filter(|(_, item)| {
                        primary.scene.tables.sources[item.source.0 as usize]
                            .as_ref()
                            .is_some_and(|source| {
                                source.adapter == crate::wire::SESSION_ITEM_SOURCE
                            })
                    })
                    .map(|(instance, _)| instance.0 as usize)
                    .collect();
                for index in hidden {
                    primary.scene.tables.items[index] = None;
                    primary.scene.tables.item_order[index] = None;
                }
                mere_route::fill(&mut self.model, &sessions, &graph, &self.cards);
                self.records = mere_route::node_cards(&graph, &self.cards);
                if let Some(material) = &self.material {
                    mere_route::add_node_facets(&mut self.records, &material.facets)?;
                }
                if self
                    .inspected
                    .as_ref()
                    .is_some_and(|key| !self.records.contains_key(key))
                {
                    self.inspected = None;
                }
                self.model.layouts = vec![
                    ("spectral.default".into(), "Spectral".into()),
                    ("grid.default".into(), "Grid".into()),
                    ("phyllotaxis.default".into(), "Spiral".into()),
                ];
                self.model.layout = "spectral.default".into();
                if let Some(strategy) = self
                    .transient_layout
                    .as_ref()
                    .or(self.saved_view.strategy.as_ref())
                {
                    self.model.layout = strategy.clone();
                    let positions = mere_view::lay_out(&self.model.graph, strategy, 1280, 720);
                    for item in primary.scene.tables.items.iter_mut().flatten() {
                        if let Some(source) =
                            primary.scene.tables.sources[item.source.0 as usize].as_ref()
                        {
                            if let Some((x, y)) = positions.get(&source.id) {
                                item.transform.translate.x = x * 1280.;
                                item.transform.translate.y = y * 720.;
                            }
                        }
                    }
                    primary.scene.tables.bounds.origin.x = 0.;
                    primary.scene.tables.bounds.origin.y = 0.;
                    primary.scene.tables.bounds.size.w = 1280.;
                    primary.scene.tables.bounds.size.h = 720.;
                }
                self.primary = Some(primary);
                if self.read_only {
                    if let Some(session) =
                        self.model.sessions.iter_mut().find(|entry| entry.attached)
                    {
                        session.detail = Some("Read-only archive".into());
                    }
                }
                if let Some(archive) = &self.archive {
                    let archive = self.snapshot(archive)?;
                    mere_route::fill_archive(&mut self.model, &sessions, &archive, &self.cards);
                }
                // Choices hold only for conflicts still pending.
                let pending: Vec<String> = self
                    .merge
                    .iter()
                    .flat_map(|merge| &merge.conflicts)
                    .map(|conflict| mere_route::conflict_key(&conflict.part))
                    .collect();
                self.merge_choices.retain(|key, _| pending.contains(key));
                mere_route::fill_merge(&mut self.model, self.merge.as_ref(), &self.merge_choices);
                self.model.status = ViewStatus::Ready;
                self.model.notice = self.notice.clone();
                self.model.actions = vec![HostAction::new("refresh", "Refresh")];
                if self.drain_notices()? {
                    return self.refresh();
                }
                return Ok(());
            };
            let progress = match &task {
                Task::Mount(index) => self
                    .driver
                    .core_mut()
                    .ok_or("The route is closed.")?
                    .mount(*index)?,
                Task::Resolve(session, instance, _) => self
                    .driver
                    .core_mut()
                    .ok_or("The route is closed.")?
                    .resolve(session, *instance)?,
                Task::Media(session, instance) | Task::MergeMedia(session, instance) => self
                    .driver
                    .core_mut()
                    .ok_or("The route is closed.")?
                    .media_resource(session, *instance, 0)?,
                Task::RecordAction {
                    key,
                    target,
                    action,
                    command,
                } => {
                    if self.focus_key().as_deref() != Some(key) {
                        return Err("The selected record changed; reopen its owner action.".into());
                    }
                    self.driver
                        .core_mut()
                        .ok_or("The route is closed.")?
                        .submit_command(target, action, command)?
                },
                Task::Invoke(request) => {
                    let sessions = self.snapshot(&self.sessions)?;
                    let invocation = self
                        .view_invocation(request)?
                        .or_else(|| mere_route::intent(request, &sessions))
                        .or_else(|| {
                            self.merge.as_ref().and_then(|merge| {
                                mere_route::merge_intent(
                                    request,
                                    &sessions,
                                    merge,
                                    &self.merge_choices,
                                )
                            })
                        })
                        .or_else(|| {
                            self.archive
                                .as_ref()
                                .and_then(|archive| self.snapshot(archive).ok())
                                .and_then(|archive| {
                                    mere_route::archive_intent(request, &sessions, &archive)
                                })
                        })
                        .ok_or("The owner does not advertise that action.")?;
                    let snapshot = self.snapshot(&invocation.session)?;
                    let action = snapshot
                        .presentation
                        .offers_for(invocation.target)
                        .and_then(|offers| {
                            offers
                                .iter()
                                .flat_map(|offer| offer.semantics.actions.iter())
                                .find(|action| action.intent.0 == invocation.intent)
                        })
                        .cloned()
                        .ok_or("The owner no longer advertises that action.")?;
                    let payload: serde_json::Value =
                        serde_json::from_slice(&invocation.payload).map_err(|e| e.to_string())?;
                    self.driver
                        .core_mut()
                        .ok_or("The route is closed.")?
                        .invoke(&invocation.session, invocation.target, &action, &payload)?
                },
            };
            self.current = Some(task);
            let advance = self.driver.begin(progress)?;
            let completed = matches!(&advance, Advance::Done(_));
            self.accept(advance)?;
            if !completed {
                return Ok(());
            }
        }
    }
    fn view_invocation(
        &self,
        request: &MereViewRequest,
    ) -> Result<Option<chirograph::IntentInvocation>, String> {
        let mut view = self.saved_view.clone();
        match request {
            MereViewRequest::Activate(key) => {
                view.focus = Some(
                    self.records
                        .get(key)
                        .and_then(|card| card.values.iter().find(|value| value.label == "Address"))
                        .ok_or("The selected record has no disclosed address.")?
                        .value
                        .clone(),
                );
                view.strategy = Some(view.strategy.unwrap_or_else(|| "spectral.default".into()));
            },
            MereViewRequest::Layout(key) => view.strategy = Some(key.clone()),
            _ => return Ok(None),
        }
        if self.read_only {
            return Err("A read-only archive cannot save this view implicitly.".into());
        }
        let snapshot = self.snapshot(&self.graph)?;
        let target = snapshot
            .scene
            .active_items_in_order()
            .into_iter()
            .find(|(_, item)| {
                snapshot.scene.tables.sources[item.source.0 as usize]
                    .as_ref()
                    .is_some_and(|source| source.adapter == crate::wire::SESSION_ITEM_SOURCE)
            })
            .map(|(instance, _)| instance)
            .ok_or("The owner offers no session view target.")?;
        Ok(Some(chirograph::IntentInvocation {
            session: self.graph.clone(),
            target,
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: crate::wire::SET_VIEW_INTENT.into(),
            payload: serde_json::to_vec(&crate::wire::SetViewV1::new(self.view_name(), view))
                .map_err(|e| e.to_string())?,
        }))
    }
    fn drain_notices(&mut self) -> Result<bool, String> {
        let mut moved = false;
        while let Some(notice) = self.driver.take_notice() {
            if notice.session != self.sessions
                && notice.session != self.graph
                && self.archive.as_ref() != Some(&notice.session)
            {
                return Err("The resident notice names an undisclosed projection role.".into());
            }
            let current = self
                .driver
                .core()
                .and_then(|core| core.client().mounted(&notice.session));
            if current.is_some_and(|mounted| {
                notice.epoch.0 < mounted.scene.epoch.0
                    || (notice.epoch == mounted.scene.epoch
                        && notice.revision.0 <= mounted.scene.revision.0)
            }) {
                continue;
            }
            moved = true;
        }
        Ok(moved)
    }
    fn accept(&mut self, advance: Advance) -> Result<(), String> {
        match advance {
            Advance::Send(line) => self.outbox.push(line),
            Advance::Noted => {
                if self.current.is_none() && self.tasks.is_empty() {
                    if self.drain_notices()? {
                        self.refresh()?;
                    }
                }
            },
            Advance::Done(outcome) => {
                let task = self
                    .current
                    .take()
                    .ok_or("Unexpected resident operation completion.")?;
                match (task, outcome) {
                    (Task::Mount(_), Outcome::Mounted(session)) => {
                        let snapshot = self.snapshot(&session)?;
                        if session == self.sessions {
                            mere_route::unique_merge_card(&snapshot)?;
                        }
                        let mut resolves = Vec::new();
                        for binding in &snapshot.presentation.bindings {
                            if let Some(offer) = snapshot
                                .presentation
                                .offers
                                .get(&binding.key)
                                .and_then(|offers| offers.first())
                            {
                                resolves.push(Task::Resolve(
                                    session.clone(),
                                    binding.instance,
                                    offer.resource,
                                ));
                            }
                        }
                        for task in resolves.into_iter().rev() {
                            self.tasks.push_front(task);
                        }
                    },
                    (Task::Resolve(session, instance, hash), Outcome::Resolved(presentation)) => {
                        if let ResolvedContent::PortableCard(card) = presentation.content {
                            let snapshot = self.snapshot(&session)?;
                            let is_session = session == self.graph
                                && snapshot.scene.tables.items[instance.0 as usize]
                                    .as_ref()
                                    .and_then(|item| {
                                        snapshot.scene.tables.sources[item.source.0 as usize]
                                            .as_ref()
                                    })
                                    .is_some_and(|source| {
                                        source.adapter == crate::wire::SESSION_ITEM_SOURCE
                                    });
                            if is_session {
                                crate::wire::session_media(&card)?;
                                self.tasks
                                    .push_front(Task::Media(session.clone(), instance));
                            }
                            let is_merge = session == self.sessions
                                && mere_route::merge_cards(&snapshot).contains(&instance);
                            if is_merge {
                                mere_route::pending_merge_media(&card)?;
                                self.tasks.push_front(Task::MergeMedia(session, instance));
                            }
                            self.cards.insert(
                                hash,
                                serde_json::to_vec(&card).map_err(|e| e.to_string())?,
                            );
                        } else {
                            return Err("The mere panel requires a disclosed portable card.".into());
                        }
                    },
                    (Task::Media(_, instance), Outcome::MediaResource(resource)) => {
                        let material: crate::wire::SessionGraphV1 =
                            serde_json::from_slice(&resource.bytes)
                                .map_err(|e| format!("Invalid scoped graph material: {e}"))?;
                        let graph = self.snapshot(&self.graph)?;
                        let attached = graph.scene.tables.items[instance.0 as usize]
                            .as_ref()
                            .and_then(|item| {
                                graph.scene.tables.sources[item.source.0 as usize].as_ref()
                            })
                            .and_then(|source| uuid::Uuid::parse_str(&source.id).ok())
                            .ok_or("The material card names no attached session.")?;
                        crate::validate_graph_material(
                            &material,
                            &self.application,
                            pandect::SessionId(attached),
                            graph.scene.epoch,
                            graph.scene.revision,
                        )?;
                        if self
                            .material
                            .as_ref()
                            .is_some_and(|previous| previous.session != material.session)
                        {
                            self.inspected = None;
                            self.transient_layout = None;
                        }
                        self.saved_view = material
                            .views
                            .get(&self.view_name())
                            .cloned()
                            .unwrap_or_default();
                        self.read_only = material.read_only;
                        self.material = Some(material);
                    },
                    (Task::MergeMedia(..), Outcome::MediaResource(resource)) => {
                        let merge = mere_route::decode_pending_merge(&resource.bytes)?;
                        mere_route::adopt_merge(&mut self.merge_choices, &merge);
                        self.merge = Some(merge);
                    },
                    (Task::Invoke(request), Outcome::Intent(result)) => {
                        self.notice = mere_route::declined(&result);
                        if matches!(
                            request,
                            MereViewRequest::Merge(MergeRequest::Apply | MergeRequest::Cancel)
                        ) && *result == chirograph::IntentResult::Accepted
                        {
                            self.merge_choices.clear();
                        }
                        self.merge = None;
                        if matches!(request, MereViewRequest::Layout(_)) {
                            self.transient_layout = None;
                        }
                        self.tasks
                            .extend(self.indices.iter().copied().map(Task::Mount));
                    },
                    (Task::RecordAction { .. }, Outcome::Intent(result)) => {
                        self.notice = Some(match *result {
                            chirograph::IntentResult::Accepted => {
                                "Request accepted by the owner.".into()
                            },
                            chirograph::IntentResult::Rejected { reason } => reason,
                            chirograph::IntentResult::Stale { .. } => {
                                "The record changed; read it again.".into()
                            },
                        });
                        self.tasks
                            .extend(self.indices.iter().copied().map(Task::Mount));
                    },
                    _ => return Err("The resident answered another operation.".into()),
                }
            },
        }
        Ok(())
    }
}

/// The panel over a native admitted stream: `graphshell_client::pump`
/// carries its lines (transport agnosticism note, T3), as a browser's data
/// channel does. A failure is the panel's own (`fail`), so the pump goes on
/// only while the panel is open.
impl graphshell_client::LineDriven for MereRouteSession {
    fn take_outgoing(&mut self) -> Vec<String> {
        MereRouteSession::take_outgoing(self)
    }

    fn on_line(&mut self, line: &str) -> Result<(), String> {
        MereRouteSession::on_line(self, line);
        if !self.closed {
            return Ok(());
        }
        Err(match &self.model.status {
            ViewStatus::Unavailable(note) => note.message.clone(),
            _ => "The resident mere is unavailable.".into(),
        })
    }
}

#[cfg(test)]
mod busy_tests {
    use super::*;
    use chirograph::{
        BoundsRelationship, CarrierOutput, CarrierRequest, CarrierResponse, CarrierResponseBody,
        PresentationBinding, PresentationCapability, PresentationCodec, PresentationKey,
        PresentationManifest, PresentationOffer, PresentationSemantics, ProjectionOffer,
        ProjectionRequest, SemanticRole,
    };
    use sceno::{
        Arrangement, Footprint, ProjectedItem, Representation, Scene, Score, SourceRef, Transform2,
    };
    use scenotime::{Revision, SceneEpoch, SceneSnapshot};

    fn panel() -> MereRouteSession {
        let descriptor = EndpointDescriptor {
            label: "Resident mere".into(),
            projections: [MERE_SESSIONS, MERE_GRAPH]
                .into_iter()
                .map(|name| ProjectionOffer {
                    label: name.into(),
                    request: ProjectionRequest {
                        version: ProtocolVersion::V1,
                        session: ProjectionSession(name.into()),
                        score: Score::new(Arrangement::Spiral(Default::default())),
                    },
                })
                .collect(),
        };
        MereRouteSession::from_descriptor(
            descriptor,
            CapabilityProfile::new([PresentationCapability::PortableCard]),
            "graphshell",
        )
        .unwrap()
    }

    fn closed_panel() -> MereRouteSession {
        let mut panel = panel();
        panel.fail("The resident route closed.".into());
        assert!(!panel.in_flight());
        assert_eq!(panel.status(), "unavailable");
        panel
    }

    fn seed_partial_state(panel: &mut MereRouteSession) {
        let hash = ContentHash::of(b"partial card");
        panel.cards.insert(hash, b"partial card".to_vec());
        panel.records.insert(
            "partial".into(),
            chirograph::PortableCardV1 {
                title: "partial".into(),
                values: vec![],
                badges: vec![],
                media: vec![],
            },
        );
        panel.inspected = Some("partial".into());
        panel.transient_layout = Some("grid.default".into());
        panel
            .merge_choices
            .insert("partial".into(), MergeOption::Source);
    }

    fn send_response(panel: &mut MereRouteSession, body: CarrierResponseBody) {
        let outgoing = panel.take_outgoing();
        assert_eq!(outgoing.len(), 1);
        let request: CarrierRequest = serde_json::from_str(&outgoing[0]).unwrap();
        let line = chirograph::line::encode(&CarrierOutput::Response(CarrierResponse {
            id: request.id,
            body: Ok(body),
        }))
        .unwrap();
        panel.on_line(&line);
    }

    fn graph_snapshot(card_bytes: &[u8]) -> ProjectionSnapshot {
        let session_id = uuid::Uuid::from_u128(9).to_string();
        let mut scene = Scene::new();
        let source = scene.intern_source(SourceRef::new(
            crate::wire::SESSION_ITEM_SOURCE,
            &session_id,
        ));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(0.0, 0.0),
            footprint: Footprint::Circle { radius: 8.0 },
            representation: Representation::Card,
            layer: 0,
            visible: true,
            hit: None,
            channels: vec![],
        });
        let key = PresentationKey("session-card".into());
        let card_hash = ContentHash::of(card_bytes);
        let presentation = PresentationManifest {
            bindings: vec![PresentationBinding {
                instance: InstanceId(0),
                key: key.clone(),
            }],
            offers: [(
                key,
                vec![PresentationOffer {
                    codec: PresentationCodec::PortableCardV1,
                    resource: card_hash,
                    byte_size: card_bytes.len() as u64,
                    requires: PresentationCapability::PortableCard,
                    semantics: PresentationSemantics {
                        label: "session".into(),
                        role: SemanticRole::Graphic,
                        bounds: BoundsRelationship::FillFootprint,
                        actions: vec![],
                    },
                }],
            )]
            .into_iter()
            .collect(),
        };
        ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: ProjectionSession(MERE_GRAPH.into()),
            scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap(),
            presentation,
            cache_policy: Default::default(),
        }
    }

    fn sessions_snapshot(merge_cards: &[Vec<ContentHash>]) -> ProjectionSnapshot {
        let mut scene = Scene::new();
        let mut bindings = Vec::new();
        let mut offers = BTreeMap::new();
        for (index, media) in merge_cards.iter().enumerate() {
            let source = scene.intern_source(SourceRef::new(
                crate::wire::MERGE_SOURCE,
                &format!("pending-{index}"),
            ));
            scene.items.push(ProjectedItem {
                source,
                space: Scene::WORLD,
                transform: Transform2::translation(index as f32 * 16.0, 0.0),
                footprint: Footprint::Circle { radius: 8.0 },
                representation: Representation::Card,
                layer: 0,
                visible: true,
                hit: None,
                channels: vec![],
            });
            let key = PresentationKey(format!("merge-card-{index}"));
            let card = chirograph::PortableCardV1 {
                title: format!("pending merge {index}"),
                values: vec![],
                badges: vec![],
                media: media.clone(),
            };
            let bytes = serde_json::to_vec(&card).unwrap();
            let hash = ContentHash::of(&bytes);
            bindings.push(PresentationBinding {
                instance: InstanceId(index as u32),
                key: key.clone(),
            });
            offers.insert(
                key,
                vec![PresentationOffer {
                    codec: PresentationCodec::PortableCardV1,
                    resource: hash,
                    byte_size: bytes.len() as u64,
                    requires: PresentationCapability::PortableCard,
                    semantics: PresentationSemantics {
                        label: "pending merge".into(),
                        role: SemanticRole::Graphic,
                        bounds: BoundsRelationship::FillFootprint,
                        actions: vec![],
                    },
                }],
            );
        }
        ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: ProjectionSession(MERE_SESSIONS.into()),
            scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap(),
            presentation: PresentationManifest { bindings, offers },
            cache_policy: Default::default(),
        }
    }

    fn pending_merge_bytes() -> Vec<u8> {
        serde_json::to_vec(&crate::wire::PendingMergeV1 {
            schema: crate::wire::PENDING_MERGE_SCHEMA.into(),
            source: pandect::SessionId(uuid::Uuid::from_u128(9)),
            source_name: "source".into(),
            target: pandect::SessionId(uuid::Uuid::from_u128(10)),
            target_name: "target".into(),
            conflicts: vec![],
            carried: vec![],
            dropped: vec![],
        })
        .unwrap()
    }

    fn mount_empty_route_with_merges(
        merge_media: &[Vec<ContentHash>],
        pending_bytes: &[u8],
    ) -> MereRouteSession {
        let mut panel = panel();
        let empty = SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), Scene::new()).unwrap();
        let graph = ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: ProjectionSession(MERE_GRAPH.into()),
            scene: empty,
            presentation: PresentationManifest::default(),
            cache_policy: Default::default(),
        };
        let mut resources = HashMap::new();
        for (index, media) in merge_media.iter().enumerate() {
            let card = chirograph::PortableCardV1 {
                title: format!("pending merge {index}"),
                values: vec![],
                badges: vec![],
                media: media.clone(),
            };
            let bytes = serde_json::to_vec(&card).unwrap();
            resources.insert(ContentHash::of(&bytes), bytes);
        }
        resources.insert(ContentHash::of(pending_bytes), pending_bytes.to_vec());
        seed_partial_state(&mut panel);
        while !panel.outbox.is_empty() && !panel.closed {
            let outgoing = panel.take_outgoing();
            assert_eq!(outgoing.len(), 1);
            let request: CarrierRequest = serde_json::from_str(&outgoing[0]).unwrap();
            let body = match request.body {
                chirograph::CarrierRequestBody::Snapshot(snapshot)
                    if snapshot.session.0 == MERE_SESSIONS =>
                {
                    CarrierResponseBody::Snapshot(Box::new(sessions_snapshot(merge_media)))
                },
                chirograph::CarrierRequestBody::Snapshot(snapshot)
                    if snapshot.session.0 == MERE_GRAPH =>
                {
                    CarrierResponseBody::Snapshot(Box::new(graph.clone()))
                },
                chirograph::CarrierRequestBody::Resource(resource) => {
                    let bytes = resources.get(&resource.resource).unwrap_or_else(|| {
                        panic!("unexpected resource request: {:?}", resource.resource)
                    });
                    CarrierResponseBody::Resource(chirograph::ResourceResponse::new(
                        resource.session,
                        bytes.clone(),
                    ))
                },
                other => panic!("unexpected carrier request: {other:?}"),
            };
            let line = chirograph::line::encode(&CarrierOutput::Response(CarrierResponse {
                id: request.id,
                body: Ok(body),
            }))
            .unwrap();
            panel.on_line(&line);
        }
        panel
    }

    #[test]
    fn carrier_admits_zero_or_one_pending_merge_with_first_media() {
        let pending_bytes = pending_merge_bytes();
        let pending_hash = ContentHash::of(&pending_bytes);

        let zero = mount_empty_route_with_merges(&[], &pending_bytes);
        assert!(!zero.closed, "no pending merge should be healthy");
        assert!(matches!(zero.model.status, ViewStatus::Ready));
        assert!(zero.merge.is_none());

        let ignored_second = ContentHash::of(b"ignored second merge media");
        let one =
            mount_empty_route_with_merges(&[vec![pending_hash, ignored_second]], &pending_bytes);
        assert!(
            !one.closed,
            "one pending merge with first media should be healthy"
        );
        assert!(matches!(one.model.status, ViewStatus::Ready));
        assert_eq!(one.merge, serde_json::from_slice(&pending_bytes).ok());
    }

    #[test]
    fn carrier_rejects_pending_merge_without_media_and_purges_partial_state() {
        let pending_bytes = pending_merge_bytes();
        let missing = mount_empty_route_with_merges(&[vec![]], &pending_bytes);
        assert!(
            missing.closed,
            "a pending merge without media must be rejected"
        );
        assert!(
            matches!(&missing.model.status, ViewStatus::Unavailable(note) if note.message.contains("media resource"))
        );
        assert!(missing.cards.is_empty());
        assert!(missing.records.is_empty());
        assert!(missing.inspected.is_none());
        assert!(missing.transient_layout.is_none());
        assert!(missing.merge_choices.is_empty());
        assert!(missing.merge.is_none());
        assert!(missing.material.is_none());
    }

    #[test]
    fn carrier_rejects_multiple_pending_merge_cards_and_purges_partial_state() {
        let pending_bytes = pending_merge_bytes();
        let pending_hash = ContentHash::of(&pending_bytes);
        let multiple = mount_empty_route_with_merges(
            &[vec![pending_hash], vec![pending_hash]],
            &pending_bytes,
        );
        assert!(
            multiple.closed,
            "multiple pending merge cards must be rejected"
        );
        assert!(
            matches!(&multiple.model.status, ViewStatus::Unavailable(note) if note.message.contains("Multiple pending merge cards"))
        );
        assert!(multiple.cards.is_empty());
        assert!(multiple.records.is_empty());
        assert!(multiple.inspected.is_none());
        assert!(multiple.transient_layout.is_none());
        assert!(multiple.merge_choices.is_empty());
        assert!(multiple.merge.is_none());
        assert!(multiple.material.is_none());
    }

    #[test]
    fn live_route_validates_session_media_before_fetching_it() {
        for count in [0, 1, 2] {
            let mut panel = panel();
            seed_partial_state(&mut panel);
            let empty =
                SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), Scene::new()).unwrap();
            send_response(
                &mut panel,
                CarrierResponseBody::Snapshot(Box::new(ProjectionSnapshot {
                    version: ProtocolVersion::V1,
                    session: ProjectionSession(MERE_SESSIONS.into()),
                    scene: empty,
                    presentation: PresentationManifest::default(),
                    cache_policy: Default::default(),
                })),
            );

            let first_media = ContentHash::of(b"scoped graph envelope");
            let media = match count {
                0 => vec![],
                1 => vec![first_media],
                _ => vec![first_media, ContentHash::of(b"unexpected second resource")],
            };
            let card_bytes = serde_json::to_vec(&chirograph::PortableCardV1 {
                title: "attached session".into(),
                values: vec![],
                badges: vec![],
                media,
            })
            .unwrap();
            let graph = graph_snapshot(&card_bytes);
            send_response(&mut panel, CarrierResponseBody::Snapshot(Box::new(graph)));
            send_response(
                &mut panel,
                CarrierResponseBody::Resource(chirograph::ResourceResponse::new(
                    ProjectionSession(MERE_GRAPH.into()),
                    card_bytes,
                )),
            );

            if count == 1 {
                assert!(!panel.closed);
                let outgoing = panel.take_outgoing();
                assert_eq!(outgoing.len(), 1);
                let request: CarrierRequest = serde_json::from_str(&outgoing[0]).unwrap();
                assert!(matches!(
                    request.body,
                    chirograph::CarrierRequestBody::Resource(resource)
                        if resource.session.0 == MERE_GRAPH && resource.resource == first_media
                ));
            } else {
                assert!(panel.closed);
                assert!(
                    matches!(&panel.model.status, ViewStatus::Unavailable(note) if note.message.contains("exactly one media"))
                );
                assert!(panel.take_outgoing().is_empty());
                assert!(panel.cards.is_empty());
                assert!(panel.records.is_empty());
                assert!(panel.inspected.is_none());
                assert!(panel.transient_layout.is_none());
                assert!(panel.merge_choices.is_empty());
                assert!(panel.material.is_none());
            }
        }
    }

    #[test]
    fn purged_panel_waits_immediately_through_reconnect_and_admission() {
        let panel = closed_panel();
        let mut transport = graphshell_client::RemoteSession::new(CapabilityProfile::new([
            PresentationCapability::PortableCard,
        ]));
        transport.channel_closed();
        assert!(!panel.in_flight_with_transport(&transport));

        transport.reconnecting();
        assert!(panel.in_flight_with_transport(&transport));
        transport.set_status("rejoining: challenge, admission");
        assert!(panel.in_flight_with_transport(&transport));
        assert!(
            !panel.in_flight(),
            "no replacement panel requests exist yet"
        );

        transport.rejoined();
        assert!(
            panel.in_flight_with_transport(&transport),
            "new discovery is pending after transport admission"
        );
    }

    #[test]
    fn refused_reconnect_releases_wait_and_keeps_panel_purged() {
        let panel = closed_panel();
        let mut transport = graphshell_client::RemoteSession::new(CapabilityProfile::default());
        transport.reconnecting();
        assert!(panel.in_flight_with_transport(&transport));
        transport.link_failed("rejoin refused: grant revoked");
        assert!(!panel.in_flight_with_transport(&transport));
        assert_eq!(panel.status(), "unavailable");
        assert!(panel.model().sessions.is_empty());
        assert!(panel.records().is_empty());
        assert!(panel.primary().is_none());
    }
}
