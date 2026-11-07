// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use graphshell_door::projection_offer::{PROJECTION_PROVIDER_ROUTE, ProviderAnswer};
pub fn fixture() -> String {
    assert!(!PROJECTION_PROVIDER_ROUTE.is_empty());
    serde_json::to_string(&ProviderAnswer::Idle).unwrap()
}

#[cfg(feature = "forbidden-api")]
pub use ostiary::app_client::AppConnection;

#[test]
fn portable_offer_response_preserves_its_wire_shape() {
    assert_eq!(fixture(), r#"{"type":"idle"}"#);
    let response: ProviderAnswer =
        serde_json::from_str(r#"{"type":"registered","max_sessions":4}"#).unwrap();
    assert_eq!(response, ProviderAnswer::Registered { max_sessions: 4 });
}
