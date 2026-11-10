// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! The block adapter contract, and the native host that runs a guest through it.
//!
//! An adapter claims one Djot div class. A block pane looks it up by class and calls it with a
//! copied [`BlockInput`]; the adapter proposes [`Change`]s, returns the complete desired
//! attribute map, and may return a [`GuestView`] (a `sceno` scene naming the regions plus a
//! vector sheet the host paints). A call may answer [`AdapterError::Pending`] when the host
//! cannot answer in the frame (a web Worker). [`GuestAdapter`] answers in process over the
//! `block-guest` world through `block-host`. Pack review is the caller's: the adapter is
//! attached to bytes and the powers the caller granted. Nothing here names an app or a guest's
//! owner.

use sceno::{Footprint, Scene};
use std::collections::BTreeMap;
use vector_sheet::VectorSheet;

#[cfg(feature = "guest")]
mod guest;
#[cfg(feature = "guest")]
pub use guest::{GuestAdapter, GuestEngine, TURN_TICKS};

/// One proposed edit to one row, as the pane shows it for review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub index: usize,
    pub label: String,
    pub before: Option<String>,
    pub after: String,
    pub conflict: bool,
    pub use_update: bool,
}

/// The hidden text command a class offers to draw the swatch: `run`'s argument is the block's
/// address and its result is a scene document (`SceneInput` JSON). It is never listed as a
/// button.
pub const BACKDROP_SCENE: &str = "backdrop-scene";

/// What a command is for. Mirrors the WIT `command-kind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandKind {
    /// Proposes changes the user reviews, then applies.
    Choice,
    /// Writes new attributes at once.
    Write,
    /// Produces text to export.
    Text,
}

/// A text field the pane shows for an adapter: a label and its default text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub label: String,
    pub default: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub id: String,
    pub label: String,
    pub kind: CommandKind,
    /// A field whose text is sent as `run`'s argument.
    pub arg: Option<Field>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewMode {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Example {
    pub title: String,
    pub body: String,
    pub attrs: BTreeMap<String, String>,
}

/// What an adapter claims. Called once when the adapter is registered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    /// The Djot div class it edits.
    pub class: String,
    /// The attribute that carries its structured source, if it has one.
    pub source_key: Option<String>,
    pub source_versions: Vec<u32>,
    /// The pane's pattern field for this class, sent as [`BlockInput::pattern`].
    pub pattern: Option<Field>,
    pub commands: Vec<Command>,
    pub views: Vec<ViewMode>,
    pub example: Option<Example>,
}

/// One selectable row of a block: a bar of a progression, a line of a verse. `index` is what
/// [`BlockInput::selected`] and [`Change::index`] refer to; it need not be dense.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub index: u32,
    pub label: String,
}

