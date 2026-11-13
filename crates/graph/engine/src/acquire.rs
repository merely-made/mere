// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The acquire stage: bytes and bundled documents in, a graph contribution out.
//! The fetch itself stays with the host; the offline `@context` loader and the
//! contribution profile live here.

/// JSON-LD ingest: `application/ld+json` → a graph contribution.
pub mod ingest;

/// Content negotiation and Link-header discovery: the Accept policy, the
/// media-type dispatch to the parsers, and RFC 8288 links. Pure functions; the
/// host does the fetch.
pub mod negotiate;
