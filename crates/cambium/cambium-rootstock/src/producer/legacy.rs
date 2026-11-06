// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Existing immediate producer preparation and lifecycle.

use super::*;

impl ProducerRegistry {
    pub(crate) fn suspend_all(&mut self, core: Option<&genet_render_host::RenderCore>) {
        self.suspend_except(&|_, _| false, core);
        self.device = None;
        self.device_generation = None;
    }

    /// Suspend every producer but those another window owns (see
    /// [`Elsewhere`]), whose producers are theirs to keep.
    pub(crate) fn suspend_except(
        &mut self,
        elsewhere: Elsewhere<'_>,
        core: Option<&genet_render_host::RenderCore>,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        self.discard_staged(core);
        for (&key, entry) in &mut self.entries {
            let native_here = !elsewhere(key, entry.owner);
            let portable_here = entry
                .portable
                .as_ref()
                .is_some_and(|slot| !elsewhere(key, Some(slot.owner)));
            let portable_active =
                portable_here && entry.portable.as_ref().is_some_and(|slot| slot.active);
            if (native_here && entry.active) || portable_active {
                entry.producer.suspend();
            }
            if portable_here && let Some(slot) = &mut entry.portable {
                slot.active = false;
            }
            if native_here {
                entry.active = false;
                entry.image = None;
                if let Some(core) = core {
                    crate::retire_external_image(core, entry.renderer_key);
                }
            }
        }
        if let Some(core) = core {
            for key in self.pending_retire.drain(..) {
                crate::retire_external_image(core, key);
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
        core: Option<&genet_render_host::RenderCore>,
    ) -> usize {
        let orphans: Vec<u64> = self
            .entries
            .iter()
            .filter_map(|(&key, entry)| {
                entry
                    .owner
                    .or_else(|| entry.portable.as_ref().map(|slot| slot.owner))
                    .is_some_and(|n| !in_document(n))
                    .then_some(key)
            })
            .collect();
        for &key in &orphans {
            let renderer_key = self.entries[&key].renderer_key;
            self.remove(key);
            if let Some(core) = core {
                crate::retire_external_image(core, renderer_key);
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
        // Producer callbacks below rasterize on this core, so renderer access
        // is taken per operation rather than held across the window.
        let core = surface.core();
        #[cfg(not(target_arch = "wasm32"))]
        self.discard_staged(Some(core));
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
            crate::retire_external_image(core, key);
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
            crate::retire_external_image(core, renderer_key);
            stats.retirements += 1;
        }
        self.pending_retire.clear();
        for (&key, entry) in &mut self.entries {
            let Some(owner) = nodes.get(&key) else {
                continue;
            };
            #[cfg(not(target_arch = "wasm32"))]
            {
                entry.accepted_semantics = None;
            }
            let image_key = entry.renderer_key;
            let Some(node) = *owner else {
                entry.error = Some(ProducerError::DuplicateDomKey);
                suspend_entry(entry, image_key, core, &mut stats);
                stats.invalid_frames += 1;
                continue;
            };
            if entry
                .owner
                .is_some_and(|previous| previous != node && elsewhere(key, Some(previous)))
            {
                entry.error = Some(ProducerError::DuplicateDomKey);
                suspend_entry(entry, image_key, core, &mut stats);
                stats.invalid_frames += 1;
                continue;
            }
            if entry.owner.is_some_and(|previous| previous != node) {
                entry.producer.retire();
                entry.active = false;
                entry.image = None;
                crate::retire_external_image(core, image_key);
                stats.retirements += 1;
            }
            entry.owner = Some(node);
            let Some(geometry) = layout.element_geometry(dom, node) else {
                suspend_entry(entry, image_key, core, &mut stats);
                continue;
            };
            let logical_size = geometry.content_size();
            if logical_size.0 == 0.0 || logical_size.1 == 0.0 {
                suspend_entry(entry, image_key, core, &mut stats);
                continue;
            }
            let Some(size) = physical_size(
                logical_size,
                scale,
                surface.device().limits().max_texture_dimension_2d,
            ) else {
                entry.error = Some(ProducerError::InvalidExtent);
                suspend_entry(entry, image_key, core, &mut stats);
                stats.invalid_frames += 1;
                continue;
            };
            let needs_frame = entry.force_frame
                || !entry.active
                || entry.image.as_ref().is_none_or(|image| image.size != size);
            if needs_frame {
                entry.image = None;
                crate::retire_external_image(core, image_key);
            }
            entry.active = true;
            entry.logical_size = logical_size;
            let frame = ProducerFrameInfo {
                purpose: ProducerFramePurpose::Presentation,
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
                #[cfg(not(target_arch = "wasm32"))]
                native_submission: None,
            });
            stats.render_us += micros(phase.elapsed());
            stats.render_calls += 1;
            let output = match output {
                Ok(output) => output,
                Err(failure) => {
                    refuse_render_entry(entry);
                    crate::retire_external_image(core, image_key);
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
                crate::retire_external_image(core, image_key);
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
                crate::retire_external_image(core, image_key);
                self.gpu_remap_failure =
                    Some(ProducerGpuRemapError::StampExhausted { local_key: key });
                stats.invalid_frames += 1;
                break;
            };
            self.next_stamp = next_stamp;
            let phase = crate::Instant::now();
            let Ok(renderer) = crate::try_renderer(core) else {
                // Loss or another frame owner: the frame itself is refused,
                // so this producer keeps no image staged on the old access.
                refuse_render_entry(entry);
                crate::retire_external_image(core, image_key);
                stats.invalid_frames += 1;
                continue;
            };
            renderer.stage_external_image(
                image_key,
                &output.view,
                size,
                output.alpha,
                self.next_stamp,
            );
            drop(renderer);
            stats.stage_us += micros(phase.elapsed());
            stats.stages += 1;
            entry.image = Some(Image {
                output,
                size,
                stamp: self.next_stamp,
            });
            entry.force_frame = false;
        }
        stats
    }
}
