// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The frozen realization's tests that need Mere or Genet: the solved coastal
//! map (`scenomise`), the matrix reading, the AccessKit tree (`uxtree`) and
//! the DOM and probe checks (`genet-scripted-dom`, `taproot`).
//!
//! They were unit tests inside `graphshell-client`'s `frozen` module. They
//! moved here when `graphshell-client` moved to the Graphshell repository,
//! which depends on nothing in Mere (cycle breaks plan, D9; "moved code keeps
//! its tests"). The four tests that need only `sceno` stayed with the module.
//! `named` and `folded_dependencies` are the module's fixtures, restated.

use std::collections::HashMap;

use graphshell_client::frozen::*;
use sceno::{
    Footprint, HeldPlacement, InstanceId, LinkKey, LinkMarks, ProjectedItem, Representation,
    RoutedRelation, Scene, Score, SourceIx, SourceRef, Transform2, Vec2,
};
use scenotime::{FoldId, SceneSnapshot};
use uxtree::frozen::FrozenUxTree;

fn named(pairs: &[(&str, &str, &str)]) -> HashMap<SourceRef, String> {
    pairs
        .iter()
        .map(|(adapter, id, name)| (SourceRef::new(*adapter, *id), (*name).to_owned()))
        .collect()
}

/// Mere and three dependencies, folded behind Mere.
fn folded_dependencies() -> Scene {
    let mut scene = Scene::new();
    for (index, id) in ["mere", "genet", "netrender", "retinue"].iter().enumerate() {
        let source = scene.intern_source(SourceRef::new("fixture.repo", *id));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(index as f32 * 30.0, 0.0),
            footprint: Footprint::Point,
            representation: sceno::Representation::Glyph,
            layer: 0,
            visible: true,
            hit: None,
            channels: Vec::new(),
        });
        if index > 0 {
            scene.relations.push(RoutedRelation {
                from: InstanceId(0),
                to: InstanceId(index as u32),
                space: Scene::WORLD,
                points: Vec::new(),
                kind: Some("depends_on".into()),
                weight: None,
                link_key: None,
                marks: sceno::LinkMarks::default(),
                strength: None,
            });
        }
    }
    scene.folds.push(sceno::Fold {
        members: (0..4).map(InstanceId).collect(),
        stand_in: sceno::StandIn::Member(InstanceId(0)),
        rule: Some(sceno::FoldRule::Descendants {
            root: InstanceId(0),
            family: "depends_on".into(),
            direction: sceno::FoldDirection::Outgoing,
        }),
        boundary: None,
        label: None,
    });
    scene
}

/// The P5 fixture, solved: a real scene rather than a hand-built stand-in.
fn coastal() -> Scene {
    let score: Score = serde_json::from_str(include_str!(
        "../../../cambium/scenes/scenomise/fixtures/coastal_map.json"
    ))
    .expect("the coastal map fixture parses");
    scenomise::solve(&score)
}

#[test]
fn a_frozen_scene_enumerates_every_visible_instance_by_name() {
    let scene = coastal();
    let names = named(&[
        ("fixture.map", "harbor", "Harbor"),
        ("fixture.map", "beacon", "Beacon"),
    ]);
    let frozen = FrozenScene::freeze(&scene, "Coastal map", &names);

    let visible = scene.items.iter().filter(|item| item.visible).count();
    assert_eq!(
        frozen.instances.len(),
        visible,
        "every visible item is listed"
    );
    assert!(frozen.instances.iter().any(|i| i.name == "Harbor"));
    assert!(frozen.instances.iter().any(|i| i.name == "Beacon"));
    // The underlay had no supplied name, so it falls back and is counted.
    assert!(frozen.unnamed > 0);
    assert!(
        frozen
            .instances
            .iter()
            .any(|i| i.named_by_fallback && i.name == "coastal-outline")
    );
}

