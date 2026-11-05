// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

use layout_dom_api::LayoutDom;
use paint_list_api::{
    CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, items::ExternalTextureItem,
};

use super::*;
use crate::{NodeId, OwnedLayout, Surface};

#[cfg(test)]
#[path = "shadow_namespace_tests.rs"]
mod shadow_namespace_tests;

#[path = "portable.rs"]
mod portable;
pub use portable::PortableCaptureError;
pub(crate) use portable::PortablePlan;
use portable::PortableSlot;

#[path = "legacy.rs"]
mod legacy;
#[cfg(not(target_arch = "wasm32"))]
#[path = "native.rs"]
mod native;

// Rootstock physical producer IDs use the upper half of Netrender's
// lower-62-bit producer domain, below its generated shadow-mask range.
// Full renderer keys are 0xE000_0000_0000_0000..0xFFFF_0000_0000_0000.
// Direct external-image callers sharing a renderer must not register keys
// in this physical binding range. Preserve the translator's mask keys: moving
// them into the inline domain could leave a retained renderer override that
// captures an unrelated inline image in a later frame.
// Local DOM/paint/portable keys still admit the entire lower-62-bit domain:
// every local reference is remapped, even if its number lies in this range.
const LOCAL_PRODUCER_END: u64 = 1 << 62;
// Matches the translator's generated mask band; real-translator tests guard
// this supplier contract rather than inferring mask ownership from key alone.
const SHADOW_MASK_IMAGE_KEY_BASE: u64 = 0xFFFF_0000_0000_0000;
const PHYSICAL_PRODUCER_BASE: u64 = 1 << 61;
const PHYSICAL_PRODUCER_END: u64 = SHADOW_MASK_IMAGE_KEY_BASE - netrender::EXTERNAL_IMAGE_KEY_BASE;
static NEXT_PHYSICAL_PRODUCER: AtomicU64 = AtomicU64::new(PHYSICAL_PRODUCER_BASE);

fn allocate_renderer_key(next: &AtomicU64) -> Result<u64, ProducerRegistrationError> {
    next.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |key| {
        (PHYSICAL_PRODUCER_BASE..PHYSICAL_PRODUCER_END)
            .contains(&key)
            .then(|| key + 1)
    })
    .map(netrender::external_image_key)
    .map_err(|_| ProducerRegistrationError::IdentityExhausted)
}

/// A complete local producer reference could not be bound for this GPU frame.
/// Validation runs before rewriting the translated scene or touching a GPU.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerGpuRemapError {
    StampExhausted {
        local_key: u64,
    },
    MissingProducer {
        local_key: u64,
    },
    UnstagedProducer {
        local_key: u64,
    },
    StaleGeneration {
        local_key: u64,
        expected: u64,
        actual: Option<u64>,
    },
    InconsistentDrawBinding {
        local_key: u64,
        scene_op: usize,
    },
    ForeignInlineImage {
        renderer_key: u64,
    },
    InconsistentShadowMask {
        renderer_key: u64,
    },
    UnsupportedRetainedFragment {
        fragment_id: u64,
    },
}
impl std::fmt::Display for ProducerGpuRemapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "producer GPU binding refused: {self:?}")
    }
}
impl std::error::Error for ProducerGpuRemapError {}

#[derive(Clone, Copy)]
struct GpuBinding {
    renderer_key: u64,
    stamp: u64,
}

fn local_external_key(key: u64) -> Option<u64> {
    (key >= netrender::EXTERNAL_IMAGE_KEY_BASE).then(|| key - netrender::EXTERNAL_IMAGE_KEY_BASE)
}

