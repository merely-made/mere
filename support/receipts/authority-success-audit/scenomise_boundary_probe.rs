//! Bounded public API observations, not repaired-behavior tests.
use std::collections::BTreeSet;
use std::sync::Arc;
use sceno::{Size2, SourceRef, Vec2};
use scenomise::history::Change;
use scenomise::host_dataset::{parse_host_history, route_relationships};
use scenomise::projection::{DisclosedRelationship, ItemSizes, ProjectionCompiler, ProjectionDataset};
use serde_json::{Value, json};

fn binding(owner: &str) -> Value {
    json!({"authority":owner,"domain":"fixture","resource":"one"})
}
fn dataset(owner: &str, revision: &str) -> ProjectionDataset {
    let occurrence = |id: &str, x: f64| json!({
        "occurrence_id":id,"source":{"adapter":owner,"id":id},
        "values":{"id":{"kind":"text","value":id},"label":{"kind":"text","value":id},
            "x":{"kind":"number","value":x},"y":{"kind":"number","value":0.0}}
    });
    serde_json::from_value(json!({"source":binding(owner),"revision":revision,
        "fields":{"id":"text","label":"text","x":"number","y":"number"},
        "occurrences":[occurrence("a",0.0),occurrence("b",1.0)]})).unwrap()
}
fn relationship(owner: &str, revision: &str, label: &str, explanation: &str) -> DisclosedRelationship {
    serde_json::from_value(json!({"id":"edge","from_occurrence":"a","to_occurrence":"b",
        "kind":"links","label":label,"explanation":explanation,
        "provenance":{"source":binding(owner),"source_revision":revision,"method":"host.links",
            "method_version":1,"provider":"fixture","evidence":[{"adapter":owner,"id":"a"}]}})).unwrap()
}
fn compiler() -> ProjectionCompiler {
    ProjectionCompiler::new(ItemSizes { card: Size2::new(10.0,10.0) })
}
fn definition(data: &ProjectionDataset, kind: &str) -> scenograph::ProjectionDefinition {
    serde_json::from_value(json!({"version":1,"id":"view","label":"View","source":data.source,
        "reading":{"kind":"nodes","key":"id","value":null},
        "encoding":{"x":{"kind":"field","value":"x"},"y":{"kind":"field","value":"y"},
            "color":null,"label":{"kind":"field","value":"label"}},
        "arrangement":{"kind":kind,"direction":"coordinates","spacing":16,"options":{}},
        "interaction":{"selection":"single","pan":false,"zoom":false},
        "appearance":{"realization":"canvas","title":"View","theme":"slate"},
        "provenance":{"author":"fixture","source_revision":data.revision,"note":""}})).unwrap()
}
fn history(before: (&str,&str), after: (&str,&str)) -> scenomise::host_dataset::HostDatasetV2 {
    let revision = |sequence: u64, revision: &str, (label, explanation): (&str,&str)| json!({
        "sequence":sequence,"revision":revision,"dataset":dataset("owner-a",revision),
        "relationships":[relationship("owner-a",revision,label,explanation)]
    });
    parse_host_history(&json!({"schema":"scenomise.host-dataset/v2",
        "revisions":[revision(1,"r1",before),revision(2,"r2",after)]}).to_string()).unwrap()
}

#[test]
fn ordinary_history_change_control() {
    let h=history(("old","explanation"),("new","explanation"));
    assert_eq!(h.changes().relationships[0].change,Change::Updated);
    assert!(h.changes().occurrences.iter().all(|o| o.change==Change::Updated));
    let h=history(("same","same"),("same","same"));
    assert_eq!(h.changes().relationships[0].change,Change::Stable);
}

#[test]
fn parsed_relationship_text_can_hide_a_change_at_signature_separators() {
    let h=history(("a\u{1f}\u{1f}b","c"),("a","b\u{1f}\u{1f}c"));
    assert_ne!(h.revisions[0].relationships[0].label,h.revisions[1].relationships[0].label);
    assert_ne!(h.revisions[0].relationships[0].explanation,h.revisions[1].relationships[0].explanation);
    assert_eq!(h.changes().relationships[0].change,Change::Stable);
    assert!(h.changes().occurrences.iter().all(|o| o.change==Change::Stable));
    println!("Strictly parsed history contains changed label/explanation, yet reports the relationship and both endpoints stable");
}

