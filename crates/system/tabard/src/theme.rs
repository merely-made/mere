// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host theme model: `ThemeTokenSet` and the [`registry::ThemeRegistry`]
//! that resolves it, seed derivation (`seed`), custom-mode calculators
//! (`mode_calc`), chrome colours (`chrome`), the edge-style token vocabulary
//! (`edge_style`) and the theme a lens carries (`data`). It was mere-registry's
//! `theme` feature, with lens's theme data, until both moved here on
//! 2026-09-24.
//!
//! Per the proposal §B.2 "bundle, remove egui" decision (2026-05-04), the
//! registry crate combined two previously-separate root-crate modules:
//!
//! - `model/graph/edge_style_registry.rs` (666 LOC) — the portable edge
//!   visual-style vocabulary (`EdgeStyleFamily`, `EdgeStrokePattern`,
//!   `EdgeEndpointMarker`, `EdgeAccessibilityMode`, `ThemeContract`,
//!   `ThemeEdgeTokens`, `validate_theme_edge_tokens`).
//! - `shell/desktop/runtime/registries/theme.rs` (608 LOC) — the
//!   `ThemeRegistry` itself, plus `ThemeTokenSet`, `GraphNodeChromeTheme`,
//!   theme seed data, and the donor registry's `theme` /
//!   `unregister_theme` / `resolve_theme` APIs.
//!
//! The bundling rationale: `theme` was the sole consumer of `edge_style`
//! (verified by exhaustive grep), and the donor registry's `theme()`
//! validated submitted tokens via `edge_style::validate_theme_edge_tokens`. The two modules
//! move in lockstep; splitting them across crates would impose a transitive
//! dep on every consumer of the edge vocabulary.
//!
//! The "remove egui" half of the decision: the original shell-side `theme.rs`
//! had a dead `#[cfg(feature = "egui-host")] pub(crate) use egui::Color32;`
//! gate, paired with `pub(crate) use graphshell_core::color::Color32;` for
//! the non-egui case. Per root `Cargo.toml:96`, `egui-host = []` is now an
//! empty no-op feature — egui has been removed from the dependency graph.
//! The cfg branch was therefore dead code and was dropped during the move;
//! this crate always used `graphshell_core::color::Color32`, which has since
//! merged into tinct's `Srgb`.

pub mod choice;
pub mod chrome;
pub mod data;
pub mod edge_style;
pub mod mode_calc;
pub mod registry;
pub mod seed;
