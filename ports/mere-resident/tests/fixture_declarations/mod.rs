// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the custody tests' fixture applications declare, and the owner
//! grants admitting them (capability door D22). Each declares only the calls
//! its tests make; an ordinary application declares status alone.

#![allow(dead_code)]

use djinn::door::app_admission::{AppId, AppRouteGrants};
use djinn::door::declaration::{AppDeclaration, CustodyOperation, CustodyUse, NamePattern};

pub fn declaration(app: &str) -> AppDeclaration {
    use CustodyOperation as Op;
    let operations = match app {
        "dramatis" => vec![
            Op::Status,
            Op::Roster,
            Op::ChooseProfile,
            Op::LockVault,
            Op::UnlockNative,
            Op::ChangePassphraseNative,
            Op::Quarantine,
            Op::ReviewQuarantine,
            Op::UnassignedImports,
            Op::AssignImport,
        ],
        "personae-vault" => vec![Op::Status, Op::Roster, Op::VaultCommand],
        _ => vec![Op::Status],
    };
    AppDeclaration {
        routes: vec![NamePattern::exact("custody")],
        custody: CustodyUse {
            operations: operations.into_iter().collect(),
            ..CustodyUse::default()
        },
        ..AppDeclaration::default()
    }
}

/// Grant each of `apps` exactly its declaration.
pub fn grant(grants: &AppRouteGrants, apps: &[&str]) {
    mere_resident::applications::install(
        grants,
        apps.iter()
            .map(|app| mere_resident::applications::grant_for(&AppId::new(app), &declaration(app), None))
            .collect(),
    );
}
