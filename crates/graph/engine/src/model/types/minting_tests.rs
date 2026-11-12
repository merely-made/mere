// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// The minted id is device-safe in shape: time prefix + a non-zero
/// process salt (native self-seeds) + a monotonic counter, all distinct
/// across consecutive mints.
#[test]
fn statement_ids_carry_salt_and_stay_unique() {
    let a = mint_local_statement_id();
    let b = mint_local_statement_id();
    assert_ne!(a, b);
    let parts: Vec<&str> = a.split('-').collect();
    assert_eq!(parts.len(), 3, "ts-salt-counter shape: {a}");
    assert_eq!(parts[0].len(), 12);
    assert_eq!(parts[1].len(), 16);
    assert_eq!(parts[2].len(), 16);
    assert_ne!(parts[1], "0000000000000000", "native salt self-seeds");
    let b_parts: Vec<&str> = b.split('-').collect();
    assert_eq!(parts[1], b_parts[1], "salt is per-process stable");
}
