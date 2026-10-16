// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Finite native Canvas / bounded Cambium Swatch fold receipt.
//! This is an acceptance fixture, not Isometry's campaign UI.
use cambium::{
    AnyView, GRAPH_CANVAS_SWATCH_CSS, GenetCtx, GenetElement, GraphCanvasNode, GraphCanvasRelation,
    GraphCanvasSubgraph, GraphCanvasSwatch, PointerPhase, clickable, custom_leaf, el, focusable,
    graph_canvas_swatch, on_pointer, text,
};
use cambium_genet_winit_host::{
    AppCtx, HostHooks, HostOptions, Init, SceneProducer, inert_hooks, run,
};
use kernel::{
    geometry::PortablePoint,
    graph::{EdgeFamily, NodeKey, SemanticSubKind},
};
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use mesquite::{CaptureRecord, Lane, LaneConfig, Product};
use pictograph::canvas::fold_projection::FoldTraversalDirection;
use pictograph::canvas::{CameraView, Canvas, FoldEndpoint, PointerButton};
use sprigging::ColorF;
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};
use taproot::ProbeSnapshot;

const ROOT: u64 = 9101;
const SWATCH: u64 = 9102;
const W: u32 = 440;
const H: u32 = 300;
const CSS: &str = ":root { background:#f7f5f0; color:#202832; font-family:sans-serif; font-size:14px; } main { padding:16px; } .panels { display:flex; gap:20px; } button { margin:4px; padding:8px; background:#e2e8ef; color:#202832; border:1px solid #61758a; } .panel { width:440px; } .graph-canvas-swatch { background:#fff; }";
fn sheet() -> String {
    format!("{GRAPH_CANVAS_SWATCH_CSS}\n{CSS}")
}
type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
type Logic = fn(&App) -> Child;

