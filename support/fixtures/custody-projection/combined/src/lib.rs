// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An external consumer of the resident assembly and both independent doors.
use ostiary::app_admission::{AppHello, AppId, AppRouteId};
use graphshell_door::projection_offer::PROJECTION_PROVIDER_ROUTE;

// Refer to public types from every package without constructing a resident
// or requiring a live vault, listener, or network service.
pub fn accepts_residents(_: &mere_resident::resident::DjinnResident, _: &djinn::resident::Resident) {}
pub fn accepts_custody(_: &castellan_client::custody_client::CustodyClient) {}
pub fn accepts_projection(_: &graphshell_door::projection::AppBrokerClient) {}
pub use graphshell_door::projection_provider::serve;

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
