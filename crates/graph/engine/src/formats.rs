// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The formats stage: parsers and serializers over `oxjsonld` and `oxttl`.

/// JSON-LD export and the classic-reification bridge.
pub mod jsonld;

/// RDF Dataset Canonicalization (RDFC-1.0) over `oxrdf`'s `rdfc-10`: canonical N-Quads, the
/// issued-identifier map, a hash choice and a blank-node limit.
pub mod canon;

/// Turtle-family file I/O (N-Quads / TriG) for the RDF projection, via `oxttl`.
pub mod serialize;

/// Entry points that take a base IRI and return an `IndexedDataset` (no `query`
/// feature needed): Turtle, N-Triples, N-Quads, TriG, RDF/XML (parse only) and JSON-LD.
pub mod parse;

/// The RDF format enum (media types, extensions), from `oxrdfio`. The engine
/// parses every variant but N3.
pub use oxrdfio::RdfFormat;
