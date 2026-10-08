// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable titlebar composition. Applications own identity and content; the
//! shared frame supplies slots, drag regions and native caption-area spacing.

use crate::{AnyView, GenetCtx, GenetElement, el};

/// One application-owned titlebar slot. It may contain a plain title, an
/// ornament, commands, or caption controls supplied by the current host.
pub type TitleBarSlot<State> = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;

/// Layout and drag-region rules for [`title_bar`], without product colors.
/// Append these to the host stylesheet. Native hosts populate the caption-area
/// variables; other hosts use the zero inset and the default spacing.
pub const TITLE_BAR_CSS: &str = include_str!("title_bar.css");

/// Compose a titlebar from four application-owned slots.
///
/// Ornament and title content inherit the draggable frame. Actions and
/// captions carve out `no-drag` areas so controls receive ordinary pointer and
/// keyboard input. Interactive content placed in another slot should declare
/// its own `--app-region: no-drag` rule. This component has no window commands,
/// theme model or platform policy of its own.
pub fn title_bar<State: 'static>(
    ornament: TitleBarSlot<State>,
    title: TitleBarSlot<State>,
    actions: TitleBarSlot<State>,
    captions: TitleBarSlot<State>,
) -> TitleBarSlot<State> {
    Box::new(
        el(
            "header",
            (
                el("div", ornament).attr("class", "title-bar-ornament"),
                el("div", title).attr("class", "title-bar-title"),
                el("div", actions).attr("class", "title-bar-actions"),
                el("div", captions).attr("class", "title-bar-captions"),
            ),
        )
        .attr("class", "cambium-title-bar"),
    )
}
