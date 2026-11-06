// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Candidate producer images and facts share the host's one native fate.

use super::*;

#[derive(Clone, Default)]
pub(super) struct SemanticSnapshot {
    pub(super) semantics: Option<ProducerSemantics>,
    pub(super) children: BTreeMap<u64, Option<ProducerChildToken>>,
}
impl SemanticSnapshot {
    fn capture(producer: &dyn TextureProducer) -> Self {
        let semantics = producer.semantics();
        let children = semantics
            .iter()
            .flat_map(|s| &s.children)
            .map(|child| (child.key, producer.child_lifetime(child.key)))
            .collect();
        Self {
            semantics,
            children,
        }
    }
    fn is_live(&self) -> bool {
        self.children
            .values()
            .flatten()
            .all(ProducerChildToken::is_live)
    }
}

pub(super) struct Candidate {
    pub(super) renderer_key: u64,
    registration: Weak<()>,
    owner: Option<NodeId>,
    pub(super) active: bool,
    pub(super) image: Option<Image>,
    pub(super) logical_size: (f32, f32),
    error: Option<ProducerError>,
    semantics: SemanticSnapshot,
    pub(super) retire: bool,
    touched: bool,
    invoked: bool,
}
impl Candidate {
    fn capture(entry: &Entry) -> Self {
        let semantics = entry.accepted_semantics.clone().unwrap_or_default();
        Self {
            renderer_key: entry.renderer_key,
            registration: Arc::downgrade(&entry.action_lifetime),
            owner: entry.owner,
            active: entry.active,
            image: entry.image.clone(),
            logical_size: entry.logical_size,
            error: entry.error,
            semantics,
            retire: false,
            touched: false,
            invoked: false,
        }
    }
    fn belongs_to(&self, entry: &Entry) -> bool {
        self.renderer_key == entry.renderer_key
            && self.registration.upgrade().is_some()
            && self
                .registration
                .ptr_eq(&Arc::downgrade(&entry.action_lifetime))
    }
}

pub(super) struct StagedPreparation {
    pub(super) entries: BTreeMap<u64, Candidate>,
    device: wgpu::Device,
    generation: u64,
    valid: bool,
}

impl ProducerRegistry {
    /// Conservative admission: every registration must explicitly participate.
    pub(crate) fn can_stage_native(&self) -> bool {
        self.entries
            .values()
            .all(|entry| entry.producer.supports_native_staging())
    }

