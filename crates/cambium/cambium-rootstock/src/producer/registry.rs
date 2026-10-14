// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

use layout_dom_api::LayoutDom;
use paint_list_api::{
    CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, items::ExternalTextureItem,
};

use super::*;
use crate::{NodeId, OwnedLayout, Surface};

#[path = "portable.rs"]
mod portable;
pub use portable::PortableCaptureError;
pub(crate) use portable::PortablePlan;
use portable::PortableSlot;

// Rootstock physical producer IDs reserve the upper half of Netrender's
// lower-62-bit producer domain. The full renderer image keys are therefore
// 0xE000_0000_0000_0000..=u64::MAX. Direct, non-Rootstock external-image callers
// sharing a renderer must not register keys in this physical binding range.
// Local DOM/paint/portable keys still admit the entire lower-62-bit domain:
// every local reference is remapped, even if its number lies in this range.
const PHYSICAL_PRODUCER_BASE: u64 = 1 << 61;
const PHYSICAL_PRODUCER_END: u64 = 1 << 62;
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

/// Plan every rewrite before applying any of them. Image and pattern keys in
/// the external namespace are local here, never already-physical passthrough.
fn remap_gpu_display_list(
    translated: &mut paint_list_render::TranslatedDisplayList,
    binding: impl Fn(u64) -> Result<GpuBinding, ProducerGpuRemapError>,
) -> Result<(), ProducerGpuRemapError> {
    for &key in translated.scene.image_sources.keys() {
        if local_external_key(key).is_some() {
            return Err(ProducerGpuRemapError::ForeignInlineImage { renderer_key: key });
        }
    }
    for mask in &translated.box_shadow_masks {
        if local_external_key(mask.key).is_some() {
            return Err(ProducerGpuRemapError::ForeignInlineImage {
                renderer_key: mask.key,
            });
        }
    }
    let mut draw_rewrites = Vec::new();
    for (index, draw) in translated.external_textures.iter().enumerate() {
        if draw.texture_key >= PHYSICAL_PRODUCER_END
            || local_external_key(draw.image_key) != Some(draw.texture_key)
            || !matches!(translated.scene.ops.get(draw.scene_op_boundary),
                Some(netrender::SceneOp::Image(image)) if image.key == draw.image_key)
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
            netrender::SceneOp::Image(image) => image.key,
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
    producer: Box<dyn TextureProducer>,
    properties: Vec<String>,
    owner: Option<NodeId>,
    active: bool,
    image: Option<Image>,
    logical_size: (f32, f32),
    error: Option<ProducerError>,
    portable: Option<PortableSlot>,
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
        let renderer_key = allocate_renderer_key(&NEXT_PHYSICAL_PRODUCER)?;
        self.entries.insert(
            key,
            Entry {
                renderer_key,
                producer: Box::new(producer),
                properties: properties.iter().map(|s| (*s).to_owned()).collect(),
                owner: None,
                active: false,
                image: None,
                logical_size: (0.0, 0.0),
                error: None,
                portable: None,
            },
        );
        Ok(())
    }

    pub fn contains(&self, key: u64) -> bool {
        self.entries.contains_key(&key)
    }

    pub fn image_identity(&self, key: u64) -> Option<ProducerImageIdentity> {
        let entry = self.entries.get(&key)?;
        let image = entry.image.as_ref()?;
        Some(ProducerImageIdentity {
            renderer_key: entry.renderer_key,
            owner: entry.owner,
            generation: self.device_generation?,
            stamp: image.stamp,
            size: image.size,
        })
    }

    /// Removal is immediate for the producer; the next host synchronization
    /// unregisters its staged image before any reused key can be painted.
    pub fn remove(&mut self, key: u64) -> bool {
        let Some(mut entry) = self.entries.remove(&key) else {
            return false;
        };
        entry.producer.retire();
        self.pending_retire.push(entry.renderer_key);
        true
    }

    /// The registered producer's description of its slot, if it gives one.
    pub fn semantics(&mut self, key: u64) -> Option<ProducerSemantics> {
        self.entries.get_mut(&key)?.producer.semantics()
    }

    /// Hand a reader's invoked action to the producer that drew the node.
    /// Whether it carried it out; `false` for an unknown slot.
    pub fn act(&mut self, action: &super::ProducedAction) -> bool {
        self.entries
            .get_mut(&action.slot)
            .is_some_and(|entry| entry.producer.act(action.key, &action.id))
    }

    pub fn error(&self, key: u64) -> Option<ProducerError> {
        self.entries.get(&key)?.error
    }

    /// The first checked producer refusal from the latest preparation.
    pub fn render_failure(&self) -> Option<&ProducerRenderFailure> {
        self.render_failure.as_ref()
    }

    pub(crate) fn clear_render_failure(&mut self) {
        self.render_failure = None;
        self.gpu_remap_failure = None;
    }

    pub fn gpu_remap_failure(&self) -> Option<ProducerGpuRemapError> {
        self.gpu_remap_failure
    }

    pub(crate) fn bind_core(&mut self, core: &Arc<genet_render_host::RenderCore>) {
        if let Some(previous) = self.cleanup_core.upgrade() {
            if Arc::ptr_eq(&previous, core) {
                return;
            }
            self.retire_renderer_images(previous.renderer());
        }
        self.cleanup_core = Arc::downgrade(core);
    }

    fn retire_renderer_images(&self, renderer: &netrender::Renderer) {
        for key in self
            .entries
            .values()
            .map(|entry| entry.renderer_key)
            .chain(self.pending_retire.iter().copied())
        {
            renderer.unregister_external_image(key);
        }
    }

    pub(crate) fn remap_gpu_display_list(
        &self,
        translated: &mut paint_list_render::TranslatedDisplayList,
    ) -> Result<(), ProducerGpuRemapError> {
        remap_gpu_display_list(translated, |local_key| {
            let entry = self
                .entries
                .get(&local_key)
                .ok_or(ProducerGpuRemapError::MissingProducer { local_key })?;
            let image = entry
                .image
                .as_ref()
                .filter(|_| entry.active)
                .ok_or(ProducerGpuRemapError::UnstagedProducer { local_key })?;
            Ok(GpuBinding {
                renderer_key: entry.renderer_key,
                stamp: image.stamp,
            })
        })
    }

    pub(crate) fn record_gpu_remap_failure(&mut self, error: ProducerGpuRemapError) {
        self.gpu_remap_failure = Some(error);
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
        self.suspend_except(&|_, _| false, renderer);
        self.device = None;
        self.device_generation = None;
    }

    /// Suspend every producer but those another window owns (see
    /// [`Elsewhere`]), whose producers are theirs to keep.
    pub(crate) fn suspend_except(
        &mut self,
        elsewhere: Elsewhere<'_>,
        renderer: Option<&netrender::Renderer>,
    ) {
        for (&key, entry) in &mut self.entries {
            if elsewhere(key, entry.owner) {
                continue;
            }
            if entry.active {
                entry.producer.suspend();
            }
            entry.active = false;
            entry.image = None;
            if let Some(renderer) = renderer {
                renderer.unregister_external_image(entry.renderer_key);
            }
        }
        if let Some(renderer) = renderer {
            for key in self.pending_retire.drain(..) {
                renderer.unregister_external_image(key);
            }
        }
    }

    /// Forget the device the producers last rendered on, so the next frame
    /// takes whichever it is given.
    pub(crate) fn forget_device(&mut self) {
        self.device = None;
        self.device_generation = None;
    }

    /// Retire every producer whose owner node left the document: under
    /// several windows, those of a window that closed. Returns how many.
    pub(crate) fn retire_orphans(
        &mut self,
        in_document: impl Fn(NodeId) -> bool,
        renderer: Option<&netrender::Renderer>,
    ) -> usize {
        let orphans: Vec<u64> = self
            .entries
            .iter()
            .filter_map(|(&key, entry)| entry.owner.is_some_and(|n| !in_document(n)).then_some(key))
            .collect();
        for &key in &orphans {
            let renderer_key = self.entries[&key].renderer_key;
            self.remove(key);
            if let Some(renderer) = renderer {
                renderer.unregister_external_image(renderer_key);
            }
        }
        orphans.len()
    }

    #[cfg(test)]
    pub(crate) fn prepare<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        surface: &dyn Surface,
        layout: &OwnedLayout,
        dom: &D,
        scale: f32,
    ) -> ProducerFrameStats {
        self.prepare_window(surface, layout, dom, scale, &|_, _| false)
    }

    /// Stage this window's producers. One another window owns (see
    /// [`Elsewhere`]) is left alone rather than retired as absent, and a key
    /// this window lays out while another window's node still owns it is a
    /// duplicate, as two nodes with one key in one window are.
    pub(crate) fn prepare_window<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        surface: &dyn Surface,
        layout: &OwnedLayout,
        dom: &D,
        scale: f32,
        elsewhere: Elsewhere<'_>,
    ) -> ProducerFrameStats {
        self.render_failure = None;
        self.gpu_remap_failure = None;
        let mut stats = ProducerFrameStats::default();
        let renderer = surface.renderer();
        if self
            .device_generation
            .is_some_and(|generation| generation != surface.core().generation())
            || self
                .device
                .as_ref()
                .is_some_and(|device| device != surface.device())
        {
            self.suspend_all(None);
        }
        self.device = Some(surface.device().clone());
        self.device_generation = Some(surface.core().generation());
        for key in self.pending_retire.drain(..) {
            renderer.unregister_external_image(key);
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
                // A producer another window owns is not an absent one.
                (entry.owner.is_some() && !nodes.contains_key(&key) && !elsewhere(key, entry.owner))
                    .then_some(key)
            })
            .collect();
        for key in absent {
            let renderer_key = self.entries[&key].renderer_key;
            self.remove(key);
            renderer.unregister_external_image(renderer_key);
            stats.retirements += 1;
        }
        self.pending_retire.clear();
        for (&key, entry) in &mut self.entries {
            let Some(owner) = nodes.get(&key) else {
                continue;
            };
            let image_key = entry.renderer_key;
            let Some(node) = *owner else {
                entry.error = Some(ProducerError::DuplicateDomKey);
                suspend_entry(entry, image_key, renderer, &mut stats);
                stats.invalid_frames += 1;
                continue;
            };
            if entry
                .owner
                .is_some_and(|previous| previous != node && elsewhere(key, Some(previous)))
            {
                entry.error = Some(ProducerError::DuplicateDomKey);
                suspend_entry(entry, image_key, renderer, &mut stats);
                stats.invalid_frames += 1;
                continue;
            }
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
                timestamp: self.timestamp,
                physical_size: size,
                layout_scale: scale,
                needs_frame,
                appearance: layout.resolved_appearance(node, &entry.properties),
            };
            let phase = crate::Instant::now();
            let output = entry.producer.render_checked(&ProducerContext {
                device: surface.device(),
                queue: surface.queue(),
                frame: &frame,
                core: surface.core(),
            });
            stats.render_us += micros(phase.elapsed());
            stats.render_calls += 1;
            let output = match output {
                Ok(output) => output,
                Err(failure) => {
                    refuse_render_entry(entry);
                    renderer.unregister_external_image(image_key);
                    stats.invalid_frames += 1;
                    self.render_failure = Some(ProducerRenderFailure {
                        source_key: key,
                        failure,
                    });
                    // A refused source cannot be silently omitted/reused in a
                    // successfully presented document. Stop this host frame.
                    break;
                },
            };
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
            let Some(next_stamp) = self.next_stamp.checked_add(1) else {
                refuse_render_entry(entry);
                renderer.unregister_external_image(image_key);
                self.gpu_remap_failure =
                    Some(ProducerGpuRemapError::StampExhausted { local_key: key });
                stats.invalid_frames += 1;
                break;
            };
            self.next_stamp = next_stamp;
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
        if let Some(core) = self.cleanup_core.upgrade() {
            self.retire_renderer_images(core.renderer());
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
mod checked_refusal_tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };
    struct BoundSource {
        source: Rc<RefCell<String>>,
        suspended: Rc<Cell<usize>>,
    }
    impl TextureProducer for BoundSource {
        fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
            None
        }
        fn suspend(&mut self) {
            self.suspended.set(self.suspended.get() + 1);
        }
        fn portable_scene(
            &mut self,
            _: &ProducerFrameInfo,
        ) -> Result<ProducedScene, PortableProducerError> {
            assert_eq!(&*self.source.borrow(), "retained scene authority");
            Ok(ProducedScene {
                scene: netrender::Scene::new(12, 9),
                source_identity: 73,
                source_revision: 19,
                alpha: SourceAlpha::Straight,
                encoding: SourceEncoding::Srgb,
            })
        }
    }
    #[test]
    fn checked_refusal_invalidates_slot_without_retiring_bound_source() {
        let source = Rc::new(RefCell::new("retained scene authority".to_owned()));
        let suspended = Rc::new(Cell::new(0));
        let mut registry = ProducerRegistry::new();
        registry
            .register(
                7,
                BoundSource {
                    source: source.clone(),
                    suspended: suspended.clone(),
                },
                &[],
            )
            .unwrap();
        let entry = registry.entries.get_mut(&7).unwrap();
        entry.active = true;
        entry.logical_size = (12.0, 9.0);
        // This CPU control exercises real slot invalidation and source binding;
        // only the separate GPU gate can populate a hardware texture/view.
        refuse_render_entry(entry);
        assert!(!entry.active);
        assert!(entry.image.is_none());
        let portable = entry
            .producer
            .portable_scene(&ProducerFrameInfo {
                logical_size: (12.0, 9.0),
                physical_size: [12, 9],
                layout_scale: 1.0,
                needs_frame: true,
                timestamp: None,
                appearance: ResolvedAppearance::default(),
            })
            .unwrap();
        assert_eq!(
            (portable.source_identity, portable.source_revision),
            (73, 19)
        );
        assert_eq!(registry.error(7), Some(ProducerError::RenderRefused));
        assert!(registry.commands(7).unwrap().is_empty());
        assert!(registry.contains(7));
        assert_eq!(suspended.get(), 1);
        assert_eq!(&*source.borrow(), "retained scene authority");
    }
}

