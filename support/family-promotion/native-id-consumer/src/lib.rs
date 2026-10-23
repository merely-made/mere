// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

/// Compile the native consumer outside Mere's root workspace patch table.
pub fn native_host() -> cambium_winit_a11y::A11yHost {
    cambium_winit_a11y::A11yHost::new(|| {})
}
