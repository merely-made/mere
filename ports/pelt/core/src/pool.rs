// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained content custody independent of arrangement and window lifetime.
use crate::{
    PeltContent, PeltHostEffect, PeltInput, PeltLane, PeltLayer, PeltSessionIdentity, WorkspaceRect,
};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq)]
pub struct PeltPlacement<K> {
    pub content: K,
    pub window: u64,
    pub rect: WorkspaceRect,
    pub visible: bool,
    pub focused: bool,
    pub scale_factor: f32,
}

impl<K> PeltPlacement<K> {
    fn admits(&self) -> bool {
        self.visible
            && self.scale_factor.is_finite()
            && self.scale_factor > 0.0
            && [self.rect.x, self.rect.y, self.rect.width, self.rect.height]
                .into_iter()
                .all(f32::is_finite)
            && self.rect.width > 0.0
            && self.rect.height > 0.0
    }
}

struct Placement<K> {
    value: PeltPlacement<K>,
    generation: u64,
}
struct Capture<P> {
    placement: P,
    buttons: u16,
    last: PeltInput,
}

struct Prepared<P> {
    placement: P,
    generation: u64,
    identity: Option<PeltSessionIdentity>,
}

/// Content keys identify lifetimes; placement keys identify appearances. Two
/// appearances sharing a content key share its scroll/history. Give independent
/// Reader appearances their own content keys. The host remains arrangement authority.
pub struct PeltControllerPool<K, P, F> {
    contents: HashMap<K, PeltContent<F>>,
    pub(crate) a11y: crate::pool_a11y::PoolA11yState<K, P>,
    placements: HashMap<P, Placement<K>>,
    order: Vec<P>,
    generation: u64,
    captures: HashMap<i32, Capture<P>>,
    last_pump: Option<(u64, bool)>,
    pending: HashSet<K>,
    prepared: HashMap<K, Prepared<P>>,
    presentation: HashMap<K, (bool, Option<(P, u64)>)>,
}

