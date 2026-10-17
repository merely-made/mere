//! Unchanged document-host URL gate compared with retained url2.5.8 parsing.
#[path = "../../../crates/script/document-host/src/net.rs"]
#[allow(dead_code)]
mod net;

fn main() {
    let origins = vec!["allowed.test".to_owned()];
    let ordinary = "https://allowed.test/ok";
    assert!(net::host_in_origins(ordinary, &origins));
    assert_eq!(url::Url::parse(ordinary).unwrap().host_str(), Some("allowed.test"));
    assert!(!net::host_in_origins("https://outside.test/no", &origins));
    println!("PASS ordinary allowed and denied host controls");

    let input = r"https://outside.test\@allowed.test/private";
    let parsed = url::Url::parse(input).unwrap();
    assert_eq!(net::net_host_of(input), "allowed.test");
    assert!(net::host_in_origins(input, &origins));
    assert_eq!(parsed.host_str(), Some("outside.test"));
    assert!(!net::host_in_origins(parsed.as_str(), &origins));
    println!("PASS gate/parser disagreement: raw admitted host={} parsed host={} normalized request refused", net::net_host_of(input), parsed.host_str().unwrap());

    assert!(net::host_in_origins("http://allowed.test:8123/", &origins));
    println!("OBSERVATION host allowlist also admits a different scheme and port; this is its stated host policy, not an additional parser-bypass case");
}
