use djinn::door::app_client::AppConnection;
use graphshell_endpoint::door::projection_offer::PROJECTION_PROVIDER_ROUTE;
pub fn accepts_connection(_: AppConnection) -> &'static str {
    PROJECTION_PROVIDER_ROUTE
}
pub use graphshell_endpoint::door::projection_provider::serve;

#[cfg(feature = "forbidden-api")]
pub use djinn::door::custody_client::CustodyClient;

#[test]
fn native_provider_uses_the_existing_route_hello() {
    use djinn::door::app_admission::{AppHello, AppId, AppRouteId};
    let request = AppHello::for_route(
        AppId::new(" PROVIDER "),
        AppRouteId::new(PROJECTION_PROVIDER_ROUTE).unwrap(),
    )
    .accept()
    .unwrap();
    assert_eq!(request.app.as_str(), "provider");
    assert_eq!(request.route.as_str(), PROJECTION_PROVIDER_ROUTE);
}
