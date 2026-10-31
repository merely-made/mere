// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The remote board: a mounted remote scene described as Pictograph's
//! [`BoardScene`], and the [`PhysicsBoard`] that moves its cards.
//!
//! Every host that draws a remote session goes through this, so the mapping
//! from Graphshell's scene to the board is written once: the Graphshell
//! port's browser pages and native viewers alike (stack lift plan, P10). The
//! score's positions become anchor slots; the scene's relations become the
//! board's edges; a fold hides its members, and its stand-in carries "+N"
//! (stack-lift D2). The board mirrors the local canvas's live stage
//! (dynamics grammar plan, F150) and its speed dial (ruled 2026-10-04, "The
//! viewer's own dial"); nothing is written back. (Physics catalog: P3.)

use std::collections::{BTreeSet, HashMap};

use crate::view::ProjectionLayoutView;
use graphshell_client::MountedScene;
use graphshell_client::frozen::Satisfaction;
use pictograph::canvas::dynamics_spec::DynamicsSpec;
use pictograph::canvas::{
    BoardBackdrop, BoardCard, BoardFootprint, BoardRect, BoardRelation, BoardScene, PhysicsBoard,
    Speed,
};
use sceno::{InstanceId, StandIn};

#[cfg(test)]
mod tests;

/// What the scene's folds do to its instances: the card each hidden
/// instance is drawn as, how many each stand-in hides, and the summary
/// bodies that stand for a whole fold.
#[derive(Default)]
struct Folding {
    drawn_as: HashMap<u32, String>,
    hides: HashMap<u32, usize>,
    summaries: Vec<(Vec<InstanceId>, Option<String>)>,
}

fn folding(mounted: &MountedScene) -> Folding {
    let mut folding = Folding::default();
    for fold in mounted.scene.tables.folds.iter().flatten() {
        match &fold.stand_in {
            StandIn::Member(stand_in) => {
                for member in fold.members.iter().filter(|m| *m != stand_in) {
                    folding.drawn_as.insert(member.0, stand_in.0.to_string());
                }
                *folding.hides.entry(stand_in.0).or_default() += fold.members.len() - 1;
            }
            StandIn::Summary { label } => {
                let id = format!("fold:{}", folding.summaries.len());
                for member in &fold.members {
                    folding.drawn_as.insert(member.0, id.clone());
                }
                folding
                    .summaries
                    .push((fold.members.clone(), label.clone().or_else(|| fold.label.clone())));
            }
        }
    }
    folding
}

/// Describe a mounted scene as a board: one card per active item in order
/// that no fold hides, the first in the lead tone, held cards marked, a
/// stand-in carrying the count it hides; edges from the relations, lifted to
/// the stand-in of a hidden end; backdrops from the layout.
pub fn board_scene(mounted: &MountedScene) -> BoardScene {
    let tables = &mounted.scene.tables;
    let bounds = tables.bounds;
    let layout = ProjectionLayoutView::from_scene(&mounted.scene);
    let folding = folding(mounted);
    let mut cards: Vec<BoardCard> = mounted
        .scene
        .active_items_in_order()
        .into_iter()
        .filter(|(instance, _)| !folding.drawn_as.contains_key(&instance.0))
        .map(|(instance, item)| BoardCard {
            id: instance.0.to_string(),
            // The name the endpoint's presentation gives the card.
            title: mounted
                .presentation
                .offers_for(instance)
                .and_then(|offers| offers.first())
                .map(|offer| offer.semantics.label.clone())
                .unwrap_or_default(),
            slot: (item.transform.translate.x, item.transform.translate.y),
            site: tables
                .sources
                .get(item.source.0 as usize)
                .and_then(|source| source.as_ref())
                .map(|source| source.adapter.clone())
                .unwrap_or_default(),
            footprint: BoardFootprint::from(&item.footprint),
            pinned: Satisfaction::is_pinned(tables, instance),
            lead: instance.0 == 0,
            folded: folding.hides.get(&instance.0).copied().unwrap_or(0),
        })
        .collect();
    for (index, (members, label)) in folding.summaries.iter().enumerate() {
        let items: Vec<_> = members
            .iter()
            .filter_map(|m| mounted.scene.active_item(*m))
            .collect();
        let Some(first) = items.first() else { continue };
        let n = items.len() as f32;
        let (x, y) = items.iter().fold((0.0, 0.0), |(x, y), item| {
            (x + item.transform.translate.x, y + item.transform.translate.y)
        });
        cards.push(BoardCard {
            id: format!("fold:{index}"),
            title: label.clone().unwrap_or_default(),
            slot: (x / n, y / n),
            site: String::new(),
            footprint: BoardFootprint::from(&first.footprint),
            pinned: false,
            lead: false,
            folded: members.len(),
        });
    }
    let end = |instance: InstanceId| {
        folding
            .drawn_as
            .get(&instance.0)
            .cloned()
            .unwrap_or_else(|| instance.0.to_string())
    };
    let drawn: BTreeSet<String> = cards.iter().map(|card| card.id.clone()).collect();
    let mut seen = BTreeSet::new();
    let relations = tables
        .relations
        .iter()
        .flatten()
        .filter_map(|relation| {
            let (from, to) = (end(relation.from), end(relation.to));
            let kind = relation.kind.clone();
            let shown = from != to && drawn.contains(&from) && drawn.contains(&to);
            (shown && seen.insert((from.clone(), to.clone(), kind.clone())))
                .then_some(BoardRelation { from, to, kind })
        })
        .collect();
    BoardScene {
        bounds: BoardRect::new(
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.w,
            bounds.size.h,
        ),
        backdrops: layout
            .backdrops
            .iter()
            .map(|backdrop| BoardBackdrop {
                rect: BoardRect::new(backdrop.x, backdrop.y, backdrop.width, backdrop.height),
                kind: backdrop.kind.clone(),
                collidable: backdrop.collidable,
            })
            .collect(),
        cards,
        relations,
    }
}

