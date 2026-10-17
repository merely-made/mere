use mere::kernel::graph::{Graph, apply, RelationKind, SemanticSubKind};
use mere::{glossary, roster};

#[test]
fn corrupt_subgraph_sidecar_loads_as_fresh_and_can_overwrite_prior_definitions() {
    use mere::subgraph::{SessionSubgraphs, SUBGRAPHS_FILE};
    let root=std::env::current_dir().unwrap();
    let dir=root.join("support/receipts/authority-success-audit/mere-sidecar-fixture");
    assert!(!dir.exists());
    std::fs::create_dir(&dir).unwrap();
    let path=dir.join(SUBGRAPHS_FILE);
    let mut index=SessionSubgraphs::new().with_default_session();
    let member=apply_seed_id();
    let branch=index.record_branch(member,mere::subgraph::default_spec_for(member));
    index.save(&dir).unwrap();
    let valid=std::fs::read(&path).unwrap();
    assert_eq!(SessionSubgraphs::load(&dir).subgraphs().len(),2);
    std::fs::write(&path,&valid[..valid.len()/2]).unwrap();
    let corrupt=std::fs::read(&path).unwrap();
    let loaded=SessionSubgraphs::load(&dir);
    assert_eq!(loaded.subgraphs().len(),1);
    assert!(loaded.get(branch).is_none());
    assert_eq!(std::fs::read(&path).unwrap(),corrupt,"load alone leaves bytes intact");
    loaded.save(&dir).unwrap();
    assert_ne!(std::fs::read(&path).unwrap(),corrupt);
    assert_eq!(SessionSubgraphs::load(&dir).subgraphs().len(),1);
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(&dir).unwrap();
    println!("Valid sidecar retains two definitions; truncated sidecar silently loads one default; subsequent save replaces the corrupt file without a refusal.");
}

fn apply_seed_id() -> mere::forme::GraphMemberId {
    let mut graph=Graph::new();
    let key=apply::add_node(&mut graph,None,"https://sidecar.test/seed".into(),Default::default());
    graph.get_node(key).unwrap().id
}

#[test]
fn url_structure_outline_discards_distinct_query_scheme_and_fragment_nodes() {
    let cases=[
        ["https://example.test/search?q=one","https://example.test/search?q=two"],
        ["http://example.test/page","https://example.test/page"],
        ["https://example.test/page#one","https://example.test/page#two"],
    ];
    for addresses in cases {
        let mut graph=Graph::new();
        for address in addresses { apply::add_node(&mut graph,None,address.into(),Default::default()); }
        assert_eq!(graph.node_count(),2);
        let rows=glossary::outline_rows(&graph);
        let linked:Vec<_>=rows.iter().filter_map(|row|row.url.as_deref()).collect();
        assert_eq!(linked.len(),1);
        assert_eq!(glossary::outline_djot(&graph).matches("](").count(),1);
        println!("Graph nodes2, outline links1 for {addresses:?}: {linked:?}");
    }
    let mut control=Graph::new();
    for address in ["https://example.test/one","https://example.test/two"] { apply::add_node(&mut control,None,address.into(),Default::default()); }
    assert_eq!(glossary::outline_rows(&control).iter().filter(|r|r.url.is_some()).count(),2);
}

#[test]
fn equal_titles_interleave_distinct_relation_bundles() {
    let mut graph=Graph::new();
    let ids:Vec<_>=(0..4).map(|i| {
        let key=apply::add_node(&mut graph,None,format!("https://example.test/{i}"),Default::default());
        graph.get_node(key).unwrap().id
    }).collect();
    let input=|bundle:usize,kind,id:&str| roster::LinkRowInput {
        from:ids[bundle*2],to:ids[bundle*2+1],assertion_id:id.into(),
        source_title:"same source".into(),source_url:format!("source{bundle}"),
        target_title:"same target".into(),target_url:format!("target{bundle}"),
        kind:RelationKind::Semantic(kind),source_label:None,editable:false,selected:false,
    };
    let rows=roster::build_link_rows(vec![
        input(0,SemanticSubKind::Cites,"a-cites"),input(0,SemanticSubKind::Quotes,"a-quotes"),
        input(1,SemanticSubKind::Cites,"b-cites"),input(1,SemanticSubKind::Quotes,"b-quotes"),
    ]);
    assert_eq!(rows.iter().map(|r|r.from).collect::<Vec<_>>(),vec![ids[0],ids[2],ids[0],ids[2]]);
    assert_eq!(rows.iter().filter(|r|r.starts_bundle).count(),4);
    let mut controls=vec![input(0,SemanticSubKind::Cites,"a-cites"),input(0,SemanticSubKind::Quotes,"a-quotes"),input(1,SemanticSubKind::Cites,"b-cites"),input(1,SemanticSubKind::Quotes,"b-quotes")];
    for row in &mut controls { row.source_title=format!("source {}",row.source_url); }
    assert_eq!(roster::build_link_rows(controls).iter().filter(|r|r.starts_bundle).count(),2);
    println!("Two endpoint bundles with equal titles become four bundle starts; distinct-title control has two.");
}
