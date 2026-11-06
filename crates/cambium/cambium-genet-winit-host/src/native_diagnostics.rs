// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit diagnostic request; actual enabled device features remain authority.

use std::ffi::OsStr;

pub(crate) fn requested(value: Option<&OsStr>) -> bool {
    value == Some(OsStr::new("1"))
}

pub(crate) fn with_gpu_timing(
    mut options: netrender::NetrenderOptions,
    request_timestamps: bool,
) -> netrender::NetrenderOptions {
    if request_timestamps {
        // Optional admission keeps the existing adapter/device choice and
        // never forces an unsupported feature or changes required features.
        options.optional_features |=
            genet_render_host::RenderCore::native_worker_diagnostic_features();
    }
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_options_are_preserved_and_opt_in_adds_only_optional_timestamps() {
        let ordinary = netrender::NetrenderOptions {
            tile_cache_size: Some(17),
            enable_vello: true,
            optional_features: wgpu::Features::DEPTH_CLIP_CONTROL,
            ..Default::default()
        };
        let off = with_gpu_timing(ordinary, false);
        assert_eq!(off.optional_features, wgpu::Features::DEPTH_CLIP_CONTROL);
        assert_eq!(off.tile_cache_size, Some(17));
        assert!(off.enable_vello);
        let on = with_gpu_timing(off, true);
        assert_eq!(
            on.optional_features,
            wgpu::Features::DEPTH_CLIP_CONTROL | wgpu::Features::TIMESTAMP_QUERY
        );
        assert_eq!(on.tile_cache_size, Some(17));
        assert!(on.enable_vello);
    }

    #[test]
    fn the_diagnostic_flag_is_explicit_and_tests_do_not_change_process_environment() {
        assert!(!requested(None));
        assert!(!requested(Some(OsStr::new("0"))));
        assert!(!requested(Some(OsStr::new("true"))));
        assert!(requested(Some(OsStr::new("1"))));
    }
}
