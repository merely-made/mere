// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Which layer draws the window frame.
///
/// This is the application-visible boundary. A [`Host`](Self::Host) frame may
/// be compositor-drawn or supplied by the platform window library; the
/// application leaves both forms alone. An [`App`](Self::App) frame is part of
/// the application's own view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowFrame {
    /// Let the platform host draw and operate the frame.
    #[default]
    Host,
    /// Let the application draw the frame; the host supplies window verbs and
    /// resize affordances.
    App,
}

/// Transparent margins reserved inside an application-drawn window frame.
///
/// These are logical pixels, matching the application's CSS coordinate space.
/// The application draws its frame and effects inside them. The desktop host
/// supplies an alpha-capable surface and, on X11, publishes the corresponding
/// device-pixel `_GTK_FRAME_EXTENTS` so the window manager snaps and maximizes
/// against the visible frame rather than the transparent shadow boundary.
///
/// Being CSS pixels, they **carry zoom** — see [`scaled`](Self::scaled). That
/// is the one stated exception to the rule that window geometry rides the
/// device scale: an app-drawn frame's insets are document geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppFrameInsets {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

/// Window and pipeline configuration.
pub struct HostOptions {
    /// Browser presentation preference. Native hosts retain their wgpu route.
    pub presentation_choice: crate::PresentationChoice,
    /// Browser WebGPU presents the same portable scene as direct WebGL2,
    /// composing producer scenes in the page pass instead of sampling their
    /// textures. For applications whose producers supply portable scenes.
    pub wgpu_portable_scene: bool,
    /// The window title.
    pub title: String,
    /// The layer responsible for the window frame.
    pub window_frame: WindowFrame,
    /// Compatibility input for existing consumers. `false` selects
    /// [`WindowFrame::App`]; `true` leaves [`window_frame`](Self::window_frame)
    /// in control.
    ///
    /// New consumers should set `window_frame` directly. This field can retire
    /// after the existing Woodshed consumer moves off the boolean API.
    pub decorations: bool,
    /// Transparent margins occupied by effects around an app-drawn frame.
    ///
    /// This is geometry, not a shared visual style: the application still
    /// chooses and draws its own shadow. Keep these values equal to the outer
    /// transparent margins in that application stylesheet. They are ignored
    /// when the effective [`WindowFrame`] is [`WindowFrame::Host`].
    pub app_frame_insets: AppFrameInsets,
    /// Accessible label of the application-drawn maximize/restore control.
    ///
    /// Windows uses the laid-out button with this label for its native Snap
    /// Layout hit rectangle. Applications that localize their caption labels
    /// should set the same localized value here.
    pub maximize_control_label: String,
    /// Logical size to open at when no environment override is present.
    pub initial_logical_size: (f64, f64),
    /// Application-restored position, size, and maximized state.
    ///
    /// The winit event source validates this against the monitors present at
    /// launch before applying it. Persistence remains application-owned.
    pub initial_geometry: Option<WindowGeometry>,
    /// Environment variable names overriding the logical width and height,
    /// for reproducible scenario receipts (`WOODSHED_WIDTH`-shaped).
    pub size_env: Option<(String, String)>,
    /// Renderer options for the event source's surface boot. A factory rather
    /// than a value: `NetrenderOptions` is not `Clone`, and the surface is booted
    /// again every time the platform takes it away and hands it back
    /// (suspend/resume), which must not silently fall back to defaults.
    pub netrender: Box<dyn Fn() -> NetrenderOptions>,
    /// Hold Tab and steer focus with the arrow keys (see [`spatial`]).
    ///
    /// On by default: it is additive, costs nothing until Tab is *held*, and
    /// tapping Tab behaves exactly as it always did. Turn it off for an
    /// application whose arrow keys mean something while a control is focused
    /// and which would rather not share them.
    ///
    /// [`spatial`]: crate::Direction
    pub spatial_focus: bool,
    /// The application's own interface zoom, multiplied onto the device scale
    /// to give the one [`layout_scale`](Host::layout_scale) everything is laid
    /// out and rasterized at.
    ///
    /// `1.0` is "no zoom" and is the default. With
    /// [`fit_design`](Self::fit_design) also set this becomes a user *offset*
    /// on the fit factor rather than the whole answer, which is what makes a
    /// keyboard step meaningful in a window that is already fitting a design.
    ///
    /// The host never persists it. An application that wants the preference to
    /// survive a launch writes it through its own settings contract and hands
    /// it back here.
    pub ui_zoom: f32,
    /// Design size, in logical pixels, the interface should be scaled to fit.
    ///
    /// With this set the host recomputes zoom on every resize as
    /// [`fit_zoom`], so an application asks for "fit 1100x820" once and never
    /// handles a resize itself. `None` — the default — leaves
    /// [`ui_zoom`](Self::ui_zoom) alone.
    pub fit_design: Option<(f32, f32)>,
    /// Vector images the application draws itself, by URL: asked for each
    /// `<img>` the host's image bytes do not hold, with the layout scale
    /// (device pixels per CSS pixel, zoom included), so a face beside a
    /// mention picks its detail for its real size. `None`, the default, is
    /// raster images only.
    pub vector_images: Option<VectorImageSource>,
}

