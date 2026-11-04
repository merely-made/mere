// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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
    pub(super) fn snapshot(
        &self,
        session: &ProjectionSession,
    ) -> Result<ProjectionSnapshot, String> {
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
    pub(super) fn refresh(&mut self) -> Result<(), String> {
        self.cards.clear();
        self.merge = None;
        self.tasks
            .extend(self.indices.iter().copied().map(Task::Mount));
        self.advance()
    }
    pub(super) fn view_invocation(
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
    pub(super) fn drain_notices(&mut self) -> Result<bool, String> {
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
    pub(super) fn accept(&mut self, advance: Advance) -> Result<(), String> {
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
                        mere_route::validate_required_cards(&snapshot, false)?;
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
