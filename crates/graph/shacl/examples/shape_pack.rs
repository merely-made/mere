// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Write a locally selectable test pack from synthetic Turtle. The input
//! must be authored for the fixture and available under CC0-1.0.

use std::collections::BTreeMap;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use eidetic::pack::{BodyValue, LensBody, PACK2_FORMAT, PackHeader, PackKind, TypedBody};
use mere::dataset::View;
use mere::profile::load::Limits;
use mere_shacl::{canonical_shape_nquads, load_local_shape_pack};
use oxrdf::{Graph, Triple};

const LENS: &str = "urn:mere:shacl-fixture:lens";
const SET: &str = "urn:mere:shacl-fixture:shapes";

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(input), Some(output), None) = (arguments.next(), arguments.next(), arguments.next())
    else {
        return Err("usage: shape_pack fixture.ttl fixture.pack".into());
    };
    let turtle = std::fs::read_to_string(PathBuf::from(input))?;
    let graph: Graph = mere::parse_turtle(&turtle, None)?
        .quads(View::Live)
        .map(|quad| Triple::new(quad.subject, quad.predicate, quad.object))
        .collect();
    let lens = LensBody {
        shapes: vec![SET.into()],
        ..LensBody::default()
    };
    let canonical = PackHeader {
        format: PACK2_FORMAT.into(),
        id: LENS.into(),
        version: "1.0.0".into(),
        kind: vec![PackKind::Lens, PackKind::Shapes],
        licence: "CC0-1.0".into(),
        credit: Vec::new(),
        requires: Vec::new(),
        provides: vec!["urn:mere:shacl-fixture:".into()],
        contributes: Vec::new(),
        scopes: Vec::new(),
        parts: Vec::new(),
        body: BTreeMap::from([
            (PackKind::Lens, lens.to_value()?),
            (
                PackKind::Shapes,
                BodyValue::Map(BTreeMap::from([(
                    SET.into(),
                    BodyValue::Text(canonical_shape_nquads(&graph)?),
                )])),
            ),
        ]),
    }
    .to_canonical()?;
    load_local_shape_pack(&canonical, &Limits::default())?;
    std::fs::File::create_new(PathBuf::from(output))?.write_all(&canonical)?;
    Ok(())
}
