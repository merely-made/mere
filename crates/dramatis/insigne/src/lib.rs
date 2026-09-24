// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **insigne**, the graded identity proof of the Mere platform's dramatis
//! tier.
//!
//! *Insigne* is the Latin singular of *insignia* — a plural English uses so
//! exclusively that its singular has dropped out of ordinary use. It names one
//! badge of office or rank, and rank badges are graded by construction: which
//! one you wear is chosen for the occasion.
//!
//! An insigne is what a persona presents: a proof of identity made to be shown
//! and surviving showing. A signature reveals no key; a certificate can be
//! republished forever. The grade is chosen per audience, and it needn't be
//! the most stringent proof available:
//!
//! - a bare handle or key: continuity, "same me as last time"
//! - signed claims binding handles and endpoints to a persona key:
//!   cross-attestation
//! - delegation certificates (personae's grammar): attenuated capability
//! - chain-root linkage, rarely shown: the disclosure pseudonymous personas
//!   exist to withhold
//!
//! Your insigne is what someone else's gaz keeps: the insigne is the
//! interchange artifact, the gaz record is the ledger of insignia received. A
//! *published* insigne is what gazette resolves (a WebFinger JRD is one); a
//! *handed* insigne travels bilaterally. Same artifact, two carriages.
//!
//! The boundaries are the point:
//!
//! - **Not the secrets.** That is `chatelaine`: bearer material, damaged by
//!   disclosure. Everything here is a public-key artifact, designed for it.
//! - **Not the keeper.** That is `castellan`, which signs presentations and
//!   mans the gate.
//!
//! ## What travels, and what is concluded
//!
//! The core is plain, serializable data: the artifacts that travel, that a
//! gaz keeps, and that anyone may check again later. Checking belongs behind
//! a feature, and a check that passes yields a local conclusion that is
//! deliberately not serializable, the rule notochord's `AdmittedPrincipal`
//! already follows: a conclusion drawn from a verified artifact never travels
//! as one.
//!
//! ## Built today
//!
//! [`TypedKey`], the bare-key grade: a public key typed by its family and
//! written in that family's standard text form (`did:key`, or `rnid`'s hex for
//! a Reticulum identity). Nothing verifies yet.

#![warn(missing_docs)]
#![doc(html_no_source)]

mod encoding;
pub mod key;

pub use key::{KeyAlgorithm, KeyParseError, TypedKey};
