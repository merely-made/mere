use graphshell_endpoint::door::projection_offer::{PROJECTION_PROVIDER_ROUTE, ProviderAnswer};
pub fn fixture() -> String {
    assert!(!PROJECTION_PROVIDER_ROUTE.is_empty());
    serde_json::to_string(&ProviderAnswer::Idle).unwrap()
}

#[cfg(feature = "forbidden-api")]
pub use djinn::door::app_client::AppConnection;

#[test]
fn portable_offer_response_preserves_its_wire_shape() {
    assert_eq!(fixture(), r#"{"type":"idle"}"#);
    let response: ProviderAnswer =
        serde_json::from_str(r#"{"type":"registered","max_sessions":4}"#).unwrap();
    assert_eq!(response, ProviderAnswer::Registered { max_sessions: 4 });
}
