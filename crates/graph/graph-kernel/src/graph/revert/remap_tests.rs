// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use serde_json::json;

use super::tests::{facet, id};
use super::*;
use crate::graph::node_facets::{PROVENANCE_DERIVATIONS, SEMANTIC_PROPERTIES};
use crate::graph::{EdgeAssertion, ProvenanceSubKind, SemanticSubKind, SplitOrientation};
use crate::persistence::{
    PersistedCoupling, PersistedCouplingResponse, PersistedField, PersistedFieldLifecycle,
    PersistedNodeSelector,
};
use crate::types::{ImportRecord, ImportRecordMembership, NodeDerivation, NodeProperty};

fn node(n: u128) -> String {
    id(n).to_string()
}

/// Ids 1 to 8, the ones the edits below hold, each mapped to itself plus 100.
fn ids() -> BTreeMap<String, String> {
    (1..=8).map(|n| (node(n), node(n + 100))).collect()
}

fn edge(from: u128, to: u128) -> PersistedEdge {
    serde_json::from_value(json!({ "from_node_id": node(from), "to_node_id": node(to) })).unwrap()
}

fn hint(first: u128, second: u128) -> FrameLayoutHint {
    FrameLayoutHint::SplitHalf {
        first: node(first),
        second: node(second),
        orientation: SplitOrientation::Vertical,
    }
}

