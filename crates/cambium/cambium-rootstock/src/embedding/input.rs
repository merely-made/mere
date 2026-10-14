// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use document_session_api::SessionInput;
#[derive(Clone, Debug, PartialEq)]
pub enum OwnerInput {
    Session(SessionInput),
    Wheel { position: [f32; 2], delta: [f32; 2] },
}
/// Caller-configured logical pixels per line/page. Toolkits supply wheel units;
/// the owner chooses their meaning instead of a toolkit adapter fixing a speed.
#[derive(Clone, Copy, Debug)]
pub struct ScrollUnits {
    pub line: [f32; 2],
    pub page: [f32; 2],
}
impl ScrollUnits {
    pub fn validate(self) -> Result<Self, &'static str> {
        if self
            .line
            .iter()
            .chain(self.page.iter())
            .all(|v| v.is_finite() && *v >= 0.)
        {
            Ok(self)
        } else {
            Err("finite nonnegative caller scroll units required")
        }
    }
}
