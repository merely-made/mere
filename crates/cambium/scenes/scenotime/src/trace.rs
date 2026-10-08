// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Scene-edit history as a record: a base snapshot and the labelled steps
//! taken from it.
//!
//! A [`SceneTrace`] is a value. It validates its chain when it is built or
//! read, and replays to any step on request; it keeps no cursor, no redo
//! stack and no step bound. Editing behaviour (where a viewer stands, undo,
//! redo, truncate-on-commit) belongs to the host, which can keep traces in
//! Cambium's `edit-history`. A position to share travels in the host's own
//! link or citation, not here.
//!
//! This is scene-edit history only. Authority-revision history, the sequence
//! of source checkpoints a projection was drawn from, is a different history
//! and is not modelled here.

use serde::{Deserialize, Serialize, Serializer};

use crate::{DiffError, Revision, SceneDiff, SceneSnapshot, SnapshotError};

/// The scene-trace wire version this reader writes and reads.
pub const SCENE_TRACE_VERSION: u16 = 1;

/// One labelled step in a scene trace.
///
/// A step without a diff leaves the scene as it was; a host uses one for an
/// act that is not a scene change, such as a selection. `annotation` is the
/// host's own record of the step (a selection, a tool, a reason). Scenotime
/// stores it verbatim and never reads it. A JSON `null` annotation reads back
/// as absent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceStep {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff: Option<SceneDiff>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotation: Option<serde_json::Value>,
}

impl TraceStep {
    /// A step that changes the scene.
    pub fn diff(label: impl Into<String>, diff: SceneDiff) -> Self {
        Self {
            label: label.into(),
            diff: Some(diff),
            annotation: None,
        }
    }

    /// A step that leaves the scene as it was.
    pub fn mark(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            diff: None,
            annotation: None,
        }
    }

    /// The same step carrying the host's annotation.
    pub fn with_annotation(mut self, annotation: serde_json::Value) -> Self {
        self.annotation = Some(annotation);
        self
    }
}

/// A rejected scene trace, or a position it does not have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceError {
    UnsupportedVersion {
        found: u16,
    },
    InvalidBase(SnapshotError),
    /// Step `index` does not chain from the scene before it, or does not
    /// replay onto it.
    Step {
        index: usize,
        error: DiffError,
    },
    /// A position past the last step. Positions run from `0`, the base, to
    /// the number of steps.
    OutOfRange {
        position: usize,
        steps: usize,
    },
}

/// A base snapshot and the chained steps taken from it.
///
/// Position `n` means "after the first `n` steps": position `0` is the base
/// and position [`len`](Self::len) is the end. Every step counts, with or
/// without a diff, so a host's step numbers survive steps that change nothing.
///
/// The chain is checked whenever a trace is built or deserialized: the base is
/// valid, every diff is in the base's epoch, each diff's base is the revision
/// the steps before it reached (a step without a diff does not advance it),
/// each diff advances the revision, and each replays.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneTrace {
    base: SceneSnapshot,
    steps: Vec<TraceStep>,
}

impl SceneTrace {
    /// A trace with no steps.
    pub fn new(base: SceneSnapshot) -> Result<Self, TraceError> {
        Self::from_steps(base, Vec::new())
    }

    /// A trace from its parts, refused unless the chain holds.
    pub fn from_steps(base: SceneSnapshot, steps: Vec<TraceStep>) -> Result<Self, TraceError> {
        base.validate().map_err(TraceError::InvalidBase)?;
        let mut head = base.clone();
        for (index, step) in steps.iter().enumerate() {
            advance(&mut head, index, step)?;
        }
        Ok(Self { base, steps })
    }

    pub fn base(&self) -> &SceneSnapshot {
        &self.base
    }

    pub fn steps(&self) -> &[TraceStep] {
        &self.steps
    }

