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

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use genet_livery::VectorImage;
use paint_list_api::{
    ColorF, CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, PathCommand, PathData, PathItem,
    StrokeCap, StrokeJoin, StrokeStyle,
};

use super::{DerivedFacePalette, TextFacePalette};

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
    let palette = crate::palette_from_rgba(palette.colors());
    crate::flat_icon::commands(file, &palette, device_height, rect).ok()
}

/// The image URL a host registers an address's face under and names in the
/// `<img src>` beside a mention. One address, one URL: the palette and device
/// scale belong to the host's document, not the URL.
pub fn inline_face_url(address: &str) -> String {
    let mut url = String::with_capacity(5 + address.len() * 2);
    url.push_str("face:");
    for byte in address.bytes() {
        let _ = write!(url, "{byte:02x}");
    }
    url
}

/// The address an [`inline_face_url`] names, or `None` for any other URL.
pub fn inline_face_address(url: &str) -> Option<String> {
    let hex = url.strip_prefix("face:")?.as_bytes();
    if hex.len() % 2 != 0 {
        return None;
    }
    let nibble = |digit: u8| match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    };
    let bytes = hex
        .chunks(2)
        .map(|pair| Some(nibble(pair[0])? << 4 | nibble(pair[1])?))
        .collect::<Option<Vec<u8>>>()?;
    String::from_utf8(bytes).ok()
}

/// Every face a host's document shows, in one palette: the resolver a text
/// host hands its engine for [`inline_face_url`] images. Faces are derived
/// once per address and scale and kept.
#[derive(Debug)]
pub struct InlineFaces {
    palette: TextFacePalette,
    outline: Option<[u8; 4]>,
    made: Mutex<HashMap<(String, u32), Arc<InlineFace>>>,
}

impl InlineFaces {
    /// Faces in `palette`, built for the ground the host's text sits on.
    pub fn new(palette: TextFacePalette) -> Self {
        Self {
            palette,
            outline: None,
            made: Mutex::new(HashMap::new()),
        }
    }

    /// Ring faces in `color` where the palette could not hold its floor.
    #[must_use]
    pub fn with_outline(mut self, color: [u8; 4]) -> Self {
        self.outline = Some(color);
        self
    }

    /// The face `url` names at `device_scale`, or `None` if `url` is not an
    /// [`inline_face_url`].
    pub fn resolve(&self, url: &str, device_scale: f32) -> Option<Arc<InlineFace>> {
        let address = inline_face_address(url)?;
        let key = (address, device_scale.to_bits());
        let mut made = self
            .made
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(face) = made.get(&key) {
            return Some(face.clone());
        }
        let mut face = InlineFace::for_address(&key.0, self.palette, device_scale);
        if let Some(color) = self.outline {
            face = face.with_outline(color);
        }
        let face = Arc::new(face);
        made.insert(key, face.clone());
        Some(face)
    }
    /// [`resolve`](Self::resolve) as the text engine's vector image, the
    /// shape a host's image source returns.
    pub fn resolve_image(&self, url: &str, device_scale: f32) -> Option<Arc<dyn VectorImage>> {
        self.resolve(url, device_scale)
            .map(|face| face as Arc<dyn VectorImage>)
    }

    /// The stylesheet rule sizing every face wearing `class`, for a host
    /// that does not know its device scale when it writes its sheet: whole
    /// CSS pixels, which are whole device pixels at any integer scale.
    pub fn rule(class: &str) -> String {
        format!("img.{class} {{ {} }}", inline_face_style(1.0))
    }
}

/// A face as an embedder vector image: what a text host registers with
/// genet-livery (`LiveryDocument::set_vector_image`, or its own image lookup)
/// under [`inline_face_url`]. It draws through [`inline_face_commands`], and
/// with an outline colour set it rings itself one device pixel wide in that
/// colour when its palette marks any entry [`outlined`].
///
/// [`outlined`]: TextFacePalette::outlined
#[derive(Clone, Debug)]
pub struct InlineFace {
    file: Vec<u8>,
    palette: TextFacePalette,
    device_scale: f32,
    outline: Option<[u8; 4]>,
}

impl InlineFace {
    /// The face of canonical `address` in `palette`, for a display of
    /// `device_scale`. The address is the one the canvas keys on (G6).
    pub fn for_address(address: &str, palette: TextFacePalette, device_scale: f32) -> Self {
        let file = crate::derive(address.as_bytes()).expect("every address face encodes");
        Self::new(file, palette, device_scale)
    }

