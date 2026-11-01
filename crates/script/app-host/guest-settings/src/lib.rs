// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The settings test guest: the shared call script over this component's
//! imports.

wit_bindgen::generate!({
    path: ["../../wit", "wit"],
    world: "mere:settings-test/settings-consumer",
    // The imported interface lives in another package, mere:script.
    generate_all,
});

use mere::script::settings as imported;
// The world `use`s the guest's own types at this level, so ours are renamed.
use settings_wit::bindings::{
    Failure as WireFailure, SettingsSnapshot as WireSnapshot, WriteOutcome as WireOutcome,
    WriteRequest,
};
use settings_wit::{SettingsCalls, testkit};

settings_wit::bridge!(crate::mere::script::settings);

/// The imports as [`SettingsCalls`]. A call to an import cannot break.
struct Imports;

impl SettingsCalls for Imports {
    fn snapshot(&mut self, reference: String) -> Result<Result<WireSnapshot, WireFailure>, String> {
        Ok(imported::snapshot(&reference)
            .map(bridge::wire::snapshot)
            .map_err(bridge::wire::failure))
    }
    fn write(&mut self, request: WriteRequest) -> Result<WireOutcome, String> {
        let request = bridge::theirs::request(request);
        Ok(bridge::wire::outcome(imported::write(&request)))
    }
}

struct Component;

impl Guest for Component {
    fn run() -> Vec<(String, String)> {
        testkit::run(&mut Imports)
    }
    fn echo_snapshot(value: imported::SettingsSnapshot) -> imported::SettingsSnapshot {
        value
    }
    fn echo_outcome(value: imported::WriteOutcome) -> imported::WriteOutcome {
        value
    }
    fn echo_failure(value: imported::Failure) -> imported::Failure {
        value
    }
}

export!(Component);