/// The board a host keeps for its remote session.
#[derive(Default)]
pub struct RemoteBoard {
    board: PhysicsBoard,
    scene: BoardScene,
    position: Option<(scenotime::SceneEpoch, Option<u64>, Option<String>)>,
    /// The viewer's speed, as last mirrored.
    speed: Speed,
}

impl RemoteBoard {
    pub fn new() -> Self {
        Self::default()
    }

    /// Stage the board's repulsion on the host's device (P5c).
    #[cfg(feature = "board-gpu")]
    pub fn set_physics_device(&mut self, device: Option<pictograph::canvas::PhysicsDevice>) {
        self.board.set_physics_device(device);
    }

    /// Mirror the viewer's live `stage` and `speed` (no-ops when unchanged)
    /// and, whenever the acknowledged revision moves, reconcile the bodies to
    /// the scene: a new card spawns at its slot with a settle burst, the
    /// others keep their simulated positions, every slot is re-anchored. The
    /// stage is the canvas's [`live_stage_spec`](pictograph::canvas::Canvas::live_stage_spec)
    /// (dynamics grammar plan, F150): its law or composition, the stage under
    /// way when a schedule runs. `None`, or a stage the board refuses, leaves
    /// the board's stage as it was.
    pub fn sync(
        &mut self,
        mounted: Option<&MountedScene>,
        revision: Option<u64>,
        stage: Option<&DynamicsSpec>,
        speed: Speed,
    ) {
        self.sync_scoped(mounted, revision, None, stage, speed)
    }

    /// Read the acknowledged score's layout exactly. A resident mere has its
    /// own saved layout; simulated bodies from another attachment or layout
    /// cannot determine this reading, nor can the hidden local canvas's law.
    pub fn sync_reading(
        &mut self,
        mounted: Option<&MountedScene>,
        revision: Option<u64>,
        identity: Option<&str>,
    ) {
        let position =
            mounted.map(|mounted| (mounted.scene.epoch, revision, identity.map(str::to_owned)));
        if position != self.position {
            self.board.sync(Vec::new());
        }
        self.sync_scoped(mounted, revision, identity, None, Speed::REAL_TIME);
        self.board.halt();
    }

    /// An attached session is a separate identity even when an endpoint reuses
    /// its projection name, epoch or revision. It owns no durable graph here.
    pub fn sync_scoped(
        &mut self,
        mounted: Option<&MountedScene>,
        revision: Option<u64>,
        identity: Option<&str>,
        stage: Option<&DynamicsSpec>,
        speed: Speed,
    ) {
        if let Some(stage) = stage {
            let _ = self.board.set_stage(stage);
        }
        if speed != self.speed {
            self.speed = speed;
            self.board.set_speed(speed);
        }
        let Some(mounted) = mounted else {
            self.scene = BoardScene::default();
            self.board.sync(self.scene.items());
        self.board.set_links(self.scene.links());
            self.position = None;
            return;
        };
        let position = (mounted.scene.epoch, revision, identity.map(str::to_owned));
        if Some(&position) == self.position.as_ref() && !self.board.is_empty() {
            return;
        }
        self.scene = board_scene(mounted);
        self.board.sync(self.scene.items());
        self.board.set_links(self.scene.links());
        self.position = Some(position);
    }

    /// The step budget the board's frames run under: the page's, as its
    /// canvas's.
    pub fn set_step_budget(&mut self, budget: Option<pictograph::canvas::StepBudget>) {
        self.board.set_step_budget(budget);
    }

    /// The speed the board steps at: the viewer's, as last mirrored.
    pub fn speed(&self) -> Speed {
        self.speed
    }

    /// Advance the board one frame; whether it is still moving.
    pub fn tick(&mut self) -> bool {
        self.board.tick()
    }

    pub fn board(&self) -> &PhysicsBoard {
        &self.board
    }

    /// The board, for the viewer's own actions on its cards (F64).
    pub fn board_mut(&mut self) -> &mut PhysicsBoard {
        &mut self.board
    }

    /// The board as last described; hosts paint it with
    /// `scene().paint(board(), ..)`.
    pub fn scene(&self) -> &BoardScene {
        &self.scene
    }

    /// Presentation-only emphasis of the reader's selected instance.
    pub fn set_focus(&mut self, instance: Option<&str>) {
        for card in &mut self.scene.cards {
            card.lead = instance == Some(card.id.as_str());
        }
    }

    /// Overlapping footprints at the positions drawn.
    pub fn overlaps(&self) -> usize {
        self.scene.overlaps(&self.board)
    }

    pub fn energy(&self) -> f32 {
        self.board.energy()
    }

    pub fn gap(&self, a: &str, b: &str) -> Option<f32> {
        self.board.gap(a, b)
    }

    pub fn is_settling(&self) -> bool {
        self.board.is_settling()
    }
}

