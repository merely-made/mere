// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shape sets of an admitted declared lens. Each set is a separate graph.

use std::collections::BTreeMap;
use std::fmt;

use eidetic::pack::{BodyValue, Pin, SignerReview, TrustPolicy};
use mere::dataset::{IndexedDataset, View};
use mere::formats::canon::{CanonError, CanonOptions, canonicalize_quads};
use mere::profile::lens::SelectionRecord;
use mere::profile::load::{
    Limits, LoadRefusal, Profile, ShapePackBody, ShapeSelection, load_local_shape_selection,
    load_shape_selection,
};
use mere::profile::resolve::{PackRef, PackSource};
use oxrdf::{Graph, GraphName, NamedNodeRef, Quad, Triple};
use shacl::ir::IRSchema;

use crate::{GraphSelector, ValidationError, ValidationReport, compile_shapes, validate_compiled};

const META_SHAPES: &str = include_str!("../assets/shacl-shacl.ttl");
const OWL_IMPORTS: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("http://www.w3.org/2002/07/owl#imports");
const SH_ENTAILMENT: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("http://www.w3.org/ns/shacl#entailment");

/// An active shape set with its independent graph and the pin that declared it.
#[derive(Debug, Clone)]
pub struct PackedShape {
    iri: String,
    pack_pin: Pin,
    graph: Graph,
}

impl PackedShape {
    pub fn iri(&self) -> &str {
        &self.iri
    }

    pub fn pack_pin(&self) -> &Pin {
        &self.pack_pin
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }
}

/// The loaded profile with explicitly activated and checked shape sets.
#[derive(Debug, Clone)]
pub struct ShapeProfile {
    profile: Profile,
    shapes: BTreeMap<String, PackedShape>,
}

impl ShapeProfile {
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn selection(&self) -> &SelectionRecord {
        &self.profile.selection
    }

    pub fn shapes(&self) -> impl ExactSizeIterator<Item = &PackedShape> {
        self.shapes.values()
    }

    pub fn shape(&self, iri: &str) -> Option<&PackedShape> {
        self.shapes.get(iri)
    }
}

#[derive(Debug)]
pub enum ShapeLoadError {
    Load(LoadRefusal),
    Body {
        pack: String,
    },
    Set {
        pack: String,
        set: String,
        error: ShapeSetError,
    },
    Duplicate {
        set: String,
        packs: [String; 2],
    },
    Missing {
        lens: String,
        set: String,
    },
    NoActiveSet {
        lens: String,
    },
    Meta(ValidationError),
}

#[derive(Debug)]
pub enum ShapeSetError {
    InvalidIri,
    Undeclared,
    ExpectedText,
    Rdf(String),
    NamedGraph,
    Canonicalization(CanonError),
    NonCanonical,
    UnsupportedDirective(NamedNodeRef<'static>),
    InvalidShape(ValidationError),
    MetaReport(Box<ValidationReport>),
}

impl fmt::Display for ShapeLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(error) => write!(formatter, "Pack load refused: {error:?}"),
            Self::Body { pack } => write!(formatter, "Pack {pack} must map shape set IRIs to RDF"),
            Self::Set { pack, set, error } => {
                write!(formatter, "Shape set {set} in {pack}: {error}")
            },
            Self::Duplicate { set, packs: [a, b] } => {
                write!(formatter, "Both {a} and {b} declare shape set {set}")
            },
            Self::Missing { lens, set } => write!(formatter, "Lens {lens} needs shape set {set}"),
            Self::NoActiveSet { lens } => write!(formatter, "Lens {lens} selects no shape set"),
            Self::Meta(error) => write!(formatter, "SHACL-SHACL validation failed: {error}"),
        }
    }
}

impl fmt::Display for ShapeSetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIri => formatter.write_str("Expected an absolute set IRI"),
            Self::Undeclared => formatter.write_str("Set IRI is outside the provided namespaces"),
            Self::ExpectedText => formatter.write_str("Expected canonical N-Quads text"),
            Self::Rdf(error) => write!(formatter, "Could not parse N-Quads: {error}"),
            Self::NamedGraph => formatter.write_str("A set must contain one default graph"),
            Self::Canonicalization(error) => write!(formatter, "RDFC-1.0 refused the set: {error}"),
            Self::NonCanonical => formatter.write_str("Expected RDFC-1.0 canonical N-Quads"),
            Self::UnsupportedDirective(iri) => write!(formatter, "Unsupported directive {iri}"),
            Self::InvalidShape(error) => write!(formatter, "Could not compile SHACL: {error}"),
            Self::MetaReport(report) => {
                write!(
                    formatter,
                    "SHACL-SHACL found {} problems",
                    report.findings.len()
                )
            },
        }
    }
}

impl std::error::Error for ShapeLoadError {}

/// Resolve and check inline shapes for a lens using the caller's pack source
/// and trust policy. A refusal returns before the host can use a shape set.
///
/// # Errors
///
/// A pack load, shape identity, canonicalization or SHACL refusal.
pub fn load_shape_profile<R: SignerReview>(
    lens: &PackRef,
    source: &dyn PackSource,
    policy: &TrustPolicy<R>,
    limits: &Limits,
) -> Result<ShapeProfile, ShapeLoadError> {
    interpret(load_shape_selection(lens, source, policy, limits).map_err(ShapeLoadError::Load)?)
}

