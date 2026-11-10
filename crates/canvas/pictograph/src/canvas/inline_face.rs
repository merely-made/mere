// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A face inline in text (iconography T3): the sizing every host shares and
//! the paths a host hands its text engine.
//!
//! The text engine knows nothing of faces. A host registers each face as a
//! vector image (genet-livery's `VectorImage`), writes [`inline_face_style`]
//! on the `<img>` naming it, and paints it with [`inline_face_commands`]. The
//! engine lays the image out like any other: a square of the font's cap
//! height, bottom on the baseline.

use paint_list_api::{LayoutRect, PaintCmd};

use super::DerivedFacePalette;

/// The smallest inline face, in device pixels.
pub const INLINE_FACE_MIN_DEVICE_PX: f32 = 8.0;

/// The CSS a host puts on an inline face for `device_scale` device pixels per
/// CSS pixel: the font's cap height, at least
/// [`INLINE_FACE_MIN_DEVICE_PX`], rounded to a whole device pixel, with
/// `width: auto` taking the face's square ratio. Tabs, breadcrumbs, rows and
/// running text all use it.
pub fn inline_face_style(device_scale: f32) -> String {
    let scale = if device_scale.is_finite() && device_scale > 0.0 {
        device_scale
    } else {
        1.0
    };
    let pixel = 1.0 / scale;
    let floor = INLINE_FACE_MIN_DEVICE_PX / scale;
    format!(
        "height: round(nearest, max({floor}px, 1cap), {pixel}px); width: auto; \
         vertical-align: baseline"
    )
}

/// The paint commands for face bytes `file` filling `rect` (CSS pixels) on a
/// display of `device_scale`. The arm is chosen by the device height, so a 2×
/// display draws the arm for its real size. `None` if the bytes do not
/// decode or use a gradient.
pub fn inline_face_commands(
    file: &[u8],
    palette: DerivedFacePalette,
    device_scale: f32,
    rect: LayoutRect,
) -> Option<Vec<PaintCmd>> {
    let device_height = (rect.max.y - rect.min.y) * device_scale;
    super::derived_face::commands(file, palette, device_height, rect).ok()
}

#[cfg(test)]
mod tests {
    use paint_list_api::LayoutPoint;

    use super::*;

    #[test]
    fn the_style_rounds_to_device_pixels_with_an_eight_pixel_floor() {
        assert_eq!(
            inline_face_style(1.0),
            "height: round(nearest, max(8px, 1cap), 1px); width: auto; vertical-align: baseline"
        );
        assert_eq!(
            inline_face_style(2.0),
            "height: round(nearest, max(4px, 1cap), 0.5px); width: auto; vertical-align: baseline"
        );
        assert_eq!(inline_face_style(0.0), inline_face_style(1.0));
    }

    #[test]
    fn the_device_height_picks_the_arm() {
        let face = crate::derive(b"https://example.test/").expect("face");
        let rect = LayoutRect::new(LayoutPoint::new(0.0, 0.0), LayoutPoint::new(10.0, 10.0));
        let palette = DerivedFacePalette::default();
        let draw = |scale| format!("{:?}", inline_face_commands(&face, palette, scale, rect));
        // 10 device pixels is the text arm, 20 the coarse arm: one rect, two
        // drawings.
        assert_ne!(draw(1.0), draw(2.0));
        // Any scale that keeps the device height inside one arm draws alike.
        assert_eq!(draw(1.0), draw(1.3));
    }
}
