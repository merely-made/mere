pub use graphshell_endpoint::ProjectionCatalog;
#[cfg(feature = "forbidden-api")]
pub use graphshell_endpoint::door::custody_client::CustodyClient;
pub use graphshell_endpoint::door::projection::{AppBrokerClient, AppRouteCarrier};

#[cfg(test)]
mod tests {
    use graphshell_endpoint::door::app_admission::AppHello;

    #[test]
    fn legacy_projection_hello_selects_identity_without_custody() {
        let hello: AppHello = serde_json::from_str(
            r#"{"schema":"mere.graphshell/app-broker-hello/v1","app":" TURNSTONE "}"#,
        )
        .unwrap();
        let request = hello.accept().unwrap();
        assert_eq!(request.app.as_str(), "turnstone");
        assert_eq!(request.route.as_str(), "identity");
    }
}