#[test]
fn history_admits_duplicate_occurrences_that_the_compiler_rejects() {
    let mut before=dataset("owner-a","r1");
    let mut duplicate=before.occurrences[0].clone();
    duplicate.values.insert("label".into(),scenomise::projection::ProjectionValue::Text("replacement".into()));
    before.occurrences.push(duplicate);
    assert!(compiler().compile(&definition(&before,"grid"),&before).unwrap_err().iter()
        .any(|issue|issue.field.ends_with("occurrence_id")));
    let mut after=dataset("owner-a","r2");
    after.occurrences[0].values.insert("label".into(),scenomise::projection::ProjectionValue::Text("replacement".into()));
    let h=parse_host_history(&json!({"schema":"scenomise.host-dataset/v2","revisions":[
        {"sequence":1,"revision":"r1","dataset":before,"relationships":[]},
        {"sequence":2,"revision":"r2","dataset":after,"relationships":[]}
    ]}).to_string()).unwrap();
    assert_eq!(h.revisions[0].dataset.occurrences.len(),3);
    assert!(h.changes().occurrences.iter().all(|occurrence|occurrence.change==Change::Stable));
    println!("History parsing admits duplicate occurrence IDs that compile rejects; comparison keeps the last predecessor and reports all current items stable");
}

#[test]
fn routing_accepts_another_source_with_the_same_revision_and_occurrence_ids() {
    let a=dataset("owner-a","r1");
    let b=dataset("owner-b","r1");
    let mut same=compiler().compile(&definition(&a,"grid"),&a).unwrap();
    assert!(route_relationships(&mut same,&a,&[relationship("owner-a","r1","edge","evidence")]).is_ok());
    let mut foreign=compiler().compile(&definition(&a,"grid"),&a).unwrap();
    let b_projection=compiler().compile(&definition(&b,"grid"),&b).unwrap();
    assert_eq!(foreign.score.generation,b_projection.score.generation);
    let rel=relationship("owner-b","r1","edge","evidence");
    let routed=route_relationships(&mut foreign,&b,&[rel]).unwrap();
    assert_eq!(routed[0].disclosure.provenance.source,b.source);
    assert!(foreign.scene.sources.iter().all(|source|source.adapter=="owner-a"));
    let later=dataset("owner-b","r2");
    assert!(route_relationships(&mut foreign,&later,&[relationship("owner-b","r2","edge","evidence")]).is_err());
    println!("Foreign owner-b relationships route onto an owner-a projection; different-revision control refuses");
}

#[test]
fn coordinate_validation_uses_spacing_instead_of_the_overridden_scale() {
    let mut data=dataset("owner-a","r1");
    data.occurrences[1].values.insert("x".into(),scenomise::projection::ProjectionValue::Number(1e6));
    let mut recipe=definition(&data,"geographic");
    let ordinary=compiler().compile(&recipe,&data).unwrap();
    assert!(ordinary.scene.items.iter().all(|i|i.transform.translate.x.is_finite()));
    recipe.arrangement.options.insert("units_per_coordinate".into(),"1e38".into());
    let compiled=compiler().compile(&recipe,&data).unwrap();
    assert!(compiled.scene.items[compiled.instance_by_occurrence["b"].0 as usize].transform.translate.x.is_infinite());
    println!("Finite source coordinate and finite accepted scale return Ok with infinite geometry");
}

#[test]
fn relaxation_moves_items_but_not_their_existing_routes() {
    let data=dataset("owner-a","r1");
    let mut projection=compiler().compile(&definition(&data,"grid"),&data).unwrap();
    route_relationships(&mut projection,&data,&[relationship("owner-a","r1","edge","evidence")]).unwrap();
    let mut scene=projection.scene;
    let before=scene.clone();
    scenomise::relax(&mut scene,&scenomise::Relaxation { steps:0,..Default::default() });
    assert_eq!(scene,before);
    scenomise::relax(&mut scene,&scenomise::Relaxation { steps:1,spring:0.0,..Default::default() });
    assert_ne!(scene.items[0].transform.translate,before.items[0].transform.translate);
    assert_eq!(scene.relations,before.relations);
    assert_ne!(scene.relations[0].points[0],scene.items[scene.relations[0].from.0 as usize].transform.translate);
    println!("One bounded relax step moves endpoint items; the original routed polyline stays behind");
}