/// Edits whose other strings read as the id of node 1, with the leaves of
/// each that are ids.
fn cases() -> Vec<(CapturedDelta, Vec<&'static str>)> {
    use CapturedDelta as D;
    let text = node(1);
    let record = ImportRecord {
        record_id: text.clone(),
        source_id: text.clone(),
        source_label: text.clone(),
        imported_at_secs: 0,
        memberships: vec![
            ImportRecordMembership {
                node_id: node(1),
                suppressed: false,
            },
            ImportRecordMembership {
                node_id: node(2),
                suppressed: true,
            },
        ],
    };
    let derivation = NodeDerivation {
        sub_kind: ProvenanceSubKind::CopiedFrom,
        source_node: node(2),
        source_graph: Some(text.clone()),
    };
    let field = PersistedField {
        id: node(5),
        name: Some(text.clone()),
        definition_json: text.clone(),
        extent: PersistedFieldExtent::AttachedToNode(node(1)),
        lifecycle: PersistedFieldLifecycle::Active,
    };
    let coupling = PersistedCoupling {
        id: node(6),
        field_id: node(5),
        selector: PersistedNodeSelector::Tagged(text.clone()),
        response: PersistedCouplingResponse::AttractToMin,
        strength: 1.0,
    };
    let resource = PersistedResourceRecord {
        canonical_iri: text.clone(),
        facets: vec![PersistedResourceFacet {
            facet: text.clone(),
            value_json: format!("\"{text}\""),
        }],
    };
    vec![
        (
            D::ReplayAddNodeWithIdIfMissing {
                id: node(1),
                url: text.clone(),
                position: [0.0, 0.0],
            },
            vec!["/id"],
        ),
        (
            D::ReplaySetNodeTitleById {
                node_id: node(1),
                title: text.clone(),
            },
            vec!["/node_id"],
        ),
        (
            D::ReplaySetNodeUrlById {
                node_id: node(1),
                new_url: text.clone(),
            },
            vec!["/node_id"],
        ),
        (
            D::ReplayInsertNodeTagById {
                node_id: node(1),
                tag: text.clone(),
            },
            vec!["/node_id"],
        ),
        (
            D::ReplaySetNodeBodyById {
                node_id: node(1),
                body: Some(text.clone()),
            },
            vec!["/node_id"],
        ),
        (
            D::ReplaySetNodeNestedById {
                node_id: node(1),
                nested: Some(text.clone()),
            },
            vec!["/node_id"],
        ),
        (
            D::ReplayUpdateNodeHistoryById {
                node_id: node(1),
                entries: vec![text.clone()],
                current_index: 0,
            },
            vec!["/node_id"],
        ),
        (
            D::ReplayAppendNodePropertyById {
                node_id: node(1),
                property: NodeProperty::new(text.clone(), text.clone()),
            },
            vec!["/node_id"],
        ),
        (
            D::ReplayRecordNodeDerivationById {
                node_id: node(1),
                derivation,
            },
            vec!["/node_id"],
        ),
        (
            D::ReplaySetNodeFacetById {
                node_id: node(1),
                facet: text.clone(),
                value_json: format!("\"{text}\""),
            },
            vec!["/node_id"],
        ),
        (
            D::ReplayAppendFrameLayoutHintById {
                node_id: node(1),
                hint: hint(2, 3),
            },
            vec![
                "/node_id",
                "/hint/SplitHalf/first",
                "/hint/SplitHalf/second",
            ],
        ),
        (
            D::ReplayBranchHistoryByIds {
                child_id: node(1),
                parent_id: node(2),
            },
            vec!["/child_id", "/parent_id"],
        ),
        (
            D::ReplayAssertSemanticPredicateByIds {
                from_id: node(1),
                to_id: node(2),
                predicate: text.clone(),
            },
            vec!["/from_id", "/to_id"],
        ),
        (
            D::ReplayAssertRelationByIds {
                from_id: node(1),
                to_id: node(2),
                assertion: EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::Cites,
                    label: Some(text.clone()),
                },
            },
            vec!["/from_id", "/to_id"],
        ),
        (
            D::ReplaySetEdgesByIds {
                from_id: node(1),
                to_id: node(2),
                edges: vec![edge(1, 2)],
            },
            vec![
                "/from_id",
                "/to_id",
                "/edges/0/from_node_id",
                "/edges/0/to_node_id",
            ],
        ),
        (
            D::ReplaySetImportRecords {
                import_records: vec![record],
            },
            vec![
                "/import_records/0/memberships/0/node_id",
                "/import_records/0/memberships/1/node_id",
            ],
        ),
        (
            D::ReplayAddField { field },
            vec!["/field/id", "/field/extent/AttachedToNode"],
        ),
        (
            D::ReplaySetFieldCouplingStrengthByFieldId {
                field_id: node(5),
                strength: 0.5,
            },
            vec!["/field_id"],
        ),
        (
            D::ReplayAddCoupling { coupling },
            vec!["/coupling/id", "/coupling/field_id"],
        ),
        (
            D::ReplayRetractCouplingById {
                coupling_id: node(6),
            },
            vec!["/coupling_id"],
        ),
        (
            D::ReplaySetResourceRecordById {
                resource_id: node(7),
                record: Some(resource),
            },
            vec!["/resource_id"],
        ),
        (
            D::ReplaySetResourceEdgesByIds {
                from_resource_id: node(7),
                to_resource_id: node(8),
                edges: vec![edge(7, 8)],
            },
            vec![
                "/from_resource_id",
                "/to_resource_id",
                "/edges/0/from_node_id",
                "/edges/0/to_node_id",
            ],
        ),
        (
            D::ReplaySetShownResourceById {
                surface_id: node(1),
                resource_id: Some(node(7)),
            },
            vec!["/surface_id", "/resource_id"],
        ),
    ]
}

/// The variant's fields, without its name.
fn body(edit: &CapturedDelta) -> Value {
    let value = serde_json::to_value(edit).unwrap();
    value.as_object().unwrap().values().next().unwrap().clone()
}

/// The paths of the leaves that differ.
fn changed(before: &Value, after: &Value, at: String, out: &mut Vec<String>) {
    match (before, after) {
        (Value::Object(a), Value::Object(b)) if a.len() == b.len() => {
            for (key, value) in a {
                changed(value, &b[key], format!("{at}/{key}"), out);
            }
        },
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (n, (x, y)) in a.iter().zip(b).enumerate() {
                changed(x, y, format!("{at}/{n}"), out);
            }
        },
        _ if before != after => out.push(at),
        _ => {},
    }
}

