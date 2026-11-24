// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Host-side admission of bundled and operator-selected local ontology packs.
mod catalog;
mod links;
pub use catalog::load;
pub use links::{RecordIndex, RecordLink};

pub use eidetic::pack::TermKind;
pub use mere::profile::load::Profile;

pub const WEB_ANNOTATION: &str = include_str!("../packs/web-annotation.toml");
pub const CITATION: &str = include_str!("../packs/citation.toml");

#[cfg(test)]
mod tests;