#[cfg(test)]
mod namespace_controls {
    use super::*;

    struct CpuSource;
    impl TextureProducer for CpuSource {
        fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
            None
        }
    }

    fn draw(key: u64, stamp: u64) -> PaintCmd {
        PaintCmd::DrawExternalTexture(ExternalTextureItem {
            placement: CommonPlacement::new(LayoutRect::new(
                LayoutPoint::new(0.0, 0.0),
                LayoutPoint::new(8.0, 6.0),
            )),
            texture_key: key,
            content_generation: Some(stamp),
            opacity: 0.73,
        })
    }

    fn translated(commands: &[PaintCmd]) -> paint_list_render::TranslatedDisplayList {
        paint_list_render::translate_paint_cmd_stream(
            paint_list_api::DeviceIntSize::new(8, 6),
            commands,
            &[],
            &[],
        )
    }

    fn keys(display: &paint_list_render::TranslatedDisplayList) -> Vec<u64> {
        display
            .scene
            .ops
            .iter()
            .filter_map(|op| match op {
                netrender::SceneOp::Image(image) => Some(image.key),
                netrender::SceneOp::Pattern(pattern) => Some(pattern.tile),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn physical_identity_is_distinct_stable_and_never_recycled_for_same_local_key() {
        let mut first = ProducerRegistry::new();
        let mut second = ProducerRegistry::new();
        first.register(7, CpuSource, &[]).unwrap();
        second.register(7, CpuSource, &[]).unwrap();
        let first_key = first.entries[&7].renderer_key;
        let second_key = second.entries[&7].renderer_key;
        assert_ne!(first_key, second_key);
        assert!(first_key >= (netrender::EXTERNAL_IMAGE_KEY_BASE | PHYSICAL_PRODUCER_BASE));
        first.suspend_all(None);
        assert_eq!(first.entries[&7].renderer_key, first_key);
        assert_eq!(second.entries[&7].renderer_key, second_key);
        assert!(first.remove(7));
        assert_eq!(first.pending_retire, [first_key]);
        first.register(7, CpuSource, &[]).unwrap();
        let rebound_key = first.entries[&7].renderer_key;
        assert_ne!(rebound_key, first_key);
        assert_ne!(rebound_key, second_key);
        // Dropping the first registry cannot retire or replace a registration
        // belonging to the second; actual GPU cleanup is qualified separately.
        drop(first);
        assert!(second.contains(7));
        assert_eq!(second.entries[&7].renderer_key, second_key);
    }

    #[test]
    fn physical_identity_exhaustion_is_typed_and_never_wraps() {
        let next = AtomicU64::new(PHYSICAL_PRODUCER_END - 1);
        assert_eq!(allocate_renderer_key(&next), Ok(u64::MAX));
        for _ in 0..2 {
            assert_eq!(
                allocate_renderer_key(&next),
                Err(ProducerRegistrationError::IdentityExhausted)
            );
            assert_eq!(next.load(Ordering::Relaxed), PHYSICAL_PRODUCER_END);
        }
    }

    #[test]
    fn equal_local_keys_stamps_and_extents_remap_to_distinct_physical_resources() {
        let mut first = ProducerRegistry::new();
        let mut second = ProducerRegistry::new();
        first.register(7, CpuSource, &[]).unwrap();
        second.register(7, CpuSource, &[]).unwrap();
        let mut first_display = translated(&[draw(7, 1)]);
        let mut second_display = translated(&[draw(7, 1)]);
        remap_gpu_display_list(&mut first_display, |_| {
            Ok(GpuBinding {
                renderer_key: first.entries[&7].renderer_key,
                stamp: 1,
            })
        })
        .unwrap();
        remap_gpu_display_list(&mut second_display, |_| {
            Ok(GpuBinding {
                renderer_key: second.entries[&7].renderer_key,
                stamp: 1,
            })
        })
        .unwrap();
        assert_ne!(keys(&first_display), keys(&second_display));
        assert_eq!(first_display.external_textures[0].texture_key, 7);
        assert_eq!(second_display.external_textures[0].texture_key, 7);
        assert_eq!(
            first_display.external_textures[0].content_generation,
            Some(1)
        );
        assert_eq!(
            second_display.external_textures[0].content_generation,
            Some(1)
        );
        assert_eq!(
            (
                first_display.scene.viewport_width,
                first_display.scene.viewport_height
            ),
            (
                second_display.scene.viewport_width,
                second_display.scene.viewport_height
            )
        );
    }

    #[test]
    fn authored_high_local_key_is_remapped_with_local_command_identity_preserved() {
        let local = PHYSICAL_PRODUCER_END - 1;
        let mut registry = ProducerRegistry::new();
        registry.register(local, CpuSource, &[]).unwrap();
        let physical = registry.entries[&local].renderer_key;
        let commands = [draw(local, 1)];
        let mut display = translated(&commands);
        display.scene.push_pattern(
            netrender::external_image_key(local),
            [0.0, 0.0, 8.0, 6.0],
            [1.0, 1.0],
        );
        assert_eq!(keys(&display), [u64::MAX, u64::MAX]);
        // Pure production remapping consumes copied binding metadata; this CPU
        // control does not claim an actual GPU view was staged.
        remap_gpu_display_list(&mut display, |key| {
            assert_eq!(key, local);
            Ok(GpuBinding {
                renderer_key: physical,
                stamp: 1,
            })
        })
        .unwrap();
        assert_eq!(keys(&display), [physical, physical]);
        assert_ne!(physical, u64::MAX);
        assert_eq!(display.external_textures[0].texture_key, local);
        assert_eq!(display.external_textures[0].content_generation, Some(1));
        assert_eq!(display.external_textures[0].image_key, physical);
        assert!(
            matches!(&commands[0], PaintCmd::DrawExternalTexture(item) if item.texture_key == local)
        );
        assert_eq!(display.scene.ops.len(), 2);
    }

    #[test]
    fn missing_late_binding_refuses_without_partial_scene_or_sidechannel_rewrite() {
        let mut display = translated(&[draw(7, 1), draw(8, 1)]);
        let before = keys(&display);
        let mut registry = ProducerRegistry::new();
        registry.register(7, CpuSource, &[]).unwrap();
        let physical = registry.entries[&7].renderer_key;
        let result = remap_gpu_display_list(&mut display, |key| {
            if key == 7 {
                Ok(GpuBinding {
                    renderer_key: physical,
                    stamp: 1,
                })
            } else {
                Err(ProducerGpuRemapError::MissingProducer { local_key: key })
            }
        });
        assert_eq!(
            result,
            Err(ProducerGpuRemapError::MissingProducer { local_key: 8 })
        );
        assert_eq!(keys(&display), before);
        assert_eq!(
            display
                .external_textures
                .iter()
                .map(|draw| draw.image_key)
                .collect::<Vec<_>>(),
            before
        );
        assert!(registry.pending_retire.is_empty());
        assert_eq!(registry.next_stamp, 0);
    }

    #[test]
    fn actual_registry_refuses_unstaged_and_unknown_local_sources() {
        let mut registry = ProducerRegistry::new();
        registry.register(7, CpuSource, &[]).unwrap();
        let mut known = translated(&[draw(7, 1)]);
        assert_eq!(
            registry.remap_gpu_display_list(&mut known),
            Err(ProducerGpuRemapError::UnstagedProducer { local_key: 7 })
        );
        let mut missing = translated(&[draw(8, 1)]);
        assert_eq!(
            registry.remap_gpu_display_list(&mut missing),
            Err(ProducerGpuRemapError::MissingProducer { local_key: 8 })
        );
        assert_eq!(keys(&known), [netrender::external_image_key(7)]);
        assert_eq!(keys(&missing), [netrender::external_image_key(8)]);
    }

    #[test]
    fn stale_generation_and_foreign_inline_payload_are_typed_refusals() {
        let physical = allocate_renderer_key(&AtomicU64::new(PHYSICAL_PRODUCER_BASE)).unwrap();
        let mut stale = translated(&[draw(7, 3)]);
        assert_eq!(
            remap_gpu_display_list(&mut stale, |_| Ok(GpuBinding {
                renderer_key: physical,
                stamp: 4
            })),
            Err(ProducerGpuRemapError::StaleGeneration {
                local_key: 7,
                expected: 4,
                actual: Some(3)
            })
        );
        assert_eq!(keys(&stale), [netrender::external_image_key(7)]);
        let mut foreign = translated(&[]);
        foreign.scene.image_sources.insert(
            physical,
            netrender::ImageData::from_bytes(1, 1, vec![0, 0, 0, 0]),
        );
        assert_eq!(
            remap_gpu_display_list(&mut foreign, |_| unreachable!(
                "reserved inline data must refuse before lookup"
            )),
            Err(ProducerGpuRemapError::ForeignInlineImage {
                renderer_key: physical
            })
        );
    }
}
