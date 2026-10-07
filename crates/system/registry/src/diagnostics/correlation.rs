// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

/// Opaque producer-supplied identity. No text, time, or channel inference.
/// Run and source scope prevent unrelated operations with the same local id
/// from completing one another. Products redact and bound construction first.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DiagnosticCorrelation {
    pub run: String,
    pub source: String,
    pub operation: String,
}

impl DiagnosticCorrelation {
    pub(crate) fn accounted_bytes(&self) -> Option<usize> {
        [
            self.run.as_str(),
            self.source.as_str(),
            self.operation.as_str(),
        ]
        .into_iter()
        .try_fold(0usize, |sum, value| {
            sum.checked_add(8)?.checked_add(value.len())
        })
    }
}
