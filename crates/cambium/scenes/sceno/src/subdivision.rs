// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Arrangements that divide an area into shares (graph grammar scenes, Merely
//! Design System DECISIONS 2026-10-16): Partition, Sets and Mosaic.
//!
//! Partition subdivides the viewport (Mark, 2026-10-16: "Partition being
//! subdivisions of the viewport makes more sense"); Sets draws fixed regions
//! whose positions mean membership. Mosaic is not bounded: it lives on the
//! infinite canvas, its extent grown from the weight it holds ("Same behavior,
//! just pannable, organizable, resizable").

use serde::{Deserialize, Serialize};

use crate::{Rect, Size2, Vec2};

/// Shares of a measured whole, nested one level: an item's categorical axis
/// names its group, its weight is its measured size. Whatever the items do
/// not account for of `whole` is the remainder, and is drawn as a region of
/// its own so every share is honest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Partition {
    /// The area divided: the host's viewport, in scene units.
    pub bounds: Rect,
    /// The measured whole. `None` takes the items' total, so nothing remains.
    pub whole: Option<f32>,
    pub form: PartitionForm,
    /// Inset between a group's edge and its members, in scene units.
    pub gap: f32,
}

impl Default for Partition {
    fn default() -> Self {
        Self {
            bounds: Rect::new(Vec2::new(-400.0, -250.0), Size2::new(800.0, 500.0)),
            whole: None,
            form: PartitionForm::default(),
            gap: 8.0,
        }
    }
}

/// Two forms of the same partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartitionForm {
    /// Nested rectangular enclosures, area by share.
    #[default]
    Tiles,
    /// Concentric rings, angle by share: groups inside, members outside.
    Rings,
}

/// Fixed set regions as the backdrop. An item's categorical axis lists the
/// sets it belongs to, separated by `|`, which picks its region; a weight of
/// at least one half places it in the core of those sets (forward), anything
/// else in the rim (background).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Setscape {
    /// The area the sets fill, in scene units.
    pub bounds: Rect,
    /// The sets, in drawing order. Up to four overlap pairwise around the
    /// centre; more ring it.
    pub sets: Vec<String>,
}

impl Default for Setscape {
    fn default() -> Self {
        Self {
            bounds: Rect::new(Vec2::new(-400.0, -300.0), Size2::new(800.0, 600.0)),
            sets: Vec::new(),
        }
    }
}

/// The separator between set names on a [`Setscape`] item's axis.
pub const SET_SEPARATOR: char = '|';

/// The weight at or above which a [`Setscape`] item sits in a core.
pub const SET_CORE_WEIGHT: f32 = 0.5;

/// Tiles by weight on the infinite canvas, grouped by an item's categorical
/// axis. Weight alone sizes a tile, so selection never reflows the mosaic;
/// the order is explicit, so a person's arrangement holds. Groups and members
/// split by weighted balanced bisection, the workbench's recursive splits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mosaic {
    /// The mosaic's centre, in scene units.
    pub origin: Vec2,
    /// Scene area one unit of weight occupies.
    pub unit_area: f32,
    /// Width over height of the whole mosaic.
    pub aspect: f32,
    /// Gutter between tiles and around groups.
    pub gap: f32,
    /// The band along a group's top that carries its label.
    pub header: f32,
    /// Group order, left to right and top to bottom as the splits fall.
    /// Groups not named here follow in first-appearance order; members keep
    /// the score's order within their group.
    pub group_order: Vec<String>,
}

impl Default for Mosaic {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            unit_area: 9_000.0,
            aspect: 1.6,
            gap: 5.0,
            header: 22.0,
            group_order: Vec::new(),
        }
    }
}