#[test]
fn snapshot_freeze_preserves_sparse_instances_and_full_source_mapping() {
    use scenotime::{Revision, SceneEpoch, SceneSnapshot};

    let template = coastal().items[0].clone();
    let mut scene = Scene::new();
    let shared = scene.intern_source(SourceRef::new("fixture.alpha", "shared"));
    let colliding = scene.intern_source(SourceRef::new("fixture.beta", "shared"));
    for source in [shared, shared, colliding, shared] {
        scene.items.push(ProjectedItem {
            source,
            ..template.clone()
        });
    }
    let mut snapshot =
        SceneSnapshot::from_dense(SceneEpoch(8), Revision(3), scene).expect("snapshot");
    snapshot.tables.items[0] = None;
    snapshot.tables.item_order[0] = None;
    snapshot.validate().expect("sparse snapshot remains valid");

    let names = HashMap::from([
        (InstanceId(1), "Alpha row".to_owned()),
        (InstanceId(2), "Beta cell".to_owned()),
        (InstanceId(3), "Alpha column".to_owned()),
    ]);
    let frozen = FrozenScene::freeze_snapshot(&snapshot, "Sparse projection", &names);
    let mapping = frozen
        .instances
        .iter()
        .map(|instance| (instance.instance, instance.source.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        mapping,
        vec![
            (InstanceId(1), SourceRef::new("fixture.alpha", "shared")),
            (InstanceId(2), SourceRef::new("fixture.beta", "shared")),
            (InstanceId(3), SourceRef::new("fixture.alpha", "shared")),
        ],
        "tombstones do not renumber instances and repeated sources stay repeated"
    );
    assert_eq!(
        frozen
            .instances
            .iter()
            .map(|instance| instance.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Alpha row", "Beta cell", "Alpha column"],
        "presentation labels stay per instance even when sources repeat"
    );

    let html = frozen.to_html("sparse");
    for (instance, source) in mapping {
        let carried = format!(
            "data-projection-instance=\"{}\" data-source-adapter=\"{}\" data-source-id=\"{}\"",
            instance.0, source.adapter, source.id
        );
        assert_eq!(
            html.matches(&carried).count(),
            2,
            "the navigable list and table both carry {carried}"
        );
    }
}

/// A static viewer renders a portable V2 capture from its bytes alone: the
/// score rides along for whoever wants it, but freezing the scene never
/// runs the compiler (site canvas plan, S6).
#[test]
fn a_v2_capture_freezes_without_the_compiler() {
    use chirograph::{
        CaptureAuthorityV1, PROJECTION_CAPTURE_V2, PresentationManifest, ProjectionCaptureV2,
        Sha256NamedInformation,
    };
    use scenotime::{Revision, SceneEpoch, SceneSnapshot};

    let score: Score = serde_json::from_str(include_str!(
        "../../../cambium/scenes/scenomise/fixtures/coastal_map.json"
    ))
    .expect("the coastal map fixture parses");
    let scene = scenomise::solve(&score);
    let names = named(&[("fixture.map", "harbor", "Harbor")]);
    let dense = FrozenScene::freeze(&scene, "Coastal map", &names);

    let capture = ProjectionCaptureV2 {
        version: PROJECTION_CAPTURE_V2,
        scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(0), scene).expect("snapshot"),
        presentation: PresentationManifest::default(),
        authority: Some(CaptureAuthorityV1 {
            adapter: "fixture.map".to_owned(),
            schema: "fixture.map/v1".to_owned(),
            sha256: Sha256NamedInformation::of(b"coastal authority"),
            generation: score.generation,
        }),
        score: Some(score),
    };
    let bytes = capture.encode().expect("capture encodes");

    let reopened = ProjectionCaptureV2::decode(&bytes).expect("capture decodes");
    let instance_names = reopened
        .scene
        .tables
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let source =
                reopened.scene.tables.sources[item.as_ref()?.source.0 as usize].as_ref()?;
            Some((InstanceId(index as u32), names.get(source)?.clone()))
        })
        .collect::<HashMap<_, _>>();
    let frozen = FrozenScene::freeze_snapshot(&reopened.scene, "Coastal map", &instance_names);

    assert_eq!(frozen.instances.len(), dense.instances.len());
    assert_eq!(frozen.rows(), dense.rows());
    assert!(frozen.to_html("capture").contains("Harbor"));
}

#[test]
fn the_summary_counts_what_the_listing_shows() {
    let scene = coastal();
    let frozen = FrozenScene::freeze(&scene, "Coastal map", &HashMap::new());
    // The alternate must not be able to drift from the enumeration under it.
    assert!(
        frozen
            .summary
            .contains(&format!("{} items", frozen.instances.len()))
    );
    assert!(frozen.instances.iter().all(|i| i.named_by_fallback));
    assert_eq!(frozen.unnamed, frozen.instances.len());
}

#[test]
fn an_invisible_item_is_not_something_a_reader_is_missing() {
    let mut scene = coastal();
    let before = FrozenScene::freeze(&scene, "Coastal map", &HashMap::new())
        .instances
        .len();
    scene.items[0].visible = false;
    let after = FrozenScene::freeze(&scene, "Coastal map", &HashMap::new());
    assert_eq!(after.instances.len(), before - 1);
}