/// A copied snapshot of one block; never a handle into the document.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockInput {
    pub class: String,
    pub body: String,
    pub attrs: BTreeMap<String, String>,
    /// Content hash of the whole document source.
    pub revision: String,
    /// Selected rows or bars.
    pub selected: Vec<u32>,
    /// The pane's pattern field for this adapter.
    pub pattern: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunResult {
    /// The complete attribute map after an explicit write.
    Attrs(BTreeMap<String, String>),
    /// Text for an export.
    Text(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ViewAsk {
    pub mode: String,
    pub width: f32,
    pub selected: Option<(u32, u32)>,
}

/// One focusable event region read from a view's scene.
#[derive(Clone, Debug, PartialEq)]
pub struct EventRegion {
    pub bar: u32,
    pub event: u32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub label: String,
}

/// A view: a scene that names the regions, and the sheet the host paints.
#[derive(Clone, Debug, PartialEq)]
pub struct GuestView {
    pub scene: Scene,
    pub sheet: VectorSheet,
    pub width: f32,
    pub height: f32,
    pub description: String,
}

/// The two payload version bytes a view carries (a version byte, then CBOR).
pub const SCENE_PAYLOAD_VERSION: u8 = 1;

impl GuestView {
    /// Decode the two payloads a guest's view returns, refusing any that is over the 1 MiB cap
    /// or has an unknown version byte. The sheet is validated.
    pub fn decode(
        scene: &[u8],
        sheet: &[u8],
        width: f32,
        height: f32,
        description: String,
    ) -> Result<Self, String> {
        if scene.len() > vector_sheet::MAX_PAYLOAD_BYTES {
            return Err("The view's scene is over the 1 MiB payload cap".into());
        }
        let (version, body) = scene.split_first().ok_or("The view's scene is empty")?;
        if *version != SCENE_PAYLOAD_VERSION {
            return Err(format!("Unsupported scene version {version}"));
        }
        let scene: Scene = ciborium::from_reader(body)
            .map_err(|e| format!("The view's scene did not decode: {e}"))?;
        let sheet = VectorSheet::decode(sheet).map_err(|e| e.to_string())?;
        Ok(Self {
            scene,
            sheet,
            width,
            height,
            description,
        })
    }

    /// The event regions the scene names: one per region that has a label and a member item
    /// whose source id is `bar:<n>/event:<m>`. Items without that id are not events.
    pub fn regions(&self) -> Vec<EventRegion> {
        let mut out = Vec::new();
        for region in &self.scene.regions {
            let (Some(label), Some(&member)) = (region.label.as_ref(), region.members.first())
            else {
                continue;
            };
            let Some(item) = self.scene.items.get(member.0 as usize) else {
                continue;
            };
            let Some(source) = self.scene.sources.get(item.source.0 as usize) else {
                continue;
            };
            let Some((bar, event)) = parse_event_id(&source.id) else {
                continue;
            };
            let Footprint::Rect { size } = item.hit.as_ref().unwrap_or(&item.footprint) else {
                continue;
            };
            let c = item.transform.translate;
            out.push(EventRegion {
                bar,
                event,
                x: c.x - size.w / 2.0,
                y: c.y - size.h / 2.0,
                w: size.w,
                h: size.h,
                label: label.clone(),
            });
        }
        out
    }
}

fn parse_event_id(id: &str) -> Option<(u32, u32)> {
    let rest = id.strip_prefix("bar:")?;
    let (bar, event) = rest.split_once("/event:")?;
    Some((bar.parse().ok()?, event.parse().ok()?))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdapterError {
    /// The adapter cannot answer in this frame; ask again after the host wakes.
    Pending,
    /// The adapter refused, with the text to show.
    Refused(String),
}

impl From<&str> for AdapterError {
    fn from(text: &str) -> Self {
        Self::Refused(text.to_owned())
    }
}

impl From<String> for AdapterError {
    fn from(text: String) -> Self {
        Self::Refused(text)
    }
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => f.write_str("Waiting for the block adapter"),
            Self::Refused(text) => f.write_str(text),
        }
    }
}

pub type Answer<T> = Result<T, AdapterError>;

/// An adapter for one block class.
pub trait BlockAdapter {
    fn claim(&self) -> &Claim;
    /// The block's selectable rows, in order.
    fn rows(&self, block: &BlockInput) -> Answer<Vec<Row>>;
    /// Proposed edits for the selection; `command` is a [`CommandKind::Choice`] command.
    fn choices(&self, block: &BlockInput, command: &str) -> Answer<Vec<Change>>;
    /// The complete desired attribute map after the user's decisions.
    fn apply(
        &self,
        block: &BlockInput,
        command: &str,
        changes: &[Change],
    ) -> Answer<BTreeMap<String, String>>;
    /// An explicit write or text export.
    fn run(&self, block: &BlockInput, command: &str, arg: &str) -> Answer<RunResult>;
    /// A view, or `Ok(None)` when the adapter has none.
    fn view(&self, block: &BlockInput, ask: &ViewAsk) -> Answer<Option<GuestView>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event_scene(source_id: &str, labelled: bool) -> GuestView {
        use sceno::{
            ProjectedItem, Region, Representation, Size2, SourceIx, SourceRef, Transform2, Vec2,
        };
        let mut scene = Scene::new();
        scene.sources.push(SourceRef::new("woodshed.event", source_id));
        scene.items.push(ProjectedItem {
            source: SourceIx(0),
            space: Scene::WORLD,
            transform: Transform2 { translate: Vec2::new(60.0, 40.0), ..Transform2::IDENTITY },
            footprint: Footprint::Rect { size: Size2::new(20.0, 30.0) },
            representation: Representation::Open { kind: "event".into() },
            layer: 1,
            visible: true,
            hit: None,
            channels: vec![],
        });
        scene.regions.push(Region {
            members: vec![sceno::InstanceId(0)],
            space: Scene::WORLD,
            contour: None,
            label: labelled.then(|| "Bar 2, event 3".to_string()),
            confidence: 1.0,
        });
        GuestView {
            scene,
            sheet: VectorSheet { width: 100.0, height: 80.0, ops: vec![] },
            width: 100.0,
            height: 80.0,
            description: String::new(),
        }
    }

    #[test]
    fn a_view_decodes_from_its_two_payloads_and_refuses_bad_ones() {
        let view = event_scene("bar:0/event:0", true);
        let mut scene = vec![SCENE_PAYLOAD_VERSION];
        ciborium::into_writer(&view.scene, &mut scene).unwrap();
        let sheet = VectorSheet {
            width: 10.0,
            height: 10.0,
            ops: vec![],
        };
        let sheet_bytes = sheet.encode().unwrap();
        let decoded = GuestView::decode(&scene, &sheet_bytes, 10.0, 10.0, "d".into()).unwrap();
        assert_eq!(decoded.scene, view.scene);
        assert_eq!(decoded.regions().len(), 1);
        assert!(GuestView::decode(&[], &sheet_bytes, 1.0, 1.0, String::new()).is_err());
        assert!(GuestView::decode(&[9, 0], &sheet_bytes, 1.0, 1.0, String::new()).is_err());
        assert!(GuestView::decode(&scene, &[1, 0xff], 1.0, 1.0, String::new()).is_err());
        let huge = vec![1u8; vector_sheet::MAX_PAYLOAD_BYTES + 1];
        assert!(GuestView::decode(&huge, &sheet_bytes, 1.0, 1.0, String::new()).is_err());
    }

    #[test]
    fn regions_come_from_the_scene() {
        let view = event_scene("bar:1/event:2", true);
        assert_eq!(
            view.regions(),
            vec![EventRegion { bar: 1, event: 2, x: 50.0, y: 25.0, w: 20.0, h: 30.0, label: "Bar 2, event 3".into() }]
        );
        assert!(event_scene("bar:1/event:2", false).regions().is_empty());
        assert!(event_scene("something-else", true).regions().is_empty());
    }
}
