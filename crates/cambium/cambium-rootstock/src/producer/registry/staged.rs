// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Browser whole-frame ledger. Accepted registry state is unchanged during awaits.

use super::*;
use genet_render_host::{AcceptedBrowserFrame, BrowserFrameCommands, RasterizeFailure};

#[derive(Clone, PartialEq)]
struct Baseline {
    key: u64,
    renderer_key: u64,
    owner: Option<NodeId>,
    active: bool,
    logical: (f32, f32),
    image: Option<(wgpu::TextureView, u64, u64, [u32; 2])>,
}
fn baseline(registry: &ProducerRegistry) -> Vec<Baseline> {
    registry.entries.iter().map(|(&key, entry)| Baseline {
        key, renderer_key:entry.renderer_key, owner:entry.owner, active:entry.active,
        logical:entry.logical_size,
        image:entry.image.as_ref().map(|image| (image.output.view.clone(), image.output.generation, image.stamp, image.size)),
    }).collect()
}

struct SlotPlan {
    key: u64,
    owner: NodeId,
    logical_size: (f32, f32),
    frame: Option<ProducerFrameInfo>, // None means exact zero-area/suspended slot.
}
struct Candidate {
    key: u64,
    owner: NodeId,
    logical_size: (f32, f32),
    renderer_key: u64,
    image: Option<Image>,
    private: bool,
}

pub(crate) struct PreparedProducerWindow {
    pub(crate) stats: ProducerFrameStats,
    pub(crate) payload_bytes: usize,
    baseline: Vec<Baseline>,
    previous_stamp: u64,
    next_stamp: u64,
    previous_generation: Option<u64>,
    generation: u64,
    previous_retire: Vec<u64>,
    absent: Vec<u64>,
    candidates: BTreeMap<u64, Candidate>,
    cleanup: Weak<genet_render_host::RenderCore>,
    committed: bool,
}
impl PreparedProducerWindow {
    #[cfg(feature = "browser-frame-diagnostics")]
    pub(crate) fn capture_source_keys(&self) -> Vec<u64> {
        self.candidates.values().filter_map(|candidate|candidate.image.is_some().then_some(candidate.key)).collect()
    }
    pub(crate) fn source_count(&self) -> usize {
        self.candidates.values().filter(|candidate| candidate.image.is_some()).count()
    }
    pub(crate) fn commands(&self, key: u64) -> Option<Vec<PaintCmd>> {
        let candidate = self.candidates.get(&key)?;
        let Some(image) = &candidate.image else { return Some(Vec::new()) };
        Some(vec![PaintCmd::DrawExternalTexture(ExternalTextureItem {
            placement:CommonPlacement::new(LayoutRect::new(LayoutPoint::new(0.0,0.0), LayoutPoint::new(candidate.logical_size.0,candidate.logical_size.1))),
            texture_key:key, opacity:1.0, content_generation:Some(image.stamp),
        })])
    }
    pub(crate) fn remap(&self, translated: &mut paint_list_render::TranslatedDisplayList) -> Result<(), ProducerGpuRemapError> {
        remap_gpu_display_list(translated, |local_key| {
            let candidate = self.candidates.get(&local_key).ok_or(ProducerGpuRemapError::MissingProducer { local_key })?;
            let image = candidate.image.as_ref().ok_or(ProducerGpuRemapError::UnstagedProducer { local_key })?;
            Ok(GpuBinding { renderer_key:candidate.renderer_key, stamp:image.stamp })
        })
    }
}
impl Drop for PreparedProducerWindow {
    fn drop(&mut self) {
        if self.committed { return; }
        if let Some(core) = self.cleanup.upgrade() {
            for candidate in self.candidates.values().filter(|candidate| candidate.private) {
                core.retire_owned_external_image(candidate.renderer_key);
            }
        }
    }
}

// Retire application producer ownership only after leaving Host RefMut.
struct RetiredEntry(Entry);
impl Drop for RetiredEntry {
    fn drop(&mut self) { self.0.producer.retire(); }
}

fn refusal(message: &str) -> RasterizeFailure { crate::producer::staged_producer_refusal(message) }