/// Load a canonical bare pack explicitly selected by the local user.
/// The same bytes can be held by the caller and re-admitted offline.
///
/// # Errors
///
/// A pack load, shape identity, canonicalization or SHACL refusal, including
/// a local pack whose lens selects no shape set.
pub fn load_local_shape_pack(
    bytes: &[u8],
    limits: &Limits,
) -> Result<ShapeProfile, ShapeLoadError> {
    let selected = load_local_shape_selection(bytes, limits).map_err(ShapeLoadError::Load)?;
    if selected.profile.lens.shapes.is_empty() {
        return Err(ShapeLoadError::NoActiveSet {
            lens: selected.profile.selection.lens.id,
        });
    }
    interpret(selected)
}

/// Produce the canonical inline N-Quads for a shape graph. The pack uses
/// RDFC-1.0 with the default SHA-256 hash and bounded canonicalization work.
///
/// # Errors
///
/// An RDF 1.2 triple term or an exhausted RDFC work budget.
pub fn canonical_shape_nquads(graph: &Graph) -> Result<String, CanonError> {
    let quads: Vec<Quad> = graph
        .iter()
        .map(|triple| triple.into_owned().in_graph(GraphName::DefaultGraph))
        .collect();
    Ok(canonicalize_quads(&quads, &CanonOptions::default())?.nquads)
}

fn interpret(selected: ShapeSelection) -> Result<ShapeProfile, ShapeLoadError> {
    let mut declared: BTreeMap<String, PackedShape> = BTreeMap::new();
    if !selected.packs.is_empty() {
        let meta = mere::parse_turtle(META_SHAPES, None)
            .map_err(|error| ShapeLoadError::Meta(crate::validation_error(error)))?;
        let meta = compile_shapes(&graph_from(&meta)).map_err(ShapeLoadError::Meta)?;
        for pack in &selected.packs {
            let BodyValue::Map(sets) = &pack.body else {
                return Err(ShapeLoadError::Body {
                    pack: pack.pin.id.clone(),
                });
            };
            for (iri, text) in sets {
                let refusal = |error| ShapeLoadError::Set {
                    pack: pack.pin.id.clone(),
                    set: iri.clone(),
                    error,
                };
                if let Some(prior) = declared.get(iri) {
                    return Err(ShapeLoadError::Duplicate {
                        set: iri.clone(),
                        packs: [prior.pack_pin.id.clone(), pack.pin.id.clone()],
                    });
                }
                let graph = check_set(pack, iri, text, &meta).map_err(refusal)?;
                declared.insert(
                    iri.clone(),
                    PackedShape {
                        iri: iri.clone(),
                        pack_pin: pack.pin.clone(),
                        graph,
                    },
                );
            }
        }
    }
    let shapes = selected
        .profile
        .lens
        .shapes
        .iter()
        .map(|iri| {
            declared
                .remove(iri)
                .map(|shape| (iri.clone(), shape))
                .ok_or_else(|| ShapeLoadError::Missing {
                    lens: selected.profile.selection.lens.id.clone(),
                    set: iri.clone(),
                })
        })
        .collect::<Result<_, _>>()?;
    Ok(ShapeProfile {
        profile: selected.profile,
        shapes,
    })
}

fn check_set(
    pack: &ShapePackBody,
    iri: &str,
    value: &BodyValue,
    meta: &IRSchema,
) -> Result<Graph, ShapeSetError> {
    NamedNodeRef::new(iri).map_err(|_| ShapeSetError::InvalidIri)?;
    if !pack
        .provides
        .iter()
        .any(|namespace| iri.starts_with(namespace.as_str()))
    {
        return Err(ShapeSetError::Undeclared);
    }
    let BodyValue::Text(text) = value else {
        return Err(ShapeSetError::ExpectedText);
    };
    let data =
        mere::parse_nquads(text, None).map_err(|error| ShapeSetError::Rdf(error.to_string()))?;
    if data
        .quads(View::Live)
        .any(|quad| quad.graph_name != GraphName::DefaultGraph)
    {
        return Err(ShapeSetError::NamedGraph);
    }
    let graph = graph_from(&data);
    let canonical = canonical_shape_nquads(&graph).map_err(ShapeSetError::Canonicalization)?;
    if canonical != *text {
        return Err(ShapeSetError::NonCanonical);
    }
    for directive in [OWL_IMPORTS, SH_ENTAILMENT] {
        if graph.triples_for_predicate(directive).next().is_some() {
            return Err(ShapeSetError::UnsupportedDirective(directive));
        }
    }
    let report = validate_compiled(data, View::Live, GraphSelector::Default, meta)
        .map_err(ShapeSetError::InvalidShape)?;
    if !report.conforms {
        return Err(ShapeSetError::MetaReport(Box::new(report)));
    }
    compile_shapes(&graph).map_err(ShapeSetError::InvalidShape)?;
    Ok(graph)
}

fn graph_from(dataset: &IndexedDataset) -> Graph {
    dataset
        .quads(View::Live)
        .map(|quad| Triple::new(quad.subject, quad.predicate, quad.object))
        .collect()
}

#[cfg(test)]
#[path = "shape_pack_tests.rs"]
mod tests;
