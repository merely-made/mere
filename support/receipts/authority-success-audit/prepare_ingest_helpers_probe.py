from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
source=ROOT/'crates/graph/linked-data/src/ingest.rs';text=source.read_text();start=text.index('pub fn referenced_context_urls(');end=text.index('const RDF_REIFIES:',start);code=text[start:end]
policy_path=ROOT/'crates/distillery/alembic/src/memory_levels.rs';policy=policy_path.read_text();start=policy.index('    pub fn cycled(');end=policy.index('\n    /// Whether `node`',start);method=policy[start:end]
fixture='''// Source-extracted JSON context scanner and exact policy method; policy enum storage is a local matching fixture.
'''+code+'''
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum EvictionPolicy { KeepForever, KeepDays(u32), KeepSessions(u32) }
impl EvictionPolicy {
'''+method+'''}
#[test]
fn top_level_and_graph_contexts_are_found_while_ordinary_iri_fields_are_ignored() {
    let input=br#"{"@context":"https://ctx.test/root","@graph":[{"@context":"https://ctx.test/graph","url":"https://not-a-context.test/"}]}"#;
    assert_eq!(referenced_context_urls(input), ["https://ctx.test/graph","https://ctx.test/root"]);
    assert!(referenced_context_urls(b"not JSON").is_empty());
}
#[test]
fn a_nested_context_inside_an_inline_term_definition_is_omitted() {
    let input=br#"{"@context":{"child":{"@id":"urn:child","@context":"https://ctx.test/scoped"}},"@id":"urn:subject"}"#;
    assert!(referenced_context_urls(input).is_empty());
}
#[test]
fn the_same_nested_context_is_omitted_inside_a_context_array() {
    let input=br#"{"@context":["https://ctx.test/root",{"child":{"@id":"urn:child","@context":"https://ctx.test/scoped"}}]}"#;
    assert_eq!(referenced_context_urls(input), ["https://ctx.test/root"]);
}
#[test]
fn ordinary_eviction_rungs_work_but_arbitrary_values_below_seven_skip_the_next_rung() {
    use EvictionPolicy::*;
    assert_eq!(KeepDays(7).cycled(), KeepDays(30));
    assert_eq!(KeepDays(30).cycled(), KeepDays(90));
    assert_eq!(KeepDays(90).cycled(), KeepForever);
    assert_eq!(KeepForever.cycled(), KeepDays(7));
    assert_eq!(KeepSessions(3).cycled(), KeepDays(7));
    for days in 0..7 { assert_eq!(KeepDays(days).cycled(), KeepDays(30)); }
}
'''
with (HERE/'ingest_helpers_probe.rs').open('x',encoding='utf-8') as f:f.write(fixture)
with (HERE/'ingest_helpers_extraction.json').open('x',encoding='utf-8') as f:f.write(json.dumps({'source_sha256':{source.relative_to(ROOT).as_posix():hashlib.sha256(source.read_bytes()).hexdigest(),policy_path.relative_to(ROOT).as_posix():hashlib.sha256(policy_path.read_bytes()).hexdigest()},'extracted_code_sha256':[hashlib.sha256(c.encode()).hexdigest() for c in [code,method]],'qualification':'Exact original scanner functions and cycled method; local enum fixture uses matching variants without serde derives. No whole linked-data/Alembic feature, kernel, JSON-LD expansion or network execution.'},indent=2)+'\n')
