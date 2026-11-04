// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// A capture armed by the application, run inside the next frame while the
/// rasterized view is still alive (scenario screenshots).
#[derive(Clone, Debug)]
pub enum CaptureRefusal {
    HostDisposed,
    ProducerResource {
        generation: u64,
        error: crate::ProducerGpuRemapError,
    },
    PresentationError {
        backend: crate::PresentationBackend,
        generation: crate::ContextGeneration,
        source_key: Option<u64>,
        error: String,
    },
    ContextLost {
        backend: crate::PresentationBackend,
        generation: crate::ContextGeneration,
    },
    DeviceLost(genet_render_host::DeviceLossNotice),
    SurfaceUnavailable {
        generation: u64,
        error: String,
    },
    ProviderError {
        generation: u64,
        source_key: Option<u64>,
        error: Arc<netrender::VelloError>,
    },
}

pub type CaptureFrame = Box<dyn crate::PendingPresentationCapture>;

pub type CaptureFn = Box<dyn FnOnce(Result<CaptureFrame, CaptureRefusal>) + 'static>;

/// Identity of one successful queue presentation in this host/window lifetime.
/// Host numbers are process-local; a receipt must also carry its product run.
/// This certifies submission of the rasterized source, not compositor visibility.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresentedFrame {
    /// Graphics lifetime that produced these pixels, scoped to `host`.
    pub context_generation: crate::ContextGeneration,
    pub host: u64,
    pub sequence: u64,
    pub width: u32,
    pub height: u32,
    pub layout_scale: f32,
}

/// A readback of that exact rasterized source, branded at presentation.
pub type StampedCaptureFrame = CaptureFrame;

pub type StampedCaptureFn = Box<dyn FnOnce(Result<StampedCaptureFrame, CaptureRefusal>) + 'static>;

/// One-shot read-only observation, before capture callbacks and pointer/AT input.
pub type PresentationObserver<State, Logic, V, T = Runner<State, Logic, V>> =
    Box<dyn FnOnce(&AppCtx<'_, State, Logic, V, T>, PresentedFrame)>;

/// The paint commands and inline font/image resources of the next presented
/// frame, before renderer translation. External GPU texture commands retain
/// references only: this does not read back their pixels. The envelope's
/// viewport is in layout coordinates, before device scale and UI zoom.
pub type PaintCaptureFn =
    Box<dyn FnOnce(Result<paint_list_api::PaintEnvelope, CaptureRefusal>) + 'static>;

impl std::fmt::Display for CaptureRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "capture refused: {self:?}")
    }
}

impl std::error::Error for CaptureRefusal {}
