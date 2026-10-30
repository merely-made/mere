// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A real custodian on a socket, served from a thread of the test, over a
//! synthetic passphrase vault. Djinn reaches it as it reaches the installed
//! one: through a client file and [`RemoteHost`].
#![allow(dead_code)]

use std::sync::Arc;

use mere_resident::custodian::InstalledCustodian;
use mere_resident::custody_host::remote::RemoteHost;
use djinn_custodian::fixture::{Fixture, Spec};
use serde_json::{Value, json};

pub struct TestCustodian {
    pub dir: tempfile::TempDir,
    pub fixture: Fixture,
    pub passphrase: String,
}

impl std::ops::Deref for TestCustodian {
    type Target = Fixture;

    fn deref(&self) -> &Fixture {
        &self.fixture
    }
}

impl TestCustodian {
    /// Profiles `(id, master seed)`; the first is the one the custodian opens.
    pub fn start(passphrase: &str, profiles: &[(&str, [u8; 32])], grants: Value) -> Self {
        Self::start_with(passphrase, profiles, grants, "deny", None)
    }

    pub fn start_with(
        passphrase: &str,
        profiles: &[(&str, [u8; 32])],
        grants: Value,
        consent: &str,
        mechanism: Option<Arc<dyn djinn_custodian::consent::Mechanism>>,
    ) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let profiles: Vec<_> = profiles.iter().map(|(id, seed)| (*id, *id, *seed)).collect();
        let fixture = djinn_custodian::fixture::start(
            dir.path(),
            Spec {
                passphrase,
                profiles: &profiles,
                grants,
                consent,
                mechanism,
                agent: false,
            },
        )
        .unwrap();
        Self {
            dir,
            fixture,
            passphrase: passphrase.into(),
        }
    }

    pub fn installed(&self) -> InstalledCustodian {
        InstalledCustodian::from_path(&self.client_file, false).unwrap()
    }

    /// Djinn's custody: a client of this custodian.
    pub fn remote(&self) -> Arc<RemoteHost> {
        RemoteHost::connect(self.installed()).unwrap()
    }

    /// The vault opened directly, to check what the custodian answered.
    pub fn direct(&self, profile: &str) -> castellan::custody::IdentityVault<Box<dyn castellan::custody::IdentityStorage>> {
        self.fixture.direct(&self.passphrase, profile).unwrap()
    }
}

/// Every non-signing H2 operation, unattended (a development custodian).
pub fn manage_grants() -> Value {
    json!([
        {"operation": "status"}, {"operation": "lock"}, {"operation": "unlock"}, {"operation": "snapshot"},
        {"operation": "manage", "consent": "none"},
        {"operation": "wallet", "consent": "none"},
        {"operation": "payload", "consent": "none"},
        {"operation": "credentials", "consent": "none"}
    ])
}

/// [`manage_grants`] plus release, attestation and signing for `salts`.
pub fn grants_with(salts: Value) -> Value {
    let mut grants = manage_grants();
    let list = grants.as_array_mut().unwrap();
    list.push(json!({"operation": "release", "profile": "any", "salts": salts, "consent": "none"}));
    list.push(json!({"operation": "release", "profile": "current", "source": "wallet", "salts": salts, "consent": "none"}));
    list.push(json!({"operation": "sign", "profile": "current", "purpose": {"purpose": "derived", "salts": salts}, "consent": "none"}));
    list.push(json!({"operation": "sign", "profile": "current", "purpose": {"purpose": "delegation", "audience": "mere.graphshell"}, "consent": "none"}));
    grants
}