struct App {
    canvas: Canvas,
    ids: [uuid::Uuid; 5],
    keys: [NodeKey; 5],
    places: Vec<(NodeKey, PortablePoint)>,
    truth: Vec<u8>,
    reopened: bool,
    fresh_process: bool,
    generation: usize,
}
fn truth(canvas: &Canvas) -> Vec<u8> {
    let mut snapshot = canvas.graph().to_snapshot();
    snapshot.timestamp_secs = 0;
    serde_json::to_vec(&snapshot).unwrap()
}
impl App {
    fn new() -> Self {
        let mut graph = kernel::graph::Graph::new();
        let mut next_id = 0u128;
        let keys = ["Root", "Leaf A", "Leaf B", "Other", "Leaf C"].map(|name| {
            next_id += 1;
            kernel::graph::apply::add_node(
                &mut graph,
                Some(uuid::Uuid::from_u128(next_id)),
                format!("https://fold.test/{name}"),
                Default::default(),
            )
        });
        let ids = keys.map(|key| graph.get_node(key).unwrap().id);
        for (from, to) in [(0, 1), (0, 2), (3, 4)] {
            kernel::graph::apply::assert_relation(
                &mut graph,
                keys[from],
                keys[to],
                kernel::graph::EdgeAssertion::Containment {
                    sub_kind: kernel::graph::ContainmentSubKind::Domain,
                },
            );
        }
        let mut canvas = Canvas::with_graph(graph);
        for (a, b, kind) in [
            (1, 3, SemanticSubKind::Cites),
            (2, 3, SemanticSubKind::Quotes),
            (3, 1, SemanticSubKind::Cites),
        ] {
            assert!(canvas.assert_relation_between_members(ids[a], ids[b], kind));
        }
        canvas.set_physics_paused(true);
        canvas.set_layout_strategy(Some("acceptance-fixed".into()));
        let places = keys
            .into_iter()
            .zip([
                (75., 75.),
                (75., 180.),
                (185., 180.),
                (340., 75.),
                (340., 205.),
            ])
            .map(|(key, (x, y))| (key, PortablePoint::new(x, y)))
            .collect::<Vec<_>>();
        canvas.apply_strategy_positions(&places);
        canvas.set_camera(CameraView {
            offset: (0., 0.),
            zoom: 1.,
        });
        let _ = canvas.frame(W, H);
        let truth = truth(&canvas);
        let fresh_process = if let Some(path) = std::env::var_os("FOLD_ACCEPTANCE_RESTORE") {
            let saved = std::fs::read(path).unwrap();
            let records: Vec<curation::FoldRecord> = serde_json::from_slice(&saved).unwrap();
            assert!(canvas.restore_folds(records.clone()).is_empty());
            assert_eq!(canvas.fold_records(), records);
            true
        } else {
            false
        };
        Self {
            canvas,
            ids,
            keys,
            places,
            truth,
            reopened: false,
            fresh_process,
            generation: 0,
        }
    }
    fn select(&mut self, a: usize, b: usize) {
        assert!(self.canvas.select_member(self.ids[a]));
        assert!(self.canvas.toggle_select_member(self.ids[b]));
    }
    fn command(&mut self, command: &str) {
        match command {
            "Fold A and B" => {
                self.select(1, 2);
                assert!(self.canvas.fold_selected("fold-acceptance").is_some());
            },
            "Fold other pair" => {
                self.select(3, 4);
                assert!(self.canvas.fold_selected("fold-acceptance").is_some());
            },
            "Expand all" => while self.canvas.expand_selected_fold() {},
            "Undo fold" => assert!(self.canvas.undo_fold()),
            "Redo fold" => assert!(self.canvas.redo_fold()),
            "Collapse containment outgoing" => {
                assert!(
                    self.canvas
                        .collapse_descendants(
                            self.keys[0],
                            EdgeFamily::Containment,
                            FoldTraversalDirection::Outgoing,
                            "fold-acceptance"
                        )
                        .is_some()
                );
            },
            "Collapse containment incoming" => {
                assert!(
                    self.canvas
                        .collapse_descendants(
                            self.keys[1],
                            EdgeFamily::Containment,
                            FoldTraversalDirection::Incoming,
                            "fold-acceptance"
                        )
                        .is_some()
                );
            },
            "Collapse semantic outgoing" => {
                assert!(
                    self.canvas
                        .collapse_descendants(
                            self.keys[1],
                            EdgeFamily::Semantic,
                            FoldTraversalDirection::Outgoing,
                            "fold-acceptance"
                        )
                        .is_some()
                );
            },
            "Reload serialized folds" => {
                let json = serde_json::to_vec(self.canvas.fold_records()).unwrap();
                if let Some(path) = std::env::var_os("FOLD_ACCEPTANCE_STATE") {
                    std::fs::write(path, &json).unwrap();
                }
                let records: Vec<curation::FoldRecord> = serde_json::from_slice(&json).unwrap();
                assert!(self.canvas.restore_folds([]).is_empty());
                assert!(self.canvas.restore_folds(records).is_empty());
                self.reopened = true;
            },
            _ => panic!("unknown command"),
        }
        assert_eq!(truth(&self.canvas), self.truth);
        self.generation += 1;
    }
    fn endpoint(&self, endpoint: FoldEndpoint) -> String {
        match endpoint {
            FoldEndpoint::Node(key) => self.canvas.graph().get_node(key).unwrap().id.to_string(),
            FoldEndpoint::Fold(i) => format!("fold:{}", self.canvas.fold_projections()[i].fold_id),
        }
    }
    fn swatch(&self) -> GraphCanvasSwatch<String, bool> {
        let folds = self.canvas.fold_projections();
        let membership = folds
            .iter()
            .flat_map(|f| f.members.iter().copied())
            .collect::<BTreeSet<_>>();
        let names = ["Root", "Leaf A", "Leaf B", "Other", "Leaf C"];
        let mut nodes = self
            .keys
            .iter()
            .enumerate()
            .filter(|(_, k)| !membership.contains(k))
            .map(|(i, &k)| {
                let p = self.canvas.node_position(k).unwrap();
                GraphCanvasNode {
                    id: self.ids[i].to_string(),
                    kind: false,
                    position: (p.x / W as f32, p.y / H as f32),
                    label: names[i].into(),
                    key: None,
                }
            })
            .collect::<Vec<_>>();
        for (i, fold) in folds.iter().enumerate() {
            let p = fold
                .stand_in
                .and_then(|k| self.canvas.node_position(k))
                .or_else(|| fold.summary_center(|k| self.canvas.node_position(k)))
                .unwrap();
            nodes.push(GraphCanvasNode {
                id: format!("fold:{}", fold.fold_id),
                kind: true,
                position: (p.x / W as f32, p.y / H as f32),
                label: format!("Expand fold {i} {}", fold.badge()),
                key: Some(format!("fold-{i}")),
            });
        }
        let mut relations = Vec::new();
        for (i, route) in self.canvas.fold_routes().into_iter().enumerate() {
            relations.push(GraphCanvasRelation {
                link_key: None,
                marks: Default::default(),
                strength: 1.0,
                id: format!("boundary:{i}"),
                from: self.endpoint(route.from),
                to: self.endpoint(route.to),
                kind: format!("{:?}", route.family),
                label: format!("{} cells", route.count),
                route: vec![],
                visible: true,
                emphasized: route.count > 1,
            });
        }
        for (i, (_, r)) in self.canvas.graph().projected_relations().enumerate() {
            if membership.contains(&r.from) || membership.contains(&r.to) {
                continue;
            }
            relations.push(GraphCanvasRelation {
                link_key: None,
                marks: Default::default(),
                strength: 1.0,
                id: format!("source:{i}"),
                from: self.endpoint(FoldEndpoint::Node(r.from)),
                to: self.endpoint(FoldEndpoint::Node(r.to)),
                kind: format!("{:?}", r.kind.family()),
                label: "1 cell".into(),
                route: vec![],
                visible: true,
                emphasized: false,
            });
        }
        GraphCanvasSwatch::new(
            SWATCH,
            GraphCanvasSubgraph {
                nodes,
                edges: vec![],
            },
        )
        .with_size(W, H)
        .with_expand(false)
        .with_node_labels(true)
        .with_relations(relations)
        .with_label("Fold acceptance Swatch")
    }
    fn unchanged_positions(&self) -> bool {
        self.places
            .iter()
            .all(|(k, p)| self.canvas.node_position(*k) == Some(*p))
    }
}
fn button(name: &'static str) -> Child {
    Box::new(focusable(clickable(
        el("button", text(name)),
        move |s: &mut App, _| s.command(name),
    )))
}
fn root(state: &App) -> Child {
    let swatch = state.swatch();
    let controls = [
        "Fold A and B",
        "Fold other pair",
        "Expand all",
        "Undo fold",
        "Redo fold",
        "Reload serialized folds",
    ]
    .map(button)
    .into_iter()
    .collect::<Vec<_>>();
    let native = on_pointer(
        custom_leaf(ROOT, W, H)
            .attr("aria-label", "Native Canvas")
            .attr("role", "button")
            .attr("data-native-fold", "true"),
        |s: &mut App, e: cambium::PointerEvent| match e.phase {
            PointerPhase::Down => {
                s.canvas
                    .pointer_down(PointerButton::Left, e.local.0, e.local.1);
            },
            PointerPhase::Up => {
                s.canvas
                    .pointer_up(PointerButton::Left, e.local.0, e.local.1);
                s.generation += 1;
            },
            _ => {},
        },
    );
    Box::new(el(
        "main",
        (
            el("h1", text("Native folds: Canvas and bounded Swatch")),
            el("div", controls),
            el(
                "div",
                [
                    "Collapse containment outgoing",
                    "Collapse containment incoming",
                    "Collapse semantic outgoing",
                ]
                .map(button)
                .into_iter()
                .collect::<Vec<_>>(),
            ),
            el(
                "div",
                (
                    el("section", (el("h2", text("Root Canvas")), native)).attr("class", "panel"),
                    el(
                        "section",
                        (
                            el("h2", text("Bounded Swatch")),
                            graph_canvas_swatch(
                                &swatch,
                                |s: &mut App, id: String| {
                                    if let Some(index) = s
                                        .canvas
                                        .fold_projections()
                                        .iter()
                                        .position(|f| format!("fold:{}", f.fold_id) == id)
                                    {
                                        let id = s.canvas.fold_projections()[index].fold_id.clone();
                                        assert!(s.canvas.expand_fold(&id));
                                        s.generation += 1;
                                    }
                                },
                                |_: &mut App, _| {},
                                |_: &mut App| {},
                            ),
                        ),
                    )
                    .attr("class", "panel"),
                ),
            )
            .attr("class", "panels"),
            el(
                "p",
                text(format!(
                    "Folds: {} / boundary source cells: {} / graph truth unchanged: {}",
                    state.canvas.fold_records().len(),
                    state
                        .canvas
                        .fold_routes()
                        .iter()
                        .map(|r| r.count)
                        .sum::<usize>(),
                    truth(&state.canvas) == state.truth
                )),
            ),
        ),
    ))
}
struct Acceptance;
impl Product for Acceptance {
    type State = App;
    type Logic = Logic;
    type View = Child;
    const KIND: &'static str = "native-fold-acceptance";
    const SURFACE: &'static str = "app";
    const LOG_PREFIX: &'static str = "folds";
    fn sheet(&self) -> &str {
        CSS
    }
    fn snapshot(&self, ctx: &AppCtx<'_, App, Logic, Child>, _: usize, _: f32) -> ProbeSnapshot {
        let s = ctx.runner.state();
        let routes = s.canvas.fold_routes();
        ProbeSnapshot::default()
            .with_field("folds", s.canvas.fold_records().len().to_string())
            .with_field("routes", routes.len().to_string())
            .with_field(
                "cells",
                routes.iter().map(|r| r.count).sum::<usize>().to_string(),
            )
            .with_field(
                "max_bundle",
                routes
                    .iter()
                    .map(|r| r.count)
                    .max()
                    .unwrap_or(0)
                    .to_string(),
            )
            .with_field("swatch_nodes", s.swatch().graph.nodes.len().to_string())
            .with_field("truth", (truth(&s.canvas) == s.truth).to_string())
            .with_field("positions", s.unchanged_positions().to_string())
            .with_field("reopened", s.reopened.to_string())
            .with_field("fresh_process", s.fresh_process.to_string())
    }
    fn target_point(
        &self,
        ctx: &AppCtx<'_, App, Logic, Child>,
        node: genet_scripted_dom::NodeId,
        rect: [f32; 4],
    ) -> (f32, f32) {
        if ctx
            .runner
            .dom()
            .borrow()
            .attribute(
                node,
                &Namespace::from(""),
                &LocalName::from("data-native-fold"),
            )
            .is_some()
        {
            let s = ctx.runner.state();
            if let Some(fold) = s.canvas.fold_projections().first() {
                let p = fold.summary_center(|k| s.canvas.node_position(k)).unwrap();
                return (rect[0] + p.x, rect[1] + p.y);
            }
        }
        (rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5)
    }
    fn receipt_checks(&self, captures: &[CaptureRecord]) -> Vec<String> {
        if captures.len() < 5
            || captures
                .iter()
                .map(|c| c.digest)
                .collect::<BTreeSet<_>>()
                .len()
                < 3
        {
            vec!["Need five presented captures and at least three distinct frames".into()]
        } else {
            vec![]
        }
    }
}
fn main() {
    let config = LaneConfig::from_env("FOLD_ACCEPTANCE").expect("finite scenario required");
    let lane = Rc::new(RefCell::new(
        Lane::from_config(config, Acceptance, cambium_genet_winit_host::read_file).unwrap(),
    ));
    let mut hooks: HostHooks<App, Logic, Child> = inert_hooks();
    let mut painted = None;
    let source: Rc<RefCell<Option<(netrender::Scene, u64)>>> = Rc::new(RefCell::new(None));
    hooks.frame = Box::new(move |ctx| {
        let generation = ctx.runner.state().generation;
        if painted == Some(generation) {
            return false;
        }
        let mut scene = None;
        ctx.runner.update(|s| {
            scene = Some(s.canvas.frame(W, H).0);
            assert!(s.unchanged_positions());
            assert_eq!(truth(&s.canvas), s.truth);
        });
        *source.borrow_mut() = Some((scene.unwrap(), generation as u64));
        if !ctx.producers.contains(ROOT) {
            ctx.producers
                .register(
                    ROOT,
                    Rc::new(RefCell::new(SceneProducer::new(
                        source.clone(),
                        ROOT,
                        |s, _, _| s.as_ref().unwrap().0.clone(),
                        |s| s.as_ref().unwrap().1,
                        |_| None,
                    ))),
                    &[],
                )
                .unwrap();
        }
        ctx.leaves.insert(
            SWATCH,
            Box::new(ctx.runner.state().swatch().paint_leaf(|fold| {
                if *fold {
                    ColorF {
                        r: 0.6,
                        g: 0.25,
                        b: 0.65,
                        a: 1.,
                    }
                } else {
                    ColorF {
                        r: 0.15,
                        g: 0.4,
                        b: 0.7,
                        a: 1.,
                    }
                }
            })),
        );
        painted = Some(generation);
        true
    });
    hooks.after_frame = Box::new(move |ctx| lane.borrow_mut().after_frame(ctx));
    run(
        HostOptions {
            title: "Native fold acceptance".into(),
            initial_logical_size: (950., 550.),
            ..Default::default()
        },
        |_, _, _| Init {
            state: App::new(),
            logic: root as Logic,
            sheet: sheet(),
            fonts: vec![],
            images: vec![],
        },
        hooks,
    )
    .expect("native event loop");
}
