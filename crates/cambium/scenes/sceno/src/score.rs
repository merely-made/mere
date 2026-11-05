// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Scores — persisted, product-free instructions for making a scene.
//!
//! A score names an arrangement and supplies the measured instances that the
//! arrangement places. It deliberately carries opaque [`SourceRef`]s rather
//! than source truth: a Mere graph adapter, an Isometry campaign adapter, and
//! a geographic fixture can serialize the same vocabulary without sharing a
//! model.

use serde::{Deserialize, Serialize};

use crate::{Footprint, Rect, Representation, Size2, SourceRef, Vec2};

/// The persisted-score wire version.
///
/// Version 5 gives the holds their three arrangement roles (dynamics grammar
/// plan, F25, F44): "Seeded", "Anchored" (returns) and "Pinned". Up to version
/// 4, "Anchored" meant best effort, which is now `Seeded`, so an older score's
/// "Anchored" reads as [`Hold::Seeded`] and keeps its meaning. A reader
/// refuses a score newer than itself ([`ScoreVersionError`]), and a score it
/// reads is held in its own version's meaning from then on.
///
/// Version 4 absorbs the `arrangements` catalog: seven new [`Arrangement`]
/// families, the [`Arrangement::Custom`] escape, and the three per-item
/// disclosure fields they read ([`ScoreItem::axis`], [`ScoreItem::embedding`],
/// [`ScoreItem::weight`]). A version 3 reader must reject a version 4 score:
/// the families it does not know are not decorative, and an item whose
/// position depends on a disclosed axis value would be placed by ordinal
/// instead — silently, and somewhere else entirely.
///
/// Version 3 renames the regular-cell arrangement from `Board` to [`Grid`].
/// Version 2 added [`Score::holds`]. It differed from version 1 in nothing else,
/// but an adapter that only understands version 1 must reject a version 2
/// score rather than accept it, because the one thing it would drop is an
/// authored placement someone asked to be honored. A silently dropped pin is
/// the failure this field exists to prevent.
pub const SCORE_VERSION: u16 = 5;

/// The first version whose "Anchored" hold returns (F44).
const RETURNING_ANCHOR_SINCE: u16 = 5;

/// A score newer than its reader, refused rather than half-read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreVersionError {
    /// The score's version.
    pub found: u16,
    /// The newest version the reader knows.
    pub reader: u16,
}

impl std::fmt::Display for ScoreVersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "score version {} is newer than this reader's {}",
            self.found, self.reader
        )
    }
}

impl std::error::Error for ScoreVersionError {}

/// A complete, serializable projection request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ScoreWire")]
pub struct Score {
    /// Allows an adapter to reject a score it cannot interpret.
    pub version: u16,
    /// The analytic placement family that will realize [`Self::items`].
    pub arrangement: Arrangement,
    /// Ordered, measured source instances. A source may appear more than
    /// once, which remains distinct from source truth by design.
    pub items: Vec<ScoreItem>,
    /// Authored placements that outrank the arrangement, keyed by source.
    ///
    /// Sparse by construction: an unheld source has no entry, and the common
    /// case costs one empty vec. Keyed by [`SourceRef`] rather than by item
    /// index so a hold survives re-ordering, re-solving, and a changed
    /// authority, which is what lets a citation name a hold without shipping
    /// the score that contains it.
    #[serde(default)]
    pub holds: Vec<HeldPlacement>,
    /// Adapter-stamped input generation, copied into the realized scene.
    pub generation: u64,
}

/// A score as written, before it is read in the reader's meaning.
#[derive(Deserialize)]
struct ScoreWire {
    version: u16,
    arrangement: Arrangement,
    items: Vec<ScoreItem>,
    #[serde(default)]
    holds: Vec<HeldPlacement>,
    generation: u64,
}

impl TryFrom<ScoreWire> for Score {
    type Error = ScoreVersionError;

    fn try_from(wire: ScoreWire) -> Result<Self, Self::Error> {
        Score::read(wire, SCORE_VERSION)
    }
}

impl Score {
    /// How a reader at version `reader` reads `wire`: a newer score is
    /// refused; an older one's "Anchored", best effort then, becomes
    /// [`Hold::Seeded`]; and the score is stamped with the reader's version,
    /// so it is never written back under an older meaning.
    fn read(wire: ScoreWire, reader: u16) -> Result<Self, ScoreVersionError> {
        if wire.version > reader {
            return Err(ScoreVersionError {
                found: wire.version,
                reader,
            });
        }
        let mut holds = wire.holds;
        if wire.version < RETURNING_ANCHOR_SINCE && reader >= RETURNING_ANCHOR_SINCE {
            for held in &mut holds {
                if held.hold == Hold::Anchored {
                    held.hold = Hold::Seeded;
                }
            }
        }
        Ok(Self {
            version: reader,
            arrangement: wire.arrangement,
            items: wire.items,
            holds,
            generation: wire.generation,
        })
    }

    pub fn new(arrangement: Arrangement) -> Self {
        Self {
            version: SCORE_VERSION,
            arrangement,
            items: Vec::new(),
            holds: Vec::new(),
            generation: 0,
        }
    }

    /// The hold authored for `source`, if any. First entry wins; a score with
    /// two holds on one source is malformed and the later one is ignored.
    pub fn hold_for(&self, source: &SourceRef) -> Option<&HeldPlacement> {
        self.holds.iter().find(|held| &held.source == source)
    }
}

