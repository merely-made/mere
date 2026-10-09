// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Composing a facet's cells into one scene (Scenograph editor plan, SE64).
//!
//! The host compiles each cell's swatch to a scene with the ordinary
//! compiler, resolving a scope to a dataset and a reflection to the recipe it
//! follows. [`compose_facet`] then places those scenes in one scene: a space
//! per cell, an item per axis heading and per cell frame, each cell scaled by
//! its axes' declared rule (SE56). Every index a cell scene holds (sources,
//! spaces, instances) is renumbered, so relations, regions, folds and holds
//! keep pointing at the same things.

use sceno::{
    Footprint, InstanceId, ProjectedItem, Rect, Representation, Scene, Size2, SourceRef, Space,
    SpaceId, StandIn, Transform2, Vec2, fold::FoldRule,
};
use scenograph::ValidationIssue;
use scenograph::swatch::{AxisScale, Facet};

/// The source adapter of an axis heading: `columns:<i>` or `rows:<i>`, whose
/// text is the facet's label at that position.
pub const FACET_AXIS_ADAPTER: &str = "scenomise.facet-axis/v1";
/// The source adapter of a cell's frame, keyed by its swatch id.
pub const FACET_CELL_ADAPTER: &str = "scenomise.facet-cell/v1";

/// Where cells sit: every cell gets the same box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FacetLayout {
    pub cell: Size2,
    pub gap: f32,
    /// The band the headings take, above the columns and left of the rows.
    pub heading: f32,
}

