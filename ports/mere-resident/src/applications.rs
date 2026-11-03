// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The owner's admission policy for declared applications (capability door
//! plan, M1, D24, D25).
//!
//! An application states on its hello what it opens, offers and needs
//! (`ostiary::declaration`). The owner's grants decide what it receives;
//! a connection gets only what both name. There are no default grants: the
//! first time an application nobody has granted connects, Djinn asks the
//! owner ([`OwnerConsent`]), and a yes is written to `OwnerSettings::applications`.
//! The one built-in grant is the resident's own command line
//! ([`resident_grant`]). Nothing here names any other application.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use djinn::door::app_admission::AppRouteGrants;
use ostiary::app_admission::AppId;
use ostiary::declaration::{
    AppDeclaration, AppGrant, AppIdentity, CustodyGrant, NamePattern, WindowsAppIdentity,
};

use crate::identity_ui::NativeIdentityUi;
use crate::resident_devices::DEVICE_DIRECTORY_ROUTE;
use crate::resident_status::{RESIDENT_APP, RESIDENT_CONTROL_ROUTE, RESIDENT_STATUS_ROUTE};
use crate::settings::OwnerSettings;

/// Why an application policy was refused.
#[derive(Debug, thiserror::Error)]
pub enum ApplicationPolicyError {
    #[error("application grants are malformed: {0}")]
    Malformed(String),
    #[error("{0} is granted twice")]
    Duplicate(String),
}

/// The routes the resident's own command line opens: its status, its
/// control (stop and unlock) and the device directory.
const RESIDENT_ROUTES: [&str; 3] = [
    RESIDENT_STATUS_ROUTE,
    RESIDENT_CONTROL_ROUTE,
    DEVICE_DIRECTORY_ROUTE,
];

/// What the resident's own command line declares.
pub fn resident_declaration() -> AppDeclaration {
    AppDeclaration {
        routes: RESIDENT_ROUTES.into_iter().map(NamePattern::exact).collect(),
        ..AppDeclaration::default()
    }
}

