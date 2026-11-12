// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! # mere-rdf
//!
//! A re-export shim over the `mere` engine crate (engine split P5 L2, F6) and,
//! for the surface projection and importer that moved there in P5 L3, over
//! `kernel::rdf`. Every
//! path this crate had still resolves, so consumers build unchanged until they
//! repoint to `mere` and this crate is deleted.

/// The ingest module, with the importer that moved to `kernel::rdf` (P5 L3).
pub mod ingest {
    #[cfg(not(target_arch = "wasm32"))]
    pub use kernel::rdf::{
        ApplyOutcome, apply_contribution, apply_contribution_with_identity, apply_import,
        apply_import_with_identity,
    };
    pub use mere::acquire::ingest::*;
}
pub use mere::formats::serialize;
pub use mere::profile::vocab;
#[cfg(feature = "query")]
pub use mere::query;

pub use kernel::rdf::node_quads;
#[cfg(not(target_arch = "wasm32"))]
pub use kernel::rdf::{
    ApplyOutcome, apply_contribution, apply_contribution_with_identity, apply_import,
    apply_import_with_identity,
};
pub use mere::{
    ASSERTION_HISTORY_GRAPH_IRI, Alignment, AssertionProjection, ContextCache, EdgeContribution,
    GraphContribution, ImportEnvelope, IngestError, NodeContribution, SubjectIdentity, alignment,
    dataset_quads, dataset_quads_with_history, from_jsonld, from_jsonld_envelope,
    from_jsonld_envelope_with_contexts, from_jsonld_envelope_with_contexts_and_base_iri,
    from_jsonld_with_contexts, from_jsonld_with_contexts_and_base_iri, from_nquads,
    from_nquads_envelope, from_quads, from_quads_envelope, from_trig, from_trig_envelope,
    is_bundled_context, referenced_context_urls, to_jsonld, to_jsonld_compact, to_jsonld_string,
    to_jsonld_with_history, to_nquads, to_nquads_with_history, to_trig, to_trig_with_history,
    vocabulary_alignment_quads,
};

#[cfg(test)]
mod tests {
    /// The old module paths and flat names resolve to the same items.
    #[test]
    fn old_paths_resolve() {
        let flat = crate::GraphContribution::default();
        let via_module: crate::ingest::GraphContribution = flat.clone();
        assert_eq!(flat, via_module);
        let _: fn(&str, &str) -> _ = crate::serialize::from_nquads;
        let _ = crate::vocab::alignment;
    }
}
