// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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
        if key >= LOCAL_PRODUCER_END {
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
                action_lifetime: Arc::new(()),
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

    pub(crate) fn action_lifetime(&self, key: u64) -> Option<Weak<()>> {
        Some(Arc::downgrade(&self.entries.get(&key)?.action_lifetime))
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
    pub fn semantics(&self, key: u64) -> Option<ProducerSemantics> {
        self.entries.get(&key)?.producer.semantics()
    }

    pub(crate) fn child_lifetime(
        &self,
        slot: u64,
        child: u64,
    ) -> Option<super::super::ProducerChildToken> {
        self.entries.get(&slot)?.producer.child_lifetime(child)
    }

    /// Hand a reader's invoked action to the producer that drew the node.
    /// Whether it carried it out; `false` for an unknown slot.
    pub fn act(&mut self, action: &super::super::ProducedAction) -> bool {
        self.entries
            .get_mut(&action.slot)
            .is_some_and(|entry| entry.producer.act(action.key, &action.id))
    }

    /// The host uses this after semantic admission, preserving the captured
    /// child token instead of acquiring a new life for an old request.
    pub fn act_checked(
        &mut self,
        action: &super::super::ProducedAction,
        expected: Option<&super::super::ProducerChildToken>,
    ) -> super::super::ProducerActionDispatch {
        self.entries
            .get_mut(&action.slot)
            .map_or(super::super::ProducerActionDispatch::StaleTarget, |entry| {
                entry.producer.act_checked(action.key, &action.id, expected)
            })
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
            self.retire_renderer_images(&previous);
        }
        self.cleanup_core = Arc::downgrade(core);
    }

    pub(super) fn retire_renderer_images(&self, core: &genet_render_host::RenderCore) {
        for key in self
            .entries
            .values()
            .map(|entry| entry.renderer_key)
            .chain(self.pending_retire.iter().copied())
        {
            crate::retire_external_image(core, key);
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
            let needs_frame =
                !entry.active || entry.image.as_ref().is_none_or(|image| image.size != size);
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
        }
        stats
    }
}
