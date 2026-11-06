// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mere's concrete resident composition, served by independent Djinn.
//!
//! This package constructs Knot, graph/reservoir, Distillery and published-site
//! services and supplies their authorities and shutdown order. Djinn owns the
//! application door and resident task lifetime. Custodian remains a separately
//! protected authority using custody-wire. Executable names remain `djinn`.

#![doc(html_no_source)]

pub mod applications;
pub mod conditions;
pub mod custody;
pub mod door;
pub mod embedded_reservoir;
pub mod identity_ui;
pub mod knot_relation_intake;
pub mod lock_triggers;
pub mod pairing;
pub mod personal_sync;
pub mod personal_facts;
pub mod resident;
pub mod resident_blobs;
pub mod resident_devices;
pub mod resident_distillery;
pub mod resident_events;
pub mod resident_inference;
pub mod resident_knot;
pub mod resident_mere;
pub mod resident_projection;
pub mod resident_reservoir;
pub mod retained_custody;
pub mod resident_site;
#[cfg(feature = "moot-site")]
pub mod moot_site;
pub mod resident_status;
pub mod settings;
pub mod sync;
