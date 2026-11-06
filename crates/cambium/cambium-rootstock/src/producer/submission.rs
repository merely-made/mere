// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Raster access through the producer's actual host attribution capability.

use super::ProducerContext;

impl ProducerContext<'_> {
    /// Rasterize in the active native batch, or through the existing checked
    /// path for direct/browser callers. No second batch or completion wait is
    /// introduced by an opted-in native producer.
    pub fn try_rasterize_scaled_for(
        &self,
        surface: u64,
        scene: &netrender::Scene,
        width: u32,
        height: u32,
        clear: netrender::ColorLoad,
        scale: f32,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), genet_render_host::RasterizeFailure> {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(submission) = self.native_submission {
            return submission.rasterize(surface, scene, width, height, clear, scale);
        }
        self.core
            .try_rasterize_scaled_for(surface, scene, width, height, clear, scale)
    }
}
