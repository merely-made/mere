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

#[cfg(target_arch = "wasm32")]
mod a11y;
#[cfg(target_arch = "wasm32")]
mod files;
#[cfg(target_arch = "wasm32")]
mod input;
#[cfg(target_arch = "wasm32")]
mod mount;
#[cfg(target_arch = "wasm32")]
mod surface;

#[cfg(target_arch = "wasm32")]
pub use a11y::DomAccessibility;
#[cfg(target_arch = "wasm32")]
pub use files::WebFileChooser;
#[cfg(target_arch = "wasm32")]
pub use input::{
    CompositionKind, composition_from_dom, key_press_from_dom, modifiers_from_dom,
    wheel_delta_from_dom,
};
#[cfg(target_arch = "wasm32")]
pub use mount::{Mounted, mount};
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
