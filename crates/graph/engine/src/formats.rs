// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The formats stage: parsers and serializers over `oxjsonld` and `oxttl`.

/// JSON-LD export and the classic-reification bridge.
pub mod jsonld;

/// Turtle-family file I/O (N-Quads / TriG) for the RDF projection, via `oxttl`.
pub mod serialize;

/// Entry points that take a base IRI and return an `IndexedDataset` (no `query`
/// feature needed): Turtle, N-Triples, N-Quads, TriG, RDF/XML (parse only) and JSON-LD.
pub mod parse;
