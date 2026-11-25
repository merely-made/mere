// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Host-side admission of bundled and operator-selected local ontology packs.
mod catalog;
mod links;
mod saved;
pub use catalog::load;
pub use links::{RecordIndex, RecordLink};
pub use saved::{AdmittedSource, SavedView, SourceResult, ViewQuery};

pub use eidetic::pack::TermKind;
pub use mere::profile::load::Profile;

pub const WEB_ANNOTATION: &str = include_str!("../packs/web-annotation.toml");
pub const CITATION: &str = include_str!("../packs/citation.toml");

/// Register the bundled W3C Web Annotation context in a host's offline loader.
pub fn with_web_annotation_context(cache: mere::ContextCache) -> mere::ContextCache {
    let context = include_bytes!("../assets/anno.jsonld").as_slice();
    cache
        .with("http://www.w3.org/ns/anno.jsonld", context)
        .with("https://www.w3.org/ns/anno.jsonld", context)
}

#[cfg(test)]
mod tests;
