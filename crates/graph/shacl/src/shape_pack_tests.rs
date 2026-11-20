// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use eidetic::pack::{
    LensBody, NoReview, Origin, PACK2_FORMAT, PackHeader, PackKind, Refusal, Selection, TypedBody,
    sign_canonical,
};
use eidetic::schema::TrustEnvelope;
use identity::Ed25519Keypair;
use mere::profile::core::CORE_PACK_ID;
use mere::profile::lens::AdmittedAs;
use mere::profile::resolve::Offered;

use super::*;
use crate::validate;

const LENS: &str = "urn:shape-test:lens";
const SHAPES: &str = "urn:shape-test:shapes";
const PERSON_SHAPE: &str = r#"
    @prefix sh: <http://www.w3.org/ns/shacl#> .
    <urn:PersonShape> a sh:NodeShape ;
      sh:targetNode <urn:person> ;
      sh:property [ sh:path <urn:name> ; sh:minCount 1 ] .
"#;

fn canonical_turtle(turtle: &str) -> String {
    let data = mere::parse_turtle(turtle, None).unwrap();
    canonical_shape_nquads(&graph_from(&data)).unwrap()
}

fn header(set: &str, rdf: String) -> PackHeader {
    let lens = LensBody {
        shapes: vec![set.into()],
        ..LensBody::default()
    };
    PackHeader {
        format: PACK2_FORMAT.into(),
        id: LENS.into(),
        version: "1.0.0".into(),
        kind: vec![PackKind::Lens, PackKind::Shapes],
        licence: "CC0-1.0".into(),
        credit: Vec::new(),
        requires: Vec::new(),
        provides: vec!["urn:shape-test:".into()],
        contributes: Vec::new(),
        scopes: Vec::new(),
        parts: Vec::new(),
        body: BTreeMap::from([
            (PackKind::Lens, lens.to_value().unwrap()),
            (
                PackKind::Shapes,
                BodyValue::Map(BTreeMap::from([(set.into(), BodyValue::Text(rdf))])),
            ),
        ]),
    }
}

fn load(header: &PackHeader) -> Result<ShapeProfile, ShapeLoadError> {
    load_local_shape_pack(&header.to_canonical().unwrap(), &Limits::default())
}

#[test]
fn local_pack_reopens_exactly_and_validates() {
    let header = header(SHAPES, canonical_turtle(PERSON_SHAPE));
    let one = load(&header).expect("admit");
    let retained = header.to_canonical().unwrap();
    let two = load_local_shape_pack(&retained, &Limits::default()).expect("re-admit offline");
    assert_eq!(
        one.selection().to_canonical().unwrap(),
        two.selection().to_canonical().unwrap()
    );
    let shape = two.shape(SHAPES).unwrap();
    assert_eq!(two.shapes().len(), 1);
    assert_eq!(shape.iri(), SHAPES);
    assert_eq!(&two.selection().lens, shape.pack_pin());
    assert!(shape.pack_pin().matches(&retained));
    assert_eq!(
        two.selection().pins.last().unwrap().admitted,
        AdmittedAs::Local
    );

    let missing = validate(
        mere::parse_nquads("", None).unwrap(),
        View::Live,
        GraphSelector::Default,
        shape.graph(),
    )
    .unwrap();
    assert!(!missing.conforms);
    assert_eq!(missing.findings.len(), 1);
    let present = validate(
        mere::parse_nquads("<urn:person> <urn:name> \"Ada\" .", None).unwrap(),
        View::Live,
        GraphSelector::Default,
        shape.graph(),
    )
    .unwrap();
    assert!(present.conforms);
}

#[test]
fn malformed_shape_returns_a_meta_report() {
    let bad = canonical_turtle(&PERSON_SHAPE.replace("sh:minCount 1", "sh:minCount \"one\""));
    let ShapeLoadError::Set {
        pack,
        set,
        error: ShapeSetError::MetaReport(report),
    } = load(&header(SHAPES, bad)).unwrap_err()
    else {
        panic!("expected SHACL-SHACL findings");
    };
    assert_eq!(pack, LENS);
    assert_eq!(set, SHAPES);
    assert!(!report.conforms);
    assert!(!report.findings.is_empty());
    assert!(!report.dataset.is_empty());
}

