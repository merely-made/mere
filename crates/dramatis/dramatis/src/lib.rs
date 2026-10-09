// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Name reservation for **dramatis**, the cast-list tier of the Mere platform.
//!
//! *Dramatis personae*: the persons of the drama. The tier holds both sides of
//! identity, your faces and the other players, which is why the name is the
//! full cast list rather than any one role:
//!
//! - **personae** — the trust-plane spine: master keypair, per-protocol
//!   derivation, vault, sealed records, carry.
//! - **gaz** — stored contacts: anchored records, petnames, per-endpoint
//!   trust, kith/kin tiers.
//! - **gazette** — handle resolution: turning a name into reachable,
//!   unverified address claims.
//!
//! The boundaries are the point:
//!
//! - **Not the data plane.** Persistence is the eidetic family (muniment,
//!   muniment journals, chartulary). The planes bond at the seal seam and the sync gate;
//!   dramatis holds keys and trust, never the bytes they seal.
//! - **Not a product.** *Persona* is an in-product term for a face; dramatis
//!   names the tier so the term stays free.
//!
//! Ruled 2026-10-01: this becomes the facade that repos outside mere pin,
//! re-exporting personae, insigne and gaz behind features, so one dependency
//! at one revision carries the tier. The dramatis repo plan's DR-A moves the
//! identity surface's plain types here first (ruling D24): [`intents`] so
//! far.

#![doc(html_no_source)]

pub mod intents;