#[test]
fn a_fold_is_a_disclosure_in_the_markup() {
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::LayoutDom;

    let frozen = FrozenScene::freeze(
        &folded_dependencies(),
        "Repositories",
        &named(&[("fixture.repo", "mere", "Mere")]),
    );
    let html = frozen.to_html("repos");
    let dom = ScriptedDom::from_serialized_document(&format!(
        "<!doctype html><html><body>{html}</body></html>"
    ));
    let root = dom.document();
    let details = descendants(&dom, root, "details");
    assert_eq!(details.len(), 1);
    let summary = descendants(&dom, details[0], "summary");
    assert_eq!(
        text(&dom, summary[0]),
        "Mere, +3 folded: Mere and everything it reaches by depends on"
    );
    let members = descendants(&dom, details[0], "li")
        .into_iter()
        .filter_map(|li| attribute(&dom, li, "data-projection-instance"))
        .collect::<Vec<_>>();
    assert_eq!(members, ["1", "2", "3"]);
    assert_eq!(
        dom.inner_html(root).matches("<tr").count(),
        frozen.rows().len() + 1,
        "every row reached the table"
    );
}

#[test]
fn a_labelled_fold_reads_its_label_in_place_of_the_rule() {
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::LayoutDom;

    let mut scene = folded_dependencies();
    scene.folds[0].label = Some("Mere's dependencies".into());
    let frozen = FrozenScene::freeze(
        &scene,
        "Repositories",
        &named(&[("fixture.repo", "mere", "Mere")]),
    );
    let fold = &frozen.folds[0];
    assert_eq!(fold.label.as_deref(), Some("Mere's dependencies"));
    assert_eq!(
        fold.rule.as_deref(),
        Some("Mere and everything it reaches by depends on"),
        "the rule is still recorded"
    );
    assert_eq!(fold.heading(), "Mere, +3 folded: Mere's dependencies");
    assert!(frozen.rows().contains(&(
        "fold".to_owned(),
        "Mere +3".to_owned(),
        "Mere's dependencies".to_owned()
    )));

    let html = frozen.to_html("repos");
    let dom = ScriptedDom::from_serialized_document(&format!(
        "<!doctype html><html><body>{html}</body></html>"
    ));
    let root = dom.document();
    let details = descendants(&dom, root, "details");
    let summary = descendants(&dom, details[0], "summary");
    assert_eq!(
        text(&dom, summary[0]),
        "Mere, +3 folded: Mere's dependencies"
    );
    let members = descendants(&dom, details[0], "li")
        .into_iter()
        .filter_map(|li| attribute(&dom, li, "data-projection-instance"))
        .collect::<Vec<_>>();
    assert_eq!(members, ["1", "2", "3"], "the members are still listed");

    let wire = serde_json::to_string(&frozen).unwrap();
    assert!(wire.contains(r#""label":"Mere's dependencies""#));
    let unlabelled = serde_json::to_string(&FrozenScene::freeze(
        &folded_dependencies(),
        "Repositories",
        &HashMap::new(),
    ))
    .unwrap();
    assert!(!unlabelled.contains("\"label\""), "{unlabelled}");
}

#[test]
fn a_fold_free_freeze_writes_no_fold_key() {
    let frozen = FrozenScene::freeze(&coastal(), "Coastal map", &HashMap::new());
    let wire = serde_json::to_string(&frozen).unwrap();
    assert!(!wire.contains("folds"));
}

#[test]
fn an_unmet_placement_reaches_the_reader_too() {
    // A sighted reader can see a pin sitting in the wrong place. Without
    // this the frozen form is the one realization that cannot say so.
    let mut scene = coastal();
    scene.unmet_holds.push(sceno::HeldPlacement::pinned(
        SourceRef::new("fixture.map", "ghost"),
        Vec2::new(3.0, 4.0),
    ));
    let frozen = FrozenScene::freeze(&scene, "Coastal map", &HashMap::new());
    assert_eq!(frozen.unmet_holds.len(), 1);
    assert!(frozen.summary.contains("could not be placed"));
    let row = frozen
        .rows()
        .into_iter()
        .find(|(kind, _, _)| kind == "unmet placement")
        .expect("the violation is in the tabular alternate");
    assert_eq!(row.1, "ghost");
    assert!(row.2.contains("(3, 4)"));
}

#[test]
fn the_frozen_form_is_a_real_tree_a_host_can_traverse() {
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::LayoutDom;

    let scene = coastal();
    let names = named(&[("fixture.map", "harbor", "Harbor")]);
    let frozen = FrozenScene::freeze(&scene, "Coastal map", &names);
    let html = frozen.to_html("coastal");

    // Parsed, not string-matched: a markup bug that a regex would miss
    // becomes a tree that does not contain what it should.
    let dom = ScriptedDom::from_serialized_document(&format!(
        "<!doctype html><html><body>{html}</body></html>"
    ));
    let serialized = dom.inner_html(dom.document());

    assert!(
        serialized.contains("graphics-document"),
        "document role survives parsing"
    );
    assert!(serialized.contains("graphics-symbol") || serialized.contains("graphics-object"));
    assert!(serialized.contains("Harbor"));
    assert!(serialized.contains("Every item and relationship in this projection"));
    // One row per instance plus the header row.
    assert_eq!(
        serialized.matches("<tr").count(),
        frozen.rows().len() + 1,
        "every row reached the tree"
    );
}

#[test]
fn the_document_names_and_describes_itself() {
    let frozen = FrozenScene::freeze(&coastal(), "Coastal map", &HashMap::new());
    let html = frozen.to_html("coastal");
    // SVG's title/desc pairing, expressed the way ARIA carries it.
    assert!(html.contains("aria-labelledby=\"coastal-name\""));
    assert!(html.contains("aria-describedby=\"coastal-summary\""));
    assert!(html.contains("id=\"coastal-name\">Coastal map<"));
    assert!(html.contains(&frozen.summary));
}

#[test]
fn every_instance_carries_a_role_and_a_label() {
    let frozen = FrozenScene::freeze(&coastal(), "Coastal map", &HashMap::new());
    let html = frozen.to_html("coastal");
    assert_eq!(
        html.matches("aria-label=").count(),
        frozen.instances.len(),
        "no instance reaches a reader unlabelled"
    );
    assert_eq!(
        html.matches("role=\"graphics-").count(),
        frozen.instances.len() + 1
    );
}

#[test]
fn a_hostile_name_cannot_reshape_the_tree() {
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::LayoutDom;

    // Names come from an adapter and may contain anything. A projection
    // whose accessible form can be broken by a node called `<script>` is
    // not accessible, and is a hole besides.
    let mut scene = Scene::new();
    let source = scene.intern_source(SourceRef::new("fixture", "x"));
    scene.items.push(ProjectedItem {
        source,
        space: Scene::WORLD,
        transform: Transform2::translation(0.0, 0.0),
        footprint: Footprint::Point,
        representation: Representation::Glyph,
        layer: 0,
        visible: true,
        hit: None,
        channels: Vec::new(),
    });
    let hostile = "</li></ul><script>alert(1)</script>";
    let frozen = FrozenScene::freeze(&scene, "Hostile", &named(&[("fixture", "x", hostile)]));
    let html = frozen.to_html("h");
    assert!(
        !html.contains("<script>"),
        "the tag never survives as markup"
    );
    assert!(html.contains("&lt;script&gt;"), "it survives as text");

    let dom = ScriptedDom::from_serialized_document(&format!(
        "<!doctype html><html><body>{html}</body></html>"
    ));
    let serialized = dom.inner_html(dom.document());
    assert!(
        !serialized.contains("<script>"),
        "and the parser agrees, which is the check that matters"
    );
}

/// Walk the tree the way an assistive stack does: from the root, through
/// children, collecting labels. If a node is unreachable or unlabelled it
/// simply does not appear, which is the failure this is here to catch.
fn traverse(tree: &uxtree::UxTree) -> Vec<String> {
    use std::collections::HashMap;
    let by_id: HashMap<_, _> = tree.nodes.iter().map(|(id, node)| (*id, node)).collect();
    let mut labels = Vec::new();
    let mut stack = vec![tree.root];
    while let Some(id) = stack.pop() {
        let Some(node) = by_id.get(&id) else { continue };
        if let Some(label) = node.label() {
            labels.push(label.to_string());
        }
        let mut children = node.children().to_vec();
        children.reverse();
        stack.extend(children);
    }
    labels
}

#[test]
fn a_screen_reader_reaches_a_fold_and_its_members() {
    let frozen = FrozenScene::freeze(
        &folded_dependencies(),
        "Repositories",
        &named(&[("fixture.repo", "mere", "Mere")]),
    );
    let labels = traverse(&frozen.to_ux_tree("repos"));
    let at = |label: &str| labels.iter().position(|l| l == label).unwrap();
    let group = at("Mere, +3 folded: Mere and everything it reaches by depends on");
    for member in ["genet", "netrender", "retinue"] {
        assert!(at(member) > group, "{member} is read inside its fold");
    }
    assert!(at("Mere") < group, "the stand-in comes first");
}

#[test]
fn a_screen_reader_traversal_reaches_every_instance_and_relation_by_name() {
    let mut scene = coastal();
    // Give the scene a relation so the traversal has one to find.
    scene.relations.push(sceno::RoutedRelation {
        from: InstanceId(1),
        to: InstanceId(2),
        space: Scene::WORLD,
        points: vec![Vec2::ZERO, Vec2::ZERO],
        kind: Some("sights".to_owned()),
        weight: None,
        link_key: None,
        marks: sceno::LinkMarks::default(),
        strength: None,
    });
    let names = named(&[
        ("fixture.map", "harbor", "Harbor"),
        ("fixture.map", "beacon", "Beacon"),
    ]);
    let frozen = FrozenScene::freeze(&scene, "Coastal map", &names);
    let tree = frozen.to_ux_tree("coastal");
    let labels = traverse(&tree);

    // B1's validation, literally: the traversal enumerates instances and
    // relations with names.
    for instance in &frozen.instances {
        assert!(
            labels.contains(&instance.name),
            "traversal never reached {}",
            instance.name
        );
    }
    assert!(
        labels
            .iter()
            .any(|l| l.contains("Harbor") && l.contains("sights")),
        "the relation is announced with both ends named"
    );
    assert_eq!(
        labels[0], "Coastal map",
        "the document announces itself first"
    );
}

#[test]
fn repeated_source_instances_keep_distinct_accessibility_nodes() {
    use std::collections::HashSet;

    let mut scene = coastal();
    scene.items.push(scene.items[1].clone());
    let frozen = FrozenScene::freeze(&scene, "Repeated source", &HashMap::new());
    let tree = frozen.to_ux_tree("repeated");
    let ids = tree.nodes.iter().map(|(id, _)| *id).collect::<HashSet<_>>();
    assert_eq!(
        ids.len(),
        tree.nodes.len(),
        "two appearances of one source need separate accessibility nodes"
    );
}

#[test]
fn no_node_reaches_a_reader_unlabelled() {
    let frozen = FrozenScene::freeze(&coastal(), "Coastal map", &HashMap::new());
    let tree = frozen.to_ux_tree("coastal");
    for (id, node) in &tree.nodes {
        assert!(
            node.label().is_some(),
            "node {id:?} would be announced as nothing"
        );
    }
}

#[test]
fn an_unmet_placement_is_announced_too() {
    let mut scene = coastal();
    scene.unmet_holds.push(sceno::HeldPlacement::pinned(
        SourceRef::new("fixture.map", "ghost"),
        Vec2::new(3.0, 4.0),
    ));
    let frozen = FrozenScene::freeze(&scene, "Coastal map", &HashMap::new());
    let labels = traverse(&frozen.to_ux_tree("coastal"));
    assert!(
        labels
            .iter()
            .any(|l| l.contains("ghost") && l.contains("could not be placed")),
        "the violation a sighted reader sees is spoken too"
    );
}

#[test]
fn the_tree_is_deterministic_and_root_last() {
    let frozen = FrozenScene::freeze(&coastal(), "Coastal map", &HashMap::new());
    let once = frozen.to_ux_tree("coastal");
    let twice = frozen.to_ux_tree("coastal");
    assert_eq!(
        once.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        twice.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>()
    );
    // UxTree documents that the root is pushed last so a consumer can zip
    // straight into a TreeUpdate; honour that contract.
    assert_eq!(once.nodes.last().expect("a root").0, once.root);
}

/// Build the probe's view of the frozen realization.
///
/// A `ProbeSurface` needs only a parsed DOM, so selector resolution is
/// testable without a frame pump. What this cannot cover is a driven
/// scenario against a live app, which needs a host that renders this
/// realization and owns a winit loop. None exists yet.
fn probe_dom(frozen: &FrozenScene) -> genet_scripted_dom::ScriptedDom {
    genet_scripted_dom::ScriptedDom::from_serialized_document(&format!(
        "<!doctype html><html><body>{}</body></html>",
        frozen.to_html("coastal")
    ))
}

#[test]
fn a_probe_can_reach_every_instance_by_carried_identity() {
    use taproot::{ProbeSurface, Selector, resolve_estimated_fixture};

    let names = named(&[
        ("fixture.map", "harbor", "Harbor"),
        ("fixture.map", "beacon", "Beacon"),
    ]);
    let frozen = FrozenScene::freeze(&coastal(), "Coastal map", &names);
    let dom = probe_dom(&frozen);
    let surfaces = [ProbeSurface {
        name: "frozen",
        dom: &dom,
        rect: [0.0, 0.0, 800.0, 600.0],
        sheet: "",
    }];

    // Identity the DOM carries, which is the only way a probe drives
    // anything: the source id in a data attribute, not a coordinate.
    for instance in &frozen.instances {
        let selector = Selector::role("graphics-symbol")
            .with_attr("data-source-id", instance.source.id.clone());
        let by_symbol = resolve_estimated_fixture(&surfaces, &selector);
        let by_object = resolve_estimated_fixture(
            &surfaces,
            &Selector::role("graphics-object")
                .with_attr("data-source-id", instance.source.id.clone()),
        );
        assert!(
            by_symbol.is_some() || by_object.is_some(),
            "no selector reaches {} by its carried id",
            instance.source.id
        );
    }
}

#[test]
fn a_probe_resolves_an_instance_by_its_announced_name() {
    use taproot::{ProbeSurface, Selector, resolve_estimated_fixture, text_present};

    let names = named(&[("fixture.map", "harbor", "Harbor")]);
    let frozen = FrozenScene::freeze(&coastal(), "Coastal map", &names);
    let dom = probe_dom(&frozen);
    let surfaces = [ProbeSurface {
        name: "frozen",
        dom: &dom,
        rect: [0.0, 0.0, 800.0, 600.0],
        sheet: "",
    }];

    // The aria-label path: a scenario written against what a person hears
    // resolves the same element as one written against the id.
    assert!(
        resolve_estimated_fixture(
            &surfaces,
            &Selector::role("graphics-object").containing("Harbor")
        )
        .is_some()
            || resolve_estimated_fixture(
                &surfaces,
                &Selector::role("graphics-symbol").containing("Harbor")
            )
            .is_some(),
        "the announced name is not selectable"
    );
    assert!(text_present(&surfaces, "Harbor"));
    assert!(
        text_present(&surfaces, &frozen.summary),
        "the long-form alternate is present for a scenario to assert"
    );
}

#[test]
fn two_instances_sharing_a_name_stay_distinguishable() {
    use taproot::{ProbeSurface, Selector, resolve_estimated_fixture};

    // The documented reason data attributes exist: a visible label that is
    // not unique. Two sources, one name.
    let mut scene = Scene::new();
    for id in ["first", "second"] {
        let source = scene.intern_source(SourceRef::new("fixture", id));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(0.0, 0.0),
            footprint: Footprint::Point,
            representation: Representation::Glyph,
            layer: 0,
            visible: true,
            hit: None,
            channels: Vec::new(),
        });
    }
    let frozen = FrozenScene::freeze(
        &scene,
        "Twins",
        &named(&[
            ("fixture", "first", "Example"),
            ("fixture", "second", "Example"),
        ]),
    );
    let dom = probe_dom(&frozen);
    let surfaces = [ProbeSurface {
        name: "frozen",
        dom: &dom,
        rect: [0.0, 0.0, 800.0, 600.0],
        sheet: "",
    }];

    let first = resolve_estimated_fixture(
        &surfaces,
        &Selector::role("graphics-symbol").with_attr("data-source-id", "first"),
    );
    let second = resolve_estimated_fixture(
        &surfaces,
        &Selector::role("graphics-symbol").with_attr("data-source-id", "second"),
    );
    // A Hit is a window-space point, because probe drives pointer input
    // rather than node handles. So what proves distinguishability is that
    // each id-filtered selector resolves at all: a name-only selector
    // could not tell these two apart, and the carried attribute can.
    assert!(first.is_some(), "the first twin is unreachable by its id");
    assert!(second.is_some(), "the second twin is unreachable by its id");
    assert!(
        resolve_estimated_fixture(
            &surfaces,
            &Selector::role("graphics-symbol").with_attr("data-source-id", "third"),
        )
        .is_none(),
        "a selector for an absent id must miss rather than match a sibling"
    );
}

