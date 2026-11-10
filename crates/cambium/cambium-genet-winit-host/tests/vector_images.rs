// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An application's vector images (`HostOptions::vector_images`): the route a
//! face beside a node's name takes into layout and paint (iconography T4).

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use cambium::{AnyView, GenetCtx, GenetElement, el};
use cambium_genet_winit_host::{Harness, HostOptions, Init, VectorImage, inert_hooks};
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use paint_list_api::{LayoutRect, PaintCmd};

type Child = Box<dyn AnyView<(), (), GenetCtx, GenetElement>>;

#[derive(Debug, Default)]
struct Probe {
    scales: Mutex<Vec<f32>>,
    painted: Mutex<Vec<LayoutRect>>,
}

#[derive(Debug)]
struct Square(Arc<Probe>);

impl VectorImage for Square {
    fn intrinsic_size(&self) -> (f32, f32) {
        (64.0, 64.0)
    }

    fn paint(&self, rect: LayoutRect) -> Vec<PaintCmd> {
        self.0.painted.lock().unwrap().push(rect);
        Vec::new()
    }
}

fn root(_: &()) -> Child {
    Box::new(el(
        "p",
        (
            el("img", ()).attr("src", "face:61").attr("class", "face"),
            el("img", ())
                .attr("src", "https://elsewhere.test/a.png")
                .attr("class", "face"),
        ),
    ))
}

fn harness(probe: &Arc<Probe>, zoom: f32) -> Harness<(), fn(&()) -> Child, Child> {
    let source = probe.clone();
    let options = HostOptions {
        ui_zoom: zoom,
        vector_images: Some(Rc::new(move |url: &str, scale: f32| {
            source.scales.lock().unwrap().push(scale);
            (url == "face:61").then(|| Arc::new(Square(source.clone())) as Arc<dyn VectorImage>)
        })),
        ..Default::default()
    };
    let init = Init {
        state: (),
        logic: root as fn(&()) -> Child,
        sheet: "img.face { height: 20px; width: auto; }".into(),
        fonts: Vec::new(),
        images: Vec::new(),
    };
    let mut harness = Harness::with_hooks_and_options(init, inert_hooks(), options);
    harness.layout_at(400.0, 200.0);
    harness
}

fn images(harness: &Harness<(), fn(&()) -> Child, Child>) -> Vec<(f32, f32, f32, f32)> {
    let nodes = harness.with_dom(|dom| {
        let src = LocalName::from("src");
        let mut found = Vec::new();
        let mut stack = vec![dom.document()];
        while let Some(node) = stack.pop() {
            if dom.attribute(node, &Namespace::default(), &src).is_some() {
                found.push(node);
            }
            stack.extend(dom.dom_children(node));
        }
        found.sort_by_key(|node| {
            dom.attribute(*node, &Namespace::default(), &LocalName::from("src"))
                .map(str::to_owned)
        });
        found
    });
    nodes
        .into_iter()
        .map(|node| harness.painted_rect(node).expect("laid out"))
        .collect()
}

#[test]
fn a_vector_image_from_the_options_is_laid_out_at_its_ratio_and_painted() {
    let probe = Arc::new(Probe::default());
    let mut harness = harness(&probe, 1.0);
    let rects = images(&harness);
    let face = rects[0];
    assert_eq!(
        (face.2, face.3),
        (20.0, 20.0),
        "the face keeps its square ratio"
    );
    assert_ne!(
        rects[1].2, 20.0,
        "an unknown URL is not the face: {rects:?}"
    );
    harness.capture_portable().expect("paints without a GPU");
    let painted = probe.painted.lock().unwrap().clone();
    assert!(
        painted.iter().any(|rect| rect.height() == 20.0),
        "painted into its box: {painted:?}"
    );
    assert!(
        probe
            .scales
            .lock()
            .unwrap()
            .iter()
            .all(|scale| *scale == 1.0)
    );
}

#[test]
fn the_source_draws_at_the_layout_scale() {
    let probe = Arc::new(Probe::default());
    let mut harness = harness(&probe, 2.0);
    harness.capture_portable().expect("paints without a GPU");
    let scales = probe.scales.lock().unwrap().clone();
    assert!(!scales.is_empty());
    assert!(scales.iter().all(|scale| *scale == 2.0), "{scales:?}");
}