/// Resolves an image URL to a vector image the application paints, given the
/// layout scale. See [`HostOptions::vector_images`].
pub type VectorImageSource =
    std::rc::Rc<dyn Fn(&str, f32) -> Option<std::sync::Arc<dyn genet_livery::VectorImage>>>;

impl AppFrameInsets {
    /// A frame with no transparent outer effect.
    pub const NONE: Self = Self {
        left: 0,
        right: 0,
        top: 0,
        bottom: 0,
    };

    /// The same margin on every edge.
    pub const fn uniform(value: u32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    pub const fn is_empty(self) -> bool {
        self.left == 0 && self.right == 0 && self.top == 0 && self.bottom == 0
    }

    /// The same margins in device pixels, at the **layout** scale.
    ///
    /// The application paints its frame out of its own stylesheet, so what
    /// lands on the glass is `inset * layout_scale`, zoom included. A host
    /// reserving that region — X11's `_GTK_FRAME_EXTENTS` is the one that does
    /// — must name the same number, or it tells the window manager the frame
    /// boundary is somewhere the application is not drawing. Multiplying by
    /// the device scale alone would be that mistake at any zoom but 1.
    pub fn scaled(self, layout_scale: f64) -> Self {
        let scaled = |value: u32| {
            (f64::from(value) * layout_scale)
                .round()
                .clamp(0.0, u32::MAX as f64) as u32
        };
        Self {
            left: scaled(self.left),
            right: scaled(self.right),
            top: scaled(self.top),
            bottom: scaled(self.bottom),
        }
    }
}

impl Default for HostOptions {
    fn default() -> Self {
        Self {
            presentation_choice: crate::PresentationChoice::Auto,
            wgpu_portable_scene: false,
            title: String::new(),
            window_frame: WindowFrame::Host,
            decorations: true,
            app_frame_insets: AppFrameInsets::NONE,
            maximize_control_label: "Maximize".into(),
            initial_logical_size: (1_100.0, 664.0),
            initial_geometry: None,
            size_env: None,
            netrender: Box::new(|| NetrenderOptions {
                tile_cache_size: Some(1024),
                enable_vello: true,
                ..Default::default()
            }),
            spatial_focus: true,
            ui_zoom: 1.0,
            fit_design: None,
            vector_images: None,
        }
    }
}

impl HostOptions {
    /// Resolve the named policy plus the source-compatible boolean input.
    pub fn effective_window_frame(&self) -> WindowFrame {
        if self.decorations {
            self.window_frame
        } else {
            WindowFrame::App
        }
    }

    /// Insets that actually participate in this window's frame policy.
    pub fn effective_app_frame_insets(&self) -> AppFrameInsets {
        if self.effective_window_frame() == WindowFrame::App {
            self.app_frame_insets
        } else {
            AppFrameInsets::NONE
        }
    }

    /// Whether the native window and presentation surface need alpha.
    pub fn app_frame_is_transparent(&self) -> bool {
        !self.effective_app_frame_insets().is_empty()
    }
}
