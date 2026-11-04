// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Revert {
    pub(super) fn keep(&mut self, part: Part) {
        self.kept.push(part);
    }

    pub(super) fn resource_parts(
        &mut self,
        id: Uuid,
        before: &PersistedResourceRecord,
        after: &PersistedResourceRecord,
        live: &PersistedResourceRecord,
    ) {
        let was = resource_facets(before);
        let became = resource_facets(after);
        let mut now = resource_facets(live);
        let keys: BTreeSet<_> = was.keys().chain(became.keys()).collect();
        let mut changed = false;
        for key in keys {
            if was.get(key) == became.get(key) {
                continue;
            }
            if now.get(key) != became.get(key) {
                self.keep(Part::ResourceFacet(id, key.clone()));
                continue;
            }
            match was.get(key) {
                Some(value) => {
                    now.insert(key.clone(), value.clone());
                },
                None => {
                    now.remove(key);
                },
            }
            changed = true;
        }
        if changed {
            self.edits.push(CapturedDelta::ReplaySetResourceRecordById {
                resource_id: id.to_string(),
                record: Some(PersistedResourceRecord {
                    canonical_iri: live.canonical_iri.clone(),
                    facets: now
                        .into_iter()
                        .map(|(facet, value)| PersistedResourceFacet {
                            facet,
                            value_json: value.to_string(),
                        })
                        .collect(),
                }),
            });
        }
    }

    /// Revert one part: `changed` if the change altered it, `untouched` if it
    /// still reads as the change left it.
    pub(super) fn part(&mut self, part: Part, changed: bool, untouched: bool, edit: CapturedDelta) {
        if !changed {
            return;
        }
        if untouched {
            self.edits.push(edit);
        } else {
            self.keep(part);
        }
    }

    /// Put back each node part the change altered and nobody touched since.
    pub(super) fn node_parts(
        &mut self,
        id: Uuid,
        before: &NodeState,
        after: &NodeState,
        live: &NodeState,
        recreating: bool,
    ) {
        let node_id = id.to_string();
        self.part(
            Part::Title(id),
            before.title != after.title,
            live.title == after.title,
            CapturedDelta::ReplaySetNodeTitleById {
                node_id: node_id.clone(),
                title: before.title.clone(),
            },
        );
        self.part(
            Part::Url(id),
            before.url != after.url,
            live.url == after.url,
            CapturedDelta::ReplaySetNodeUrlById {
                node_id: node_id.clone(),
                new_url: before.url.clone(),
            },
        );
        self.part(
            Part::Body(id),
            before.body != after.body,
            live.body == after.body,
            CapturedDelta::ReplaySetNodeBodyById {
                node_id: node_id.clone(),
                body: before.body.clone(),
            },
        );
        self.part(
            Part::Content(id),
            before.content != after.content,
            live.content == after.content,
            CapturedDelta::ReplaySetNodeContentById {
                node_id: node_id.clone(),
                content: before.content,
            },
        );
        self.part(
            Part::MediaType(id),
            before.media_type != after.media_type,
            live.media_type == after.media_type,
            CapturedDelta::ReplaySetNodeMimeHintById {
                node_id: node_id.clone(),
                mime_hint: before.media_type.clone(),
            },
        );
        self.part(
            Part::Nested(id),
            before.nested != after.nested,
            live.nested == after.nested,
            CapturedDelta::ReplaySetNodeNestedById {
                node_id: node_id.clone(),
                nested: before.nested.clone(),
            },
        );
        for tag in before.tags.symmetric_difference(&after.tags) {
            let untouched = live.tags.contains(tag) == after.tags.contains(tag);
            let edit = if before.tags.contains(tag) {
                CapturedDelta::ReplayInsertNodeTagById {
                    node_id: node_id.clone(),
                    tag: tag.clone(),
                }
            } else {
                CapturedDelta::ReplayRemoveNodeTagById {
                    node_id: node_id.clone(),
                    tag: tag.clone(),
                }
            };
            self.part(Part::Tag(id, tag.clone()), true, untouched, edit);
        }
        let facet_ids: BTreeSet<&String> =
            before.facets.keys().chain(after.facets.keys()).collect();
        for facet in facet_ids {
            let (was, became) = (before.facets.get(facet), after.facets.get(facet));
            let edit = match was {
                Some(value) => CapturedDelta::ReplaySetNodeFacetById {
                    node_id: node_id.clone(),
                    facet: facet.clone(),
                    value_json: value.to_string(),
                },
                None => CapturedDelta::ReplayRemoveNodeFacetById {
                    node_id: node_id.clone(),
                    facet: facet.clone(),
                },
            };
            self.part(
                Part::Facet(id, facet.clone()),
                recreating || was != became,
                live.facets.get(facet) == became,
                edit,
            );
        }
        self.part(
            Part::History(id),
            before.history != after.history,
            live.history == after.history,
            CapturedDelta::ReplayUpdateNodeHistoryById {
                node_id,
                entries: before.history.0.clone(),
                current_index: before.history.1,
            },
        );
    }

    /// Make a node the change removed again, as it stood before.
    pub(in super::super) fn recreate(&mut self, id: Uuid, before: &NodeState) {
        // What a node is born with, so every part below lands as it stood.
        let mut scratch = Graph::new();
        let _ = apply_graph_delta(
            &mut scratch,
            GraphDelta::ReplayAddNodeWithIdIfMissing {
                id,
                url: before.url.clone(),
                position: Point2D::new(0.0, 0.0),
            },
        );
        let born = node_state(&scratch, id).expect("the scratch node exists");
        self.recreate_from_birth_state(id, before, &born);
    }

    pub(super) fn recreate_from_birth_state(
        &mut self,
        id: Uuid,
        before: &NodeState,
        born: &NodeState,
    ) {
        self.edits
            .push(CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id: id.to_string(),
                url: before.url.clone(),
                position: [0.0, 0.0],
            });
        // Replay's birth clock can differ from the planner's; restore every facet.
        self.node_parts(id, before, born, born, true);
    }
}
