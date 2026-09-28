// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Drop offscreen canvas paint before lowering it into a GPU scene. Culling
//! individual paint bounds preserves captions that extend beyond node bodies
//! and edges crossing the viewport with both endpoints outside it. Layout,
//! simulation, hit testing, resource tables and paint-stack order are unchanged.

use std::borrow::Cow;

use paint_list_api::{LayoutRect, LayoutTransform, PaintCmd, StrokeJoin};

pub(super) fn visible_commands(commands: &[PaintCmd], viewport: LayoutRect) -> Cow<'_, [PaintCmd]> {
    let mut transforms = vec![LayoutTransform::identity()];
    let mut protected_layers = vec![false];
    let mut shadows = false;
    let mut kept: Option<Vec<PaintCmd>> = None;
    for (index, command) in commands.iter().enumerate() {
        match command {
            PaintCmd::PushTransform(spec) => {
                let mut local = spec.transform;
                local.m41 += spec.origin.x;
                local.m42 += spec.origin.y;
                transforms.push(local.then(transforms.last().unwrap()));
            },
            PaintCmd::PopTransform => {
                if transforms.len() > 1 {
                    transforms.pop();
                }
            },
            PaintCmd::PushLayer(spec) => protected_layers.push(
                *protected_layers.last().unwrap()
                    || !spec.filters.is_empty()
                    || spec.mask.is_some(),
            ),
            PaintCmd::PopLayer => {
                if protected_layers.len() > 1 {
                    protected_layers.pop();
                }
            },
            PaintCmd::PushShadow(_) => shadows = true,
            PaintCmd::PopAllShadows => shadows = false,
            _ => {},
        }
        let transform = transforms.last().unwrap();
        let outside = !shadows
            && !protected_layers.last().unwrap()
            && transform.is_2d()
            && paint_bounds(command)
                .and_then(|bounds| transform.outer_transformed_box2d(&bounds))
                .is_some_and(|bounds| {
                    // Retain uncertain geometry and a logical pixel for edge AA.
                    let coords = [bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y];
                    coords.into_iter().all(f32::is_finite)
                        && (bounds.max.x < viewport.min.x - 1.0
                            || bounds.min.x > viewport.max.x + 1.0
                            || bounds.max.y < viewport.min.y - 1.0
                            || bounds.min.y > viewport.max.y + 1.0)
                });
        if outside {
            kept.get_or_insert_with(|| commands[..index].to_vec());
        } else if let Some(kept) = kept.as_mut() {
            kept.push(command.clone());
        }
    }
    kept.map(Cow::Owned).unwrap_or(Cow::Borrowed(commands))
}

fn paint_bounds(command: &PaintCmd) -> Option<LayoutRect> {
    Some(match command {
        PaintCmd::DrawRect(item) => item.placement.bounds,
        PaintCmd::DrawText(item) => item.placement.bounds,
        PaintCmd::DrawImage(item) => item.placement.bounds,
        PaintCmd::DrawLinearGradient(item) => item.placement.bounds,
        PaintCmd::DrawRadialGradient(item) => item.placement.bounds,
        PaintCmd::DrawConicGradient(item) => item.placement.bounds,
        PaintCmd::DrawStroke(item) if item.join != StrokeJoin::Miter => {
            // A full width also covers a square cap's diagonal reach.
            item.placement
                .bounds
                .inflate(item.width.abs(), item.width.abs())
        },
        PaintCmd::DrawPath(item) => match &item.stroke {
            None => item.placement.bounds,
            Some(stroke) if stroke.join != StrokeJoin::Miter => item
                .placement
                .bounds
                .inflate(stroke.width.abs(), stroke.width.abs()),
            _ => return None,
        },
        // Shadows, miter spikes, retained fragments and unfamiliar primitives
        // have no proven finite ink bounds here. Leave them to the renderer.
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