/// Plan every rewrite before applying any of them. External image and pattern
/// references are local here, never already-physical passthrough. A validated
/// translator mask is identified by its ownership metadata, except at a
/// recorded producer image boundary where the same numeric key can be local.
fn remap_gpu_display_list(
    translated: &mut paint_list_render::TranslatedDisplayList,
    binding: impl Fn(u64) -> Result<GpuBinding, ProducerGpuRemapError>,
) -> Result<(), ProducerGpuRemapError> {
    let mut mask_keys = BTreeSet::new();
    for mask in &translated.box_shadow_masks {
        if mask.key < SHADOW_MASK_IMAGE_KEY_BASE
            || !mask_keys.insert(mask.key)
            || translated.scene.image_sources.contains_key(&mask.key)
        {
            return Err(ProducerGpuRemapError::InconsistentShadowMask {
                renderer_key: mask.key,
            });
        }
    }
    for &key in translated.scene.image_sources.keys() {
        if local_external_key(key).is_some() {
            return Err(ProducerGpuRemapError::ForeignInlineImage { renderer_key: key });
        }
    }
    let mut producer_boundaries = BTreeSet::new();
    let mut draw_rewrites = Vec::new();
    for (index, draw) in translated.external_textures.iter().enumerate() {
        if draw.texture_key >= LOCAL_PRODUCER_END
            || local_external_key(draw.image_key) != Some(draw.texture_key)
            || !matches!(translated.scene.ops.get(draw.scene_op_boundary),
                Some(netrender::SceneOp::Image(image)) if image.key == draw.image_key)
            || !producer_boundaries.insert(draw.scene_op_boundary)
        {
            return Err(ProducerGpuRemapError::InconsistentDrawBinding {
                local_key: draw.texture_key,
                scene_op: draw.scene_op_boundary,
            });
        }
        let mapped = binding(draw.texture_key)?;
        if draw.content_generation != Some(mapped.stamp) {
            return Err(ProducerGpuRemapError::StaleGeneration {
                local_key: draw.texture_key,
                expected: mapped.stamp,
                actual: draw.content_generation,
            });
        }
        draw_rewrites.push((index, mapped.renderer_key));
    }
    let mut op_rewrites = Vec::new();
    for (index, op) in translated.scene.ops.iter().enumerate() {
        let key = match op {
            netrender::SceneOp::Image(image) => {
                if mask_keys.contains(&image.key) && !producer_boundaries.contains(&index) {
                    continue;
                }
                image.key
            },
            netrender::SceneOp::Pattern(pattern) => pattern.tile,
            // Rootstock emits its custom leaves as complete ordinary commands.
            // Opaque retained fragments cannot be inspected for local external
            // references, so this boundary must not silently pass them through.
            netrender::SceneOp::Fragment(fragment) => {
                return Err(ProducerGpuRemapError::UnsupportedRetainedFragment {
                    fragment_id: fragment.id,
                });
            },
            _ => continue,
        };
        if let Some(local_key) = local_external_key(key) {
            op_rewrites.push((index, binding(local_key)?.renderer_key));
        }
    }
    for (index, key) in op_rewrites {
        match &mut translated.scene.ops[index] {
            netrender::SceneOp::Image(image) => image.key = key,
            netrender::SceneOp::Pattern(pattern) => pattern.tile = key,
            _ => {},
        }
    }
    for (index, key) in draw_rewrites {
        translated.external_textures[index].image_key = key;
    }
    Ok(())
}

#[derive(Clone)]
struct Image {
    output: ProducedTexture,
    size: [u32; 2],
    stamp: u64,
}

/// Copied identity of a currently staged producer image. It retains no GPU
/// resource and permits presentation observers to attribute rehydrated views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProducerImageIdentity {
    /// Full physical renderer image key. Sample/unregister this key directly;
    /// do not apply external_image_key to it again. Stable across device loss.
    pub renderer_key: u64,
    pub owner: Option<NodeId>,
    pub generation: u64,
    pub stamp: u64,
    pub size: [u32; 2],
}

struct Entry {
    renderer_key: u64,
    // Logical registration life, independent of image/device generations.
    action_lifetime: Arc<()>,
    producer: Box<dyn TextureProducer>,
    properties: Vec<String>,
    owner: Option<NodeId>,
    active: bool,
    image: Option<Image>,
    logical_size: (f32, f32),
    error: Option<ProducerError>,
    portable: Option<PortableSlot>,
    force_frame: bool,
    #[cfg(not(target_arch = "wasm32"))]
    accepted_semantics: Option<native::SemanticSnapshot>,
}

/// Registrations survive CSS hiding; removing a previously bound DOM node
/// retires its registration. Register again when recreating an absent node.
/// Whether another window owns the producer with this key and owner node: a
/// key another window last laid out, or an owner node that now lives under
/// another window's root (moved there since that window last laid out).
/// Always false for a single window.
pub(crate) type Elsewhere<'a> = &'a dyn Fn(u64, Option<NodeId>) -> bool;

#[derive(Default)]
pub struct ProducerRegistry {
    entries: BTreeMap<u64, Entry>,
    pending_retire: Vec<u64>,
    device: Option<wgpu::Device>,
    device_generation: Option<u64>,
    next_stamp: u64,
    render_failure: Option<ProducerRenderFailure>,
    gpu_remap_failure: Option<ProducerGpuRemapError>,
    // Cleanup may retire this registry's keys when other hosts retain the
    // renderer. A weak handle never keeps the lost device/core alive.
    cleanup_core: Weak<genet_render_host::RenderCore>,
    pub(super) timestamp: Option<std::time::Duration>,
    #[cfg(not(target_arch = "wasm32"))]
    staged: Option<native::StagedPreparation>,
}

impl Drop for ProducerRegistry {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.discard_staged(None);
        if let Some(core) = self.cleanup_core.upgrade() {
            self.retire_renderer_images(&core);
        }
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
    core: &genet_render_host::RenderCore,
    stats: &mut ProducerFrameStats,
) {
    if entry.active {
        entry.producer.suspend();
        stats.suspensions += 1;
    }
    entry.active = false;
    entry.image = None;
    crate::retire_external_image(core, key);
}

// The same invalidation path is exercised by CPU registry controls and
// actual GPU staging. Source bindings and portable bytes remain authoritative.
fn refuse_render_entry(entry: &mut Entry) {
    entry.error = Some(ProducerError::RenderRefused);
    entry.image = None;
    entry.active = false;
    entry.producer.suspend();
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

#[cfg(test)]
#[path = "registry/checked_refusal_tests.rs"]
mod checked_refusal_tests;

#[cfg(test)]
#[path = "registry/namespace_controls.rs"]
mod namespace_controls;

#[path = "registry/operations.rs"]
mod operations;
