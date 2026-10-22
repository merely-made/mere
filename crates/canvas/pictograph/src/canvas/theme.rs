// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One mapping from theme roles to the Map: the ground it clears to, the edge
//! stroke, the face palette and the five category colours. Hosts used to carry
//! their own copies (Graphshell read the roles from a browser cascade, Turnstone
//! did not theme the Map at all); both now resolve roles and apply this.

use crate::canvas::Canvas;
use crate::canvas::palette::{DerivedFacePalette, ThemeColor};
use tincture::{GatedRoles, Palette};

/// The theme roles the Map draws with, each as the host resolved it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanvasRoles {
    pub ground: ThemeColor,
    pub edge: ThemeColor,
    pub text: ThemeColor,
    pub primary: ThemeColor,
    pub secondary: ThemeColor,
    pub tertiary: ThemeColor,
    pub success: ThemeColor,
    pub danger: ThemeColor,
    pub categories: [ThemeColor; 5],
}

/// [`CanvasRoles`] resolved for [`Canvas::apply_theme`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasTheme {
    ground: [f32; 4],
    edge: [f32; 4],
    faces: DerivedFacePalette,
    categories: [[u8; 3]; 5],
}

fn unit(color: ThemeColor) -> [f32; 4] {
    color.to_array().map(|channel| f32::from(channel) / 255.0)
}

impl CanvasTheme {
    pub fn from_roles(roles: &CanvasRoles) -> Self {
        Self {
            ground: unit(roles.ground),
            edge: unit(roles.edge),
            faces: DerivedFacePalette::from_roles(
                [roles.primary, roles.secondary, roles.tertiary],
                [roles.success, roles.danger],
                roles.text,
            ),
            categories: roles.categories.map(|color| [color.r, color.g, color.b]),
        }
    }

    /// A derived Tinct palette with its gated roles: the ground is `bg`, the
    /// edge the dim text, and the statuses and categories the gated ones, as
    /// the stylesheet emits them.
    pub fn from_palette(palette: &Palette, gated: &GatedRoles) -> Self {
        Self::from_roles(&CanvasRoles {
            ground: palette.bg,
            edge: palette.text_dim,
            text: palette.text,
            primary: palette.primary,
            secondary: palette.secondary,
            tertiary: palette.tertiary,
            success: gated.success,
            danger: gated.danger,
            categories: gated.category,
        })
    }

    /// The ground as straight `[r, g, b, a]` (0..1), for the surface the Map
    /// is rasterized onto.
    pub fn ground(&self) -> [f32; 4] {
        self.ground
    }
}

impl Canvas {
    /// Re-theme the Map: ground, edge, derived faces and category colours.
    pub fn apply_theme(&mut self, theme: &CanvasTheme) {
        self.set_palette(theme.ground, theme.edge);
        self.set_derived_face_palette(theme.faces);
        self.set_category_colors(theme.categories);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color(value: u8) -> ThemeColor {
        ThemeColor::rgba(value, value / 2, 255 - value, 255)
    }

    #[test]
    fn roles_reach_the_face_palette_and_categories() {
        let roles = CanvasRoles {
            ground: color(10),
            edge: color(20),
            text: color(30),
            primary: color(40),
            secondary: color(50),
            tertiary: color(60),
            success: color(70),
            danger: color(80),
            categories: [color(90), color(100), color(110), color(120), color(130)],
        };
        let theme = CanvasTheme::from_roles(&roles);
        let faces = theme.faces.colors();
        assert_eq!(faces[0], color(40).to_array());
        assert_eq!(faces[3], color(70).to_array());
        assert_eq!(faces[4], color(80).to_array());
        assert_eq!(theme.categories[4], [130, 65, 125]);
        assert_eq!(theme.ground(), unit(color(10)));
    }
}