#[test]
fn satisfaction_reads_both_halves_off_the_wire() {
    use scenotime::{Revision, SceneEpoch, SceneSnapshot};

    let mut scene = coastal();
    // Read the held position off the instance rather than naming a
    // coordinate. A snapshot validates that an honored hold agrees with
    // where its instance actually sits, so an invented coordinate makes
    // `from_dense` reject the scene and this test never reaches the
    // counting it exists to check. What is under test is that both halves
    // of satisfaction survive the wire, not where the harbor is.
    let harbor = scene.items[1].transform.translate;
    scene.honored_holds.push(sceno::HonoredHold {
        instance: InstanceId(1),
        placement: sceno::HeldPlacement::pinned(SourceRef::new("fixture.map", "harbor"), harbor),
    });
    scene.unmet_holds.push(sceno::HeldPlacement::pinned(
        SourceRef::new("fixture.map", "ghost"),
        Vec2::new(9.0, 9.0),
    ));
    let snapshot = SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).expect("snapshot");

    let satisfaction = Satisfaction::of(&snapshot.tables);
    assert_eq!(satisfaction.honored, 1);
    assert_eq!(satisfaction.unmet, 1);
    assert_eq!(
        satisfaction.line().as_deref(),
        Some("1 pin unmet, 1 pin held")
    );
    // The distinction a host draws with, rather than infers.
    assert!(Satisfaction::is_pinned(&snapshot.tables, InstanceId(1)));
    assert!(!Satisfaction::is_pinned(&snapshot.tables, InstanceId(0)));
}