#[test]
fn relaxation_discards_footprint_bounds_even_when_no_item_moves() {
    let data=dataset("owner-a","r1");
    let mut data=data;
    data.occurrences.truncate(1);
    let mut scene=compiler().compile(&definition(&data,"grid"),&data).unwrap().scene;
    assert_eq!(scene.bounds.size,Size2::new(10.0,10.0));
    let before=scene.items.clone();
    scenomise::relax(&mut scene,&scenomise::Relaxation { steps:1,repulsion:0.0,spring:0.0,dt:0.0,..Default::default() });
    assert_eq!(scene.items,before);
    assert_eq!(scene.bounds.size,Size2::new(0.0,0.0));
    println!("Unmoved 10x10 item retains its footprint while scene bounds become 0x0");
}

struct Offset(f32);
impl scenomise::Solver for Offset {
    fn capability(&self)->scenomise::SolverCapability { scenomise::SolverCapability::new("host:offset","Offset") }
    fn place(&self,_:&Value,items:&[&sceno::ScoreItem])->Result<Vec<Vec2>,scenomise::SolveError> {
        Ok((0..items.len()).map(|i|Vec2::new(self.0+i as f32,0.0)).collect())
    }
}
#[test]
fn refresh_reuses_geometry_from_a_different_solver_registration() {
    let make=|offset| {
        let mut registry=scenomise::SolverRegistry::new();
        registry.register(Arc::new(Offset(offset))).unwrap();
        ProjectionCompiler::with_registry(ItemSizes {card:Size2::new(10.0,10.0)},registry)
    };
    let data=dataset("owner-a","r1");
    let recipe=definition(&data,"host:offset");
    let first=make(0.0).compile(&recipe,&data).unwrap();
    let second=make(100.0);
    assert_eq!(second.compile(&recipe,&data).unwrap().scene.items[0].transform.translate.x,100.0);
    let refreshed=second.refresh(&first,&recipe,&data).unwrap();
    assert!(refreshed.placement_reused);
    assert_eq!(refreshed.scene.items[0].transform.translate.x,0.0);
    println!("Qualified cache boundary: a different registration for the same custom ID reuses the previous compiler's geometry");
}

#[test]
fn entered_focus_uses_a_visibility_set_beyond_portable_fold_hiding() {
    use scenomise::host_dataset::folds::{FoldViewState,HostDatasetFolds};
    let mut data=dataset("owner-a","r1");
    let mut unrelated=data.occurrences[0].clone();
    unrelated.occurrence_id="unrelated".into();
    unrelated.source=SourceRef::new("owner-a","unrelated");
    unrelated.values.insert("id".into(),scenomise::projection::ProjectionValue::Text("unrelated".into()));
    data.occurrences.push(unrelated);
    let compiled=compiler().compile(&definition(&data,"grid"),&data).unwrap();
    let relation=relationship("owner-a","r1","edge","evidence");
    let host=HostDatasetFolds::new(&data,&[relation],"links",compiled.scene,compiled.instance_by_occurrence).unwrap();
    let projection=host.project(&FoldViewState { entered:Some("a".into()),expanded:BTreeSet::new() }).unwrap();
    let id=host.instances()["unrelated"];
    assert!(!projection.visible.contains("unrelated"));
    assert!(projection.scene.fold_effect().is_shown(id,true));
    println!("Qualified focus boundary: entered visible set omits unrelated, while the complete Scene fold effect still shows it");
}

#[test]
fn matrix_derived_sources_alias_distinct_adapters() {
    use scenomise::matrix::*;
    let axis=|reading:&str,sources:Vec<MatrixAxisSource>|MatrixAxis { authority:reading.into(),record:"r1".into(),
        reading:reading.into(),focus:None,generation:"1".into(),sources };
    let matrix=derive_matrix(axis("rows",vec![
        MatrixAxisSource {source:SourceRef::new("adapter-a","same"),label:"A".into()},
        MatrixAxisSource {source:SourceRef::new("adapter-b","same"),label:"B".into()},
    ]),axis("columns",vec![MatrixAxisSource {source:SourceRef::new("adapter-c","column"),label:"C".into()}]),
        |row,_|Ok(CellReading::Value(row.source.adapter.clone()))).unwrap();
    assert_ne!(matrix.cells[0].row,matrix.cells[1].row);
    assert_ne!(matrix.cells[0].value,matrix.cells[1].value);
    assert_eq!(matrix.cells[0].source,matrix.cells[1].source);
    let scene=matrix.scene(1);
    assert_eq!(scene.items[matrix.cell_instance(0).0 as usize].source,scene.items[matrix.cell_instance(1).0 as usize].source);
    println!("Qualified identity boundary: distinct adapter-backed value cells share one derived SourceRef and interned source");
}
