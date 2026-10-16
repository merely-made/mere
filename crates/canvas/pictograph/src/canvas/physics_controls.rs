// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Arrangement and physics actions a host's controls apply to the canvas.
//!
//! A host's forms and panels read their controls into these typed inputs and
//! call the same actions, so there is one implementation of each Apply.
//! Graphshell's old page and its tree panel do; this is the reducer Woodshed
//! wants too. It came out of Graphshell's `canvas_physics` (Graphshell stack
//! lift plan, P2 and L5) once `ProjectionClock` moved to `scenotime`. The
//! canvas owns the state; nothing here reads a DOM. Catalogs come from the
//! canvas (`CANVAS_LAYOUT_STRATEGIES`, `CANVAS_PHYSICS_*`).

use std::collections::HashMap;

use kernel::geometry::PortablePoint;
use kernel::graph::NodeKey;

use scenotime::ProjectionClock;

use super::{CANVAS_LAYOUT_STRATEGIES, Canvas, LayoutStats, PhysicsChoice, PhysicsOverlay};

/// The arrangement picker's "no arrangement" choice: physics alone.
pub const FREE_ARRANGEMENT: &str = "free";
/// The latest settle's positions, an arrangement like any other (F30).
pub use super::SETTLED_ARRANGEMENT;
/// What a profile picker reads when the live pair names no profile.
pub const CUSTOM_PROFILE: &str = "custom";

/// The arrangement picker's choices, `(id, label)`: the canvas's graph-wide
/// strategies, then the latest settle, then free.
pub fn arrangement_choices() -> impl Iterator<Item = (&'static str, &'static str)> {
    CANVAS_LAYOUT_STRATEGIES.iter().copied().chain([
        (SETTLED_ARRANGEMENT, "Settled (last rest)"),
        (FREE_ARRANGEMENT, "Free (physics alone)"),
    ])
}

/// The live profile id, or [`CUSTOM_PROFILE`].
pub fn profile_id(canvas: &Canvas) -> &'static str {
    PhysicsChoice::live(canvas)
        .profile_id()
        .unwrap_or(CUSTOM_PROFILE)
}

/// The overlays a form has ticked, in catalog order.
pub fn ticked_overlays(ticked: impl Fn(PhysicsOverlay) -> bool) -> Vec<PhysicsOverlay> {
    PhysicsOverlay::ALL
        .into_iter()
        .filter(|overlay| ticked(*overlay))
        .collect()
}

