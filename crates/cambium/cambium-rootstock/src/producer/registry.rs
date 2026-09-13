// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::{BTreeMap, HashMap};

use layout_dom_api::LayoutDom;
use paint_list_api::{
    CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, items::ExternalTextureItem,
};

use super::*;
use crate::{NodeId, OwnedLayout, Surface};

struct Image {
    output: ProducedTexture,
    size: [u32; 2],
    stamp: u64,
}

struct Entry {
    producer: Box<dyn TextureProducer>,
    properties: Vec<String>,
    owner: Option<NodeId>,
    active: bool,
    image: Option<Image>,
    logical_size: (f32, f32),
    error: Option<ProducerError>,
}

/// Registrations survive CSS hiding; removing a previously bound DOM node
/// retires its registration. Register again when recreating an absent node.
#[derive(Default)]
pub struct ProducerRegistry {
    entries: BTreeMap<u64, Entry>,
    pending_retire: Vec<u64>,
    device: Option<wgpu::Device>,
    next_stamp: u64,
}

impl ProducerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        key: u64,
        producer: impl TextureProducer + 'static,
        properties: &[&str],
    ) -> Result<(), ProducerRegistrationError> {
        if key >= 1_u64 << 62 {
            return Err(ProducerRegistrationError::KeyOutOfRange);
        }
        if self.entries.contains_key(&key) {
            return Err(ProducerRegistrationError::AlreadyRegistered);
        }
        self.entries.insert(
            key,
            Entry {
                producer: Box::new(producer),
                properties: properties.iter().map(|s| (*s).to_owned()).collect(),
                owner: None,
                active: false,
                image: None,
                logical_size: (0.0, 0.0),
                error: None,
            },
        );
        Ok(())
    }

    pub fn contains(&self, key: u64) -> bool {
        self.entries.contains_key(&key)
    }

    /// Removal is immediate for the producer; the next host synchronization
    /// unregisters its staged image before any reused key can be painted.
    pub fn remove(&mut self, key: u64) -> bool {
        let Some(mut entry) = self.entries.remove(&key) else {
            return false;
        };
        entry.producer.retire();
        self.pending_retire.push(key);
        true
    }

    pub fn error(&self, key: u64) -> Option<ProducerError> {
        self.entries.get(&key)?.error
    }

    pub(crate) fn commands(&self, key: u64) -> Option<Vec<PaintCmd>> {
        let entry = self.entries.get(&key)?;
        let Some(image) = &entry.image else {
            return Some(Vec::new());
        };
        Some(vec![PaintCmd::DrawExternalTexture(ExternalTextureItem {
            placement: CommonPlacement::new(LayoutRect::new(
                LayoutPoint::new(0.0, 0.0),
                LayoutPoint::new(entry.logical_size.0, entry.logical_size.1),
            )),
            texture_key: key,
            opacity: 1.0,
            content_generation: Some(image.stamp),
        })])
    }

    pub(crate) fn suspend_all(&mut self, renderer: Option<&netrender::Renderer>) {
        for (&key, entry) in &mut self.entries {
            if entry.active {
                entry.producer.suspend();
            }
            entry.active = false;
            entry.image = None;
            if let Some(renderer) = renderer {
                renderer.unregister_external_image(netrender::external_image_key(key));
            }
        }
        if let Some(renderer) = renderer {
            for key in self.pending_retire.drain(..) {
                renderer.unregister_external_image(netrender::external_image_key(key));
            }
        }
        self.device = None;
    }

    pub(crate) fn prepare<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        surface: &dyn Surface,
        layout: &OwnedLayout,
        dom: &D,
        scale: f32,
    ) -> ProducerFrameStats {
        let mut stats = ProducerFrameStats::default();
        let renderer = surface.renderer();
        if self
            .device
            .as_ref()
            .is_some_and(|device| device != surface.device())
        {
            self.suspend_all(None);
        }
        self.device = Some(surface.device().clone());
        for key in self.pending_retire.drain(..) {
            renderer.unregister_external_image(netrender::external_image_key(key));
            stats.retirements += 1;
        }
        if self.entries.is_empty() {
            return stats;
        }
        let mut nodes = HashMap::new();
        for (key, node) in layout.custom_leaf_nodes(dom) {
            nodes
                .entry(key)
                .and_modify(|owner| *owner = None)
                .or_insert(Some(node));
        }
        let absent: Vec<_> = self
            .entries
            .iter()
            .filter_map(|(&key, entry)| {
                (entry.owner.is_some() && !nodes.contains_key(&key)).then_some(key)
            })
            .collect();
        for key in absent {
            self.remove(key);
            renderer.unregister_external_image(netrender::external_image_key(key));
            stats.retirements += 1;
        }
        self.pending_retire.clear();
        for (&key, entry) in &mut self.entries {
            let Some(owner) = nodes.get(&key) else {
                continue;
            };
            let image_key = netrender::external_image_key(key);
            let Some(node) = *owner else {
                entry.error = Some(ProducerError::DuplicateDomKey);
                suspend_entry(entry, image_key, renderer, &mut stats);
                stats.invalid_frames += 1;
                continue;
            };
            if entry.owner.is_some_and(|previous| previous != node) {
                entry.producer.retire();
                entry.active = false;
                entry.image = None;
                renderer.unregister_external_image(image_key);
                stats.retirements += 1;
            }
            entry.owner = Some(node);
            let Some(geometry) = layout.element_geometry(dom, node) else {
                suspend_entry(entry, image_key, renderer, &mut stats);
                continue;
            };
            let logical_size = geometry.content_size();
            if logical_size.0 == 0.0 || logical_size.1 == 0.0 {
                suspend_entry(entry, image_key, renderer, &mut stats);
                continue;
            }
            let Some(size) = physical_size(
                logical_size,
                scale,
                surface.device().limits().max_texture_dimension_2d,
            ) else {
                entry.error = Some(ProducerError::InvalidExtent);
                suspend_entry(entry, image_key, renderer, &mut stats);
                stats.invalid_frames += 1;
                continue;
            };
            let needs_frame =
                !entry.active || entry.image.as_ref().is_none_or(|image| image.size != size);
            if needs_frame {
                entry.image = None;
                renderer.unregister_external_image(image_key);
            }
            entry.active = true;
            entry.logical_size = logical_size;
            let frame = ProducerFrameInfo {
                logical_size,
                physical_size: size,
                layout_scale: scale,
                needs_frame,
                appearance: layout.resolved_appearance(node, &entry.properties),
            };
            let phase = crate::Instant::now();
            let output = entry.producer.render(&ProducerContext {
                device: surface.device(),
                queue: surface.queue(),
                frame: &frame,
            });
            stats.render_us += micros(phase.elapsed());
            stats.render_calls += 1;
            let Some(output) = output else { continue };
            if let Err(error) = validate_output(&output, size) {
                entry.error = Some(error);
                entry.image = None;
                renderer.unregister_external_image(image_key);
                stats.invalid_frames += 1;
                continue;
            }
            entry.error = None;
            if entry.image.as_ref().is_some_and(|image| {
                image.size == size
                    && image.output.generation == output.generation
                    && image.output.view == output.view
                    && image.output.alpha == output.alpha
                    && image.output.encoding == output.encoding
            }) {
                continue;
            }
            self.next_stamp = self
                .next_stamp
                .checked_add(1)
                .expect("producer image generation exhausted");
            let phase = crate::Instant::now();
            renderer.stage_external_image(
                image_key,
                &output.view,
                size,
                output.alpha,
                self.next_stamp,
            );
            stats.stage_us += micros(phase.elapsed());
            stats.stages += 1;
            entry.image = Some(Image {
                output,
                size,
                stamp: self.next_stamp,
            });
        }
        stats
    }
}

