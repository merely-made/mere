// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn compile_metre(out: &mut BackdropScene, src: sceno::SourceIx, grid: &MetreGrid) {
    let slot = 47.0;
    let row = 154.0;
    let n = grid.feet as usize * grid.positions_per_foot as usize;
    let max_syllables = grid.extent() as usize;
    let template = SequenceTemplate {
        lanes: vec![String::new(); grid.lines.len().max(1)],
        extent: max_syllables.max(1) as u32,
        ticks_per_unit: 1,
        unit_label: "metrical position".into(),
        groups: grid
            .lines
            .iter()
            .enumerate()
            .flat_map(|(i, line)| {
                (0..grid.feet).map(move |foot| TemplateGroup {
                    key: format!("{}/foot/{foot}", line.key),
                    lane: i as u16,
                    span: Span {
                        start: u32::from(foot) * u32::from(grid.positions_per_foot),
                        length: u32::from(grid.positions_per_foot),
                    },
                    parent: None,
                    label: String::new(),
                })
            })
            .collect(),
    };
    draw_template(out, src, &template);
    // Free verse retains source text without inventing syllable positions.
    // Caption width reserves presentation space only.
    let caption_width = |line: &VerseLine| line.text.chars().count() as f32 * 11.0;
    let text_right = grid
        .lines
        .iter()
        .filter(|line| line.syllables.is_empty())
        .map(|line| 48.0 + caption_width(line) + 38.0)
        .fold(0.0_f32, f32::max);
    let right = (template.x(max_syllables as f32) + 38.0).max(text_right);
    out.scene.bounds = Rect::new(
        Vec2::ZERO,
        Size2::new(right + 90.0, 42.0 + row * grid.lines.len().max(1) as f32),
    );
    let mut rows = Vec::new();
    for (i, line) in grid.lines.iter().enumerate() {
        let y = template.y(i as u16);
        let detail = if n == 0 {
            format!(
                "{} supplied units; form unspecified; rhyme {}",
                line.syllables.len(),
                line.rhyme_group.as_deref().unwrap_or("unspecified")
            )
        } else {
            format!(
                "{} syllables / {} expected; rhyme {}",
                line.syllables.len(),
                n,
                line.rhyme_group.as_deref().unwrap_or("unspecified")
            )
        };
        let id = mark(
            out,
            line.key.clone(),
            &line.source,
            line.text.clone(),
            detail,
            MarkRole::Node,
            Vec2::new(24.0, y),
            Footprint::Circle { radius: 5.0 },
            true,
            None,
        );
        out.scene.items[id.0 as usize].hit = Some(Footprint::Rect {
            size: Size2::new(36.0, 44.0),
        });
        rows.push(id);
        if line.syllables.is_empty() {
            out.guides.push(GuideLabel {
                at: Vec2::new(48.0 + caption_width(line) / 2.0, y + 17.0),
                text: line.text.clone(),
            });
        }
        for (j, syllable) in line.syllables.iter().enumerate() {
            let Some(span) = grid.placement(i, j) else {
                out.unplaced.push(Unplaced {
                    key: format!("{}/syllable/{j}", line.key),
                    source: line.source.clone(),
                    label: syllable.text.clone(),
                    reason: "Authored unplaced syllable; lexical stress is retained".into(),
                });
                continue;
            };
            let sx = template.centre(i as u16, span).x;
            let h = if !syllable.stress_known {
                12.0
            } else if syllable.stressed {
                24.0
            } else {
                7.0
            };
            mark(
                out,
                format!("{}/syllable/{j}", line.key),
                &line.source,
                syllable.text.clone(),
                format!(
                    "{} stress · syllable {} · start {} / span {} metrical positions",
                    if !syllable.stress_known {
                        "unknown"
                    } else if syllable.stressed {
                        "strong"
                    } else {
                        "weak"
                    },
                    j + 1,
                    span.start,
                    span.length
                ),
                if !syllable.stress_known {
                    MarkRole::Node
                } else if syllable.stressed {
                    MarkRole::StrongStress
                } else {
                    MarkRole::WeakStress
                },
                Vec2::new(sx, y - h / 2.0),
                Footprint::Rect {
                    size: Size2::new((slot * span.length as f32 - 8.0).max(3.0), h),
                },
                true,
                Some(Vec2::new(sx, y + 17.0)),
            );
        }
        if let Some(group) = &line.rhyme_group {
            out.guides.push(GuideLabel {
                at: Vec2::new(right, y),
                text: group.clone(),
            });
        }
        if n != 0 && line.syllables.len() != n {
            out.diagnostics.push(format!(
                "{}: {} syllables, template expects {n}",
                line.key,
                line.syllables.len()
            ));
        }
    }
    for b in 0..grid.lines.len() {
        let line = &grid.lines[b];
        let Some(group) = &line.rhyme_group else {
            continue;
        };
        let Some(a) = (0..b)
            .rev()
            .find(|&a| grid.lines[a].rhyme_group.as_ref() == Some(group))
        else {
            continue;
        };
        let previous = &grid.lines[a];
        let matches = previous
            .rhyme_tail
            .as_ref()
            .zip(line.rhyme_tail.as_ref())
            .is_some_and(|(a, b)| a == b);
        let y1 = template.y(a as u16);
        let y2 = template.y(b as u16);
        let points = (0..=24)
            .map(|step| {
                let t = step as f32 / 24.0;
                Vec2::new(right + 6.0 + 52.0 * 4.0 * t * (1.0 - t), y1 + (y2 - y1) * t)
            })
            .collect();
        relation(
            out,
            rows[a],
            rows[b],
            points,
            if matches {
                "derived.rhyme"
            } else {
                "expected.rhyme"
            },
            group.clone(),
            if matches {
                format!(
                    "Source-disclosed rhyme tails match: {}",
                    line.rhyme_tail.as_deref().unwrap()
                )
            } else {
                format!("Template group {group}; rhyme tails differ or are unknown")
            },
        );
        if !matches {
            out.diagnostics.push(format!(
                "{} / {}: expected rhyme {group} is not established",
                previous.key, line.key
            ));
        }
    }
}