impl ProducerRegistry {
    pub(crate) fn prepare_browser_window<D: LayoutDom<NodeId = NodeId>>(
        &mut self, surface:&dyn Surface, layout:&OwnedLayout, dom:&D,
        scale:f32, elsewhere:Elsewhere<'_>, commands:&BrowserFrameCommands<'_>,
    ) -> Result<PreparedProducerWindow, RasterizeFailure> {
        if !self.browser_retirements.is_empty() {
            return Err(refusal("prior frame resources must retire outside the host borrow"));
        }
        if surface.core().generation() != commands.generation() {
            return Err(refusal("producer plan is foreign to the frame Core"));
        }
        let mut nodes = HashMap::new();
        for (key,node) in layout.custom_leaf_nodes(dom) {
            nodes.entry(key).and_modify(|owner| *owner = None).or_insert(Some(node));
        }
        // Full known ledger preflight before the first source target/staging.
        let mut slots = Vec::new();
        let limits = paint_list_render::bundle::BundleLimits::default();
        let mut payload_bytes = 0usize;
        for (&key, entry) in &self.entries {
            let Some(owner) = nodes.get(&key) else { continue };
            let Some(owner) = *owner else { return Err(refusal("duplicate DOM producer key")) };
            if entry.owner.is_some_and(|previous| previous != owner && elsewhere(key,Some(previous))) {
                return Err(refusal("producer is already owned by another window"));
            }
            let Some(geometry) = layout.element_geometry(dom,owner) else { return Err(refusal("producer geometry is unavailable")) };
            let logical_size = geometry.content_size();
            if !logical_size.0.is_finite() || !logical_size.1.is_finite()
                || logical_size.0 < 0.0 || logical_size.1 < 0.0 {
                return Err(refusal("invalid producer logical geometry"));
            }
            if slots.len() >= limits.max_sources {
                return Err(refusal("producer ledger exceeds resource admission limits"));
            }
            if logical_size.0 == 0.0 || logical_size.1 == 0.0 {
                slots.push(SlotPlan { key, owner, logical_size, frame:None });
                continue;
            }
            let size = physical_size(logical_size,scale,surface.device().limits().max_texture_dimension_2d)
                .ok_or_else(|| refusal("invalid producer extent"))?;
            let pixels = (size[0] as usize).checked_mul(size[1] as usize).ok_or_else(|| refusal("producer pixel budget overflow"))?;
            if size.iter().any(|dimension| *dimension > limits.max_dimension) || pixels > limits.max_pixels {
                return Err(refusal("producer extent exceeds resource admission limits"));
            }
            payload_bytes = pixels.checked_mul(4).and_then(|bytes| payload_bytes.checked_add(bytes))
                .ok_or_else(|| refusal("producer payload budget overflow"))?;
            if payload_bytes > limits.max_bytes || slots.len() >= limits.max_sources {
                return Err(refusal("producer ledger exceeds resource admission limits"));
            }
            if !entry.producer.supports_deferred_acceptance() {
                return Err(refusal("producer does not support deferred frame acceptance"));
            }
            let needs_frame = !entry.active || entry.owner != Some(owner)
                || self.device_generation != Some(surface.core().generation())
                || entry.image.as_ref().is_none_or(|image| image.size != size);
            slots.push(SlotPlan { key, owner, logical_size, frame:Some(ProducerFrameInfo {
                logical_size, physical_size:size, layout_scale:scale, needs_frame,
                timestamp:self.timestamp, appearance:layout.resolved_appearance(owner,&entry.properties),
            }) });
        }
        // Reserve worst-case stamps without advancing accepted state.
        self.next_stamp.checked_add(slots.len() as u64).ok_or_else(|| refusal("producer stamp range exhausted"))?;
        let mut plan = PreparedProducerWindow {
            stats:ProducerFrameStats::default(),
            payload_bytes,
            baseline:baseline(self), previous_stamp:self.next_stamp, next_stamp:self.next_stamp,
            previous_generation:self.device_generation, generation:surface.core().generation(),
            previous_retire:self.pending_retire.clone(),
            absent:self.entries.iter().filter_map(|(&key,entry)| (entry.owner.is_some() && !nodes.contains_key(&key) && !elsewhere(key,entry.owner)).then_some(key)).collect(),
            candidates:BTreeMap::new(), cleanup:self.cleanup_core.clone(), committed:false,
        };
        let prepared = (|| {
            for slot in slots {
                let entry = self.entries.get_mut(&slot.key).expect("preflighted producer");
                let stable = entry.renderer_key;
                let mut candidate = Candidate { key:slot.key, owner:slot.owner, logical_size:slot.logical_size,
                    renderer_key:stable, image:None, private:false };
                if let Some(frame) = slot.frame {
                    let phase = crate::Instant::now();
                    let output = entry.producer.prepare_checked(&ProducerContext {
                        device:surface.device(), queue:surface.queue(), core:surface.core(), frame:&frame,
                    }, commands)?;
                    plan.stats.render_calls += 1;
                    plan.stats.render_us += micros(phase.elapsed());
                    match output {
                        None if !frame.needs_frame => { candidate.image = entry.image.clone(); },
                        None => return Err(refusal("required producer output was omitted")),
                        Some(output) => {
                            validate_output(&output,frame.physical_size).map_err(|_| refusal("producer alpha/encoding/extent mismatch"))?;
                            if !entry.producer.prepared_is_current() { return Err(refusal("producer source changed during preparation")); }
                            let private = allocate_renderer_key(&NEXT_PHYSICAL_PRODUCER).map_err(|_| refusal("private producer identity exhausted"))?;
                            plan.next_stamp = plan.next_stamp.checked_add(1).ok_or_else(|| refusal("producer stamp exhausted"))?;
                            candidate.renderer_key = private; candidate.private = true;
                            candidate.image = Some(Image { output, size:frame.physical_size, stamp:plan.next_stamp });
                            // Register cleanup ownership before the GPU operation.
                            plan.candidates.insert(slot.key,candidate);
                            let image = plan.candidates[&slot.key].image.as_ref().expect("candidate image");
                            let phase = crate::Instant::now();
                            commands.renderer().stage_external_image(private,&image.output.view,image.size,image.output.alpha,image.stamp);
                            plan.stats.stage_us += micros(phase.elapsed()); plan.stats.stages += 1;
                            continue;
                        },
                    }
                    if candidate.image.is_none() || !entry.producer.prepared_is_current() {
                        return Err(refusal("accepted producer reuse is no longer current"));
                    }
                }
                plan.candidates.insert(slot.key,candidate);
            }
            Ok(())
        })();
        if let Err(error) = prepared { self.abort_browser_window(plan); return Err(error); }
        Ok(plan)
    }