#[test]
fn rdf_must_be_canonical_and_scoped_to_one_graph() {
    let nquads = canonical_turtle(PERSON_SHAPE);
    let cases = [
        (format!("{nquads}\n"), ShapeSetError::NonCanonical),
        (
            "<urn:s> <urn:p> <urn:o> <urn:other> .\n".into(),
            ShapeSetError::NamedGraph,
        ),
        (
            "<urn:s> <urn:p> <<( <urn:s> <urn:p> <urn:o> )>> .\n".into(),
            ShapeSetError::Canonicalization(CanonError::TripleTerm),
        ),
        (
            "<urn:s> <http://www.w3.org/2002/07/owl#imports> <urn:remote> .\n".into(),
            ShapeSetError::UnsupportedDirective(OWL_IMPORTS),
        ),
    ];
    for (rdf, expected) in cases {
        let ShapeLoadError::Set { error, .. } = load(&header(SHAPES, rdf)).unwrap_err() else {
            panic!("expected a shape-set refusal");
        };
        assert_eq!(error.to_string(), expected.to_string());
    }
}

#[test]
fn poison_budget_runs_before_meta_shacl() {
    let mut nquads = String::new();
    for left in 0..10 {
        for right in 0..10 {
            use std::fmt::Write;
            writeln!(nquads, "_:n{left} <urn:link> _:n{right} .").unwrap();
        }
    }
    assert!(matches!(
        load(&header(SHAPES, nquads)),
        Err(ShapeLoadError::Set {
            error: ShapeSetError::Canonicalization(
                CanonError::HndqCallLimitExceeded(_) | CanonError::PermutationLimitExceeded(_)
            ),
            ..
        })
    ));
}

#[test]
fn lens_requires_admitted_namespaced_sets_and_cannot_replace_core() {
    let mut fixture = header("urn:elsewhere:set", canonical_turtle(PERSON_SHAPE));
    assert!(matches!(
        load(&fixture),
        Err(ShapeLoadError::Set {
            error: ShapeSetError::Undeclared,
            ..
        })
    ));
    fixture
        .body
        .insert(PackKind::Shapes, BodyValue::Map(BTreeMap::new()));
    assert!(matches!(
        load(&fixture),
        Err(ShapeLoadError::Missing { .. })
    ));
    fixture.id = CORE_PACK_ID.into();
    assert!(matches!(
        load(&fixture),
        Err(ShapeLoadError::Load(LoadRefusal::DigestMismatch { .. }))
    ));
}

struct Source(Offered);

impl PackSource for Source {
    fn offer(&self, id: &str) -> Vec<Offered> {
        if id == LENS {
            vec![self.0.clone()]
        } else {
            Vec::new()
        }
    }

    fn handler_known(&self, _: &str) -> bool {
        false
    }
}

#[test]
fn crossing_trust_still_requires_accepted_signature() {
    let mut source = Source(Offered {
        canonical: header(SHAPES, canonical_turtle(PERSON_SHAPE))
            .to_canonical()
            .unwrap(),
        envelope: TrustEnvelope::self_asserted(),
        origin: Origin::CrossesTrust,
    });
    let release = Ed25519Keypair::from_seed([42; 32]);
    let policy = TrustPolicy {
        release_key: release.public_key(),
        review: NoReview,
    };
    let lens = PackRef::Select(Selection {
        pack: LENS.into(),
        version: "1.0.0".into(),
    });
    assert!(matches!(
        load_shape_profile(&lens, &source, &policy, &Limits::default()),
        Err(ShapeLoadError::Load(LoadRefusal::Trust {
            refusal: Refusal::Unsigned,
            ..
        }))
    ));
    source
        .0
        .envelope
        .signatures
        .push(sign_canonical(&source.0.canonical, &release));
    let granted = load_shape_profile(&lens, &source, &policy, &Limits::default()).unwrap();
    assert_eq!(
        granted.selection().pins.last().unwrap().admitted,
        AdmittedAs::Release
    );
}