impl Default for FacetLayout {
    fn default() -> Self {
        Self {
            cell: Size2::new(320.0, 240.0),
            gap: 24.0,
            heading: 32.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FacetError {
    Invalid(Vec<ValidationIssue>),
    /// A cell's swatch has no compiled scene.
    MissingScene(String),
    /// A scene names no cell of the facet.
    UnknownCell(String),
}

/// Place each cell's compiled scene, keyed by swatch id, in one scene.
pub fn compose_facet(
    facet: &Facet,
    scenes: &[(String, Scene)],
    layout: &FacetLayout,
) -> Result<Scene, FacetError> {
    facet.validate().map_err(FacetError::Invalid)?;
    if let Some((id, _)) = scenes
        .iter()
        .find(|(id, _)| !facet.cells.iter().any(|cell| &cell.swatch.id == id))
    {
        return Err(FacetError::UnknownCell(id.clone()));
    }
    let scene_of = |id: &str| scenes.iter().find(|(known, _)| known == id).map(|(_, s)| s);
    let mut placed = Vec::with_capacity(facet.cells.len());
    for cell in &facet.cells {
        let scene = scene_of(&cell.swatch.id)
            .ok_or_else(|| FacetError::MissingScene(cell.swatch.id.clone()))?;
        placed.push((cell, scene));
    }

    let rows = facet.rows.as_ref().map_or(1, |axis| axis.labels.len());
    let row_band = if facet.rows.is_some() {
        layout.heading
    } else {
        0.0
    };
    let origin = |row: usize, column: usize| {
        Vec2::new(
            row_band + column as f32 * (layout.cell.w + layout.gap),
            layout.heading + row as f32 * (layout.cell.h + layout.gap),
        )
    };
    // Cells share a factor across every axis declared shared (SE56).
    let columns_shared = facet.columns.scale == AxisScale::Shared;
    let rows_shared = facet
        .rows
        .as_ref()
        .is_none_or(|axis| axis.scale == AxisScale::Shared);
    let group = |row: usize, column: usize| {
        (
            (!columns_shared).then_some(column),
            (!rows_shared).then_some(row),
        )
    };
    let factor = |row: usize, column: usize| {
        let key = group(row, column);
        let (w, h) = placed
            .iter()
            .filter(|(cell, _)| group(cell.row, cell.column) == key)
            .fold((0.0f32, 0.0f32), |(w, h), (_, scene)| {
                (w.max(scene.bounds.size.w), h.max(scene.bounds.size.h))
            });
        // A swatch shrinks to fit its box; it is never enlarged.
        let fit = |box_side: f32, content: f32| {
            if content > 0.0 {
                box_side / content
            } else {
                1.0
            }
        };
        fit(layout.cell.w, w).min(fit(layout.cell.h, h)).min(1.0)
    };

    let mut out = Scene::new();
    let heading = |out: &mut Scene, id: String, at: Vec2, size: Size2, kind: &str| {
        let source = out.intern_source(SourceRef::new(FACET_AXIS_ADAPTER, id));
        out.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(at.x, at.y),
            footprint: Footprint::Rect { size },
            representation: Representation::Open { kind: kind.into() },
            layer: 1,
            visible: true,
            hit: None,
            channels: Vec::new(),
        });
    };
    for column in 0..facet.columns.labels.len() {
        let at = origin(0, column);
        heading(
            &mut out,
            format!("columns:{column}"),
            Vec2::new(at.x, 0.0),
            Size2::new(layout.cell.w, layout.heading),
            "facet.column-heading",
        );
    }
    if let Some(axis) = &facet.rows {
        for row in 0..axis.labels.len() {
            let at = origin(row, 0);
            heading(
                &mut out,
                format!("rows:{row}"),
                Vec2::new(0.0, at.y),
                Size2::new(layout.heading, layout.cell.h),
                "facet.row-heading",
            );
        }
    }

    let mut generation: u64 = 0xcbf2_9ce4_8422_2325;
    for (cell, scene) in &placed {
        generation = (generation ^ scene.generation).wrapping_mul(0x0000_0100_0000_01b3);
        let at = origin(cell.row, cell.column);
        let f = factor(cell.row, cell.column);
        let b = scene.bounds;
        let offset = Vec2::new(
            at.x + (layout.cell.w - b.size.w * f) / 2.0 - b.origin.x * f,
            at.y + (layout.cell.h - b.size.h * f) / 2.0 - b.origin.y * f,
        );
        let frame_source = out.intern_source(SourceRef::new(FACET_CELL_ADAPTER, &cell.swatch.id));
        out.items.push(ProjectedItem {
            source: frame_source,
            space: Scene::WORLD,
            transform: Transform2::translation(at.x, at.y),
            footprint: Footprint::Rect { size: layout.cell },
            representation: Representation::Open {
                kind: "facet.cell".into(),
            },
            layer: 0,
            visible: true,
            hit: None,
            channels: vec![("facet.scale".into(), f)],
        });
        let cell_space = out.push_space(
            Scene::WORLD,
            Transform2 {
                translate: offset,
                scale: f,
                rotate: 0.0,
            },
            Some(format!("cell: {}", cell.swatch.id)),
        );
        merge(&mut out, scene, cell_space);
    }
    out.generation = generation;
    let rows_extent = layout.heading + rows as f32 * (layout.cell.h + layout.gap) - layout.gap;
    let columns_extent =
        row_band + facet.columns.labels.len() as f32 * (layout.cell.w + layout.gap) - layout.gap;
    out.bounds = Rect::new(Vec2::ZERO, Size2::new(columns_extent, rows_extent));
    Ok(out)
}

/// Copy `cell` into `out` under `parent`, renumbering its sources, spaces and
/// instances.
fn merge(out: &mut Scene, cell: &Scene, parent: SpaceId) {
    let sources: Vec<_> = cell
        .sources
        .iter()
        .map(|source| out.intern_source(source.clone()))
        .collect();
    let space_base = out.spaces.len() as u32;
    let space = |id: SpaceId| SpaceId(space_base + id.0);
    for (index, s) in cell.spaces.iter().enumerate() {
        out.spaces.push(Space {
            // The cell's world hangs from the cell's space.
            parent: Some(if index == 0 {
                parent
            } else {
                space(s.parent.unwrap_or(Scene::WORLD))
            }),
            transform: s.transform,
            name: s.name.clone(),
        });
    }
    let base = out.items.len() as u32;
    let instance = |id: InstanceId| InstanceId(base + id.0);
    for item in &cell.items {
        let mut item = item.clone();
        item.source = sources[item.source.0 as usize];
        item.space = space(item.space);
        out.items.push(item);
    }
    for backdrop in &cell.backdrops {
        let mut backdrop = backdrop.clone();
        backdrop.source = sources[backdrop.source.0 as usize];
        backdrop.space = space(backdrop.space);
        out.backdrops.push(backdrop);
    }
    for relation in &cell.relations {
        let mut relation = relation.clone();
        relation.from = instance(relation.from);
        relation.to = instance(relation.to);
        relation.space = space(relation.space);
        out.relations.push(relation);
    }
    for region in &cell.regions {
        let mut region = region.clone();
        region.members = region.members.iter().map(|m| instance(*m)).collect();
        region.space = space(region.space);
        out.regions.push(region);
    }
    for fold in &cell.folds {
        let mut fold = fold.clone();
        fold.members = fold.members.iter().map(|m| instance(*m)).collect();
        if let StandIn::Member(id) = &mut fold.stand_in {
            *id = instance(*id);
        }
        if let Some(FoldRule::Descendants { root, .. }) = &mut fold.rule {
            *root = instance(*root);
        }
        if let Some(boundary) = &mut fold.boundary {
            for bundle in &mut boundary.bundles {
                bundle.outside = instance(bundle.outside);
            }
        }
        out.folds.push(fold);
    }
    out.unmet_holds.extend(cell.unmet_holds.iter().cloned());
    for honored in &cell.honored_holds {
        let mut honored = honored.clone();
        honored.instance = instance(honored.instance);
        out.honored_holds.push(honored);
    }
}

#[cfg(test)]
#[path = "facet_tests.rs"]
mod tests;