    pub(crate) fn abort_browser_window(&mut self, plan:PreparedProducerWindow) {
        for entry in self.entries.values_mut() {
            if let Some(prepared) = entry.producer.take_prepared() { self.browser_retirements.push(prepared); }
        }
        self.browser_retirements.push(Box::new(plan));
    }

    pub(crate) fn take_browser_retirements(&mut self) -> Vec<Box<dyn std::any::Any>> {
        std::mem::take(&mut self.browser_retirements)
    }

    pub(crate) fn has_browser_retirements(&self) -> bool { !self.browser_retirements.is_empty() }

    pub(crate) fn suspend_browser_except(&mut self, elsewhere:Elsewhere<'_>, core:Option<&genet_render_host::RenderCore>) {
        for (&key,entry) in &mut self.entries {
            if elsewhere(key,entry.owner) { continue; }
            if let Some(resources) = entry.producer.take_suspended_resources() { self.browser_retirements.push(resources); }
            if let Some(image) = entry.image.take() { self.browser_retirements.push(Box::new(image)); }
            entry.active = false;
            if let Some(core) = core { core.retire_owned_external_image(entry.renderer_key); }
        }
        if let Some(core) = core {
            for key in self.pending_retire.drain(..) { core.retire_owned_external_image(key); }
        }
    }