    /// Prepare candidates inside an already admitted native batch. Accepted
    /// image/semantic metadata remains separate until publish_staged.
    pub(crate) fn prepare_window_staged<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        surface: &dyn Surface,
        layout: &OwnedLayout,
        dom: &D,
        scale: f32,
        elsewhere: Elsewhere<'_>,
        submission: &genet_render_host::NativeFrameSubmission,
    ) -> ProducerFrameStats {
        self.clear_render_failure();
        let mut stats = ProducerFrameStats::default();
        if self.staged.is_some() || !self.can_stage_native() {
            self.render_failure = Some(ProducerRenderFailure {
                source_key: 0,
                failure: genet_render_host::RasterizeFailure::SourceError(
                    "native producer preparation is pending or unsupported".into(),
                ),
            });
            stats.invalid_frames = 1;
            return stats;
        }
        let core = surface.core();
        for key in self.pending_retire.drain(..) {
            crate::retire_external_image(core, key);
            stats.retirements += 1;
        }
        let same_device = self.device.as_ref() == Some(surface.device())
            && self.device_generation == Some(core.generation());
        let mut staged = StagedPreparation {
            entries: self
                .entries
                .iter_mut()
                .map(|(&key, entry)| (key, Candidate::capture(entry)))
                .collect(),
            device: surface.device().clone(),
            generation: core.generation(),
            valid: true,
        };
        if !same_device {
            for candidate in staged.entries.values_mut() {
                if candidate.image.is_some() {
                    candidate.touched = true;
                    candidate.active = false;
                    candidate.image = None;
                    candidate.semantics = Default::default();
                }
            }
        }
        let mut nodes = HashMap::new();
        for (key, node) in layout.custom_leaf_nodes(dom) {
            nodes
                .entry(key)
                .and_modify(|owner| *owner = None)
                .or_insert(Some(node));
        }
        for (&key, candidate) in &mut staged.entries {
            let entry = self.entries.get_mut(&key).expect("captured registration");
            if !nodes.contains_key(&key) {
                if entry.owner.is_some() && !elsewhere(key, entry.owner) {
                    candidate.retire = true;
                    candidate.image = None;
                    candidate.active = false;
                }
                continue;
            }
            candidate.touched = true;
            // Only this window's source is about to mutate. An earlier
            // accepted snapshot cannot be reconstructed from live source data.
            entry
                .accepted_semantics
                .get_or_insert_with(Default::default);
            let prepared = (|| -> Result<(), genet_render_host::RasterizeFailure> {
                let node = nodes[&key].ok_or_else(|| source_error("duplicate producer DOM key"))?;
                if entry
                    .owner
                    .is_some_and(|old| old != node && elsewhere(key, Some(old)))
                {
                    return Err(source_error("producer DOM key belongs to another window"));
                }
                candidate.owner = Some(node);
                let geometry = layout.element_geometry(dom, node);
                let logical_size = geometry
                    .as_ref()
                    .map(|g| g.content_size())
                    .unwrap_or((0., 0.));
                candidate.logical_size = logical_size;
                if logical_size.0 == 0. || logical_size.1 == 0. {
                    candidate.active = false;
                    candidate.image = None;
                    candidate.semantics = Default::default();
                    return Ok(());
                }
                let size = physical_size(
                    logical_size,
                    scale,
                    surface.device().limits().max_texture_dimension_2d,
                )
                .ok_or_else(|| source_error("invalid native producer extent"))?;
                let needs_frame = !same_device
                    || entry.force_frame
                    || !entry.active
                    || entry.owner != Some(node)
                    || entry.image.as_ref().is_none_or(|image| image.size != size);
                if needs_frame {
                    candidate.image = None;
                }
                candidate.active = true;
                let frame = ProducerFrameInfo {
                    purpose: ProducerFramePurpose::Presentation,
                    logical_size,
                    timestamp: self.timestamp,
                    physical_size: size,
                    layout_scale: scale,
                    needs_frame,
                    appearance: layout.resolved_appearance(node, &entry.properties),
                };
                candidate.invoked = true;
                let phase = crate::Instant::now();
                let output = entry.producer.render_staged(&ProducerContext {
                    device: surface.device(),
                    queue: surface.queue(),
                    frame: &frame,
                    core,
                    native_submission: Some(submission),
                });
                stats.render_us += micros(phase.elapsed());
                stats.render_calls += 1;
                let output = output?;
                candidate.semantics = SemanticSnapshot::capture(&*entry.producer);
                candidate.error = None;
                let Some(output) = output else { return Ok(()) };
                validate_output(&output, size).map_err(|error| {
                    source_error(format!("invalid native producer output: {error:?}"))
                })?;
                if candidate.image.as_ref().is_some_and(|image| {
                    image.size == size
                        && image.output.generation == output.generation
                        && image.output.view == output.view
                        && image.output.alpha == output.alpha
                        && image.output.encoding == output.encoding
                }) {
                    return Ok(());
                }
                self.next_stamp = self
                    .next_stamp
                    .checked_add(1)
                    .ok_or_else(|| source_error("native producer stamp exhausted"))?;
                let phase = crate::Instant::now();
                core.renderer().stage_external_image(
                    candidate.renderer_key,
                    &output.view,
                    size,
                    output.alpha,
                    self.next_stamp,
                );
                stats.stage_us += micros(phase.elapsed());
                stats.stages += 1;
                candidate.image = Some(Image {
                    output,
                    size,
                    stamp: self.next_stamp,
                });
                Ok(())
            })();
            if let Err(failure) = prepared {
                staged.valid = false;
                stats.invalid_frames += 1;
                self.render_failure = Some(ProducerRenderFailure {
                    source_key: key,
                    failure,
                });
                break;
            }
        }
        self.staged = Some(staged);
        stats
    }

    /// Candidate identities for the document being prepared, never public
    /// image_identity observations of an unaccepted frame.
    pub(crate) fn staged_image_identities(&self) -> Vec<ProducerImageIdentity> {
        let Some(staged) = &self.staged else {
            return self.image_identities();
        };
        let mut identities: Vec<_> = staged
            .entries
            .values()
            .filter(|entry| !entry.retire)
            .filter_map(|entry| {
                entry.image.as_ref().map(|image| ProducerImageIdentity {
                    renderer_key: entry.renderer_key,
                    owner: entry.owner,
                    generation: staged.generation,
                    stamp: image.stamp,
                    size: image.size,
                })
            })
            .collect();
        identities.sort_unstable_by_key(|identity| identity.renderer_key);
        identities
    }

    pub(crate) fn staged_is_live(&self) -> bool {
        let Some(staged) = &self.staged else {
            return false;
        };
        staged.valid
            && staged.entries.len() == self.entries.len()
            && staged.entries.iter().all(|(key, candidate)| {
                self.entries.get(key).is_some_and(|entry| {
                    candidate.belongs_to(entry)
                        && (!candidate.invoked
                            || (entry.producer.staged_is_live() && candidate.semantics.is_live()))
                })
            })
    }

    /// Called only after accepted GPU work and successful host presentation.
    pub(crate) fn publish_staged(&mut self) -> bool {
        if !self.staged_is_live() {
            return false;
        }
        let staged = self.staged.take().expect("live candidate");
        self.device = Some(staged.device);
        self.device_generation = Some(staged.generation);
        for (key, candidate) in staged.entries {
            if candidate.retire {
                self.remove(key);
                continue;
            }
            if !candidate.touched {
                continue;
            }
            let entry = self
                .entries
                .get_mut(&key)
                .expect("live candidate registration");
            if candidate.invoked {
                entry.producer.publish_staged();
            } else if !candidate.active {
                entry.producer.suspend();
            }
            entry.owner = candidate.owner;
            entry.active = candidate.active;
            entry.image = candidate.image;
            entry.logical_size = candidate.logical_size;
            entry.error = candidate.error;
            entry.force_frame = false;
            entry.accepted_semantics = Some(candidate.semantics);
        }
        true
    }

    /// Failed or cancelled candidates never replace accepted metadata. Their
    /// renderer bindings are revoked and the next preparation requires a draw.
    pub(crate) fn discard_staged(&mut self, core: Option<&genet_render_host::RenderCore>) {
        let Some(staged) = self.staged.take() else {
            return;
        };
        let fallback = self.cleanup_core.upgrade();
        let core = core.or(fallback.as_deref());
        for (key, candidate) in staged.entries {
            if !candidate.touched {
                continue;
            }
            if let Some(core) = core {
                crate::retire_external_image(core, candidate.renderer_key);
            }
            if let Some(entry) = self
                .entries
                .get_mut(&key)
                .filter(|entry| candidate.belongs_to(entry))
            {
                if candidate.invoked {
                    entry.producer.discard_staged();
                }
                entry.force_frame = true;
            }
        }
    }
}

fn source_error(message: impl Into<String>) -> genet_render_host::RasterizeFailure {
    genet_render_host::RasterizeFailure::SourceError(message.into())
}

#[cfg(test)]
#[path = "native_tests.rs"]
mod tests;
