// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The `block-guest` test fixture: claims the class `fixture` and exercises every call.
//!
//! `mark` proposes one change per selected row and `apply` returns the whole attribute map;
//! `stamp` and `echo` are `run` commands; `spin` never returns, so the host's watchdog can be
//! shown to trap it; `view` returns a scene naming two event regions and a sheet with a
//! compound path, a stroke and a text run.

wit_bindgen::generate!({
    path: "../../wit",
    world: "block-guest",
});

use exports::mere::script::block_adapter::{
    Attr, Block, Change, Claim, Command, CommandKind, Example, Guest, Row, RunResult, ViewAsk,
    ViewMode, ViewResult,
};
use sceno::{
    Footprint, InstanceId, ProjectedItem, Region, Representation, Scene, Size2, SourceIx,
    SourceRef, Transform2, Vec2,
};
use vector_sheet::{
    Anchor, Cap, Face, FillRule, Join, Op, Paint, Path, Role, Seg, VectorSheet,
};

struct Component;

fn attrs_of(block: &Block) -> Vec<Attr> {
    block.attrs.clone()
}

fn with(mut attrs: Vec<Attr>, key: &str, value: String) -> Vec<Attr> {
    attrs.retain(|a| a.key != key);
    attrs.push(Attr { key: key.into(), value });
    attrs
}

impl Guest for Component {
    fn describe() -> Claim {
        Claim {
            class: "fixture".into(),
            source_key: "fixture".into(),
            source_versions: vec![1],
            commands: vec![
                Command { id: "mark".into(), label: "Mark rows".into(), kind: CommandKind::Choice },
                Command { id: "stamp".into(), label: "Stamp".into(), kind: CommandKind::Write },
                Command { id: "echo".into(), label: "Echo".into(), kind: CommandKind::Text },
                Command { id: "spin".into(), label: "Spin".into(), kind: CommandKind::Write },
            ],
            views: vec![ViewMode { id: "plain".into(), label: "Plain".into() }],
            example: Some(Example {
                title: "Fixture block".into(),
                body: "hello\n".into(),
                attrs: vec![Attr { key: "class".into(), value: "fixture".into() }],
            }),
        }
    }

    fn rows(b: Block) -> Result<Vec<Row>, String> {
        Ok(b.body
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
            .map(|(i, l)| Row { index: i as u32, label: l.to_string() })
            .collect())
    }

    fn choices(b: Block, command: String) -> Result<Vec<Change>, String> {
        if command != "mark" {
            return Err(format!("Unknown command {command}"));
        }
        Ok(b.selected
            .iter()
            .map(|&row| Change {
                index: row,
                label: format!("Row {}", row + 1),
                before: None,
                after: "x".into(),
                conflict: false,
                use_update: false,
            })
            .collect())
    }

    fn apply(b: Block, command: String, changes: Vec<Change>) -> Result<Vec<Attr>, String> {
        if command != "mark" {
            return Err(format!("Unknown command {command}"));
        }
        Ok(with(attrs_of(&b), "fixture", format!("marked:{}", changes.len())))
    }

    fn run(b: Block, command: String, arg: String) -> Result<RunResult, String> {
        match command.as_str() {
            "stamp" => Ok(RunResult::Attrs(with(attrs_of(&b), "stamp", arg))),
            "echo" => Ok(RunResult::Text(format!("{}:{}", b.body.len(), arg))),
            "spin" => loop {
                std::hint::black_box(0);
            },
            other => Err(format!("Unknown command {other}")),
        }
    }

    fn view(b: Block, ask: ViewAsk) -> Result<ViewResult, String> {
        if ask.mode != "plain" {
            return Err(format!("Unknown view {}", ask.mode));
        }
        let mut scene = Scene::new();
        let mut regions = Vec::new();
        for event in 0..2u32 {
            let source = scene.sources.len();
            scene.sources.push(SourceRef::new("fixture.event", format!("bar:0/event:{event}")));
            scene.items.push(ProjectedItem {
                source: SourceIx(source as u32),
                space: Scene::WORLD,
                transform: Transform2 {
                    translate: Vec2::new(30.0 + 40.0 * event as f32, 20.0),
                    ..Transform2::IDENTITY
                },
                footprint: Footprint::Rect { size: Size2::new(30.0, 30.0) },
                representation: Representation::Open { kind: "fixture:event".into() },
                layer: 1,
                visible: true,
                hit: None,
                channels: vec![],
            });
            regions.push(Region {
                members: vec![InstanceId(event)],
                space: Scene::WORLD,
                contour: None,
                label: Some(format!("Event {} of {}", event + 1, b.class)),
                confidence: 1.0,
            });
        }
        scene.regions = regions;
        let mut scene_bytes = vec![1u8];
        ciborium::into_writer(&scene, &mut scene_bytes).map_err(|e| e.to_string())?;

        let selected_role = if ask.selected.is_some() { Role::Accent } else { Role::Ink };
        let ring = [
            Path::rect(5.0, 5.0, 30.0, 30.0).segs,
            vec![
                Seg::Move(12.0, 12.0),
                Seg::Line(12.0, 28.0),
                Seg::Line(28.0, 28.0),
                Seg::Line(28.0, 12.0),
                Seg::Close,
            ],
        ]
        .concat();
        let sheet = VectorSheet {
            width: ask.width.max(100.0),
            height: 50.0,
            ops: vec![
                Op::Fill { path: Path { segs: ring }, rule: FillRule::NonZero, paint: Paint::new(selected_role) },
                Op::Stroke {
                    path: Path { segs: vec![Seg::Move(0.0, 45.0), Seg::Line(100.0, 45.0)] },
                    width: 1.0, cap: Cap::Butt, join: Join::Miter, paint: Paint::new(Role::Muted),
                },
                Op::Text { x: 4.0, y: 0.0, size: 12.0, text: "fixture".into(), anchor: Anchor::Start, face: Face::Reading, paint: Paint::new(Role::Ink) },
            ],
        };
        let sheet_bytes = sheet.encode().map_err(|e| e.to_string())?;
        Ok(ViewResult {
            scene: scene_bytes,
            sheet: sheet_bytes,
            width: sheet.width,
            height: sheet.height,
            description: format!("fixture view {}", ask.mode),
        })
    }
}

export!(Component);
