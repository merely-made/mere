// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::{collections::HashSet, time::Duration};

use emblem::Palette;
use kernel::graph::NodeKey;
use paint_list_api::{
    LayerSpec, LayoutPoint, LayoutTransform, PaintCmd, TransformKind, TransformSpec,
};

use crate::backdrop::{LOD_HEIGHT, ViewDigest, derive_backdrop};
use crate::flat_icon::wash_commands;

use super::{wash_height, wash_world_transform};

const STABLE_FRAMES: u8 = 4;
const DETAIL_MARGIN: f32 = 16.0;
const FADE_DURATION: Duration = Duration::from_millis(160);

#[derive(Default)]
pub(in crate::canvas) struct WashPresentation {
    current: Option<Wash>,
    outgoing: Option<Wash>,
    pending: Option<(ViewDigest, u8)>,
    fade_elapsed: Duration,
    scope: Option<HashSet<NodeKey>>,
    folded: HashSet<NodeKey>,
    #[cfg(test)]
    derivations: usize,
}

impl WashPresentation {
    pub(super) fn update_visibility(
        &mut self,
        scope: Option<HashSet<NodeKey>>,
        folded: HashSet<NodeKey>,
    ) -> bool {
        let changed = self.scope != scope || self.folded != folded;
        self.scope = scope;
        self.folded = folded;
        changed
    }

    pub(super) fn present(
        &mut self,
        digest: Option<ViewDigest>,
        visibility_changed: bool,
        dt: Duration,
        camera: LayoutTransform,
        palette: &Palette,
    ) -> (Vec<PaintCmd>, bool) {
        if self.outgoing.is_some() {
            self.fade_elapsed = self.fade_elapsed.saturating_add(dt);
            if self.fade_elapsed >= FADE_DURATION || visibility_changed {
                self.outgoing = None;
            }
        }

        let Some(digest) = digest else {
            self.current = None;
            self.outgoing = None;
            self.pending = None;
            return (Vec::new(), false);
        };

        if self
            .current
            .as_ref()
            .is_some_and(|wash| wash.digest == digest)
        {
            self.pending = None;
        } else if self.current.is_none() || visibility_changed {
            self.replace(digest, &camera, false);
        } else {
            self.observe(digest, &camera);
        }

        let mut commands = Vec::new();
        if let Some(current) = self.current.as_mut() {
            let projection = current.world.then(&camera);
            current.update_detail(wash_height(&projection));
            let incoming_alpha = if let Some(outgoing) = self.outgoing.as_mut() {
                let progress = self.fade_elapsed.as_secs_f32() / FADE_DURATION.as_secs_f32();
                outgoing.append(
                    &mut commands,
                    outgoing.world.then(&camera),
                    palette,
                    1.0 - progress,
                );
                progress
            } else {
                1.0
            };
            current.append(&mut commands, projection, palette, incoming_alpha);
        }

        (commands, self.pending.is_some() || self.outgoing.is_some())
    }

    #[cfg(test)]
    pub(super) fn work(&self) -> (usize, usize) {
        (
            self.derivations,
            self.current.as_ref().map_or(0, |wash| wash.decodes),
        )
    }

    fn observe(&mut self, digest: ViewDigest, camera: &LayoutTransform) {
        match &mut self.pending {
            Some((pending, frames)) if *pending == digest => {
                *frames = frames.saturating_add(1);
                // Finish the active fade before starting another, so two
                // snapshots suffice even if changes keep arriving.
                if *frames >= STABLE_FRAMES && self.outgoing.is_none() {
                    self.replace(digest, camera, true);
                }
            },
            _ => self.pending = Some((digest, 1)),
        }
    }

    fn replace(&mut self, digest: ViewDigest, camera: &LayoutTransform, fade: bool) {
        #[cfg(test)]
        {
            self.derivations += 1;
        }
        let inherited_detail = self.current.as_ref().map(|wash| wash.detailed);
        let next = Wash::new(digest, camera, inherited_detail);
        let previous = self.current.replace(next);
        self.outgoing = if fade { previous } else { None };
        self.fade_elapsed = Duration::ZERO;
        self.pending = None;
    }
}

struct Wash {
    digest: ViewDigest,
    file: Option<Vec<u8>>,
    world: LayoutTransform,
    detailed: bool,
    palette: Option<Palette>,
    // Each arm is decoded once per palette in camera-independent coordinates.
    // The world affine is kept separately so camera changes only move the ink.
    arms: [Option<Vec<PaintCmd>>; 2],
    #[cfg(test)]
    decodes: usize,
}

impl Wash {
    fn new(digest: ViewDigest, camera: &LayoutTransform, detail: Option<bool>) -> Self {
        let world = wash_world_transform(&digest);
        let file = derive_backdrop(&digest)
            .inspect_err(|error| tracing::debug!(%error, "could not derive content backdrop"))
            .ok();
        Self {
            digest,
            file,
            world,
            detailed: detail.unwrap_or_else(|| wash_height(&world.then(camera)) >= LOD_HEIGHT),
            palette: None,
            arms: [None, None],
            #[cfg(test)]
            decodes: 0,
        }
    }

    fn update_detail(&mut self, height: f32) {
        self.detailed = if self.detailed {
            height >= LOD_HEIGHT - DETAIL_MARGIN
        } else {
            height >= LOD_HEIGHT + DETAIL_MARGIN
        };
    }

    fn append(
        &mut self,
        commands: &mut Vec<PaintCmd>,
        transform: LayoutTransform,
        palette: &Palette,
        opacity: f32,
    ) {
        let Some(file) = &self.file else {
            return;
        };
        if opacity <= 0.0 {
            return;
        }
        if self.palette.as_ref() != Some(palette) {
            self.palette = Some(*palette);
            self.arms = [None, None];
        }
        let arm = &mut self.arms[usize::from(self.detailed)];
        let ink = arm.get_or_insert_with(|| {
            #[cfg(test)]
            {
                self.decodes += 1;
            }
            let height = if self.detailed { LOD_HEIGHT } else { 0.0 };
            wash_commands(file, palette, height)
                .inspect_err(|error| tracing::debug!(%error, "could not paint content backdrop"))
                .unwrap_or_default()
        });
        if ink.is_empty() {
            return;
        }

        if opacity < 1.0 {
            commands.push(PaintCmd::PushLayer(LayerSpec {
                opacity,
                ..LayerSpec::default()
            }));
        }
        commands.push(PaintCmd::PushTransform(TransformSpec {
            origin: LayoutPoint::zero(),
            transform,
            kind: TransformKind::Standard,
        }));
        commands.extend_from_slice(ink);
        commands.push(PaintCmd::PopTransform);
        if opacity < 1.0 {
            commands.push(PaintCmd::PopLayer);
        }
    }
}
