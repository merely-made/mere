// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A graph node's face beside its name: in a tab, a breadcrumb, a row or a
//! sentence (iconography T4).

use crate::{El, el};

/// The class every face wears, so one stylesheet rule sizes them all.
/// Pictograph's `inline_face_style` is that rule's body.
pub const FACE_CLASS: &str = "face";

/// An image of the face at `url`, decorative: the name beside it is what
/// assistive technology reads, so the image has an empty `alt` and is
/// `aria-hidden`. The host resolves `url` (a vector image, or bytes).
pub fn face<State: 'static, Action: 'static>(url: String) -> El<(), State, Action> {
    el("img", ())
        .attr("class", FACE_CLASS)
        .attr("src", url)
        .attr("alt", "")
        .attr("aria-hidden", "true")
}