    /// A face from its bytes.
    pub fn new(file: Vec<u8>, palette: TextFacePalette, device_scale: f32) -> Self {
        let device_scale = if device_scale.is_finite() && device_scale > 0.0 {
            device_scale
        } else {
            1.0
        };
        Self {
            file,
            palette,
            device_scale,
            outline: None,
        }
    }

    /// Ring the face in `color` (the text colour) where the palette could not
    /// hold its floor.
    #[must_use]
    pub fn with_outline(mut self, color: [u8; 4]) -> Self {
        self.outline = Some(color);
        self
    }

    /// The face's bytes.
    pub fn file(&self) -> &[u8] {
        &self.file
    }
}

impl VectorImage for InlineFace {
    fn intrinsic_size(&self) -> (f32, f32) {
        (16.0, 16.0)
    }

    fn paint(&self, rect: LayoutRect) -> Vec<PaintCmd> {
        let mut commands =
            inline_face_commands(&self.file, self.palette.faces(), self.device_scale, rect)
                .unwrap_or_default();
        if let Some(color) = self.outline
            && self.palette.outlined().contains(&true)
        {
            commands.push(outline(rect, color, 1.0 / self.device_scale));
        }
        commands
    }
}

/// A `width`-wide ring just inside `rect`.
fn outline(rect: LayoutRect, [r, g, b, a]: [u8; 4], width: f32) -> PaintCmd {
    let inset = width / 2.0;
    let (x0, y0) = (rect.min.x + inset, rect.min.y + inset);
    let (x1, y1) = (rect.max.x - inset, rect.max.y - inset);
    let channel = |value: u8| f32::from(value) / 255.0;
    PaintCmd::DrawPath(PathItem {
        placement: CommonPlacement::new(rect),
        path: PathData {
            commands: vec![
                PathCommand::MoveTo(LayoutPoint::new(x0, y0)),
                PathCommand::LineTo(LayoutPoint::new(x1, y0)),
                PathCommand::LineTo(LayoutPoint::new(x1, y1)),
                PathCommand::LineTo(LayoutPoint::new(x0, y1)),
                PathCommand::Close,
            ],
        },
        fill: None,
        stroke: Some(StrokeStyle {
            color: ColorF::new(channel(r), channel(g), channel(b), channel(a)),
            width,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
            dash: None,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::palette::ThemeColor;

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

    #[test]
    fn one_address_one_url() {
        assert_eq!(inline_face_url("ab"), "face:6162");
        assert_ne!(inline_face_url("a"), inline_face_url("b"));
        let address = "file:///notes/é.md";
        assert_eq!(
            inline_face_address(&inline_face_url(address)).as_deref(),
            Some(address)
        );
        for other in ["https://a.test/", "face:6", "face:zz", "face:ff"] {
            assert_eq!(inline_face_address(other), None, "{other}");
        }
    }

    #[test]
    fn faces_resolve_only_their_own_urls_and_are_made_once() {
        let faces = InlineFaces::new(TextFacePalette::new(DerivedFacePalette::default(), &[]));
        let url = inline_face_url("https://example.test/");
        let first = faces.resolve(&url, 1.0).expect("a face url");
        assert!(Arc::ptr_eq(&first, &faces.resolve(&url, 1.0).unwrap()));
        assert!(!Arc::ptr_eq(&first, &faces.resolve(&url, 2.0).unwrap()));
        assert_eq!(
            first.file(),
            crate::derive(b"https://example.test/").unwrap().as_slice()
        );
        assert!(faces.resolve("https://example.test/a.png", 1.0).is_none());
    }

    #[test]
    fn an_inline_face_draws_the_canvas_face_and_outlines_only_when_asked() {
        let rect = LayoutRect::new(LayoutPoint::new(0.0, 0.0), LayoutPoint::new(10.0, 10.0));
        let grounds = [0x00, 0x80, 0xFF].map(ThemeColor::gray);
        let palette = TextFacePalette::new(DerivedFacePalette::default(), &grounds);
        assert!(palette.outlined().contains(&true));
        let face = InlineFace::for_address("https://example.test/", palette, 2.0);
        let plain = face.paint(rect);
        assert_eq!(
            Some(plain.clone()),
            inline_face_commands(face.file(), palette.faces(), 2.0, rect)
        );
        let ringed = face.clone().with_outline([0, 0, 0, 255]).paint(rect);
        assert_eq!(ringed.len(), plain.len() + 1);
        let held = TextFacePalette::new(DerivedFacePalette::default(), &[]);
        let quiet = InlineFace::for_address("https://example.test/", held, 2.0)
            .with_outline([0, 0, 0, 255])
            .paint(rect);
        assert_eq!(quiet.len(), plain.len());
    }
}