#[test]
fn an_unpinned_scene_says_nothing_at_all() {
    use scenotime::{Revision, SceneEpoch, SceneSnapshot};

    let snapshot =
        SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), coastal()).expect("snapshot");
    let satisfaction = Satisfaction::of(&snapshot.tables);
    assert!(satisfaction.is_quiet());
    // Zero of zero is noise, not information.
    assert_eq!(satisfaction.line(), None);
}

#[test]
fn freezing_is_deterministic() {
    let scene = coastal();
    let names = named(&[("fixture.map", "harbor", "Harbor")]);
    let once = FrozenScene::freeze(&scene, "Coastal map", &names);
    let twice = FrozenScene::freeze(&scene, "Coastal map", &names);
    assert_eq!(once, twice, "a receipt that varies is not a receipt");
    assert_eq!(once.rows(), twice.rows());
}

/// A two-reading matrix from the shared derivation, frozen as a grid the
/// way a viewer does: headings and cells keep their scene instances.
fn matrix_grid() -> (scenomise::matrix::Matrix, FrozenGrid) {
    use scenomise::matrix::{CellReading, MatrixAxis, MatrixAxisSource, derive_matrix};

    let axis = |authority: &str, reading: &str, ids: &[(&str, &str)]| MatrixAxis {
        authority: authority.into(),
        record: "rev:1".into(),
        reading: reading.into(),
        focus: None,
        generation: "1".into(),
        sources: ids
            .iter()
            .map(|(id, label)| MatrixAxisSource {
                source: SourceRef::new("fixture.repo", *id),
                label: (*label).into(),
            })
            .collect(),
    };
    let matrix = derive_matrix(
        axis("live", "neighbors", &[("mere", "Mere"), ("genet", "Genet")]),
        axis(
            "specimen",
            "changes",
            &[("genet", "Genet"), ("relay", "Relay <b>")],
        ),
        |row, column| {
            Ok(if row.source.id == "mere" && column.source.id == "genet" {
                CellReading::Relations(vec![scenomise::matrix::MatrixContributor {
                    authority: "live".into(),
                    source: SourceRef::new("fixture.relation", "mere-depends-on-genet"),
                    provenance: "derived".into(),
                }])
            } else {
                CellReading::NoRelation
            })
        },
    )
    .expect("matrix");
    let heading = |instance: InstanceId, entry: &MatrixAxisSource| FrozenGridHeading {
        instance: Some(instance),
        source: entry.source.clone(),
        name: entry.label.clone(),
    };
    let width = matrix.columns.sources.len();
    let grid = FrozenGrid::new(
        format!(
            "Two-reading matrix: {} by {}",
            matrix.rows.reading, matrix.columns.reading
        ),
        "Rows / columns",
        matrix
            .columns
            .sources
            .iter()
            .enumerate()
            .map(|(index, entry)| heading(matrix.column_instance(index), entry))
            .collect(),
        matrix
            .rows
            .sources
            .iter()
            .enumerate()
            .map(|(row, entry)| FrozenGridRow {
                heading: heading(matrix.row_instance(row), entry),
                cells: (0..width)
                    .map(|column| {
                        let index = row * width + column;
                        let cell = &matrix.cells[index];
                        FrozenGridCell {
                            instance: Some(matrix.cell_instance(index)),
                            source: cell.source.clone(),
                            text: cell.value.clone(),
                            description: cell.description.clone(),
                        }
                    })
                    .collect(),
            })
            .collect(),
    )
    .expect("grid");
    (matrix, grid)
}

