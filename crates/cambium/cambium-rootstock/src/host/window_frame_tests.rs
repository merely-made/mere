// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{AppFrameInsets, HostOptions, WindowFrame};

#[test]
fn host_frame_is_the_default() {
    assert_eq!(
        HostOptions::default().effective_window_frame(),
        WindowFrame::Host
    );
}

#[test]
fn an_app_frame_is_explicit() {
    let options = HostOptions {
        window_frame: WindowFrame::App,
        ..Default::default()
    };
    assert_eq!(options.effective_window_frame(), WindowFrame::App);
}

#[test]
fn the_old_false_input_still_selects_an_app_frame() {
    let options = HostOptions {
        decorations: false,
        ..Default::default()
    };
    assert_eq!(options.effective_window_frame(), WindowFrame::App);
}

#[test]
fn app_frame_insets_only_apply_to_an_app_frame() {
    let insets = AppFrameInsets::uniform(12);
    let host = HostOptions {
        app_frame_insets: insets,
        ..Default::default()
    };
    assert_eq!(host.effective_app_frame_insets(), AppFrameInsets::NONE);
    assert!(!host.app_frame_is_transparent());

    let app = HostOptions {
        window_frame: WindowFrame::App,
        app_frame_insets: insets,
        ..Default::default()
    };
    assert_eq!(app.effective_app_frame_insets(), insets);
    assert!(app.app_frame_is_transparent());
}

#[test]
fn app_frame_insets_are_css_pixels_and_carry_zoom() {
    // Device scale 2, zoom 1.25. The stylesheet's 8 CSS px of transparent
    // margin is painted at the layout scale, so 20 device pixels of glass
    // are the frame's outer boundary. Reserving `inset * device_scale`
    // instead claims 16 and names a boundary four pixels inside the one
    // the application actually paints.
    let insets = AppFrameInsets::uniform(8);
    assert_eq!(insets.scaled(2.0 * 1.25), AppFrameInsets::uniform(20));
    assert_eq!(insets.scaled(2.0), AppFrameInsets::uniform(16));
    // Zoom 1 is the identity, which is why nothing shipping moves.
    assert_eq!(insets.scaled(2.0 * 1.0), insets.scaled(2.0));

    // Per-edge, and rounded rather than truncated.
    let uneven = AppFrameInsets {
        left: 8,
        right: 10,
        top: 12,
        bottom: 14,
    };
    assert_eq!(
        uneven.scaled(1.5),
        AppFrameInsets {
            left: 12,
            right: 15,
            top: 18,
            bottom: 21,
        }
    );
}