/// The resident's own command line: the owner at a terminal, granted by
/// Djinn rather than asked through Djinn's own dialog (D25).
pub fn resident_grant() -> AppGrant {
    let mut grant = AppGrant::new(AppId::new(RESIDENT_APP));
    grant.routes = resident_declaration().routes;
    grant
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

/// What an owner's yes grants: exactly what `declaration` names, personas
/// none (D9), pinned to the identity the door read from `peer` where the
/// platform gives one the door can check again (D25). Elsewhere the grant is
/// unpinned, in development mode.
pub fn grant_for(app: &AppId, declaration: &AppDeclaration, peer: Option<&peer_facts::Peer>) -> AppGrant {
    let custody = &declaration.custody;
    AppGrant {
        app: app.clone(),
        routes: declaration.routes.clone(),
        custody: CustodyGrant {
            operations: custody.operations.clone(),
            epoch: custody.epoch.iter().map(|used| NamePattern::exact(&used.context)).collect(),
            salts: custody.salts.clone(),
            acts: custody.acts.clone(),
            payload_scope: custody.payload_scope.clone(),
        },
        identity: pin(peer),
        // Marking an agent is the owner's own act (D27), not part of a yes.
        agent: false,
    }
}

/// The identity a grant pins, from what the door read. A Windows image is the
/// one the door reads and checks again; a macOS signature is checked against
/// a pinned team and identifier but not read, and a Linux peer is pinned
/// only by a launch, so both stay unpinned.
fn pin(peer: Option<&peer_facts::Peer>) -> Option<AppIdentity> {
    match peer? {
        peer_facts::Peer::Windows { image: Some(image), .. } => Some(AppIdentity {
            windows: Some(WindowsAppIdentity { image: image.clone() }),
            ..AppIdentity::default()
        }),
        _ => None,
    }
}

/// The owner's answer to an application nobody has granted.
pub trait ApplicationConsent: Send + Sync {
    /// Ask about `app`, once, without waiting for the answer. The connection
    /// that prompted it has already been refused; the application retries.
    fn ask(&self, app: &AppId, declaration: &AppDeclaration, peer: Option<&peer_facts::Peer>);
}

/// Djinn's consent: a native confirm, with the answer kept in the owner's
/// settings. A yes is installed for the next connection; a no is kept as a
/// refusal until the owner removes it; with no dialog the application is
/// refused and asked again only after a restart.
#[derive(Clone)]
pub struct OwnerConsent {
    settings: PathBuf,
    grants: AppRouteGrants,
    ui: Arc<dyn NativeIdentityUi>,
    asked: Arc<Mutex<BTreeSet<AppId>>>,
}

impl OwnerConsent {
    pub fn new(settings: PathBuf, grants: AppRouteGrants, ui: Arc<dyn NativeIdentityUi>) -> Self {
        Self { settings, grants, ui, asked: Arc::default() }
    }

    /// Ask now, on this thread, and keep the answer.
    pub fn decide(&self, app: &AppId, declaration: &AppDeclaration, peer: Option<&peer_facts::Peer>) -> Result<bool, String> {
        let refused = OwnerSettings::load(&self.settings)
            .map_err(|error| error.to_string())?
            .refused_applications
            .contains(app);
        if refused {
            return Ok(false);
        }
        let grant = grant_for(app, declaration, peer);
        let yes = self
            .ui
            .confirm_application(app.as_str(), &describe(&grant))
            .map_err(|failure| format!("{failure:?}"))?;
        OwnerSettings::update(&self.settings, |settings| {
            match yes {
                true => {
                    let grants = settings.applications.get_or_insert_with(Vec::new);
                    grants.retain(|existing| existing.app != grant.app);
                    grants.push(grant.clone());
                },
                false => {
                    settings.refused_applications.push(app.clone());
                },
            }
            Ok::<_, crate::settings::OwnerSettingsError>(())
        })
        .map_err(|error| error.to_string())?;
        if yes {
            self.grants.update(|routes| routes.grant_app(grant));
        }
        Ok(yes)
    }
}

impl ApplicationConsent for OwnerConsent {
    fn ask(&self, app: &AppId, declaration: &AppDeclaration, peer: Option<&peer_facts::Peer>) {
        if !self.asked.lock().expect("consent set").insert(app.clone()) {
            return;
        }
        let (consent, app, declaration, peer) = (self.clone(), app.clone(), declaration.clone(), peer.cloned());
        std::thread::spawn(move || match consent.decide(&app, &declaration, peer.as_ref()) {
            Ok(granted) => tracing::info!(app = %app, granted, "the owner answered an application's declaration"),
            Err(error) => tracing::warn!(app = %app, %error, "an application's declaration was not put to the owner"),
        });
    }
}

/// The dialog's account of a grant: every route, call and namespace, and
/// what the label rests on.
pub fn describe(grant: &AppGrant) -> String {
    let names = |patterns: &[NamePattern]| {
        patterns
            .iter()
            .map(|pattern| match pattern {
                NamePattern::Exact(name) => name.clone(),
                NamePattern::Prefix(prefix) => format!("{prefix}…"),
            })
            .collect::<Vec<_>>()
    };
    let salts = |uses: &[ostiary::declaration::SaltUse]| {
        uses.iter().map(|used| format!("{used:?}")).collect::<Vec<_>>()
    };
    let mut lines = Vec::new();
    let mut section = |title: &str, items: Vec<String>| {
        if !items.is_empty() {
            lines.push(format!("{title}: {}", items.join(", ")));
        }
    };
    section("Routes", names(&grant.routes));
    section(
        "Custody calls",
        grant.custody.operations.iter().map(|operation| operation.name().to_string()).collect(),
    );
    section("Epoch keys", names(&grant.custody.epoch));
    section("Keys released", salts(&grant.custody.salts));
    section("Keys signed with", salts(&grant.custody.acts));
    lines.push(match &grant.identity {
        Some(_) => "Identity: pinned to the executable that asked.".into(),
        None => "Identity: not pinned (development mode): any program may use this name.".into(),
    });
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ostiary::declaration::{CustodyOperation, CustodyUse, EpochUse, SaltPattern, SaltUse};

    #[test]
    fn the_resident_grant_names_only_its_own_routes() {
        let grant = resident_grant();
        assert_eq!(grant.routes.len(), RESIDENT_ROUTES.len());
        assert!(grant.custody == CustodyGrant::default(), "the command line holds no custody");
        assert!(validated(vec![grant]).is_ok());
    }

    #[test]
    fn a_yes_grants_exactly_the_declaration() {
        let declaration = AppDeclaration {
            routes: vec![NamePattern::exact("custody")],
            custody: CustodyUse {
                operations: [CustodyOperation::Status].into_iter().collect(),
                epoch: vec![EpochUse::secret("app.key", false)],
                salts: vec![SaltUse::persona(SaltPattern::Prefix("app/".into()))],
                acts: Vec::new(),
                payload_scope: None,
            },
            ..AppDeclaration::default()
        };
        let windows = peer_facts::Peer::Windows {
            sid: "S-1".into(),
            pid: 7,
            image: Some("C:\\app.exe".into()),
        };
        let grant = grant_for(&AppId::new("app"), &declaration, Some(&windows));
        assert_eq!(grant.routes, declaration.routes);
        assert_eq!(grant.custody.epoch, vec![NamePattern::exact("app.key")]);
        assert_eq!(grant.custody.salts, declaration.custody.salts);
        assert!(grant.identity.as_ref().and_then(|pin| pin.windows.as_ref()).is_some_and(|w| w.matches("c:/APP.exe")));
        let unix = peer_facts::Peer::Unix { uid: 1, gid: 1, pid: Some(2) };
        assert!(grant_for(&AppId::new("app"), &declaration, Some(&unix)).identity.is_none());
        assert!(describe(&grant).contains("custody"));
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