/// The role an authored placement plays once anything moves the scene: the
/// three arrangement roles (dynamics grammar plan, F18, F19, F25).
///
/// The classes are deliberately unequal, and naming them is the point: a
/// solver that treats a pin as a suggestion produces the silent-soft failure
/// where a person pins something, the layout quietly moves it, and nothing
/// anywhere says so. Only [`Hold::Pinned`] is reported honored or unmet.
/// A score before version 5 wrote "Anchored" for what is now `Seeded`; its
/// [`Score`] reads it so (F44).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hold {
    /// Best effort. The arrangement seeds from here and relaxation may carry
    /// it away; moving a seeded item is correct behaviour, not a failure.
    Seeded,
    /// Returns. Displaced by relaxation or a drag, the item goes back here.
    Anchored,
    /// Must be honored. The arrangement does not get a vote, and a solver that
    /// cannot honor it reports rather than repositions.
    Pinned,
}

/// One authored placement, outranking whatever the arrangement would choose.
///
/// This record is also the unit a scene citation carries as its placement
/// delta: one serialization of a pin, not two.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldPlacement {
    /// Which source this holds. Not an item index: indices move.
    pub source: SourceRef,
    /// Where, in the same coordinate space the arrangement places into.
    pub at: Vec2,
    pub hold: Hold,
}

/// An authored placement the solver *did* honor, bound to the instance that
/// received it.
///
/// The negative half of this record lives on [`crate::Scene::unmet_holds`]. The
/// positive half is here for two reasons that arrived together: a consumer that
/// wants to state "3 pins honored" should read it rather than recompute it, and
/// a scene that knows which of its instances are pinned can stop a
/// relaxation pass from quietly dragging one away.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HonoredHold {
    /// Which placed instance received the hold.
    pub instance: crate::InstanceId,
    pub placement: HeldPlacement,
}

impl HeldPlacement {
    pub fn pinned(source: SourceRef, at: Vec2) -> Self {
        Self {
            source,
            at,
            hold: Hold::Pinned,
        }
    }

    /// A placement that seeds the arrangement and may be carried away.
    pub fn seeded(source: SourceRef, at: Vec2) -> Self {
        Self {
            source,
            at,
            hold: Hold::Seeded,
        }
    }

    /// A placement the item returns to.
    pub fn anchored(source: SourceRef, at: Vec2) -> Self {
        Self {
            source,
            at,
            hold: Hold::Anchored,
        }
    }
}

/// A per-item coordinate on an arrangement's axis.
///
/// Numeric for a continuous axis ([`Timeline`], [`Stack`]'s layer, [`Radial`]'s
/// ring); categorical for a bucketed one ([`Kanban`]'s column).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AxisValue {
    /// A position on a continuous axis, in the producer's own units.
    Numeric(f64),
    /// A bucket tag. Ordering across buckets comes from the arrangement's
    /// config, not from this value.
    Categorical(String),
}

/// Optional placement data an arrangement understands. The source adapter
/// decides whether an item has an authored cell/location; the solver never
/// looks behind the opaque source reference.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum Placement {
    /// Use the item's ordinal in the arrangement's deterministic order.
    #[default]
    Ordinal,
    /// An authored cell for [`Grid`].
    Cell { column: i32, row: i32 },
    /// A disclosed geographic or local coordinate for [`Geographic`].
    Coordinate(Vec2),
}

/// One measured source instance in a score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreItem {
    pub source: SourceRef,
    /// Lower ordinal is earlier in an arrangement. An adapter maps native
    /// recency or priority to this scalar before persistence.
    pub ordinal: u32,
    /// What the host measured at the selected LOD.
    pub footprint: Footprint,
    /// The selected realization rung. A new score may select another rung for
    /// the same source when zoom, focus, or host capability changes.
    pub representation: Representation,
    pub placement: Placement,
    pub layer: i16,
    pub visible: bool,
    /// A coordinate the producer computed and disclosed, for arrangements that
    /// place along an axis ([`Timeline`], [`Kanban`], [`Stack`], [`Radial`]).
    ///
    /// This is the seam that keeps topology-derived layouts portable. A ring
    /// index, a topological rank and a timestamp are all one number per item by
    /// the time they matter to a placement; deriving them needs the source's
    /// native truth, and using them does not. The producer walks its own graph
    /// once and discloses the result, rather than serializing an adjacency list
    /// for a solver to walk again.
    #[serde(default)]
    pub axis: Option<AxisValue>,
    /// 2-D coordinates the producer computed, for [`Embedded`]. Any producer
    /// will do — a dimensionality reduction, a spectral decomposition, a hand
    /// placement exported from somewhere else.
    #[serde(default)]
    pub embedding: Option<Vec2>,
    /// A per-item scalar for arrangements that place proportionally, read today
    /// by [`RadialAngularPolicy::Weighted`]. Named to match
    /// [`RoutedRelation::weight`](crate::RoutedRelation::weight), which is the
    /// same idea on the scene side.
    #[serde(default)]
    pub weight: Option<f32>,
}

#[cfg(test)]
#[path = "score/tests.rs"]
mod tests;

#[path = "score/arrangements.rs"]
mod arrangements;

pub use arrangements::{
    Arrangement, Embedded, EmbeddingFallback, Geographic, Grid, Hulls, IterationDepth, Kanban,
    LSystem, LSystemGrammar, Penrose, PenroseVariant, Radial, RadialAngularPolicy,
    RadialUnreachablePolicy, Spiral, SpiralCurve, Stack, SubdivisionCount, Timeline,
    TimelineFallback, UnusedVertexPolicy,
};
