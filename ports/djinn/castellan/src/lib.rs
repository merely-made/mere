// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **Castellan**, Djinn's secret-custody component.
//!
//! A castellan holds a keep in trust for its lord: custody without ownership,
//! and the office of the gate. This crate is that keeper for your
//! credentials, and it has no screens. It lives at `ports/djinn/castellan`
//! (graphshell stack lift plan, ruling R8, 2026-10-15), and only djinn links
//! it. Credential release, signing and presentation happen here, inside the
//! resident: requests arrive as participant-gate petitions and are answered
//! over an agent-style channel, the way the personae ssh-agent already works.
//! Apps talk to a pipe; apps never see the key.
//!
//! The secret-free views (vault browse, credential status, code tiles) live
//! in `dramatis::view` (dramatis repo plan, ruling D24). The persona and
//! quarantine UI is `dramatis-app` (`ports/dramatis`). Castellan stays its
//! own crate so the custody code keeps its dependency boundary and the
//! `deny.toml` wrapper check.
//!
//! The vocabulary it keeps (per the dramatis tier model):
//!
//! - **chatelaine**: the secrets. Passwords, 2FA seeds, tokens, foreign key
//!   material. Never presented, only exercised.
//! - **insigne**: the proofs. Graded presentations of identity a persona hands
//!   out, from a bare handle to signed cross-attestations. Made to be shown;
//!   what lands in someone else's gaz.
//!
//! The boundaries are the point:
//!
//! - **Not personae.** The faces, their derivation roots and issuing live in
//!   `personae`; castellan is the keeper who holds and serves them.
//! - **Not gaz or gazette.** Those keep and find the other players; castellan
//!   guards and presents you.
//!
//! [`otp`] is the algorithm half of the chatelaine's 2FA codes: given a secret
//! and a clock, the digits; given an `otpauth://` URI, the configured
//! generator. The Reticulum station derivation that began here moved to
//! `personae::reticulum` and the station grant policy to
//! `pandect::station_grant` in DR-C (D17); djinn signs the grant.
//!
//! The **keeper surface** (feature `keeper`, founded 2026-08-14) moved home
//! from graphshell where it first grew:
//!
//! - [`view`] — re-exports `dramatis::view` at its old paths and loads the
//!   public carry lane into it.
//! - [`authority`] — [`authority::PersonaeHost`], the resident keeper. What
//!   only the castellan does.
//!
//! The cards (once `projection`) moved to graphshell's `identity_projection`
//! in DR-C (D15): they are a pure function of the snapshot, and djinn serves
//! them through graphshell's endpoint, which no longer names castellan.
//!
//! Since the dramatis repo plan's DR-A (2026-10-09), the plain types these
//! modules name live in the identity tier and are re-exported here at their
//! old paths: the views, intents and receipts in the `dramatis` facade, and
//! the OTP display types and the Secret Service's metadata in chatelaine.
//! castellan keeps what needs its authority: the loaders, the error it
//! raises, and the release gate.
//! [`items::ItemStore`] keeps one persona's chatelaine items, each
//! credential's secret in its own sealed payload record. OTP imports land
//! there through [`otp::OtpItemStore`], and the Secret Service's items too. [`otp::OtpReleaseGate`] returns an
//! [`otp::OtpCodeTile`] only after a participant-bound petition receives a
//! resident approval. [`otp::OtpAdmittedSession`] binds remote petitions to one
//! exact credential and the Notochord transcript that admitted their carrier.
//! [`resident::CastellanResident`] retains the process-wide sealed-record
//! authority, and [`lock`] is how the vault's lock reaches it. Feature
//! `secret-service` adds the Linux desktop adapter, and
//! [`otp::SteamGuard`] is an explicitly nonstandard Valve compatibility shape.
//! Feature `cxf` adds two-step credential import: a memory-only preview, then
//! an explicit destination per account, with native SSH keys kept in vault slots.

//! [`custody`] is every secret at rest and in hand, moved here in the dramatis
//! repo plan's DR-B (2026-10-09): personae's vault, storages, unlock ladder
//! and SSH agent, and pandect's wallet seeds and private epochs. Only the
//! resident (djinn) may link this crate; `deny.toml` holds the line (D4, D16).

#![doc(html_no_source)]
#![warn(missing_docs)]

#[cfg(feature = "keeper")]
pub mod authority;
pub mod custody;
#[cfg(feature = "cxf")]
pub mod cxf;
pub mod items;
pub mod lock;
pub mod otp;
pub mod resident;
#[cfg(feature = "cxf")]
pub mod review;
#[cfg(feature = "keeper")]
mod sealed_storage;
#[cfg(feature = "secret-service")]
pub mod secret_service;
#[cfg(feature = "keeper")]
pub mod view;
