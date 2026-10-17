// Source-extracted JSON context scanner and exact policy method; policy enum storage is a local matching fixture.
pub fn referenced_context_urls(body: &[u8]) -> Vec<String> {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return Vec::new();
    };
    let mut urls = Vec::new();
    collect_context_urls(&value, &mut urls);
    urls.sort();
    urls.dedup();
    urls
}

/// Walk every object for an `@context`, collecting its remote string references.
fn collect_context_urls(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(ctx) = map.get("@context") {
                collect_context_strings(ctx, out);
            }
            for (key, child) in map {
                if key != "@context" {
                    collect_context_urls(child, out);
                }
            }
        },
        serde_json::Value::Array(items) => items.iter().for_each(|i| collect_context_urls(i, out)),
        _ => {},
    }
}

/// A `@context` value is a URL string, an array (URL strings + inline objects), or
/// an inline object. Collect only the remote (`http(s)`) string references.
fn collect_context_strings(ctx: &serde_json::Value, out: &mut Vec<String>) {
    match ctx {
        serde_json::Value::String(s) if s.starts_with("http://") || s.starts_with("https://") => {
            out.push(s.clone());
        },
        serde_json::Value::Array(items) => {
            items.iter().for_each(|i| collect_context_strings(i, out))
        },
        _ => {},
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum EvictionPolicy { KeepForever, KeepDays(u32), KeepSessions(u32) }
impl EvictionPolicy {
    pub fn cycled(self) -> EvictionPolicy {
        match self {
            EvictionPolicy::KeepDays(d) if d < 30 => EvictionPolicy::KeepDays(30),
            EvictionPolicy::KeepDays(d) if d < 90 => EvictionPolicy::KeepDays(90),
            EvictionPolicy::KeepDays(_) => EvictionPolicy::KeepForever,
            EvictionPolicy::KeepForever => EvictionPolicy::KeepDays(7),
            EvictionPolicy::KeepSessions(_) => EvictionPolicy::KeepDays(7),
        }
    }
}
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