/// Apply physics: the picked law, overlays and sources written into the
/// canvas's spec and set as one change (dynamics grammar plan, F162), and the
/// camera follows the layout while it plays (ruled 2026-10-03, "Follow while
/// playing"). Overlays the law refuses are left out, and the status gives
/// the law's reason.
pub fn apply_physics(canvas: &mut Canvas, choice: &PhysicsChoice) -> String {
    let (choice, refused) = choice.clone().admitted();
    let running = PhysicsChoice::live(canvas);
    let applied = canvas
        .dynamics_spec()
        .map_err(|error| error.to_string())
        .and_then(|mut spec| {
            choice.write_into(&mut spec, &running);
            canvas
                .set_dynamics_spec(&spec)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
    canvas.set_view_follow(true);
    let status = with_overlays(
        format!("Physics set to {}", PhysicsChoice::live(canvas).law.label()),
        canvas,
    );
    match (applied, refused) {
        (Err(error), _) => format!("{status} · {error}"),
        (Ok(()), Some(refusal)) => format!("{status} · {}", refusal.reason),
        (Ok(()), None) => status,
    }
}

/// Apply a named profile's law and overlays. Sources are left as they are.
pub fn apply_profile(canvas: &mut Canvas, id: &str) -> Result<String, String> {
    if id.is_empty() || id == CUSTOM_PROFILE {
        return Err("choose a profile first".to_string());
    }
    let choice = PhysicsChoice::live(canvas)
        .with_profile(id)
        .ok_or_else(|| format!("unknown physics profile {id}"))?;
    let mut spec = canvas.dynamics_spec()?;
    choice.write_into(&mut spec, &PhysicsChoice::live(canvas));
    canvas
        .set_dynamics_spec(&spec)
        .map_err(|error| error.to_string())?;
    canvas.set_view_follow(true);
    Ok(with_overlays(
        format!("Profile {id}: {}", PhysicsChoice::live(canvas).law.label()),
        canvas,
    ))
}

/// The layout's room-by-mass signature where a law was applied, so a
/// receipt can say what the law did from there: rank above the start's,
/// density CV under it (Density's qualitative bar, ruled 2026-10-03).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LawStart {
    pub mass_area_rank: f32,
    pub density_cv: f32,
}

impl LawStart {
    pub fn of(canvas: &Canvas) -> Self {
        let stats = canvas.layout_stats();
        Self {
            mass_area_rank: stats.mass_area_rank,
            density_cv: stats.density_cv,
        }
    }

    /// Snapshot fields: the start's values and whether `now` beats them.
    pub fn fields(&self, now: &LayoutStats) -> [(&'static str, String); 4] {
        [
            (
                "law-start-mass-area-rank",
                format!("{:.2}", self.mass_area_rank),
            ),
            ("law-start-density-cv", format!("{:.3}", self.density_cv)),
            (
                "layout-rank-rose",
                (now.mass_area_rank > self.mass_area_rank).to_string(),
            ),
            (
                "layout-cv-fell",
                (now.density_cv < self.density_cv).to_string(),
            ),
        ]
    }
}

/// A receipt line for `log-layout <label>`: the law, the layout's signature
/// now, and where the law started.
pub fn layout_line(label: &str, canvas: &Canvas, start: Option<&LawStart>) -> String {
    let stats = canvas.layout_stats();
    let start = start.map_or_else(
        || "start unrecorded".to_string(),
        |start| {
            format!(
                "start rank {:.3} cv {:.3}",
                start.mass_area_rank, start.density_cv
            )
        },
    );
    format!(
        "layout {label}: law {} nodes {} rank {:.3} cv {:.3} overlaps {} spread {:.0}; {start}",
        PhysicsChoice::live(canvas).law.id(),
        canvas.graph().node_count(),
        stats.mass_area_rank,
        stats.density_cv,
        stats.overlaps,
        stats.spread,
    )
}

fn with_overlays(status: String, canvas: &Canvas) -> String {
    let overlays = PhysicsChoice::live(canvas).overlays;
    if overlays.is_empty() {
        return status;
    }
    let labels: Vec<_> = overlays.iter().map(|overlay| overlay.label()).collect();
    format!("{status} with {}", labels.join(", "))
}

/// An applied arrangement: its id, the status, and a transition when the
/// layout animates to its slots.
pub struct ArrangementApplied {
    pub layout_id: String,
    pub status: String,
    pub transition: Option<ArrangementTransition>,
}

/// Apply arrangement `layout_id` at `viewport`. Free drops the analytic
/// layout and lets the law alone place the graph; any other id projects its
/// strategy and animates there when positions change. Selection survives.
pub fn apply_arrangement(
    canvas: &mut Canvas,
    layout_id: &str,
    viewport: (u32, u32),
) -> Result<ArrangementApplied, String> {
    if !arrangement_choices().any(|(id, _)| id == layout_id) {
        return Err(format!("unknown arrangement {layout_id}"));
    }
    let selected = canvas.selected_members();
    if layout_id == FREE_ARRANGEMENT {
        // Reverting resumes a paused sim (the canvas's own rule).
        canvas.set_projection_score(None);
        canvas.set_layout_strategy(None);
        canvas.set_selected_members(&selected);
        canvas.set_view_follow(true);
        return Ok(ArrangementApplied {
            layout_id: layout_id.to_string(),
            status: "Arrangement set to free: physics alone".to_string(),
            transition: None,
        });
    }
    if layout_id == SETTLED_ARRANGEMENT {
        return apply_settled(canvas);
    }
    let previous_score = canvas.projection_score().cloned();
    let extents = canvas.strategy_extents();
    // The canvas is the binding: the arrangement reads its channel
    // registry's facts (the cluster partition Kinds and Group pull read).
    let zoom = canvas.camera().zoom;
    let projection = canvas.project_arrangement_for_view(
        layout_id,
        viewport.0,
        viewport.1,
        Some(&extents),
        true,
        zoom,
        previous_score.as_ref(),
    );
    let transition = ArrangementTransition::between(canvas, &projection.positions)?;
    canvas.pick_layout_strategy(Some(layout_id.to_string()));
    canvas.set_projection_score(projection.score);
    canvas.set_strategy_regions(projection.regions);
    if transition.is_none() {
        canvas.apply_strategy_positions(&projection.positions);
    }
    canvas.note_strategy_computed(layout_id, viewport.0, viewport.1, canvas.focused_key());
    canvas.set_selected_members(&selected);
    if transition.is_none() {
        canvas.fit_to_content();
    }
    Ok(ArrangementApplied {
        layout_id: layout_id.to_string(),
        status: if transition.is_some() {
            format!("Arrangement changing to {layout_id}")
        } else {
            format!("Arrangement set to {layout_id}")
        },
        transition,
    })
}

/// Settled: the latest settle's positions, with no score, animated to like
/// any analytic arrangement. Refused until the graph has come to rest once.
fn apply_settled(canvas: &mut Canvas) -> Result<ArrangementApplied, String> {
    let positions = canvas
        .settled_positions()
        .ok_or("nothing has settled yet: play physics until the graph rests")?
        .to_vec();
    let transition = ArrangementTransition::between(canvas, &positions)?;
    canvas.pick_layout_strategy(Some(SETTLED_ARRANGEMENT.to_string()));
    canvas.set_projection_score(None);
    canvas.set_strategy_regions(Vec::new());
    if transition.is_none() {
        canvas.apply_strategy_positions(&positions);
    }
    Ok(ArrangementApplied {
        layout_id: SETTLED_ARRANGEMENT.to_string(),
        status: if transition.is_some() {
            format!("Arrangement changing to {SETTLED_ARRANGEMENT}")
        } else {
            format!("Arrangement set to {SETTLED_ARRANGEMENT}")
        },
        transition,
    })
}

/// Advance a running arrangement transition at host time `host_ms`. When it
/// lands, the final slots apply, the transition clears, and the landed
/// status comes back. A stop that returned anchored items (F24) starts its
/// own transition here, from where they were to their positions.
pub fn advance_arrangement(
    canvas: &mut Canvas,
    transition: &mut Option<ArrangementTransition>,
    layout_id: &str,
    host_ms: f64,
) -> Option<String> {
    if let Some(stop) = canvas.take_stop_return() {
        *transition = ArrangementTransition::returning(canvas, &stop)
            .ok()
            .flatten();
    }
    let returning = transition.as_ref()?.returning;
    let (positions, complete) = transition.as_mut()?.advance(host_ms);
    match (returning, complete) {
        (false, true) => {
            canvas.apply_strategy_positions(&positions);
            *transition = None;
            Some(format!("Arrangement set to {layout_id}"))
        },
        (false, false) => {
            canvas.preview_strategy_positions(&positions);
            None
        },
        (true, true) => {
            canvas.land_paused_positions(&positions);
            *transition = None;
            None
        },
        (true, false) => {
            canvas.preview_paused_positions(&positions);
            None
        },
    }
}

/// One playback of an analytic arrangement change, or of a stop's return of
/// anchored items. Scenotime owns the schedule; the host owns the frame clock
/// it is advanced with.
pub struct ArrangementTransition {
    schedule: scenotime::TransitionSchedule,
    clock: ProjectionClock,
    node_of: HashMap<sceno::InstanceId, NodeKey>,
    start_positions: Vec<(NodeKey, PortablePoint)>,
    final_positions: Vec<(NodeKey, PortablePoint)>,
    /// A stop's return: it lands as the paused placement, not as slots.
    returning: bool,
}

impl ArrangementTransition {
    /// A transition from the canvas's current geometry to `final_positions`,
    /// or `None` when nothing would move.
    pub fn between(
        canvas: &Canvas,
        final_positions: &[(NodeKey, PortablePoint)],
    ) -> Result<Option<Self>, String> {
        let old_geometry = canvas.cartography_geometry();
        let old_of = old_geometry
            .iter()
            .map(|(id, (x, y))| (id, PortablePoint::new(x, y)))
            .collect::<HashMap<_, _>>();
        Self::from_to(canvas, &old_of, final_positions, false, None)
    }

    /// A stop's return (F24): from where the anchored items were to their
    /// arrangement positions, everything else holding still.
    pub fn returning(canvas: &Canvas, stop: &super::StopReturn) -> Result<Option<Self>, String> {
        let start_of = stop
            .from
            .iter()
            .filter_map(|(key, at)| Some((canvas.graph().get_node(*key)?.id, *at)))
            .collect::<HashMap<_, _>>();
        Self::from_to(canvas, &start_of, &stop.to, true, stop.duration_ms)
    }

    fn from_to(
        canvas: &Canvas,
        old_of: &HashMap<uuid::Uuid, PortablePoint>,
        final_positions: &[(NodeKey, PortablePoint)],
        returning: bool,
        duration_ms: Option<u32>,
    ) -> Result<Option<Self>, String> {
        let final_of = final_positions.iter().copied().collect::<HashMap<_, _>>();
        let extents = canvas.strategy_extents();
        let mut nodes = canvas.graph().nodes().collect::<Vec<_>>();
        nodes.sort_by_key(|(_, node)| node.id);

        let mut scene = sceno::Scene::new();
        scene.generation = canvas.graph().revision();
        let mut node_of = HashMap::new();
        let mut start_positions = Vec::with_capacity(nodes.len());
        let mut operations = Vec::new();
        for (key, node) in nodes {
            let Some(target) = final_of.get(&key).copied() else {
                continue;
            };
            let current = old_of.get(&node.id).copied().unwrap_or(target);
            let source = scene.intern_source(sceno::SourceRef::new(
                super::MERE_GRAPH_ADAPTER,
                node.id.to_string(),
            ));
            let (width, height) = extents.get(&key).copied().unwrap_or((36.0, 36.0));
            let item = sceno::ProjectedItem {
                source,
                space: sceno::Scene::WORLD,
                transform: sceno::Transform2::translation(current.x, current.y),
                footprint: sceno::Footprint::Rect {
                    size: sceno::Size2::new(width, height),
                },
                representation: canvas
                    .projection_representation(key)
                    .cloned()
                    .unwrap_or(sceno::Representation::Card),
                layer: 0,
                visible: true,
                hit: None,
                channels: Vec::new(),
            };
            let instance = sceno::InstanceId(scene.items.len() as u32);
            node_of.insert(instance, key);
            start_positions.push((key, current));
            scene.items.push(item.clone());
            if current != target {
                let mut target_item = item;
                target_item.transform = sceno::Transform2::translation(target.x, target.y);
                operations.push(scenotime::SceneOp::UpdateItem {
                    index: instance,
                    value: target_item,
                });
            }
        }

        if operations.is_empty() {
            return Ok(None);
        }
        let before = scenotime::SceneSnapshot::from_dense(
            scenotime::SceneEpoch(1),
            scenotime::Revision(1),
            scene,
        )
        .map_err(|error| format!("could not build transition start: {error:?}"))?;
        let diff = scenotime::SceneDiff {
            epoch: before.epoch,
            base: before.revision,
            revision: scenotime::Revision(before.revision.0 + 1),
            operations,
        };
        let schedule = scenotime::TransitionSchedule::from_diff(
            &before,
            &diff,
            &scenotime::TransitionSpec {
                duration_ms: duration_ms
                    .unwrap_or(scenotime::TransitionSpec::default().duration_ms),
                ..Default::default()
            },
        )
        .map_err(|error| format!("could not schedule arrangement transition: {error:?}"))?;
        Ok(Some(Self {
            schedule,
            clock: ProjectionClock::default(),
            node_of,
            start_positions,
            final_positions: final_positions.to_vec(),
            returning,
        }))
    }

    /// The positions at host time `host_ms`, and whether the transition is done.
    pub fn advance(&mut self, host_ms: f64) -> (Vec<(NodeKey, PortablePoint)>, bool) {
        let frame = self.schedule.sample_at(self.clock.observe(host_ms));
        if frame.complete {
            return (self.final_positions.clone(), true);
        }
        let mut positions = self
            .start_positions
            .iter()
            .copied()
            .collect::<HashMap<_, _>>();
        for sample in frame.items {
            let Some(key) = self.node_of.get(&sample.instance).copied() else {
                continue;
            };
            positions.insert(
                key,
                PortablePoint::new(
                    sample.value.transform.translate.x,
                    sample.value.transform.translate.y,
                ),
            );
        }
        let positions = self
            .start_positions
            .iter()
            .filter_map(|(key, _)| positions.get(key).copied().map(|position| (*key, position)))
            .collect();
        (positions, false)
    }
}

/// The framing check's inset: a node's centre must lie inside the visible
/// canvas itself.
pub const FRAMING_MARGIN: f32 = 0.0;

/// The framing fields both pages publish, from the canvas's own positions and
/// camera: nodes whose centre is off the visible canvas, the layout's and the
/// view's world extents (`min_x,min_y,max_x,max_y`), and whether the camera is
/// following the layout.
pub fn framing_fields(canvas: &Canvas) -> [(&'static str, String); 4] {
    let framing = canvas.layout_framing(FRAMING_MARGIN);
    let text = |[a, b, c, d]: [f32; 4]| format!("{a:.0},{b:.0},{c:.0},{d:.0}");
    [
        ("layout-outside", framing.outside.to_string()),
        ("layout-extent", text(framing.extent)),
        ("view-extent", text(framing.view)),
        ("view-follow", canvas.view_follows().to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{
        CANVAS_PHYSICS_PROFILES, PhysicsDepthSource, PhysicsKindSource, PhysicsLaw,
        PhysicsMassSource,
    };

    fn canvas() -> Canvas {
        let mut canvas = Canvas::with_sample_graph();
        canvas.resize(800, 600);
        canvas
    }

    #[test]
    fn physics_apply_sets_sources_overlays_and_law_together() {
        let mut canvas = canvas();
        let choice = PhysicsChoice {
            law: PhysicsLaw::Kinds,
            overlays: ticked_overlays(|overlay| {
                matches!(
                    overlay,
                    PhysicsOverlay::Skeleton | PhysicsOverlay::DegreeRepulsion
                )
            }),
            kind: PhysicsKindSource::Degree,
            groups: PhysicsKindSource::Cluster,
            mass: PhysicsMassSource::PageRank,
            depth: PhysicsDepthSource::Focus,
        };
        assert_eq!(
            choice.overlays,
            [PhysicsOverlay::DegreeRepulsion, PhysicsOverlay::Skeleton],
            "ticked overlays come in catalog order"
        );
        let status = apply_physics(&mut canvas, &choice);
        assert_eq!(PhysicsChoice::live(&canvas), choice);
        assert_eq!(status, "Physics set to Kinds with Hub room, Skeleton");
        assert_eq!(profile_id(&canvas), CUSTOM_PROFILE);
        let status = apply_physics(
            &mut canvas,
            &PhysicsChoice {
                law: PhysicsLaw::Springs,
                ..choice
            },
        );
        assert_eq!(status, "Physics set to Springs with Hub room, Skeleton");
    }

    #[test]
    fn a_profile_synchronizes_law_and_overlays_and_keeps_sources() {
        let mut canvas = canvas();
        apply_physics(
            &mut canvas,
            &PhysicsChoice {
                kind: PhysicsKindSource::Component,
                ..PhysicsChoice::default()
            },
        );
        for profile in CANVAS_PHYSICS_PROFILES {
            apply_profile(&mut canvas, profile.id).unwrap();
            let live = PhysicsChoice::live(&canvas);
            assert_eq!(live.law, profile.law, "{}", profile.id);
            assert_eq!(live.overlays, profile.overlays, "{}", profile.id);
            assert_eq!(live.kind, PhysicsKindSource::Component, "sources stay");
            assert_eq!(profile_id(&canvas), profile.id, "names itself back");
        }
        assert_eq!(
            apply_profile(&mut canvas, "liquid").unwrap(),
            "Profile liquid: Springs with Centre"
        );
        assert!(apply_profile(&mut canvas, CUSTOM_PROFILE).is_err());
        assert!(apply_profile(&mut canvas, "").is_err());
        assert!(apply_profile(&mut canvas, "plasma").is_err());
        assert_eq!(
            profile_id(&canvas),
            "liquid",
            "a refused profile changes nothing"
        );
    }

    #[test]
    fn a_custom_pair_names_no_profile_until_it_matches_again() {
        let mut canvas = canvas();
        apply_profile(&mut canvas, "void").unwrap();
        let mut choice = PhysicsChoice::live(&canvas);
        choice.overlays = vec![PhysicsOverlay::Skeleton];
        apply_physics(&mut canvas, &choice);
        assert_eq!(profile_id(&canvas), CUSTOM_PROFILE);
        choice.overlays.clear();
        apply_physics(&mut canvas, &choice);
        assert_eq!(profile_id(&canvas), "void");
    }

    /// "Follow while playing" (2026-10-03): a law, profile or Free switch
    /// follows the layout, any pan or zoom stops it, Fit graph resumes it, and
    /// an analytic arrangement leaves it as it was.
    #[test]
    fn switches_follow_the_layout_a_pan_or_zoom_stops_it_and_fit_resumes_it() {
        use crate::canvas::controls::CanvasCommand;
        let mut canvas = canvas();
        let viewport = (800, 600);
        let pan = CanvasCommand::Pan { dx: 10.0, dy: 0.0 };
        assert!(!canvas.view_follows(), "not before a switch");
        apply_physics(&mut canvas, &PhysicsChoice::default());
        assert!(canvas.view_follows(), "a law switch follows");
        pan.apply(&mut canvas, viewport);
        assert!(!canvas.view_follows(), "a pan stops it");
        apply_profile(&mut canvas, "liquid").unwrap();
        assert!(canvas.view_follows(), "a profile switch follows");
        CanvasCommand::Zoom { delta: 1.0 }.apply(&mut canvas, viewport);
        assert!(!canvas.view_follows(), "a zoom stops it");
        apply_arrangement(&mut canvas, FREE_ARRANGEMENT, viewport).unwrap();
        assert!(canvas.view_follows(), "Free follows");
        pan.apply(&mut canvas, viewport);
        CanvasCommand::Fit.apply(&mut canvas, viewport);
        assert!(canvas.view_follows(), "Fit graph resumes it");
        pan.apply(&mut canvas, viewport);
        apply_arrangement(&mut canvas, "grid.default", viewport).unwrap();
        assert!(!canvas.view_follows(), "an arrangement leaves it off");
    }

    #[test]
    fn free_drops_the_strategy_and_an_analytic_arrangement_restores_it() {
        let mut canvas = canvas();
        assert!(apply_arrangement(&mut canvas, "spiral", (800, 600)).is_err());
        assert!(!canvas.physics_paused(), "the sample canvas plays");
        let applied = apply_arrangement(&mut canvas, "grid.default", (800, 600)).unwrap();
        assert_eq!(applied.layout_id, "grid.default");
        assert!(applied.transition.is_some(), "the grid animates there");
        assert!(
            canvas.physics_paused(),
            "an analytic arrangement pauses while it lands"
        );
        if let Some(transition) = applied.transition {
            let mut transition = Some(transition);
            let mut landed = None;
            for frame in 0..600 {
                landed = advance_arrangement(
                    &mut canvas,
                    &mut transition,
                    "grid.default",
                    frame as f64 * 16.0,
                );
                if landed.is_some() {
                    break;
                }
            }
            assert_eq!(landed.as_deref(), Some("Arrangement set to grid.default"));
            assert!(transition.is_none());
            assert!(!canvas.physics_paused(), "then plays on from it (F24)");
        }
        let applied = apply_arrangement(&mut canvas, FREE_ARRANGEMENT, (800, 600)).unwrap();
        assert!(applied.transition.is_none());
        assert_eq!(applied.status, "Arrangement set to free: physics alone");
        assert!(!canvas.physics_paused(), "free resumes the law");
        assert!(
            arrangement_choices().any(|(id, _)| id == FREE_ARRANGEMENT),
            "free is offered"
        );
        assert!(
            arrangement_choices().any(|(id, _)| id == SETTLED_ARRANGEMENT),
            "settled is offered (F30)"
        );
        assert_eq!(
            arrangement_choices().count(),
            CANVAS_LAYOUT_STRATEGIES.len() + 2
        );
    }

    /// F24 through the host: a stop with anchored items starts a transition
    /// on the next advance, its frames move only the anchored items, and it
    /// lands them on their positions while the seeded item stays. Settled is
    /// refused until something has settled (G7).
    #[test]
    fn a_stop_returns_anchored_items_by_a_transition() {
        use crate::canvas::Role;
        let mut canvas = canvas();
        assert!(apply_arrangement(&mut canvas, SETTLED_ARRANGEMENT, (800, 600)).is_err());
        let applied = apply_arrangement(&mut canvas, "grid.default", (800, 600)).unwrap();
        let mut transition = applied.transition;
        for frame in 0..600 {
            if advance_arrangement(
                &mut canvas,
                &mut transition,
                "grid.default",
                frame as f64 * 16.0,
            )
            .is_some()
            {
                break;
            }
        }
        assert!(!canvas.physics_paused(), "the pick played on");
        canvas.set_arrangement_role(Role::Anchored);
        let keys: Vec<_> = canvas.graph().nodes().map(|(k, _)| k).collect();
        let seeded = canvas.graph().get_node(keys[0]).unwrap().id;
        canvas.set_member_role(seeded, Some(Role::Seeded));
        for _ in 0..90 {
            canvas.frame(800, 600);
        }
        let moved = canvas.cartography_geometry();
        canvas.set_physics_paused(true);
        assert!(canvas.has_stop_return());
        let mut transition = None;
        assert!(advance_arrangement(&mut canvas, &mut transition, "grid.default", 0.0).is_none());
        assert!(transition.is_some(), "the stop's return animates");
        assert!(!canvas.has_stop_return());
        let mut frames = 0;
        while transition.is_some() && frames < 600 {
            frames += 1;
            advance_arrangement(
                &mut canvas,
                &mut transition,
                "grid.default",
                frames as f64 * 16.0,
            );
            canvas.frame(800, 600);
        }
        assert!(transition.is_none(), "it lands");
        let landed = canvas.cartography_geometry();
        let slots = canvas.arrangement_roles().clone();
        assert_eq!(slots.default, Role::Anchored);
        let seeded_at = |geometry: &crate::canvas::CartographyGeometry| {
            geometry
                .iter()
                .find(|(id, _)| *id == seeded)
                .map(|(_, at)| at)
        };
        assert_eq!(seeded_at(&landed), seeded_at(&moved), "seeded stays");
        // Restore puts every item on its arrangement position: the anchored
        // ones were already there, the seeded one was not.
        assert!(canvas.restore_arrangement());
        let home = canvas.cartography_geometry();
        let mut returned = 0;
        for (id, at) in landed.iter() {
            let slot = home.iter().find(|(i, _)| *i == id).unwrap().1;
            let there = (at.0 - slot.0).abs() < 1e-3 && (at.1 - slot.1).abs() < 1e-3;
            if id == seeded {
                assert!(!there, "the seeded item was not returned");
            } else if there {
                returned += 1;
            }
        }
        println!(
            "stop return: {frames} frames, {returned} of {} anchored items home",
            keys.len() - 1
        );
        assert_eq!(returned, keys.len() - 1, "every anchored item landed home");
    }

    /// The law-start fields say "rose" and "fell" only when the layout beats
    /// the start; the same stats read false on both (the control).
    #[test]
    fn law_start_reads_rose_and_fell_only_past_the_start() {
        let start = LawStart {
            mass_area_rank: 0.2,
            density_cv: 0.4,
        };
        let field = |now: &LayoutStats, name: &str| {
            start
                .fields(now)
                .into_iter()
                .find(|(field, _)| *field == name)
                .map(|(_, value)| value)
                .unwrap()
        };
        let better = LayoutStats {
            mass_area_rank: 0.7,
            density_cv: 0.2,
            ..LayoutStats::default()
        };
        let same = LayoutStats {
            mass_area_rank: 0.2,
            density_cv: 0.4,
            ..LayoutStats::default()
        };
        assert_eq!(field(&better, "layout-rank-rose"), "true");
        assert_eq!(field(&better, "layout-cv-fell"), "true");
        assert_eq!(field(&same, "layout-rank-rose"), "false");
        assert_eq!(field(&same, "layout-cv-fell"), "false");
        assert_eq!(field(&same, "law-start-mass-area-rank"), "0.20");
        assert_eq!(field(&same, "law-start-density-cv"), "0.400");
    }
}
