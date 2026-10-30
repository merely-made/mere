//! An external consumer of the resident assembly and both independent doors.
use djinn::door::app_admission::{AppHello, AppId, AppRouteId};
use graphshell_endpoint::door::projection_offer::PROJECTION_PROVIDER_ROUTE;

// Refer to public types from all three packages without constructing a resident
// or requiring a live vault, listener, or network service.
pub fn accepts_residents(_: &mere_resident::resident::DjinnResident, _: &djinn::resident::Resident) {}
pub fn accepts_custody(_: &djinn::door::custody_client::CustodyClient) {}
pub fn accepts_projection(_: &graphshell_endpoint::door::projection::AppBrokerClient) {}
pub use graphshell_endpoint::door::projection_provider::serve;

#[test]
fn combined_consumer_admits_provider_route_hello() {
    let request = AppHello::for_route(
        AppId::new(" PROVIDER "),
        AppRouteId::new(PROJECTION_PROVIDER_ROUTE).unwrap(),
    )
    .accept()
    .unwrap();
    assert_eq!(request.app.as_str(), "provider");
    assert_eq!(request.route.as_str(), PROJECTION_PROVIDER_ROUTE);
    assert!(std::mem::size_of::<djinn::resident::Resident>() > 0);
}