/// What `Touched` reports, with each id it names taken through `ids`.
fn renamed(touched: &Touched, ids: &BTreeMap<String, String>) -> Touched {
    let swap = |id: &Uuid| {
        ids.get(&id.to_string())
            .map_or(*id, |new| Uuid::parse_str(new).unwrap())
    };
    let set = |set: &BTreeSet<Uuid>| set.iter().map(swap).collect::<BTreeSet<_>>();
    let pairs = |pairs: &BTreeSet<(Uuid, Uuid)>| {
        pairs
            .iter()
            .map(|(a, b)| (swap(a), swap(b)))
            .collect::<BTreeSet<_>>()
    };
    Touched {
        nodes: set(&touched.nodes),
        edges: pairs(&touched.edges),
        fields: set(&touched.fields),
        couplings: set(&touched.couplings),
        coupled_fields: set(&touched.coupled_fields),
        import_records: touched.import_records,
        resources: set(&touched.resources),
        resource_edges: pairs(&touched.resource_edges),
        shown_surfaces: set(&touched.shown_surfaces),
    }
}

#[test]
fn remap_ids_rewrites_what_touched_names_and_nothing_else() {
    let ids = ids();
    for (edit, expected) in cases() {
        let remapped = edit.remap_ids(&ids);
        let mut moved = Vec::new();
        changed(&body(&edit), &body(&remapped), String::new(), &mut moved);
        moved.sort();
        let mut expected = expected;
        expected.sort();
        assert_eq!(moved, expected, "{edit:?}");
        assert_eq!(
            Touched::of([&remapped]),
            renamed(&Touched::of([&edit]), &ids),
            "{edit:?}"
        );
    }
}

#[test]
fn remap_ids_leaves_an_id_the_map_does_not_name() {
    let ids = BTreeMap::from([(node(1), node(101))]);
    let edit = CapturedDelta::ReplayBranchHistoryByIds {
        child_id: node(1),
        parent_id: node(2),
    };
    assert_eq!(
        edit.remap_ids(&ids),
        CapturedDelta::ReplayBranchHistoryByIds {
            child_id: node(101),
            parent_id: node(2),
        }
    );
}

fn text_of(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value).unwrap()
}

#[test]
fn a_facet_value_is_remapped_only_through_the_ids_it_declares() {
    let ids = ids();
    let remapped = |name: &str, value: &str| match facet(1, name, value).remap_ids(&ids) {
        CapturedDelta::ReplaySetNodeFacetById {
            node_id,
            value_json,
            ..
        } => {
            assert_eq!(node_id, node(101), "the node key moves with the edit");
            value_json
        },
        other => panic!("{other:?}"),
    };
    // Frame-layout members are node ids.
    assert_eq!(
        remapped(ARRANGEMENT_FRAME_LAYOUT, &text_of(&[hint(2, 3)])),
        text_of(&[hint(102, 103)])
    );
    // Everything else the kernel defines, and anything it does not, carries as written.
    let derivation = vec![NodeDerivation {
        sub_kind: ProvenanceSubKind::CopiedFrom,
        source_node: node(2),
        source_graph: Some(node(3)),
    }];
    let property = vec![NodeProperty::new(node(1), node(2))];
    for (name, value) in [
        (PROVENANCE_DERIVATIONS, text_of(&derivation)),
        (SEMANTIC_PROPERTIES, text_of(&property)),
        ("extension.note", json!({ "about": node(2) }).to_string()),
        (ARRANGEMENT_FRAME_LAYOUT, "not a list of hints".to_string()),
        (
            ARRANGEMENT_FRAME_LAYOUT,
            json!([{ "Unknown": node(2) }]).to_string(),
        ),
    ] {
        assert_eq!(remapped(name, &value), value, "{name}");
    }
}