    /// The number of steps, which is also the last position.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn into_parts(self) -> (SceneSnapshot, Vec<TraceStep>) {
        (self.base, self.steps)
    }

    /// The scene at `position`: the base with the first `position` steps
    /// replayed onto it. Pure; the trace is not changed.
    pub fn snapshot_at(&self, position: usize) -> Result<SceneSnapshot, TraceError> {
        let steps = self.steps.get(..position).ok_or(TraceError::OutOfRange {
            position,
            steps: self.steps.len(),
        })?;
        let mut head = self.base.clone();
        for (index, step) in steps.iter().enumerate() {
            advance(&mut head, index, step)?;
        }
        Ok(head)
    }

    /// The revision the trace reaches at `position`, without building the
    /// scene.
    pub fn revision_at(&self, position: usize) -> Result<Revision, TraceError> {
        let steps = self.steps.get(..position).ok_or(TraceError::OutOfRange {
            position,
            steps: self.steps.len(),
        })?;
        Ok(steps
            .iter()
            .rev()
            .find_map(|step| step.diff.as_ref().map(|diff| diff.revision))
            .unwrap_or(self.base.revision))
    }

    /// The scene after every step.
    pub fn head(&self) -> SceneSnapshot {
        self.snapshot_at(self.steps.len())
            .expect("a validated trace replays to its end")
    }

    /// This trace with `step` added at the end, refused unless it chains.
    pub fn appended(&self, step: TraceStep) -> Result<Self, TraceError> {
        let mut head = self.head();
        advance(&mut head, self.steps.len(), &step)?;
        let mut steps = self.steps.clone();
        steps.push(step);
        Ok(Self {
            base: self.base.clone(),
            steps,
        })
    }

    /// This trace's first `position` steps. A prefix of a valid chain is
    /// valid, so only the position is checked.
    pub fn truncated(&self, position: usize) -> Result<Self, TraceError> {
        let steps = self.steps.get(..position).ok_or(TraceError::OutOfRange {
            position,
            steps: self.steps.len(),
        })?;
        Ok(Self {
            base: self.base.clone(),
            steps: steps.to_vec(),
        })
    }
}

/// Apply one step to `head`, requiring it to chain rather than relying on
/// `apply_diff`'s tolerance: a repeated or older diff is a successful no-op on
/// a live wire, but in a trace it is a broken chain.
fn advance(head: &mut SceneSnapshot, index: usize, step: &TraceStep) -> Result<(), TraceError> {
    let Some(diff) = &step.diff else {
        return Ok(());
    };
    let refuse = |error| TraceError::Step { index, error };
    if diff.epoch != head.epoch {
        return Err(refuse(DiffError::WrongEpoch {
            current: head.epoch,
            received: diff.epoch,
        }));
    }
    if diff.base != head.revision {
        return Err(refuse(DiffError::MissingBase {
            current: head.revision,
            required: diff.base,
        }));
    }
    if diff.revision <= diff.base {
        return Err(refuse(DiffError::InvalidRevision {
            base: diff.base,
            revision: diff.revision,
        }));
    }
    head.apply_diff(diff).map(|_| ()).map_err(refuse)
}

/// The trace as written: a version, the base and the steps.
#[derive(Serialize)]
struct WireRef<'a> {
    version: u16,
    base: &'a SceneSnapshot,
    steps: &'a [TraceStep],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    version: u16,
    base: SceneSnapshot,
    #[serde(default)]
    steps: Vec<TraceStep>,
}

impl Serialize for SceneTrace {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        WireRef {
            version: SCENE_TRACE_VERSION,
            base: &self.base,
            steps: &self.steps,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SceneTrace {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = Wire::deserialize(deserializer)?;
        if wire.version != SCENE_TRACE_VERSION {
            return Err(serde::de::Error::custom(format!(
                "unsupported scene-trace version {}",
                wire.version
            )));
        }
        Self::from_steps(wire.base, wire.steps)
            .map_err(|error| serde::de::Error::custom(format!("invalid scene trace: {error:?}")))
    }
}

#[cfg(test)]
mod tests {
    use sceno::{Footprint, ProjectedItem, Representation, Scene, SourceRef, Transform2};
    use serde_json::json;

