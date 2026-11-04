/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// Follow a ray from a point inside a rectangle to the first perimeter it
/// exits. A clamped node can already sit on that perimeter, in which case the
/// unchanged origin is the truthful endpoint.
pub(super) fn rectangle_ray_exit(
    origin: (f32, f32),
    toward: (f32, f32),
    region: GraphCanvasNodeRegion,
) -> (f32, f32) {
    const EPSILON: f32 = 0.0001;
    let dx = toward.0 - origin.0;
    let dy = toward.1 - origin.1;
    if dx.abs() < EPSILON && dy.abs() < EPSILON {
        return origin;
    }

    let right = region.left + region.width;
    let bottom = region.top + region.height;
    let tx = if dx > EPSILON {
        (right - origin.0) / dx
    } else if dx < -EPSILON {
        (region.left - origin.0) / dx
    } else {
        f32::INFINITY
    };
    let ty = if dy > EPSILON {
        (bottom - origin.1) / dy
    } else if dy < -EPSILON {
        (region.top - origin.1) / dy
    } else {
        f32::INFINITY
    };
    let distance = tx.max(0.0).min(ty.max(0.0));
    if !distance.is_finite() {
        return origin;
    }
    (
        (origin.0 + dx * distance).clamp(region.left, right),
        (origin.1 + dy * distance).clamp(region.top, bottom),
    )
}

/// A label's width estimated from its length at the labels' 10 px size, no
/// wider than [`GRAPH_LABEL_WIDTH`].
pub(super) fn label_text_width(label: &str) -> f32 {
    (label.chars().count() as f32 * LABEL_CHAR_WIDTH + 2.0).min(GRAPH_LABEL_WIDTH)
}

/// `text` cut to at most `max` characters by dropping its middle, so a long
/// label keeps both its start and its distinguishing end, as in
/// `Session (fork of … blake3:f612a))`. Names that differ only at their end
/// (two forks, two codicils) stay told apart, which an end ellipsis loses.
/// Text that fits is returned as it is.
pub(crate) fn middle_cut(text: &str, max: usize) -> std::borrow::Cow<'_, str> {
    const MARK: &str = " … ";
    let count = text.chars().count();
    if count <= max || max <= MARK.chars().count() + 1 {
        return std::borrow::Cow::Borrowed(text);
    }
    // How far a cut moves to land between words rather than inside one.
    const SNAP: usize = 6;
    let chars: Vec<char> = text.chars().collect();
    let keep = max - MARK.chars().count();
    // The end is what tells long names apart. It starts at a word: the next
    // one when that is near, else the one it falls in.
    let mut tail_start = count - keep / 2;
    let at_word = |at: usize| at == 0 || chars[at - 1].is_whitespace();
    if !at_word(tail_start) {
        if let Some(ahead) = (0..SNAP)
            .take_while(|ahead| tail_start + ahead < count)
            .find(|ahead| chars[tail_start + ahead].is_whitespace())
        {
            tail_start += ahead + 1;
        } else if let Some(back) = (1..=SNAP)
            .take_while(|back| *back < tail_start)
            .find(|back| chars[tail_start - back].is_whitespace())
        {
            tail_start -= back - 1;
        }
    }
    // The start takes the rest, ending between words when one is near.
    let mut head_end = keep.saturating_sub(count - tail_start).clamp(1, tail_start);
    if !at_word(head_end)
        && let Some(back) = (1..=SNAP)
            .take_while(|back| *back < head_end)
            .find(|back| chars[head_end - back].is_whitespace())
    {
        head_end -= back;
    }
    let start: String = chars[..head_end].iter().collect();
    let end: String = chars[tail_start..].iter().collect();
    std::borrow::Cow::Owned(format!("{}{MARK}{}", start.trim_end(), end.trim_start()))
}

pub(super) fn graph_position_at(
    viewport: GraphViewport,
    size: Size,
    inset: f32,
    local: (f32, f32),
) -> (f32, f32) {
    let width = (size.width - 2.0 * inset).max(0.0);
    let height = (size.height - 2.0 * inset).max(0.0);
    let zoom = viewport.zoom.max(0.01);
    let x = if width > 0.0 {
        (local.0 - inset) / width
    } else {
        0.5
    };
    let y = if height > 0.0 {
        (local.1 - inset) / height
    } else {
        0.5
    };
    (
        ((x - 0.5 - viewport.pan.0) / zoom + 0.5).clamp(0.0, 1.0),
        ((y - 0.5 - viewport.pan.1) / zoom + 0.5).clamp(0.0, 1.0),
    )
}

pub(super) fn graph_position_at_unclamped(
    viewport: GraphViewport,
    size: Size,
    inset: f32,
    local: (f32, f32),
) -> (f32, f32) {
    let width = (size.width - 2.0 * inset).max(0.0);
    let height = (size.height - 2.0 * inset).max(0.0);
    let zoom = viewport.zoom.max(0.01);
    let x = if width > 0.0 {
        (local.0 - inset) / width
    } else {
        0.5
    };
    let y = if height > 0.0 {
        (local.1 - inset) / height
    } else {
        0.5
    };
    (
        (x - 0.5 - viewport.pan.0) / zoom + 0.5,
        (y - 0.5 - viewport.pan.1) / zoom + 0.5,
    )
}

/// The segment crossing a polyline's half-length point. Relation hit targets
/// sit on this segment, sharing the same path as the painted link.
pub(super) fn route_midpoint_segment(points: &[(f32, f32)]) -> Option<((f32, f32), (f32, f32))> {
    let segments = points
        .windows(2)
        .map(|pair| {
            let dx = pair[1].0 - pair[0].0;
            let dy = pair[1].1 - pair[0].1;
            (pair[0], pair[1], (dx * dx + dy * dy).sqrt())
        })
        .collect::<Vec<_>>();
    let total = segments.iter().map(|(_, _, length)| *length).sum::<f32>();
    let mut walked = 0.0;
    for (from, to, length) in &segments {
        if walked + length >= total * 0.5 {
            return Some((*from, *to));
        }
        walked += length;
    }
    segments.last().map(|(from, to, _)| (*from, *to))
}