type Dom = genet_scripted_dom::ScriptedDom;
type Node = <Dom as layout_dom_api::LayoutDom>::NodeId;

fn tag(dom: &Dom, node: Node) -> Option<String> {
    use layout_dom_api::LayoutDom;
    dom.element_name(node).map(|name| name.local.to_string())
}

fn attribute(dom: &Dom, node: Node, key: &str) -> Option<String> {
    use layout_dom_api::LayoutDom;
    dom.attributes(node)
        .find(|attribute| &*attribute.name.local == key)
        .map(|attribute| attribute.value.to_owned())
}

/// Every element named `name` under `root`, in document order.
fn descendants(dom: &Dom, root: Node, name: &str) -> Vec<Node> {
    use layout_dom_api::LayoutDom;
    let mut found = Vec::new();
    for child in dom.dom_children(root).collect::<Vec<_>>() {
        if tag(dom, child).as_deref() == Some(name) {
            found.push(child);
        }
        found.extend(descendants(dom, child, name));
    }
    found
}

fn text(dom: &Dom, node: Node) -> String {
    use layout_dom_api::LayoutDom;
    let mut out = dom.text(node).unwrap_or_default().to_owned();
    for child in dom.dom_children(node).collect::<Vec<_>>() {
        out.push_str(&text(dom, child));
    }
    out
}

