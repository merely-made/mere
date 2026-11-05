// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One qualified image, retained only while its complete admitted inputs agree.
use super::{PreparedFrame, PreparedPaint};

pub(crate) struct QualifiedRasterCache<D, T> {
    entry: Option<Entry<D, T>>,
}
struct Entry<D, T> {
    device: D,
    frame: FrameKey,
    value: T,
}
struct FrameKey {
    paint: paint_list_api::PaintEnvelope,
    host: u64,
    generation: super::ContextGeneration,
    extent: [u32; 2],
    scale: f32,
    clear: [f32; 4],
    producers: Vec<crate::ProducerImageIdentity>,
}
impl<D, T> Default for QualifiedRasterCache<D, T> {
    fn default() -> Self {
        Self { entry: None }
    }
}
impl FrameKey {
    fn from_frame(frame: &PreparedFrame) -> Option<Self> {
        let PreparedPaint::Native(paint) = &frame.paint else {
            return None;
        };
        // A retained fragment can change behind a stable ID. Until its owner
        // supplies a revision witness, those packets require a fresh raster.
        if paint
            .commands
            .iter()
            .any(|command| matches!(command, paint_list_api::PaintCmd::PlaceRetainedFragment(_)))
        {
            return None;
        }
        Some(Self {
            paint: paint.clone(),
            host: frame.host,
            generation: frame.generation,
            extent: [frame.width, frame.height],
            scale: frame.scale,
            clear: frame.clear,
            producers: frame.native_producer_identities.clone(),
        })
    }
    fn matches(&self, frame: &PreparedFrame) -> bool {
        let PreparedPaint::Native(paint) = &frame.paint else {
            return false;
        };
        self.host == frame.host
            && self.generation == frame.generation
            && self.extent == [frame.width, frame.height]
            && self.scale == frame.scale
            && self.clear == frame.clear
            && self.producers == frame.native_producer_identities
            && self.paint.same_content(paint)
    }
}
impl<D: PartialEq, T> QualifiedRasterCache<D, T> {
    pub(crate) fn lookup(
        &self,
        device: &D,
        frame: &PreparedFrame,
        policy_allows_reuse: bool,
    ) -> Option<&T> {
        let entry = self.entry.as_ref()?;
        (policy_allows_reuse && entry.device == *device && entry.frame.matches(frame))
            .then_some(&entry.value)
    }
    pub(crate) fn insert(&mut self, device: D, frame: &PreparedFrame, value: T) {
        self.entry = FrameKey::from_frame(frame).map(|frame| Entry {
            device,
            frame,
            value,
        });
    }
    pub(crate) fn clear(&mut self) {
        self.entry = None;
    }
}

#[cfg(test)]
mod tests;