    use super::*;
    use crate::{RelationId, SceneEpoch, SceneOp};

    fn base() -> SceneSnapshot {
        let mut scene = Scene::new();
        let source = scene.intern_source(SourceRef::new("fixture", "only"));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(1.0, 2.0),
            footprint: Footprint::Point,
            representation: Representation::Glyph,
            layer: 0,
            visible: true,
            hit: None,
            channels: Vec::new(),
        });
        SceneSnapshot::from_dense(SceneEpoch(3), Revision(10), scene).expect("snapshot")
    }

    fn layer(base: u64, revision: u64, layer: i16) -> SceneDiff {
        SceneDiff {
            epoch: SceneEpoch(3),
            base: Revision(base),
            revision: Revision(revision),
            operations: vec![SceneOp::SetItemLayer {
                index: sceno::InstanceId(0),
                layer,
            }],
        }
    }

    /// Select, raise, select again, raise again: four steps, two diffs.
    fn trace() -> SceneTrace {
        SceneTrace::from_steps(
            base(),
            vec![
                TraceStep::mark("Select").with_annotation(json!({"kind": "node", "id": "only"})),
                TraceStep::diff("Raise", layer(10, 11, 1)),
                TraceStep::mark("Select again"),
                TraceStep::diff("Raise again", layer(11, 12, 2)),
            ],
        )
        .expect("the chain holds")
    }

    fn item_layer(snapshot: &SceneSnapshot) -> i16 {
        snapshot
            .active_item(sceno::InstanceId(0))
            .expect("item")
            .layer
    }

    #[test]
    fn every_position_replays_from_the_base() {
        let trace = trace();
        assert_eq!(trace.len(), 4);
        let layers = (0..=trace.len())
            .map(|n| item_layer(&trace.snapshot_at(n).expect("in range")))
            .collect::<Vec<_>>();
        assert_eq!(layers, [0, 0, 1, 1, 2]);
        let revisions = (0..=trace.len())
            .map(|n| trace.snapshot_at(n).expect("in range").revision.0)
            .collect::<Vec<_>>();
        assert_eq!(revisions, [10, 10, 11, 11, 12]);
        for n in 0..=trace.len() {
            assert_eq!(
                trace.revision_at(n).expect("in range"),
                trace.snapshot_at(n).expect("in range").revision
            );
        }
        assert_eq!(trace.snapshot_at(0).expect("base"), base());
        assert_eq!(trace.head(), trace.snapshot_at(4).expect("end"));
        assert_eq!(
            trace.snapshot_at(5),
            Err(TraceError::OutOfRange {
                position: 5,
                steps: 4
            })
        );
    }

    #[test]
    fn appended_and_truncated_are_plain_values() {
        let full = trace();
        let prefix = full.truncated(2).expect("in range");
        assert_eq!(prefix.len(), 2);
        assert_eq!(full.len(), 4, "the original is untouched");
        let branched = prefix
            .appended(TraceStep::diff("Lower", layer(11, 12, -1)))
            .expect("chains from revision 11");
        assert_eq!(item_layer(&branched.head()), -1);
        assert_eq!(
            full.truncated(5).map(|_| ()),
            Err(TraceError::OutOfRange {
                position: 5,
                steps: 4
            })
        );
        assert_eq!(
            full.appended(TraceStep::diff("Stale", layer(11, 13, 3)))
                .map(|_| ()),
            Err(TraceError::Step {
                index: 4,
                error: DiffError::MissingBase {
                    current: Revision(12),
                    required: Revision(11)
                }
            })
        );
    }

    #[test]
    fn a_broken_chain_is_refused() {
        let refuse = |steps| SceneTrace::from_steps(base(), steps).map(|_| ());

        // Wrong base: the mark does not advance the revision.
        assert!(matches!(
            refuse(vec![
                TraceStep::diff("Raise", layer(10, 11, 1)),
                TraceStep::mark("Select"),
                TraceStep::diff("Skip", layer(12, 13, 2)),
            ]),
            Err(TraceError::Step {
                index: 2,
                error: DiffError::MissingBase { .. }
            })
        ));
        // A repeated diff is a no-op on a live wire, but not in a trace.
        assert!(matches!(
            refuse(vec![
                TraceStep::diff("Raise", layer(10, 11, 1)),
                TraceStep::diff("Raise", layer(10, 11, 1)),
            ]),
            Err(TraceError::Step {
                index: 1,
                error: DiffError::MissingBase { .. }
            })
        ));
        // Wrong epoch.
        let mut foreign = layer(10, 11, 1);
        foreign.epoch = SceneEpoch(4);
        assert!(matches!(
            refuse(vec![TraceStep::diff("Foreign", foreign)]),
            Err(TraceError::Step {
                index: 0,
                error: DiffError::WrongEpoch { .. }
            })
        ));
        // A revision that does not advance.
        assert!(matches!(
            refuse(vec![TraceStep::diff("Still", layer(10, 10, 1))]),
            Err(TraceError::Step {
                index: 0,
                error: DiffError::InvalidRevision { .. }
            })
        ));
        // A diff that chains but does not replay.
        let dangling = SceneDiff {
            operations: vec![SceneOp::TombstoneRelation {
                index: RelationId(7),
            }],
            ..layer(10, 11, 1)
        };
        assert!(matches!(
            refuse(vec![TraceStep::diff("Dangling", dangling)]),
            Err(TraceError::Step {
                index: 0,
                error: DiffError::InvalidOperation(_)
            })
        ));
        // An invalid base.
        let mut broken = base();
        broken.tables.item_order.clear();
        assert!(matches!(
            SceneTrace::new(broken),
            Err(TraceError::InvalidBase(_))
        ));
    }

    #[test]
    fn the_host_annotation_survives_the_wire_verbatim() {
        let annotation = json!({
            "selection": {"kind": "edge", "id": "a-b"},
            "nested": [1, 2.5, null, {"deep": true}],
            "text": "unicode \u{2192} kept"
        });
        let trace = trace()
            .appended(TraceStep::mark("Annotated").with_annotation(annotation.clone()))
            .expect("chains");
        let wire = serde_json::to_string(&trace).expect("serializes");
        let read: SceneTrace = serde_json::from_str(&wire).expect("deserializes");
        assert_eq!(read, trace);
        assert_eq!(read.steps()[4].annotation, Some(annotation));
        assert_eq!(read.steps()[2].annotation, None);

        let value: serde_json::Value = serde_json::from_str(&wire).expect("json");
        assert_eq!(value["version"], json!(SCENE_TRACE_VERSION));
        assert!(
            value["steps"][2].get("annotation").is_none()
                && value["steps"][2].get("diff").is_none(),
            "absent parts are not written"
        );
    }

    #[test]
    fn deserialization_refuses_unknown_keys_versions_and_broken_chains() {
        let good = serde_json::to_value(trace()).expect("json");
        let refused = |value: &serde_json::Value| {
            serde_json::from_value::<SceneTrace>(value.clone())
                .expect_err("refused")
                .to_string()
        };

        let mut extra = good.clone();
        extra["cursor"] = json!(2);
        assert!(refused(&extra).contains("cursor"));

        let mut step_extra = good.clone();
        step_extra["steps"][0]["selection"] = json!({"kind": "node"});
        assert!(refused(&step_extra).contains("selection"));

        let mut newer = good.clone();
        newer["version"] = json!(2);
        assert!(refused(&newer).contains("version 2"));

        let mut broken = good.clone();
        broken["steps"][3]["diff"]["base"] = json!(10);
        assert!(refused(&broken).contains("MissingBase"));

        assert_eq!(
            serde_json::from_value::<SceneTrace>(good).expect("reads"),
            trace()
        );
    }
}
