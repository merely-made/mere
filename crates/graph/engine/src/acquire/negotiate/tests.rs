use super::link::{JSON_LD_CONTEXT_REL, discover, parse_links};
use super::*;
use oxrdfio::JsonLdProfileSet;
const JSONLD: RdfFormat = RdfFormat::JsonLd {
    profile: JsonLdProfileSet::empty(),
};

#[test]
fn accept_default_lists_every_format_with_rdfxml_lowest() {
    assert_eq!(
        AcceptPolicy::default().header_value(),
        "text/turtle, application/ld+json, application/trig;q=0.9, \
         application/n-quads;q=0.9, application/n-triples;q=0.8, application/rdf+xml;q=0.5"
    );
}

#[test]
fn accept_narrows_and_formats_q() {
    let mut policy = AcceptPolicy::default().narrowed_to(&[RdfFormat::Turtle, RdfFormat::RdfXml]);
    assert_eq!(
        policy.header_value(),
        "text/turtle, application/rdf+xml;q=0.5"
    );
    policy.entries[1].q_milli = 125;
    policy.entries[0].q_milli = 5000;
    assert_eq!(
        policy.header_value(),
        "text/turtle, application/rdf+xml;q=0.125"
    );
}

fn parse(bytes: &[u8], ct: &str) -> usize {
    parse_response(bytes, ct, Some("http://x/"), ContextCache::minimal())
        .unwrap()
        .len()
}

#[test]
fn dispatch_every_format_with_parameters() {
    assert_eq!(
        parse(b"<a> <http://p/> <b> .", "text/turtle; charset=UTF-8"),
        1
    );
    assert_eq!(parse(b"<a> <http://p/> <b> .", "TEXT/Turtle"), 1);
    assert_eq!(
        parse(b"<g> { <a> <http://p/> <b> . }", "application/trig"),
        1
    );
    assert_eq!(
        parse(
            b"<http://a/> <http://p/> <http://b/> .",
            "application/n-triples"
        ),
        1
    );
    assert_eq!(
        parse(
            b"<http://a/> <http://p/> <http://b/> <http://g/> .",
            "application/n-quads"
        ),
        1
    );
    let jsonld = br#"{"@id":"http://a/","http://p/":{"@id":"http://b/"}}"#;
    assert_eq!(
        parse(
            jsonld,
            r#"application/ld+json; profile="http://www.w3.org/ns/json-ld#expanded""#
        ),
        1
    );
    let xml = br#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
        xmlns:e="http://e/"><rdf:Description rdf:about="http://a/"><e:p>v</e:p></rdf:Description></rdf:RDF>"#;
    assert_eq!(parse(xml, "application/rdf+xml;charset=utf-8"), 1);
}

#[test]
fn content_type_parameters_and_case() {
    let ct =
        r#"Application/LD+JSON ; Profile="http://www.w3.org/ns/json-ld#expanded" ; charset=UTF-8"#;
    assert!(matches!(
        parse_content_type(ct),
        Ok(RdfFormat::JsonLd { .. })
    ));
    assert_eq!(
        parse_content_type("TEXT/Turtle;charset=utf-8").unwrap(),
        RdfFormat::Turtle
    );
}

#[test]
fn unknown_type_and_charset_are_errors() {
    for ct in [
        "application/json",
        "text/plain",
        "application/xml",
        "text/n3",
        "text/html",
        "",
    ] {
        assert!(matches!(
            parse_response(b"{}", ct, None, ContextCache::minimal()),
            Err(NegotiateError::UnsupportedMediaType(_))
        ));
    }
    assert!(matches!(
        parse_response(
            b"",
            "text/turtle; charset=iso-8859-1",
            None,
            ContextCache::minimal()
        ),
        Err(NegotiateError::UnsupportedMediaType(_))
    ));
    assert!(matches!(
        parse_response(
            b"<a> <b",
            "text/turtle",
            Some("http://x/"),
            ContextCache::minimal()
        ),
        Err(NegotiateError::Parse(_))
    ));
}

#[test]
fn rfc8288_examples() {
    // Section 3.5.
    let one = parse_links(
        r#"<http://example.com/TheBook/chapter2>; rel="previous"; title="previous chapter""#,
    );
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].target, "http://example.com/TheBook/chapter2");
    assert_eq!(one[0].param("title"), Some("previous chapter"));
    assert!(one[0].has_rel("previous"));

    let root = parse_links(r#"</>; rel="http://example.net/foo""#);
    assert_eq!(root[0].target, "/");
    assert!(root[0].has_rel("http://example.net/foo"));

    let ext = parse_links(
        r#"</TheBook/chapter2>; rel="previous"; title*=UTF-8'de'letztes%20Kapitel, </TheBook/chapter4>; rel="next"; title*=UTF-8'de'n%c3%a4chstes%20Kapitel"#,
    );
    assert_eq!(ext.len(), 2);
    assert_eq!(ext[0].param("title*"), Some("UTF-8'de'letztes%20Kapitel"));
    assert!(ext[1].has_rel("next"));

    let multi =
        parse_links(r#"<http://example.org/>; rel="start http://example.net/relation/other""#);
    assert!(multi[0].has_rel("start") && multi[0].has_rel("http://example.net/relation/other"));
}

#[test]
fn quoted_commas_and_malformed_values() {
    let links =
        parse_links(r#"<a>; title="x, <y>, z"; rel=next, junk, <b>; rel="prev"; rel="ignored""#);
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].param("title"), Some("x, <y>, z"));
    assert!(links[0].has_rel("next"));
    assert!(links[1].has_rel("prev") && !links[1].has_rel("ignored"));
    assert!(parse_links("").is_empty());
}

#[test]
fn discovery_resolves_against_the_response_url() {
    let header = concat!(
        r#"<data.jsonld>; rel="alternate"; type="application/ld+json", "#,
        r#"</meta/doc.ttl>; rel="alternate describedby"; type="text/turtle; charset=utf-8", "#,
        r#"<page.html>; rel=alternate; type="text/html", "#,
        r#"<https://other.example/ctx.jsonld>; rel="http://www.w3.org/ns/json-ld#context"; type="application/ld+json""#,
    );
    let found = discover(header, "http://ex.org/a/thing");
    assert_eq!(
        found
            .alternates
            .iter()
            .map(|a| (a.iri.as_str(), a.format))
            .collect::<Vec<_>>(),
        [
            ("http://ex.org/a/data.jsonld", JSONLD),
            ("http://ex.org/meta/doc.ttl", RdfFormat::Turtle)
        ]
    );
    assert_eq!(found.described_by, ["http://ex.org/meta/doc.ttl"]);
    assert_eq!(found.json_ld_contexts, ["https://other.example/ctx.jsonld"]);
    assert_eq!(JSON_LD_CONTEXT_REL, "http://www.w3.org/ns/json-ld#context");
    assert_eq!(
        discover("<x>; rel=alternate", "not a base"),
        Default::default()
    );
}

#[test]
fn turtle_body_with_a_jsonld_alternate_end_to_end() {
    let url = "http://ex.org/doc/thing";
    let body = b"<thing> <http://p/> <other> , <more> .";
    let dataset = parse_response(
        body,
        "text/turtle;charset=utf-8",
        Some(url),
        ContextCache::minimal(),
    )
    .unwrap();
    assert_eq!(dataset.len(), 2);
    let found = discover(
        r#"<thing.jsonld>; rel="alternate"; type="application/ld+json""#,
        url,
    );
    assert_eq!(
        found.alternates,
        [link::Alternate {
            iri: "http://ex.org/doc/thing.jsonld".into(),
            format: JSONLD
        }]
    );
}
