// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Register Dramatis custody on the resident runner's admitted door.

/// Serve the custody route to declared applications whose owner grant names
/// it. No application holds it by label (capability door D24).
pub fn grant(
    grants: &djinn::door::app_admission::AppRouteGrants,
) -> Result<(), ostiary::route::ResidentEndpointRouteError> {
    let route = ostiary::route::ResidentEndpointRoute::new(
        castellan_client::custody::CUSTODY_ROUTE,
        std::time::Duration::from_millis(50),
    )?;
    grants.update(|routes| routes.serve(route));
    Ok(())
}
