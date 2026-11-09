// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Bounded native geometry for authored relation cues.
use super::StrokeCue;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokePart {
    pub offset: f64,
    pub width: f64,
    pub top: f64,
    pub height: f64,
    pub rounded: bool,
}

/// At most 32 pieces per relation, even for a distant pair of appearances.
/// Explicit geometry avoids a renderer turning a one-sided styled border
/// into its solid fallback. Density adapts beyond that finite piece budget.
pub fn stroke_parts(cue: StrokeCue, length: f64) -> Vec<StrokePart> {
    if !length.is_finite() || length <= 0.0 {
        return vec![];
    }
    let part = |offset, width, top, height, rounded| StrokePart {
        offset,
        width,
        top,
        height,
        rounded,
    };
    match cue {
        StrokeCue::Solid => vec![part(0.0, length, 0.0, 3.0, false)],
        StrokeCue::Double => vec![
            part(0.0, length, -2.0, 1.5, false),
            part(0.0, length, 2.0, 1.5, false),
        ],
        StrokeCue::Dashed | StrokeCue::Dotted => {
            let pitch = if cue == StrokeCue::Dashed { 12.0 } else { 6.0 };
            let count = (length / pitch).ceil().clamp(1.0, 32.0) as usize;
            let step = length / count as f64;
            let width = if cue == StrokeCue::Dashed {
                step * 0.7
            } else {
                step.min(5.0) * 0.6
            };
            (0..count)
                .map(|i| part(i as f64 * step, width, 0.0, 3.0, cue == StrokeCue::Dotted))
                .collect()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_cues_keep_distinct_geometry_and_a_finite_budget() {
        let solid = stroke_parts(StrokeCue::Solid, 350.0);
        let dashed = stroke_parts(StrokeCue::Dashed, 350.0);
        let dotted = stroke_parts(StrokeCue::Dotted, 350.0);
        let double = stroke_parts(StrokeCue::Double, 350.0);
        assert_eq!(solid.len(), 1);
        assert!(
            dashed
                .windows(2)
                .all(|p| p[0].offset + p[0].width < p[1].offset)
        );
        assert!(dotted.iter().all(|p| p.rounded && p.width <= 3.0));
        assert_eq!(double.len(), 2);
        assert!(double[0].top + double[0].height < double[1].top);
        assert_ne!(solid, dashed);
        assert_ne!(dashed, dotted);
        for cue in StrokeCue::ALL {
            for length in [0.01, 350.0, 100_000.0] {
                let parts = stroke_parts(cue, length);
                assert!(parts.len() <= 32);
                assert!(parts.iter().all(|p| p.offset >= 0.0
                    && p.width > 0.0
                    && p.offset + p.width <= length + 1e-6));
            }
            assert!(stroke_parts(cue, f64::NAN).is_empty());
            assert!(stroke_parts(cue, -1.0).is_empty());
        }
    }
}
