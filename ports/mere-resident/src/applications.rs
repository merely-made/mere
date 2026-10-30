// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The owner's admission policy for declared applications (capability door
//! plan, M1).
//!
//! An application states on its hello what it opens, offers and needs
//! (`djinn::door::declaration`). These grants decide what it receives; a
//! connection gets only what both name. The grants are data: the owner's,
//! from `OwnerSettings::applications`, or Mere's defaults in
//! `policy/default_applications.json` for a profile whose owner has written
//! none. Nothing here names an application in code.

use std::collections::BTreeSet;

use djinn::door::app_admission::AppRouteGrants;
use djinn::door::declaration::AppGrant;

const DEFAULT_GRANTS: &str = include_str!("../policy/default_applications.json");

/// Why an application policy was refused.
#[derive(Debug, thiserror::Error)]
pub enum ApplicationPolicyError {
    #[error("application grants are malformed: {0}")]
    Malformed(String),
    #[error("{0} is granted twice")]
    Duplicate(String),
}

/// Mere's grants for a profile whose owner has written none.
pub fn default_grants() -> Vec<AppGrant> {
    let grants: Vec<AppGrant> =
        serde_json::from_str(DEFAULT_GRANTS).expect("Mere's default application grants parse");
    validated(grants).expect("Mere's default application grants are valid")
}

/// Check `grants`: each bounded, and no application granted twice.
pub fn validated(grants: Vec<AppGrant>) -> Result<Vec<AppGrant>, ApplicationPolicyError> {
    let mut seen = BTreeSet::new();
    for grant in &grants {
        grant
            .validate()
            .map_err(|error| ApplicationPolicyError::Malformed(error.to_string()))?;
        if !seen.insert(grant.app.clone()) {
            return Err(ApplicationPolicyError::Duplicate(grant.app.to_string()));
        }
    }
    Ok(grants)
}

/// Admit each granted application by its declaration, for connections from
/// now on.
pub fn install(grants: &AppRouteGrants, policy: Vec<AppGrant>) {
    grants.update(|routes| {
        for grant in policy {
            routes.grant_app(grant);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use djinn::door::app_admission::AppId;
    use djinn::door::declaration::NamePattern;

    #[test]
    fn the_defaults_parse_and_grant_by_namespace() {
        let grants = default_grants();
        assert!(!grants.is_empty());
        for grant in &grants {
            assert!(
                grant.custody.epoch.iter().all(|pattern| matches!(pattern, NamePattern::Prefix(_))),
                "{} is granted epoch contexts by namespace, not by name",
                grant.app
            );
        }
    }

    #[test]
    fn an_application_cannot_be_granted_twice() {
        let twice = vec![AppGrant::new(AppId::new("app")), AppGrant::new(AppId::new("APP"))];
        assert!(matches!(validated(twice), Err(ApplicationPolicyError::Duplicate(_))));
        let mut spaced = AppGrant::new(AppId::new("app"));
        spaced.routes.push(NamePattern::exact("a b"));
        assert!(matches!(validated(vec![spaced]), Err(ApplicationPolicyError::Malformed(_))));
    }
}
