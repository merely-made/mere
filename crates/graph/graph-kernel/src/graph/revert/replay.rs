// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// The edits that revert `change`, given the graph just before it (`before`),
/// just after it (`after`) and now (`live`), and the parts kept because they
/// changed since.
pub fn revert_change(
    change: &[CapturedDelta],
    before: &Graph,
    after: &Graph,
    live: &Graph,
) -> Revert {
    let touched = Touched::of(change);
    let mut out = Revert::default();
    let mut resource_coming = BTreeSet::new();
    let mut resource_removals = BTreeSet::new();
    let mut resource_pairs = touched.resource_edges.clone();
    for id in &touched.resources {
        resource_pairs.extend(resource_incident_pairs(before, *id));
        resource_pairs.extend(resource_incident_pairs(after, *id));
        match (
            before.resource_record(*id),
            after.resource_record(*id),
            live.resource_record(*id),
        ) {
            (None, Some(became), Some(now)) => {
                if now == became {
                    resource_removals.insert(*id);
                } else {
                    out.keep(Part::Resource(*id));
                }
            },
            (Some(was), None, None) => {
                resource_coming.insert(*id);
                out.edits.push(CapturedDelta::ReplaySetResourceRecordById {
                    resource_id: id.to_string(),
                    record: Some(was),
                });
            },
            (Some(_), None, Some(_)) => out.keep(Part::Resource(*id)),
            (Some(was), Some(became), Some(now)) => out.resource_parts(*id, &was, &became, &now),
            (Some(was), Some(became), None) if was != became => out.keep(Part::Resource(*id)),
            _ => {},
        }
    }
    let mut pairs = touched.edges.clone();
    // A node made or removed takes its relations with it.
    for id in &touched.nodes {
        pairs.extend(incident_pairs(before, *id));
        pairs.extend(incident_pairs(after, *id));
    }

    let mut going = BTreeSet::new();
    let mut coming = BTreeSet::new();
    let mut removals = Vec::new();
    for id in &touched.nodes {
        match (
            node_state(before, *id),
            node_state(after, *id),
            node_state(live, *id),
        ) {
            // Made by the change: it goes, unless changed or linked since.
            (None, Some(became), Some(now)) => {
                let unlinked_since = incident_pairs(live, *id).into_iter().all(|(from, to)| {
                    edges_between(live, from, to) == edges_between(after, from, to)
                });
                if now == became
                    && unlinked_since
                    && shown_resource(live, *id) == shown_resource(after, *id)
                {
                    going.insert(*id);
                    removals.push(CapturedDelta::ReplayRemoveNodeById {
                        node_id: id.to_string(),
                    });
                } else {
                    out.keep(Part::Node(*id));
                }
            },
            // Removed by the change: it comes back, unless made again since.
            (Some(was), None, None) => {
                coming.insert(*id);
                out.recreate(*id, &was);
            },
            (Some(_), None, Some(_)) => out.keep(Part::Node(*id)),
            (Some(was), Some(became), Some(now)) => out.node_parts(*id, &was, &became, &now, false),
            // Removed by someone since the change.
            (Some(was), Some(became), None) => {
                if was != became {
                    out.keep(Part::Node(*id));
                }
            },
            (None, _, None) | (None, None, Some(_)) => {},
        }
    }

    let present = |id: &Uuid| {
        !going.contains(id) && (coming.contains(id) || live.get_node_by_id(*id).is_some())
    };
    for (from, to) in pairs {
        let (was, became) = (
            edges_between(before, from, to),
            edges_between(after, from, to),
        );
        if was == became || going.contains(&from) || going.contains(&to) {
            continue;
        }
        let untouched = edges_between(live, from, to) == became;
        if !untouched || !present(&from) || !present(&to) {
            out.keep(Part::Edges(from, to));
            continue;
        }
        out.edits.push(CapturedDelta::ReplaySetEdgesByIds {
            from_id: from.to_string(),
            to_id: to.to_string(),
            edges: was,
        });
    }
    for (from, to) in resource_pairs {
        let was = before.persisted_resource_edges_between(from, to);
        let became = after.persisted_resource_edges_between(from, to);
        if was == became {
            continue;
        }
        let present = |id| resource_coming.contains(&id) || live.resource_record(id).is_some();
        if live.persisted_resource_edges_between(from, to) != became
            || !present(from)
            || !present(to)
        {
            out.keep(Part::ResourceEdges(from, to));
            continue;
        }
        out.edits.push(CapturedDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: from.to_string(),
            to_resource_id: to.to_string(),
            edges: was,
        });
    }
    let shown_surfaces: BTreeSet<_> = touched
        .shown_surfaces
        .union(&touched.nodes)
        .copied()
        .collect();
    for id in shown_surfaces {
        let was = shown_resource(before, id);
        let became = shown_resource(after, id);
        if was == became {
            continue;
        }
        let surface_present = coming.contains(&id) || live.get_node_by_id(id).is_some();
        let resource_present = was.is_none_or(|resource| {
            resource_coming.contains(&resource) || live.resource_record(resource).is_some()
        });
        if shown_resource(live, id) != became || !surface_present || !resource_present {
            out.keep(Part::ShownResource(id));
            continue;
        }
        out.edits.push(CapturedDelta::ReplaySetShownResourceById {
            surface_id: id.to_string(),
            resource_id: was.map(|resource| resource.to_string()),
        });
    }
    out.edits.extend(removals);

    if touched.import_records {
        let (was, became) = (before.import_records(), after.import_records());
        if was != became {
            if live.import_records() == became {
                out.edits.push(CapturedDelta::ReplaySetImportRecords {
                    import_records: was.to_vec(),
                });
            } else {
                out.keep(Part::ImportRecords);
            }
        }
    }

    for id in &touched.fields {
        let state = |graph: &Graph| {
            graph
                .field(FieldId::from_uuid(*id))
                .map(persisted_field_from_field)
        };
        let (was, became) = (state(before), state(after));
        if was == became {
            continue;
        }
        if state(live) != became {
            out.keep(Part::Field(*id));
            continue;
        }
        out.edits.push(match was {
            Some(field) => CapturedDelta::ReplayAddField { field },
            None => CapturedDelta::ReplayRemoveFieldById {
                field_id: id.to_string(),
            },
        });
    }

    let mut couplings = touched.couplings.clone();
    for graph in [before, after] {
        couplings.extend(
            graph
                .couplings()
                .filter(|coupling| touched.coupled_fields.contains(&coupling.field.as_uuid()))
                .map(|coupling| coupling.id.as_uuid()),
        );
    }
    for id in &couplings {
        let state = |graph: &Graph| {
            graph
                .couplings()
                .find(|coupling| coupling.id == CouplingId::from_uuid(*id))
                .map(persisted_coupling_from_coupling)
        };
        let (was, became) = (state(before), state(after));
        if was == became {
            continue;
        }
        if state(live) != became {
            out.keep(Part::Coupling(*id));
            continue;
        }
        out.edits.push(match was {
            Some(coupling) => CapturedDelta::ReplayAddCoupling { coupling },
            None => CapturedDelta::ReplayRetractCouplingById {
                coupling_id: id.to_string(),
            },
        });
    }
    if !resource_removals.is_empty() {
        let mut scratch = live.clone();
        super::super::capture::replay_captured_deltas_onto(&mut scratch, out.edits.iter().cloned());
        for resource in resource_removals {
            if scratch.set_resource_record(resource, None) {
                out.edits.push(CapturedDelta::ReplaySetResourceRecordById {
                    resource_id: resource.to_string(),
                    record: None,
                });
            } else {
                out.keep(Part::Resource(resource));
            }
        }
    }
    out
}