impl<K: Eq + Hash + Clone, P: Eq + Hash + Clone, F: 'static> Default
    for PeltControllerPool<K, P, F>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + Clone, P: Eq + Hash + Clone, F: 'static> PeltControllerPool<K, P, F> {
    pub fn new() -> Self {
        Self {
            contents: HashMap::new(),
            a11y: Default::default(),
            placements: HashMap::new(),
            order: Vec::new(),
            generation: 0,
            captures: HashMap::new(),
            last_pump: None,
            pending: HashSet::new(),
            prepared: HashMap::new(),
            presentation: HashMap::new(),
        }
    }
    pub fn content(&self, key: &K) -> Option<&PeltContent<F>> {
        self.contents.get(key)
    }
    pub fn content_mut(&mut self, key: &K) -> Option<&mut PeltContent<F>> {
        self.last_pump = None;
        self.prepared.remove(key);
        self.a11y_invalidate(key);
        self.contents.get_mut(key)
    }
    pub fn insert(&mut self, key: K, content: PeltContent<F>) -> Option<PeltContent<F>> {
        self.a11y_invalidate(&key);
        let placements: Vec<P> = self
            .placements
            .iter()
            .filter(|(_, p)| p.value.content == key)
            .map(|(id, _)| id.clone())
            .collect();
        for id in &placements {
            self.cancel_placement(id);
        }
        if let Some(old) = self.contents.get_mut(&key) {
            old.focus(false);
            old.set_hidden(true);
        }
        let old = self.contents.insert(key.clone(), content);
        for id in placements {
            self.generation = self
                .generation
                .checked_add(1)
                .expect("placement generation exhausted");
            self.placements.get_mut(&id).unwrap().generation = self.generation;
        }
        self.prepared.remove(&key);
        self.pending.remove(&key);
        self.presentation.remove(&key);
        self.last_pump = None;
        self.sync_visibility();
        old
    }
    pub fn remove_content(&mut self, key: &K) -> Option<PeltContent<F>> {
        self.a11y_invalidate(key);
        let removed: Vec<P> = self
            .placements
            .iter()
            .filter(|(_, p)| &p.value.content == key)
            .map(|(id, _)| id.clone())
            .collect();
        for id in removed {
            self.cancel_placement(&id);
            self.placements.remove(&id);
        }
        self.order.retain(|id| self.placements.contains_key(id));
        self.last_pump = None;
        self.prepared.remove(key);
        self.pending.remove(key);
        self.presentation.remove(key);
        if let Some(content) = self.contents.get_mut(key) {
            content.focus(false);
            content.set_hidden(true);
        }
        self.contents.remove(key)
    }
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.contents.keys()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &PeltContent<F>)> {
        self.contents.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut PeltContent<F>)> {
        self.last_pump = None;
        self.prepared.clear();
        let keys: Vec<K> = self.contents.keys().cloned().collect();
        for key in keys {
            self.a11y_invalidate(&key);
        }
        self.contents.iter_mut()
    }
    /// Mutable producer custody without invalidating unrelated document
    /// publications. Hosts polling external surfaces should prefer this over
    /// unrestricted mutable iteration.
    pub fn surfaces_mut(&mut self) -> impl Iterator<Item = (&K, &mut PeltContent<F>)> {
        if self
            .contents
            .values()
            .any(|c| c.lane() == PeltLane::Surface)
        {
            self.last_pump = None;
        }
        self.contents
            .iter_mut()
            .filter(|(_, content)| content.lane() == PeltLane::Surface)
    }
    pub(crate) fn get(&self, key: &K) -> Option<&PeltContent<F>> {
        self.content(key)
    }
    pub(crate) fn get_mut(&mut self, key: &K) -> Option<&mut PeltContent<F>> {
        self.content_mut(key)
    }
    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.contents.contains_key(key)
    }
    pub(crate) fn remove(&mut self, key: &K) -> Option<PeltContent<F>> {
        self.remove_content(key)
    }

    /// Atomically replace arrangement observations. Missing placements close;
    /// missing content keys are not fabricated, retired, or respawned.
    pub fn set_placements(&mut self, values: impl IntoIterator<Item = (P, PeltPlacement<K>)>) {
        let mut next = HashMap::new();
        let mut order = Vec::new();
        for (id, value) in values {
            let generation = match self.placements.get(&id) {
                Some(old) if old.value == value => old.generation,
                _ => {
                    self.generation = self
                        .generation
                        .checked_add(1)
                        .expect("placement generation exhausted");
                    self.generation
                },
            };
            if !next.contains_key(&id) {
                order.push(id.clone());
            }
            next.insert(id, Placement { value, generation });
        }
        let changed_contents: HashSet<K> = self
            .placements
            .iter()
            .filter(|(id, old)| {
                next.get(*id)
                    .is_none_or(|new| old.generation != new.generation)
            })
            .map(|(_, p)| p.value.content.clone())
            .chain(
                next.iter()
                    .filter(|(id, new)| {
                        self.placements
                            .get(*id)
                            .is_none_or(|old| old.generation != new.generation)
                    })
                    .map(|(_, p)| p.value.content.clone()),
            )
            .collect();
        for key in changed_contents {
            self.a11y_invalidate(&key);
        }
        let cancelled: Vec<P> = self
            .placements
            .iter()
            .filter(|(id, old)| {
                next.get(*id).is_none_or(|new| {
                    !new.value.admits()
                        || new.value.content != old.value.content
                        || new.value.window != old.value.window
                        || (old.value.focused && !new.value.focused)
                })
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in cancelled {
            self.cancel_placement(&id);
        }
        if self.order != order
            || self.placements.len() != next.len()
            || next.iter().any(|(id, p)| {
                self.placements
                    .get(id)
                    .is_none_or(|old| old.generation != p.generation)
            })
        {
            self.last_pump = None;
        }
        self.placements = next;
        self.order = order;
        self.sync_visibility();
    }
    fn sync_visibility(&mut self) {
        let keys: Vec<K> = self.contents.keys().cloned().collect();
        for key in keys {
            let visible = self
                .placements
                .values()
                .any(|p| p.value.content == key && p.value.admits());
            let focused = self.order.iter().find_map(|id| {
                let p = self.placements.get(id)?;
                (p.value.content == key && p.value.admits() && p.value.focused)
                    .then(|| (id.clone(), p.generation))
            });
            let previous = self.presentation.get(&key).cloned();
            if previous.as_ref().map(|state| state.0) != Some(visible) {
                self.contents.get_mut(&key).unwrap().set_hidden(!visible);
            }
            let mut delivered_focus = focused.clone();
            if previous.as_ref().map(|state| &state.1) != Some(&focused) {
                if let Some((id, _)) = &focused {
                    if self.prepare(id).is_err() {
                        delivered_focus = None;
                    }
                }
                self.contents
                    .get_mut(&key)
                    .unwrap()
                    .focus(delivered_focus.is_some());
            }
            self.presentation.insert(key, (visible, delivered_focus));
        }
    }
    pub fn placement(&self, id: &P) -> Option<&PeltPlacement<K>> {
        self.placements.get(id).map(|p| &p.value)
    }
    pub fn placement_generation(&self, id: &P) -> Option<u64> {
        self.placements.get(id).map(|p| p.generation)
    }
    pub fn admits(&self, id: &P, generation: u64) -> bool {
        self.placements.get(id).is_some_and(|p| {
            p.generation == generation
                && p.value.admits()
                && self.contents.contains_key(&p.value.content)
        })
    }
    pub fn frame(&mut self, id: &P) -> Option<PeltLayer<F>> {
        let (p, generation) = self
            .placements
            .get(id)
            .map(|p| (p.value.clone(), p.generation))?;
        if !p.admits() {
            return None;
        }
        self.a11y_observe(&p.content);
        let content = self.contents.get_mut(&p.content)?;
        self.prepared.remove(&p.content);
        let layer = content.frame(p.rect, p.scale_factor);
        if !matches!(&layer, PeltLayer::Surface(Err(_))) {
            self.prepared.insert(
                p.content.clone(),
                Prepared {
                    placement: id.clone(),
                    generation,
                    identity: content.session_identity(),
                },
            );
            self.a11y_after_layout(&p.content);
        }
        Some(layer)
    }
    /// Prepare the addressed layout before input or semantic revalidation.
    /// Preparation can change a document projection revision. Accessibility
    /// callers must revalidate their captured target after this succeeds.
    pub fn prepare(&mut self, id: &P) -> Result<(), String> {
        let (p, generation) = self
            .placements
            .get(id)
            .filter(|p| p.value.admits())
            .map(|p| (p.value.clone(), p.generation))
            .ok_or_else(|| "placement has no admitted geometry".to_owned())?;
        self.a11y_observe(&p.content);
        let content = self
            .contents
            .get_mut(&p.content)
            .ok_or_else(|| "placement content was retired".to_owned())?;
        if content.lane() == PeltLane::Surface
            && self.prepared.get(&p.content).is_some_and(|ready| {
                ready.placement == *id
                    && ready.generation == generation
                    && ready.identity == content.session_identity()
            })
        {
            return Ok(());
        }
        self.prepared.remove(&p.content);
        content.prepare_input(p.rect, p.scale_factor)?;
        self.prepared.insert(
            p.content.clone(),
            Prepared {
                placement: id.clone(),
                generation,
                identity: content.session_identity(),
            },
        );
        self.a11y_after_layout(&p.content);
        Ok(())
    }
    /// Pump all retained documents. Engines decide whether hidden work remains;
    /// surfaces require polling only while an appearance is visible.
    pub fn pump(&mut self) -> bool {
        self.last_pump = None;
        let keys: Vec<K> = self.contents.keys().cloned().collect();
        for key in &keys {
            self.a11y_observe(key);
        }
        let mut more = false;
        self.pending.clear();
        for (key, content) in &mut self.contents {
            if content.lane() == PeltLane::Document
                || self
                    .placements
                    .values()
                    .any(|p| &p.value.content == key && p.value.admits())
            {
                if content.pump() {
                    more = true;
                    self.pending.insert(key.clone());
                }
            }
        }
        for key in &keys {
            self.a11y_observe(key);
        }
        more
    }
    /// Observe the most recent pump outcomes without advancing any session.
    /// Hosts may schedule hidden pending work on idle turns while requesting
    /// visual redraw only for admitted visible appearances with pending work.
    pub fn visible_needs_frame(&self) -> bool {
        self.placements
            .values()
            .any(|p| p.value.admits() && self.pending.contains(&p.value.content))
    }
    /// Retained content that requested another turn at the last pump.
    pub fn pending_contents(&self) -> impl Iterator<Item = &K> {
        self.pending.iter()
    }

    /// Whether this window has admitted content awaiting another frame.
    pub fn window_needs_frame(&self, window: u64) -> bool {
        self.placements.values().any(|p| {
            p.value.window == window && p.value.admits() && self.pending.contains(&p.value.content)
        })
    }

    /// Share a host's clock epoch across windows, avoiding duplicate logical
    /// ticks. The pool never creates clocks or substitutes wall-clock time.
    pub fn pump_at(&mut self, epoch_ms: f64) -> bool {
        if let Some((epoch, more)) = self.last_pump {
            if epoch == epoch_ms.to_bits() {
                return more;
            }
        }
        let more = self.pump();
        self.last_pump = epoch_ms.is_finite().then_some((epoch_ms.to_bits(), more));
        more
    }
    pub fn captured_placement(&self, pointer_id: i32) -> Option<&P> {
        self.captures.get(&pointer_id).map(|c| &c.placement)
    }
    pub fn has_pointer_capture(&self) -> bool {
        !self.captures.is_empty()
    }
    pub fn focused_content(&self, window: u64) -> Option<&K> {
        self.order
            .iter()
            .filter_map(|id| self.placement(id))
            .find(|p| p.window == window && p.admits() && p.focused)
            .map(|p| &p.content)
    }
    fn cancel_placement(&mut self, id: &P) {
        let ids: Vec<i32> = self
            .captures
            .iter()
            .filter(|(_, c)| &c.placement == id)
            .map(|(id, _)| *id)
            .collect();
        for pointer in ids {
            if let Some(capture) = self.captures.remove(&pointer) {
                if let Some(p) = self.placements.get(id) {
                    if let Some(content) = self.contents.get_mut(&p.value.content) {
                        let _ = content.input_event(capture.last.cancel(), p.value.scale_factor);
                    }
                }
            }
        }
    }
    /// Route host/window coordinates through authoritative placements.
    pub fn input_at(&mut self, window: u64, mut input: PeltInput) -> PeltHostEffect {
        if matches!(
            &input,
            PeltInput::Session(inker::SessionInput::Focus(false))
        ) {
            let targets: Vec<P> = self
                .placements
                .iter()
                .filter(|(_, p)| p.value.window == window)
                .map(|(id, _)| id.clone())
                .collect();
            let mut effect = PeltHostEffect {
                pointer_capture: Some(false),
                ..Default::default()
            };
            for id in targets {
                self.cancel_placement(&id);
                if self.placement(&id).is_some_and(|p| p.focused) {
                    effect = self.input(&id, input.clone());
                }
            }
            return effect;
        }
        let capture = input
            .pointer()
            .and_then(|(id, _, _)| self.captured_placement(id))
            .cloned();
        let target = capture.or_else(|| {
            self.order
                .iter()
                .find(|id| {
                    let Some(p) = self.placement(id) else {
                        return false;
                    };
                    p.window == window
                        && p.admits()
                        && match input.position() {
                            Some((x, y)) => p.rect.contains(x, y),
                            None => p.focused,
                        }
                })
                .cloned()
        });
        let Some(id) = target else {
            return PeltHostEffect::default();
        };
        let Some(p) = self.placement(&id) else {
            return PeltHostEffect::default();
        };
        if p.window != window {
            return PeltHostEffect::default();
        }
        input.translate(-p.rect.x, -p.rect.y);
        self.input(&id, input)
    }
    /// Dispatch content-local input to one current, visible appearance.
    pub fn input(&mut self, id: &P, input: PeltInput) -> PeltHostEffect {
        let Some(p) = self.placement(id).cloned().filter(|p| p.admits()) else {
            return PeltHostEffect::default();
        };
        if input
            .position()
            .is_some_and(|(x, y)| !x.is_finite() || !y.is_finite())
        {
            return PeltHostEffect::default();
        }
        let pointer = input.pointer();
        if let Some((pointer_id, phase, button)) = pointer {
            if let Some(capture) = self.captures.get(&pointer_id) {
                if &capture.placement != id
                    || (phase == inker::PointerPhase::Up
                        && capture.buttons & crate::input::button_bit(button) == 0)
                {
                    return PeltHostEffect::default();
                }
            } else if matches!(phase, inker::PointerPhase::Up | inker::PointerPhase::Cancel) {
                return PeltHostEffect::default();
            }
        }
        let losing_focus = matches!(
            &input,
            PeltInput::Session(inker::SessionInput::Focus(false))
        );
        if losing_focus {
            self.cancel_placement(id);
        }
        let cancelling = matches!(&input, PeltInput::Session(inker::SessionInput::Cancel))
            || pointer.is_some_and(|(_, phase, _)| phase == inker::PointerPhase::Cancel);
        if !losing_focus && !cancelling {
            if let Err(error) = self.prepare(id) {
                return PeltHostEffect {
                    error: Some(error),
                    ..Default::default()
                };
            }
        }
        self.last_pump = None;
        let Some(content) = self.contents.get_mut(&p.content) else {
            return PeltHostEffect::default();
        };
        let mut effect = content.input_event(input.clone(), p.scale_factor);
        if losing_focus {
            effect.pointer_capture = Some(false);
        }
        if let Some((pointer_id, phase, button)) = pointer {
            use inker::PointerPhase;
            match phase {
                PointerPhase::Down if effect.handled || effect.pointer_capture == Some(true) => {
                    let buttons = self.captures.get(&pointer_id).map_or(0, |c| c.buttons)
                        | crate::input::button_bit(button);
                    self.captures.insert(
                        pointer_id,
                        Capture {
                            placement: id.clone(),
                            buttons,
                            last: input,
                        },
                    );
                    effect.pointer_capture = Some(true);
                },
                PointerPhase::Move => {
                    if let Some(c) = self.captures.get_mut(&pointer_id) {
                        c.last = input;
                    }
                },
                PointerPhase::Up => {
                    if let Some(c) = self.captures.get_mut(&pointer_id) {
                        c.buttons &= !crate::input::button_bit(button);
                        c.last = input;
                    }
                    if self
                        .captures
                        .get(&pointer_id)
                        .is_some_and(|c| c.buttons == 0)
                    {
                        self.captures.remove(&pointer_id);
                    }
                    effect.pointer_capture = Some(self.captures.contains_key(&pointer_id));
                },
                PointerPhase::Cancel => {
                    self.captures.remove(&pointer_id);
                    effect.pointer_capture = Some(false);
                },
                _ => {},
            }
        }
        if effect.handled {
            self.a11y_invalidate(&p.content);
        }
        effect
    }
}
impl<K: Eq + Hash + Clone, P: Eq + Hash + Clone, F: 'static> From<HashMap<K, PeltContent<F>>>
    for PeltControllerPool<K, P, F>
{
    fn from(contents: HashMap<K, PeltContent<F>>) -> Self {
        Self {
            contents,
            ..Self::new()
        }
    }
}
impl<K: Eq + Hash + Clone, P: Eq + Hash + Clone, F: 'static> FromIterator<(K, PeltContent<F>)>
    for PeltControllerPool<K, P, F>
{
    fn from_iter<T: IntoIterator<Item = (K, PeltContent<F>)>>(iter: T) -> Self {
        HashMap::from_iter(iter).into()
    }
}
