// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The browser event source for a Cambium application host.
//!
//! The scion to [`cambium_rootstock`]'s stock. Everything a Cambium
//! application is made of already builds for `wasm32-unknown-unknown`: the
//! retained layout over the runner's DOM, hit testing, input routing, focus
//! and spatial navigation, frame pacing, accessibility projection. What was
//! missing was something to tell it what happened, and something to present
//! onto. That is this crate, and it is deliberately small.
//!
//! Three pieces, each answering one seam the host defines:
//!
//! | seam | here | on the desktop |
//! |---|---|---|
//! | [`Surface`] | [`WebSurface`], a canvas | a winit window's surface |
//! | [`HostWindow`] | [`WebWindow`] | the winit window |
//! | [`Accessibility`] | [`DomAccessibility`], a DOM mirror with ARIA | an AccessKit tree |
//! | [`FileChooser`] | [`WebFileChooser`], a file input in the page | the platform's open dialog |
//!
//! A capture hook reads a presented frame back as a [`PendingFrame`] through
//! [`capture_into`], where the desktop reads it at once with `read_frame`.
//! [`WebCapture`] supplies asynchronous readback to Mesquite's scenario lane.
//!
//! ## Off wasm, only the mirror's plan compiles
//!
//! Everything that touches the browser is `#[cfg(target_arch = "wasm32")]`,
//! so a native `cargo check --workspace` sees only [`mirror`], the pure
//! lowering of the accessibility projection, whose tests run natively. That
//! has a cost worth stating: a green workspace check says nothing about the
//! rest of this crate, because a target cargo never builds cannot fail. Check
//! it for the target it is for:
//!
//! ```text
//! cargo check -p cambium-genet-web-host --target wasm32-unknown-unknown
//! ```

pub mod mirror;

/// DOM delta modes are pixels (0), lines (1) and pages (2). Preserve the
/// DOM sign while resolving units into the host's logical scroll pixels.
#[cfg(any(target_arch = "wasm32", test))]
fn wheel_delta_pixels(x: f64, y: f64, mode: u32, line_px: f32, page_px: f32) -> (f32, f32) {
    let scale = match mode {
        1 => line_px,
        2 => page_px,
        _ => 1.0,
    };
    (x as f32 * scale, y as f32 * scale)
}

#[cfg(test)]
mod wheel_tests {
    use super::wheel_delta_pixels;

    #[test]
    fn browser_direction_and_units_match_host_scroll_offsets() {
        assert_eq!(wheel_delta_pixels(5.0, 40.0, 0, 20.0, 300.0), (5.0, 40.0));
        assert_eq!(
            wheel_delta_pixels(-5.0, -40.0, 0, 20.0, 300.0),
            (-5.0, -40.0)
        );
        assert_eq!(wheel_delta_pixels(1.0, 2.0, 1, 20.0, 300.0), (20.0, 40.0));
        assert_eq!(
            wheel_delta_pixels(-1.0, -2.0, 2, 20.0, 300.0),
            (-300.0, -600.0)
        );
    }
}

#[cfg(target_arch = "wasm32")]
mod a11y;
#[cfg(target_arch = "wasm32")]
mod capture;
#[cfg(target_arch = "wasm32")]
mod files;
#[cfg(target_arch = "wasm32")]
mod input;
#[cfg(target_arch = "wasm32")]
mod lane;
#[cfg(target_arch = "wasm32")]
mod mount;
#[cfg(target_arch = "wasm32")]
mod start;
#[cfg(target_arch = "wasm32")]
mod surface;

#[cfg(target_arch = "wasm32")]
pub use a11y::DomAccessibility;
#[cfg(target_arch = "wasm32")]
pub use capture::{PendingFrame, capture_into};
#[cfg(target_arch = "wasm32")]
pub use files::WebFileChooser;
#[cfg(target_arch = "wasm32")]
pub use input::{
    CompositionKind, composition_from_dom, key_press_from_dom, modifiers_from_dom,
    wheel_delta_from_dom,
};
#[cfg(target_arch = "wasm32")]
pub use lane::WebCapture;
#[cfg(target_arch = "wasm32")]
pub use mount::{Mounted, mount};
#[cfg(target_arch = "wasm32")]
pub use start::run_static_constructors_once;
#[cfg(target_arch = "wasm32")]
pub use surface::{WebSurface, WebWindow};

/// How far one wheel line scrolls, in logical pixels.
///
/// The DOM may report a wheel notch in lines rather than pixels and does not
/// say how tall a line is. Firefox reports lines by default where Chromium
/// reports pixels, so without this the same gesture scrolls a different
/// distance per browser.
pub const WHEEL_LINE_PX: f32 = 16.0;

/// How far one wheel page scrolls, in logical pixels.
///
/// A fallback: `DOM_DELTA_PAGE` is rare, and a viewport-relative figure would
/// need a viewport this constant does not have.
pub const WHEEL_PAGE_PX: f32 = 400.0;
