// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Native caption controls over the existing portable titlebar slots and
//! window-command queue. Platform framing remains the host's responsibility.

use cambium::{TitleBarSlot, button, el};

use crate::WindowCommands;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CaptionPlatform {
    Windows,
    MacOs,
    Linux,
    Other,
}

fn current_caption_platform() -> CaptionPlatform {
    if cfg!(target_os = "windows") {
        CaptionPlatform::Windows
    } else if cfg!(target_os = "macos") {
        CaptionPlatform::MacOs
    } else if cfg!(target_os = "linux") {
        CaptionPlatform::Linux
    } else {
        CaptionPlatform::Other
    }
}

fn default_window_frame(platform: CaptionPlatform) -> crate::WindowFrame {
    match platform {
        CaptionPlatform::Windows => crate::WindowFrame::App,
        CaptionPlatform::MacOs | CaptionPlatform::Linux | CaptionPlatform::Other => {
            crate::WindowFrame::Host
        }
    }
}

fn uses_explicit_caption_controls(
    platform: CaptionPlatform,
    frame: crate::WindowFrame,
) -> bool {
    frame == crate::WindowFrame::App && platform != CaptionPlatform::MacOs
}

/// The desktop host's suggested frame policy for this platform.
///
/// Applications can still override this through [`crate::HostOptions::window_frame`].
/// Windows defaults to an application frame; macOS and Linux default to the
/// platform-owned frame.
pub fn platform_window_frame() -> crate::WindowFrame {
    default_window_frame(current_caption_platform())
}

/// Accessible names for the three window controls.
///
/// `maximize` must equal [`crate::HostOptions::maximize_control_label`] so the
/// Windows Snap bridge finds the same retained maximize button by its name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptionLabels {
    pub minimize: String,
    pub maximize: String,
    pub close: String,
}

impl Default for CaptionLabels {
    fn default() -> Self {
        Self {
            minimize: "Minimize".into(),
            maximize: "Maximize".into(),
            close: "Close".into(),
        }
    }
}

/// Three ordinary Cambium buttons sharing the host's window-command queue.
///
/// Hosts drain these commands through their existing window and close-policy
/// paths. The returned slot owns no native window handle and supplies no
/// product theme. It remains available on every platform for explicit custom
/// frames; use [`platform_caption_controls`] for the usual platform policy.
pub fn window_caption_controls<State: 'static>(
    commands: &WindowCommands,
    labels: &CaptionLabels,
) -> TitleBarSlot<State> {
    let minimize = commands.clone();
    let maximize = commands.clone();
    let close = commands.clone();
    let buttons: Vec<TitleBarSlot<State>> = vec![
        Box::new(
            button("—", move |_: &mut State, _| minimize.minimize())
                .attr("aria-label", labels.minimize.clone())
                .attr("data-window-action", "minimize"),
        ),
        Box::new(
            button("□", move |_: &mut State, _| maximize.toggle_maximize())
                .attr("aria-label", labels.maximize.clone())
                .attr("data-window-action", "maximize"),
        ),
        Box::new(
            button("×", move |_: &mut State, _| close.close())
                .attr("aria-label", labels.close.clone())
                .attr("data-window-action", "close"),
        ),
    ];
    Box::new(
        el("div", buttons)
            .attr("class", "window-caption-controls")
            .attr("style", "display: flex; --app-region: no-drag;"),
    )
}

/// Platform caption controls for an application-drawn frame: macOS retains its
/// traffic lights and receives an empty slot; other platforms receive explicit
/// controls. When the selected frame may be host-owned, use
/// [`caption_controls_for_frame`] instead.
pub fn platform_caption_controls<State: 'static>(
    commands: &WindowCommands,
    labels: &CaptionLabels,
) -> TitleBarSlot<State> {
    if cfg!(target_os = "macos") {
        Box::new(String::new())
    } else {
        window_caption_controls(commands, labels)
    }
}

/// Caption controls appropriate for the selected frame on this platform.
///
/// Host frames keep their native caption controls. App frames use explicit
/// controls except on macOS, where the host retains the native traffic lights
/// even when the content extends into the titlebar area.
pub fn caption_controls_for_frame<State: 'static>(
    commands: &WindowCommands,
    labels: &CaptionLabels,
    frame: crate::WindowFrame,
) -> TitleBarSlot<State> {
    if uses_explicit_caption_controls(current_caption_platform(), frame) {
        window_caption_controls(commands, labels)
    } else {
        Box::new(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::{CaptionPlatform, default_window_frame, uses_explicit_caption_controls};
    use crate::WindowFrame;

    #[test]
    fn platform_frame_defaults_are_explicit_for_desktop_targets() {
        assert_eq!(
            default_window_frame(CaptionPlatform::Windows),
            WindowFrame::App
        );
        for platform in [
            CaptionPlatform::MacOs,
            CaptionPlatform::Linux,
            CaptionPlatform::Other,
        ] {
            assert_eq!(default_window_frame(platform), WindowFrame::Host);
        }
    }

    #[test]
    fn frame_caption_policy_preserves_native_controls_where_the_host_owns_them() {
        for platform in [
            CaptionPlatform::Windows,
            CaptionPlatform::Linux,
            CaptionPlatform::Other,
        ] {
            assert!(!uses_explicit_caption_controls(platform, WindowFrame::Host));
            assert!(uses_explicit_caption_controls(platform, WindowFrame::App));
        }
        assert!(!uses_explicit_caption_controls(
            CaptionPlatform::MacOs,
            WindowFrame::Host
        ));
        assert!(!uses_explicit_caption_controls(
            CaptionPlatform::MacOs,
            WindowFrame::App
        ));
    }
}
