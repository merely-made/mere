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
                force_frame: false,
                #[cfg(not(target_arch = "wasm32"))]
                accepted_semantics: None,
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

    /// Conservative complete snapshot after current producer binding succeeds.
    /// Sorting removes HashMap iteration order from final-output retention.
    pub(crate) fn image_identities(&self) -> Vec<ProducerImageIdentity> {
        let mut identities: Vec<_> = self
            .entries
            .keys()
            .filter_map(|key| self.image_identity(*key))
            .collect();
        identities.sort_unstable_by_key(|identity| identity.renderer_key);
        identities
    }

    /// Removal is immediate for the producer; the next host synchronization
    /// unregisters its staged image before any reused key can be painted.
    pub fn remove(&mut self, key: u64) -> bool {
        let Some(mut entry) = self.entries.remove(&key) else {
            return false;
        };
        entry.producer.discard_staged();
        entry.producer.retire();
        self.pending_retire.push(entry.renderer_key);
        true
    }

    /// The registered producer's description of its slot, if it gives one.
    pub fn semantics(&self, key: u64) -> Option<ProducerSemantics> {
        let entry = self.entries.get(&key)?;
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(accepted) = &entry.accepted_semantics {
            return accepted.semantics.clone();
        }
        entry.producer.semantics()
    }

    pub(crate) fn child_lifetime(
        &self,
        slot: u64,
        child: u64,
    ) -> Option<super::super::ProducerChildToken> {
        let entry = self.entries.get(&slot)?;
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(accepted) = &entry.accepted_semantics {
            return accepted.children.get(&child).cloned().flatten();
        }
        entry.producer.child_lifetime(child)
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
            #[cfg(not(target_arch = "wasm32"))]
            self.discard_staged(Some(&previous));
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
            let (renderer_key, active, image, _) = self
                .preparation_entry(local_key)
                .ok_or(ProducerGpuRemapError::MissingProducer { local_key })?;
            let image = image
                .filter(|_| active)
                .ok_or(ProducerGpuRemapError::UnstagedProducer { local_key })?;
            Ok(GpuBinding {
                renderer_key,
                stamp: image.stamp,
            })
        })
    }

    pub(crate) fn record_gpu_remap_failure(&mut self, error: ProducerGpuRemapError) {
        self.gpu_remap_failure = Some(error);
    }

    pub(crate) fn commands(&self, key: u64) -> Option<Vec<PaintCmd>> {
        let (_, _, image, logical_size) = self.preparation_entry(key)?;
        let Some(image) = image else {
            return Some(Vec::new());
        };
        Some(vec![PaintCmd::DrawExternalTexture(ExternalTextureItem {
            placement: CommonPlacement::new(LayoutRect::new(
                LayoutPoint::new(0.0, 0.0),
                LayoutPoint::new(logical_size.0, logical_size.1),
            )),
            texture_key: key,
            opacity: 1.0,
            content_generation: Some(image.stamp),
        })])
    }

    fn preparation_entry(&self, key: u64) -> Option<(u64, bool, Option<&Image>, (f32, f32))> {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(staged) = &self.staged {
            return staged
                .entries
                .get(&key)
                .filter(|entry| !entry.retire)
                .map(|entry| {
                    (
                        entry.renderer_key,
                        entry.active,
                        entry.image.as_ref(),
                        entry.logical_size,
                    )
                });
        }
        self.entries.get(&key).map(|entry| {
            (
                entry.renderer_key,
                entry.active,
                entry.image.as_ref(),
                entry.logical_size,
            )
        })
    }
}
