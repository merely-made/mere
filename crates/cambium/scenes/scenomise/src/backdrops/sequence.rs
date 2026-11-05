// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Shared groups and coordinates. Domain-specific marks and judgments stay
/// with their owning recipe/adapter.
pub(super) fn draw_template(
    out: &mut BackdropScene,
    src: sceno::SourceIx,
    template: &SequenceTemplate,
) {
    for (i, label) in template.lanes.iter().enumerate() {
        if !label.is_empty() {
            out.guides.push(GuideLabel {
                at: Vec2::new(
                    template.x(template.extent as f32 / 2.0),
                    template.y(i as u16) - 40.0,
                ),
                text: label.clone(),
            });
        }
    }
    let depths = template.depths();
    let mut lane_depth = vec![0; template.lanes.len()];
    for group in &template.groups {
        lane_depth[group.lane as usize] =
            lane_depth[group.lane as usize].max(depths[group.key.as_str()]);
    }
    for group in &template.groups {
        let depth = depths[group.key.as_str()];
        let max_depth = lane_depth[group.lane as usize];
        let y = template.y(group.lane) + 33.0 + (max_depth - depth) as f32 * 20.0;
        let a = template.x(group.span.start as f32);
        let b = template.x(group.span.end().unwrap() as f32) - 5.0;
        guide(
            out,
            src,
            vec![
                Vec2::new(a, y),
                Vec2::new(a, y + 4.0),
                Vec2::new(b, y + 4.0),
                Vec2::new(b, y),
            ],
        );
        if !group.label.is_empty() {
            out.guides.push(GuideLabel {
                at: Vec2::new((a + b) / 2.0, y + 16.0),
                text: group.label.clone(),
            });
        }
    }
}

pub(super) fn compile_sequence(
    out: &mut BackdropScene,
    src: sceno::SourceIx,
    sequence: &TemplatedSequence,
) {
    use std::collections::HashMap;
    let t = &sequence.template;
    draw_template(out, src, t);
    out.scene.bounds = Rect::new(
        Vec2::ZERO,
        Size2::new(t.x(t.extent as f32) + 48.0, 154.0 * t.lanes.len() as f32),
    );
    let source = out.scene.sources[src.0 as usize].clone();
    let mut anchors = HashMap::new();
    for slot in &sequence.slots {
        let centre = t.centre(slot.lane, slot.span);
        let at = Vec2::new(centre.x, centre.y + 20.0);
        let id = mark(
            out,
            slot.key.clone(),
            &source,
            slot.label.clone(),
            format!(
                "Expected: {} · {} ticks at {} · {}",
                slot.expectation, slot.span.length, slot.span.start, t.unit_label
            ),
            MarkRole::Node,
            at,
            Footprint::Circle { radius: 3.0 },
            true,
            Some(Vec2::new(at.x, at.y + 12.0)),
        );
        anchors.insert(slot.key.as_str(), (id, at));
        if !sequence
            .occurrences
            .iter()
            .any(|o| o.slot.as_ref() == Some(&slot.key))
        {
            out.diagnostics.push(format!(
                "{}: no source occurrence aligned; expectation remains open",
                slot.key
            ));
        }
    }
    for occurrence in &sequence.occurrences {
        let Some(p) = &occurrence.placement else {
            out.unplaced.push(Unplaced {
                key: occurrence.key.clone(),
                source: occurrence.source.clone(),
                label: occurrence.label.clone(),
                reason: format!("No supplied alignment · {}", occurrence.detail),
            });
            continue;
        };
        let centre = t.centre(p.lane, p.span);
        let at = Vec2::new(centre.x, centre.y - 10.0);
        let width = t.x(p.span.length as f32) - t.x(0.0) - 8.0;
        let id = mark(
            out,
            occurrence.key.clone(),
            &occurrence.source,
            occurrence.label.clone(),
            format!(
                "{} · start {} / span {} ticks · {}",
                occurrence.detail, p.span.start, p.span.length, t.unit_label
            ),
            MarkRole::Other,
            at,
            Footprint::Rect {
                size: Size2::new(width.max(3.0), 12.0),
            },
            true,
            Some(Vec2::new(at.x, at.y - 12.0)),
        );
        anchors.insert(occurrence.key.as_str(), (id, at));
        if let Some(slot) = &occurrence.slot {
            let (to, point) = anchors[slot.as_str()];
            relation(
                out,
                id,
                to,
                vec![at, point],
                "template.alignment",
                "Alignment".into(),
                "Source-declared alignment to expectation; no conformity judgment inferred".into(),
            );
        }
    }
    for r in &sequence.relations {
        if let (Some(&(from, a)), Some(&(to, b))) =
            (anchors.get(r.from.as_str()), anchors.get(r.to.as_str()))
        {
            let points = (0..=24)
                .map(|i| {
                    let u = i as f32 / 24.0;
                    Vec2::new(
                        a.x + (b.x - a.x) * u,
                        a.y + (b.y - a.y) * u - 22.0 * 4.0 * u * (1.0 - u),
                    )
                })
                .collect();
            relation(
                out,
                from,
                to,
                points,
                &r.kind,
                r.label.clone(),
                r.why.clone(),
            );
        } else {
            out.diagnostics.push(format!(
                "{}: relation has an unplaced endpoint; {}",
                r.label, r.why
            ));
        }
    }
}
