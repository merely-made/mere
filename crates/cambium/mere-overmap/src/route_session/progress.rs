// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl MereRouteSession {
    pub(super) fn advance(&mut self) -> Result<(), String> {
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
}
