/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Cambium-native presentation for Nematic smolweb content (feature
//! `nematic`).
//!
//! Nematic retains `EngineDocument` lowering, while Errand owns the protocol
//! ASTs consumed here. This module projects those ASTs into reactive Cambium
//! views; the palette is tabard's. The engine-native smolweb lane that Pelt,
//! Turnstone and Signalman read through is mere-document-lanes'. This was the
//! `cambium-nematic` crate until 2026-09-24.

pub mod views;

pub use views::{SmolwebPalette, SmolwebTheme, SmolwebView, stylesheet};