    /// Read-only admission; keeps the private plan uncommitted across phase two.
    pub(crate) fn check_browser_window_plan(
        &self, plan:&PreparedProducerWindow, accepted:&AcceptedBrowserFrame,
    ) -> Result<Arc<genet_render_host::RenderCore>, RasterizeFailure> {
        accepted.check_current()?;
        if accepted.generation() != plan.generation || self.next_stamp != plan.previous_stamp
            || self.device_generation != plan.previous_generation || self.pending_retire != plan.previous_retire
            || baseline(self) != plan.baseline
        { return Err(refusal("stale producer acceptance plan")); }
        if plan.candidates.values().filter(|candidate| candidate.image.is_some()).any(|candidate| !self.entries[&candidate.key].producer.prepared_is_current()) {
            return Err(refusal("producer source changed before publication"));
        }
        let core = plan.cleanup.upgrade().ok_or_else(|| refusal("producer Core was retired"))?;
        if core.generation() != plan.generation { return Err(refusal("producer Core generation changed")); }
        accepted.check_current()?;
        Ok(core)
    }

    pub(crate) fn commit_browser_window(
        &mut self, mut plan:PreparedProducerWindow, accepted:&AcceptedBrowserFrame,
    ) -> Result<(), RasterizeFailure> {
        let core = match self.check_browser_window_plan(&plan,accepted) {
            Ok(core) => core,
            Err(error) => { self.abort_browser_window(plan); return Err(error); },
        };
        let moves:Vec<_> = plan.candidates.values().filter(|candidate| candidate.private).map(|candidate| {
            let image = candidate.image.as_ref().expect("private candidate image");
            netrender::ExternalImagePromotion { temporary:candidate.renderer_key,
                stable:self.entries[&candidate.key].renderer_key, size:image.size, generation:image.stamp }
        }).collect();
        if let Err(error) = accepted.check_current() { self.abort_browser_window(plan); return Err(error); }
        if let Err(error) = accepted.promote_external_images(&moves) {
            self.abort_browser_window(plan); return Err(refusal(&format!("private image publication refused: {error:?}")));
        }
        // No GPU submission, source callback or fallible admission follows.
        for key in self.pending_retire.drain(..) { core.retire_owned_external_image(key); }
        for key in &plan.absent {
            if let Some(entry) = self.entries.remove(key) {
                core.retire_owned_external_image(entry.renderer_key);
                self.browser_retirements.push(Box::new(RetiredEntry(entry)));
            }
        }
        for (key,candidate) in std::mem::take(&mut plan.candidates) {
            let entry = self.entries.get_mut(&key).expect("preflighted producer");
            let retired = if candidate.image.is_some() { entry.producer.commit_prepared_retiring() }
                else { core.retire_owned_external_image(entry.renderer_key); entry.producer.take_suspended_resources() };
            if let Some(resources) = retired { self.browser_retirements.push(resources); }
            entry.owner = Some(candidate.owner); entry.active = candidate.image.is_some();
            entry.logical_size = candidate.logical_size;
            if let Some(image) = std::mem::replace(&mut entry.image,candidate.image) {
                self.browser_retirements.push(Box::new(image));
            }
            entry.error = None;
        }
        self.next_stamp = plan.next_stamp;
        self.device = Some(core.device().clone()); self.device_generation = Some(plan.generation);
        plan.committed = true;
        // Baseline clones also own GPU views. Transfer the completed ledger
        // to outside-borrow cleanup instead of dropping it on this turn.
        self.browser_retirements.push(Box::new(plan));
        Ok(())
    }
}

#[cfg(feature = "browser-frame-diagnostics")]
impl ProducerRegistry {
    pub(crate) fn browser_capture_source_stamps(&self,keys:&[u64]) -> Option<Vec<crate::BrowserSurfaceCaptureSource>> {
        if keys.len()>paint_list_render::bundle::BundleLimits::default().max_resources { return None }
        keys.iter().map(|&key| {
            let entry=self.entries.get(&key)?;
            if !entry.active || !entry.producer.prepared_is_current() { return None }
            Some(crate::BrowserSurfaceCaptureSource { local_key:key,image:self.image_identity(key)? })
        }).collect()
    }
    pub(crate) fn browser_capture_sources_current(&self,expected:&[crate::BrowserSurfaceCaptureSource]) -> bool {
        expected.iter().all(|source|self.entries.get(&source.local_key).is_some_and(|entry|
            entry.active && entry.producer.prepared_is_current()
                && self.image_identity(source.local_key)==Some(source.image)))
    }
}