/// What a screen reader announces on landing in each data cell of the
/// first table: the cell's row header, its column header (resolved
/// through `scope`, by position, as the HTML table model does), and the
/// cell's accessible name, which is its `aria-label` over its text.
fn announcements(dom: &Dom) -> (String, Vec<(String, String, String, String)>) {
    use layout_dom_api::LayoutDom;
    let table = descendants(dom, dom.document(), "table")[0];
    let caption = text(dom, descendants(dom, table, "caption")[0]);
    let rows = descendants(dom, table, "tr");
    let column_headers = descendants(dom, rows[0], "th")
        .into_iter()
        .map(|header| {
            assert_eq!(attribute(dom, header, "scope").as_deref(), Some("col"));
            text(dom, header)
        })
        .collect::<Vec<_>>();
    let mut announced = Vec::new();
    for row in &rows[1..] {
        let cells = dom
            .dom_children(*row)
            .filter(|child| tag(dom, *child).is_some())
            .collect::<Vec<_>>();
        assert_eq!(tag(dom, cells[0]).as_deref(), Some("th"));
        assert_eq!(attribute(dom, cells[0], "scope").as_deref(), Some("row"));
        let row_header = text(dom, cells[0]);
        for (position, cell) in cells.iter().enumerate().skip(1) {
            assert_eq!(tag(dom, *cell).as_deref(), Some("td"));
            announced.push((
                row_header.clone(),
                column_headers[position].clone(),
                attribute(dom, *cell, "aria-label").unwrap_or_else(|| text(dom, *cell)),
                text(dom, *cell),
            ));
        }
    }
    (caption, announced)
}

