// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Immutable product readings paired with the rasterized source, not later state.
use crate::{Ctx, Product, Viewport};
use cambium_rootstock::PresentedFrame;
use serde::Serialize;
use std::{collections::BTreeMap, io};

/// Product-owned readings. Names, revision domains and redaction stay local.
/// A graph/save revision is not a whole-application semantic revision.
#[derive(Serialize)]
pub struct CaptureProjection {
    pub fields: BTreeMap<String, String>,
    pub viewport: Option<Viewport>,
    pub product: serde_json::Value,
}

/// A factory result, evaluated once at the successful presentation boundary.
/// `run` is a product-supplied fresh run identity, not a clock/sequence guess.
pub type CaptureObserverFn<P> =
    Box<dyn FnOnce(&Ctx<'_, P>, PresentedFrame) -> Result<CaptureProjection, String>>;

pub struct CaptureObserver<P: Product> {
    pub run: String,
    pub observe: CaptureObserverFn<P>,
}

/// Limits only the copied receipt metadata. It does not cap product allocation,
/// pixel storage, PNG files or legacy uncorrelated receipt fields.
#[derive(Clone, Copy, Debug)]
pub struct CaptureProjectionLimits {
    pub max_projection_bytes: usize,
    pub max_total_bytes: usize,
    pub max_captures: usize,
}
impl Default for CaptureProjectionLimits {
    fn default() -> Self {
        Self {
            max_projection_bytes: 65_536,
            max_total_bytes: 1_048_576,
            max_captures: 128,
        }
    }
}

/// Frame identity is qualified by `run`; `host` identifies a host/window
/// lifetime inside the process. Presentation means queue submission, not a
/// compositor or physical display acknowledgement.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Presentation {
    pub host: u64,
    pub sequence: u64,
    pub width: u32,
    pub height: u32,
    pub layout_scale: f32,
}
impl From<PresentedFrame> for Presentation {
    fn from(value: PresentedFrame) -> Self {
        Self {
            host: value.host,
            sequence: value.sequence,
            width: value.width,
            height: value.height,
            layout_scale: value.layout_scale,
        }
    }
}

#[derive(Serialize)]
pub struct CapturePairing {
    pub run: String,
    pub request: u64,
    pub presentation: Presentation,
    pub product_projection: serde_json::Value,
}

/// An armed request's identity, transported independently with its pixels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureRequest {
    pub run: String,
    pub request: u64,
}

pub(crate) struct SealedProjection {
    pub presentation: PresentedFrame,
    pub projection: CaptureProjection,
    pub encoded_bytes: usize,
}

/// Count encoded input without allocating another potentially oversized JSON
/// buffer. Products must also bound their own observer's construction work.
pub(crate) fn encoded_size(value: &impl Serialize, limit: usize) -> Result<usize, String> {
    struct Counter {
        bytes: usize,
        limit: usize,
    }
    impl io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let next = self
                .bytes
                .checked_add(bytes.len())
                .filter(|next| *next <= self.limit)
                .ok_or_else(|| io::Error::other("capture projection byte limit exceeded"))?;
            self.bytes = next;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter { bytes: 0, limit };
    serde_json::to_writer(&mut counter, value).map_err(|error| error.to_string())?;
    Ok(counter.bytes)
}
