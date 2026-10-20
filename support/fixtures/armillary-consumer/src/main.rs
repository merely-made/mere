// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Independent synchronous consumer of the immutable published execution API.
use std::sync::Arc;
fn main() -> Result<(), armillary::ActorError> {
    let pool = armillary::Pool::new();
    let (actor, updates) = armillary::spawn_on::<u32, u32, _>(&pool, Arc::new(|| {}), |commands, out| {
        let mut sum = 0;
        while let Ok(value) = commands.recv() { sum += value; out.emit(sum); }
    });
    assert!(actor.command(2));
    assert!(actor.command(3));
    actor.join()?;
    assert_eq!(updates.iter().collect::<Vec<_>>(), vec![2, 5]);
    Ok(())
}
