// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! # gaz
//!
//! The contact layer: your records about other people.
//!
//! A contact is the local rollup that says these addresses and keys are all
//! the same person. Your petname for them, filed under an anchor that never
//! moves, with handles and endpoints that each carry their own trust state.
//! Records are persona-scoped, tiered kith or kin, and carry recency.
//!
//! ## What a record is filed under
//!
//! An [`Anchor`], never a name. Handles change and hosts move; an anchor does
//! not, so a peer who switches providers stays the same person rather than
//! becoming a second row. The anchor is whatever the peer's own system holds
//! fixed:
//!
//! - **a root key**, typed by its family ([`TypedKey`]): for a personae peer,
//!   the master key its attestations name, not whichever derived key arrived
//!   first;
//! - **a `did:plc`**, for an atproto account, whose signing keys belong to
//!   its server and change on every move;
//! - **a local id**, for someone who has shown you no key yet.
//!
//! Keys come in two shapes. The root line holds the keys that have stood for
//! the person as a whole, each succeeding the last. Attested keys are
//! concurrent: one per protocol and one per device, each vouched for by a root.
//!
//! ## What this crate is not
//!
//! - **Not the resolver.** A gazetteer turns a name, handle, or key into
//!   reachable endpoints. gaz is where the ones you keep live. The two are
//!   siblings on the persona tier, and gaz is not short for gazetteer.
//! - **Not identity.** `personae` owns *me*, the key-bag and its carry. gaz
//!   owns *them*, your own records about other people's keys.
//! - **Not trust arithmetic.** Trust state is stored per key, endpoint and
//!   handle; how it is earned belongs to the trust plane. gaz therefore
//!   depends on no cryptography, holding keys as bytes it compares but never
//!   verifies.
//!
//! ## Two habits worth knowing
//!
//! gaz never reads a clock or a random source. Every timestamp is a `now_ms`
//! you pass in, unix milliseconds, and a local id is minted from bytes you
//! supply, which keeps the crate deterministic under test and usable on wasm.
//! And every recency update is monotonic, so a replayed or late-arriving event
//! can never rewind a record.
//!
//! ## Quick start
//!
//! ```
//! use gaz::{Anchor, Contact, ContactBook, Endpoint, EndpointKind, Handle, PersonaScope, TypedKey};
//!
//! let mut book = ContactBook::new(PersonaScope::new("work"));
//! let alice_key = TypedKey::ed25519([1u8; 32]);
//!
//! book.insert(
//!     Contact::new("Alice", alice_key)
//!         .with_handle(Handle::acct("acct:Alice@example.org"))
//!         .with_endpoint(Endpoint::new(EndpointKind::Misfin, "alice@example.org")),
//! );
//!
//! // She writes back; you note it, supplying the clock yourself.
//! book.mark_contacted(&Anchor::Key(alice_key), 1_754_000_000_000);
//!
//! assert_eq!(book.recent(5)[0].petname, "Alice");
//! assert!(book.by_handle("alice@example.org").is_some());
//! assert_eq!(book.by_key(&alice_key).count(), 1);
//! ```
//!
//! ## Status
//!
//! Pre-1.0. The data model exists and is tested. Persistence over `muniment`
//! and the adapters that turn resolver output into records are the next
//! lifts; see the founding plan in `design_docs/`.

#![warn(missing_docs)]

pub mod anchor;
pub mod book;
pub mod contact;
mod encoding;
pub mod endpoint;
pub mod handle;
pub mod key;
pub mod trust;

pub use anchor::{Anchor, AnchorParseError, LocalId, PlcDid};
pub use book::{ContactBook, PersonaScope, ScopeMismatch};
pub use contact::{AttestError, AttestedKey, Contact, ContactError, ContactTier, RootKey};
pub use endpoint::{Endpoint, EndpointKind};
pub use handle::{Handle, HandleKind};
pub use key::{KeyAlgorithm, KeyParseError, TypedKey};
pub use trust::{ProofMethod, TrustState};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