impl Drop for ProducerRegistry {
    fn drop(&mut self) {
        for entry in self.entries.values_mut() {
            entry.producer.retire();
        }
    }
}

fn micros(duration: crate::Duration) -> u64 {
    duration.as_micros().min(u64::MAX as u128) as u64
}

fn suspend_entry(
    entry: &mut Entry,
    key: u64,
    renderer: &netrender::Renderer,
    stats: &mut ProducerFrameStats,
) {
    if entry.active {
        entry.producer.suspend();
        stats.suspensions += 1;
    }
    entry.active = false;
    entry.image = None;
    renderer.unregister_external_image(key);
}

pub(super) fn physical_size(logical: (f32, f32), scale: f32, limit: u32) -> Option<[u32; 2]> {
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let dims = [
        f64::from(logical.0) * f64::from(scale),
        f64::from(logical.1) * f64::from(scale),
    ];
    dims.iter()
        .all(|v| v.is_finite() && *v > 0.0 && v.ceil() <= f64::from(limit))
        .then(|| dims.map(|v| v.ceil() as u32))
}

fn validate_output(output: &ProducedTexture, size: [u32; 2]) -> Result<(), ProducerError> {
    if output.encoding != SourceEncoding::Srgb {
        return Err(ProducerError::UnsupportedEncoding);
    }
    let texture = output.view.texture();
    let extent = texture.size();
    if [extent.width, extent.height] != size
        || extent.depth_or_array_layers != 1
        || texture.sample_count() != 1
        || texture.dimension() != wgpu::TextureDimension::D2
        || !texture
            .usage()
            .contains(wgpu::TextureUsages::TEXTURE_BINDING)
        || !matches!(
            texture.format(),
            wgpu::TextureFormat::Rgba8Unorm
                | wgpu::TextureFormat::Rgba8UnormSrgb
                | wgpu::TextureFormat::Bgra8Unorm
                | wgpu::TextureFormat::Bgra8UnormSrgb
        )
    {
        return Err(ProducerError::InvalidTexture);
    }
    Ok(())
}
