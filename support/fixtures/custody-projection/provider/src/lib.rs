// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use ostiary::app_client::AppConnection;
use graphshell_endpoint::door::projection_offer::PROJECTION_PROVIDER_ROUTE;
pub fn accepts_connection(_: AppConnection) -> &'static str {
    PROJECTION_PROVIDER_ROUTE
}
pub use graphshell_endpoint::door::projection_provider::serve;

#[cfg(feature = "forbidden-api")]
pub use ostiary::custody_client::CustodyClient;

#[test]
fn native_provider_uses_the_existing_route_hello() {
    use ostiary::app_admission::{AppHello, AppId, AppRouteId};
    let request = AppHello::for_route(
        AppId::new(" PROVIDER "),
        AppRouteId::new(PROJECTION_PROVIDER_ROUTE).unwrap(),
    )
    .accept()
    .unwrap();
    assert_eq!(request.app.as_str(), "provider");
    assert_eq!(request.route.as_str(), PROJECTION_PROVIDER_ROUTE);
}