#[test]
fn a_matrix_grid_reads_as_a_grid_to_a_screen_reader() {
    let (matrix, grid) = matrix_grid();
    let dom = Dom::from_serialized_document(&format!(
        "<!doctype html><html><body>{}</body></html>",
        grid.to_html("matrix")
    ));
    let (caption, announced) = announcements(&dom);
    assert_eq!(caption, "Two-reading matrix: neighbors by changes");
    // Every crossing is reachable, once, with both of its headers.
    assert_eq!(announced.len(), matrix.cells.len());
    let expected = matrix
        .cells
        .iter()
        .enumerate()
        .map(|(index, cell)| {
            (
                matrix.rows.sources[index / 2].label.clone(),
                matrix.columns.sources[index % 2].label.clone(),
                cell.description.clone(),
                cell.value.clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(announced, expected);
    // An absence is stated, never blank.
    assert!(
        announced
            .iter()
            .all(|(_, _, _, shown)| !shown.trim().is_empty())
    );
    assert_eq!(
        announced[1],
        (
            "Mere".into(),
            "Relay <b>".into(),
            "No direct relation from Mere to Relay <b>".into(),
            "no relation".into()
        )
    );
    assert_eq!(announced[0].3, "relation");
    assert_eq!(announced[2].3, "same source");
}

#[test]
fn a_matrix_grid_carries_instance_and_source_identity() {
    let (matrix, grid) = matrix_grid();
    let html = grid.to_html("matrix");
    let dom =
        Dom::from_serialized_document(&format!("<!doctype html><html><body>{html}</body></html>"));
    let cells = descendants(&dom, layout_dom_api::LayoutDom::document(&dom), "td");
    for (index, cell) in cells.iter().enumerate() {
        assert_eq!(
            attribute(&dom, *cell, "data-projection-instance"),
            Some(matrix.cell_instance(index).0.to_string())
        );
        assert_eq!(
            attribute(&dom, *cell, "data-source-id"),
            Some(matrix.cells[index].source.id.clone())
        );
    }
    assert!(html.contains("id=\"matrix-caption\""));
    assert!(!html.contains("<b>"), "a hostile label stays text");
}

#[test]
fn a_grid_refuses_ragged_rows_and_blank_cells() {
    let (_, grid) = matrix_grid();
    let mut ragged = grid.rows.clone();
    ragged[1].cells.pop();
    assert_eq!(
        FrozenGrid::new("Caption", "Corner", grid.columns.clone(), ragged),
        Err(FrozenGridError::Ragged {
            row: 1,
            cells: 1,
            columns: 2
        })
    );
    let mut blank = grid.rows.clone();
    blank[0].cells[1].text = " ".into();
    assert_eq!(
        FrozenGrid::new("Caption", "Corner", grid.columns.clone(), blank),
        Err(FrozenGridError::BlankCell { row: 0, column: 1 })
    );
    assert_eq!(
        FrozenGrid::new("", "Corner", grid.columns.clone(), grid.rows.clone()),
        Err(FrozenGridError::Unlabelled)
    );
    let mut unnamed = grid.columns.clone();
    unnamed[0].name.clear();
    assert!(matches!(
        FrozenGrid::new("Caption", "Corner", unnamed, grid.rows),
        Err(FrozenGridError::BlankHeading { .. })
    ));
}
