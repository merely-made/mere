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

use super::channels::Channel;
use super::dynamics_spec::{GroupRoles, Node, Target, schedule_node};
use super::physics_catalog::PhysicsLaw;
use super::schedule::{PhysicsStage, StageStop};
use super::{
    CANVAS_LAYOUT_STRATEGIES, Canvas, LayoutStats, OverlayRefusal, PhysicsChoice,
    PhysicsKindSource, PhysicsOverlay, Role, Speed,
};

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

/// What writing a physics choice into a canvas came to: the overlays the law
/// refused (left out of the write) and the error if the canvas refused the
/// spec.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PhysicsWrite {
    pub refusal: Option<OverlayRefusal>,
    pub error: Option<String>,
}

/// Write `choice`, admitted, into the canvas's spec as one change (dynamics
/// grammar plan, F162). The one write both [`apply_physics`] and
/// [`edit_physics`] make.
pub fn write_physics(canvas: &mut Canvas, choice: &PhysicsChoice) -> PhysicsWrite {
    let (choice, refusal) = choice.clone().admitted();
    let running = PhysicsChoice::live(canvas);
    let error = canvas
        .dynamics_spec()
        .map_err(|error| error.to_string())
        .and_then(|mut spec| {
            choice.write_into(&mut spec, &running);
            canvas
                .set_dynamics_spec(&spec)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
        .err();
    PhysicsWrite { refusal, error }
}

/// Change the running flat view and write it back as one spec change, for a
/// host whose controls edit one field at a time. The camera is left alone;
/// [`apply_physics`] is the form's Apply, which also follows the layout.
pub fn edit_physics(canvas: &mut Canvas, edit: impl FnOnce(&mut PhysicsChoice)) -> PhysicsWrite {
    let mut choice = PhysicsChoice::live(canvas);
    edit(&mut choice);
    write_physics(canvas, &choice)
}

/// Apply physics: the picked law, overlays and sources written into the
/// canvas's spec and set as one change (dynamics grammar plan, F162), and the
/// camera follows the layout while it plays (ruled 2026-10-03, "Follow while
/// playing"). Overlays the law refuses are left out, and the status gives
/// the law's reason.
pub fn apply_physics(canvas: &mut Canvas, choice: &PhysicsChoice) -> String {
    let PhysicsWrite { refusal, error } = write_physics(canvas, choice);
    canvas.set_view_follow(true);
    let status = with_overlays(
        format!("Physics set to {}", PhysicsChoice::live(canvas).law.label()),
        canvas,
    );
    match (error, refusal) {
        (Some(error), _) => format!("{status} · {error}"),
        (None, Some(refusal)) => format!("{status} · {}", refusal.reason),
        (None, None) => status,
    }
}

/// The simulation-speed presets, `(id, label)`: the id is the factor, or
/// [`MAX_SPEED`] for as fast as the host's step budget allows.
pub const SPEED_PRESETS: [(&str, &str); 8] = [
    ("0.2", "0.2x"),
    ("0.5", "0.5x"),
    ("1", "1x"),
    ("2", "2x"),
    ("5", "5x"),
    ("10", "10x"),
    ("50", "50x"),
    (MAX_SPEED, "Max"),
];
/// The preset a canvas runs at unless its host says otherwise: 1x.
pub const DEFAULT_SPEED_PRESET: usize = 2;
/// The uncapped preset's id.
pub const MAX_SPEED: &str = "max";

/// The preset showing the canvas's speed; a host may have set any speed, and
/// the nearest preset answers for it.
pub fn speed_preset(canvas: &Canvas) -> usize {
    let speed = canvas.physics_speed();
    if speed == Speed::UNCAPPED {
        return SPEED_PRESETS.len() - 1;
    }
    let factor = speed.factor();
    SPEED_PRESETS[..SPEED_PRESETS.len() - 1]
        .iter()
        .enumerate()
        .min_by(|(_, (a, _)), (_, (b, _))| {
            let distance = |id: &str| (id.parse::<f32>().unwrap_or(1.0).ln() - factor.ln()).abs();
            distance(a).total_cmp(&distance(b))
        })
        .map_or(DEFAULT_SPEED_PRESET, |(index, _)| index)
}

/// A speed as a person reads it: "2x", "0.2x", "3.7x", "Max".
pub fn speed_label(speed: Speed) -> String {
    if speed == Speed::UNCAPPED {
        return "Max".into();
    }
    let text = format!("{:.3}", speed.factor());
    format!("{}x", text.trim_end_matches('0').trim_end_matches('.'))
}

/// Run the canvas at the preset `id` names; the status line to show.
pub fn apply_speed(canvas: &mut Canvas, id: &str) -> Result<String, String> {
    let (id, label) = SPEED_PRESETS
        .iter()
        .find(|(known, _)| *known == id)
        .ok_or_else(|| format!("unknown speed {id}"))?;
    canvas.set_physics_speed(if *id == MAX_SPEED {
        Speed::UNCAPPED
    } else {
        Speed::from_factor(id.parse().map_err(|_| format!("unknown speed {id}"))?)
    });
    Ok(format!("Speed set to {label}"))
}

/// A bounded two-law composition with explicit force shares. Shares must be
/// finite. The dynamics grammar remains responsible for its own admission
/// rules for force weights.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwoLawMix {
    pub first: PhysicsLaw,
    pub second: PhysicsLaw,
    pub first_share: f64,
    pub second_share: f64,
}

/// A bounded two-law grouping: `outer` acts between groups and `inner` acts
/// within each group. The outer share is a force multiplier, not a percentage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwoLawGrouping {
    pub partition: PhysicsKindSource,
    pub outer: PhysicsLaw,
    pub inner: PhysicsLaw,
    pub outer_share: f64,
}

/// The editable parts of one law stage in a two-stage schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageDraft {
    pub law: PhysicsLaw,
    pub stop: StageStop,
    pub capture: Option<Role>,
}

/// The schedule form this control can read and write canonically. Its shared
/// overlays are unique and ordered like `PhysicsOverlay::ALL`, as the editor
/// checkboxes present them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TwoStageSchedule {
    pub first: StageDraft,
    pub second: StageDraft,
    pub overlays: Vec<PhysicsOverlay>,
}

/// Replace the spec root with one admitted two-law force mix. The existing
/// spec remains the authority for channels, target, seed, bars and
/// realization; root overlays are carried onto the mix.
pub fn apply_mix(canvas: &mut Canvas, mix: &TwoLawMix) -> Result<String, String> {
    let mut spec = canvas.dynamics_spec()?;
    let overlays = spec.root.overlays().to_vec();
    spec.root = Node::Mix {
        parts: vec![
            weighted_preset(mix.first, mix.first_share),
            weighted_preset(mix.second, mix.second_share),
        ],
        weight: 1.0,
        overlays,
    };
    // Canvas owns admission and refuses before changing its live state.
    canvas
        .set_dynamics_spec(&spec)
        .map_err(|error| error.to_string())?;
    canvas.set_view_follow(true);
    Ok(format!(
        "Physics mix set to {} + {}",
        mix.first.label(),
        mix.second.label()
    ))
}

/// Replace the spec root with one admitted flat two-law grouping. The current
/// spec remains the authority for channels, target, seed, bars and realization;
/// root overlays are carried onto the grouping.
pub fn apply_grouping(canvas: &mut Canvas, grouping: &TwoLawGrouping) -> Result<String, String> {
    let mut spec = canvas.dynamics_spec()?;
    let overlays = spec.root.overlays().to_vec();
    spec.root = Node::Grouped {
        partition: Channel::Groups(grouping.partition).id(),
        outer: Box::new(weighted_preset(grouping.outer, grouping.outer_share)),
        inner: Box::new(weighted_preset(grouping.inner, 1.0)),
        weight: 1.0,
        overlays,
    };
    // Canvas owns admission and refuses before changing its live state.
    canvas
        .set_dynamics_spec(&spec)
        .map_err(|error| error.to_string())?;
    canvas.set_view_follow(true);
    Ok(format!(
        "Physics grouping set to {} between groups and {} within groups",
        grouping.outer.label(),
        grouping.inner.label()
    ))
}

/// Replace the spec root with two law stages that share one explicit overlay
/// list. Canvas binding admits the whole recipe before changing live state.
/// Applying an already-authored identical schedule restarts it at stage zero.
pub fn apply_two_stage_schedule(
    canvas: &mut Canvas,
    schedule: &TwoStageSchedule,
) -> Result<String, String> {
    for (name, draft) in [("first", schedule.first), ("second", schedule.second)] {
        if draft.stop == StageStop::Frames(0) {
            return Err(format!("{name}.stop must use a positive frame count"));
        }
    }
    let mut previous = None;
    for overlay in &schedule.overlays {
        let index = PhysicsOverlay::ALL
            .iter()
            .position(|candidate| candidate == overlay)
            .expect("a PhysicsOverlay is always in its catalog");
        if let Some(previous) = previous {
            if index == previous {
                return Err(format!("duplicate schedule overlay {}", overlay.id()));
            }
            if index < previous {
                return Err("schedule overlays must use catalog order".into());
            }
        }
        previous = Some(index);
    }
    let stage = |draft: StageDraft| PhysicsStage {
        law: draft.law,
        overlays: schedule.overlays.clone(),
        composition: None,
        stop: draft.stop,
        capture: draft.capture,
    };
    let stages = vec![stage(schedule.first), stage(schedule.second)];
    let root = schedule_node(&stages)?;
    let mut spec = canvas.dynamics_spec()?;
    let restart = spec.root == root;
    spec.root = root;
    canvas
        .set_dynamics_spec(&spec)
        .map_err(|error| error.to_string())?;
    if restart {
        canvas.run_physics_schedule(stages);
    }
    canvas.set_view_follow(true);
    Ok(format!(
        "Two-stage schedule set to {} then {}",
        schedule.first.law.label(),
        schedule.second.law.label()
    ))
}

/// A plain law preset a bounded editor can faithfully expose. Per-part
/// overlays, terms and rungs need richer controls and must not be hidden.
fn plain_part(node: &Node) -> Option<(PhysicsLaw, f64)> {
    match node {
        Node::Preset {
            id,
            weight,
            terms,
            rung,
            overlays,
        } if terms.is_empty() && rung.is_none() && overlays.is_empty() => {
            Some((PhysicsLaw::parse(id)?, *weight))
        },
        _ => None,
    }
}

/// Read the authored root when it is exactly the two-law mix
/// [`apply_mix`] writes. Other roots return `Ok(None)`; a mix with richer
/// parts returns a reason for the host form to show, as
/// [`read_two_stage_schedule`] does.
pub fn read_two_law_mix(canvas: &Canvas) -> Result<Option<TwoLawMix>, String> {
    let spec = canvas.dynamics_spec()?;
    let Node::Mix { parts, weight, .. } = &spec.root else {
        return Ok(None);
    };
    if *weight != 1.0 {
        return Err("root weight".into());
    }
    let [first, second] = parts.as_slice() else {
        return Err("part count".into());
    };
    let (Some((first, first_share)), Some((second, second_share))) =
        (plain_part(first), plain_part(second))
    else {
        return Err("part data".into());
    };
    Ok(Some(TwoLawMix {
        first,
        second,
        first_share,
        second_share,
    }))
}

/// Read the authored root when it is exactly the grouped form
/// [`apply_grouping`] writes; the same three outcomes as [`read_two_law_mix`].
pub fn read_two_law_grouping(canvas: &Canvas) -> Result<Option<TwoLawGrouping>, String> {
    let spec = canvas.dynamics_spec()?;
    let Node::Grouped {
        partition,
        outer,
        inner,
        weight,
        ..
    } = &spec.root
    else {
        return Ok(None);
    };
    if *weight != 1.0 {
        return Err("root weight".into());
    }
    let Some(Channel::Groups(partition)) = Channel::parse(partition) else {
        return Err("partition".into());
    };
    let (Some((outer, outer_share)), Some((inner, inner_share))) =
        (plain_part(outer), plain_part(inner))
    else {
        return Err("law data".into());
    };
    if inner_share != 1.0 {
        return Err("inner weight".into());
    }
    Ok(Some(TwoLawGrouping {
        partition,
        outer,
        inner,
        outer_share,
    }))
}

/// Read a full canvas spec when its schedule is exactly the two-stage form
/// this control edits. Other roots return `Ok(None)`; a richer schedule shape
/// returns a reason for the host form to show.
pub fn read_two_stage_schedule(canvas: &Canvas) -> Result<Option<TwoStageSchedule>, String> {
    let spec = canvas.dynamics_spec()?;
    let Node::Schedule {
        stages,
        weight,
        overlays: root_overlays,
    } = spec.root
    else {
        return Ok(None);
    };
    if weight != 1.0 || !root_overlays.is_empty() {
        return Err("schedule root has unsupported weight or overlays".into());
    }
    if stages.len() != 2 {
        return Err("schedule must have exactly two stages".into());
    }

    fn read_stage(
        stage: &seiche::spec::Stage,
    ) -> Result<(StageDraft, Vec<PhysicsOverlay>), String> {
        let Node::Preset {
            id,
            weight,
            terms,
            rung,
            overlays,
        } = &stage.node
        else {
            return Err("schedule stage must be a plain law preset".into());
        };
        if *weight != 1.0 || !terms.is_empty() || rung.is_some() {
            return Err("schedule stage has unsupported weight, overrides, or rung".into());
        }
        let law =
            PhysicsLaw::parse(id).ok_or_else(|| format!("unknown schedule law preset {id}"))?;
        let mut parsed_overlays = Vec::with_capacity(overlays.len());
        let mut previous_catalog_index = None;
        for overlay in overlays {
            let Node::Preset {
                id,
                weight,
                terms,
                rung,
                overlays,
            } = overlay
            else {
                return Err("schedule overlays must be plain overlay presets".into());
            };
            if *weight != 1.0 || !terms.is_empty() || rung.is_some() || !overlays.is_empty() {
                return Err("schedule overlay has unsupported fields".into());
            }
            let overlay = PhysicsOverlay::parse(id)
                .ok_or_else(|| format!("unknown schedule overlay preset {id}"))?;
            let catalog_index = PhysicsOverlay::ALL
                .iter()
                .position(|candidate| *candidate == overlay)
                .expect("a parsed PhysicsOverlay is in its catalog");
            if let Some(previous) = previous_catalog_index {
                if catalog_index == previous {
                    return Err(format!("duplicate schedule overlay {}", overlay.id()));
                }
                if catalog_index < previous {
                    return Err("schedule overlays must use catalog order".into());
                }
            }
            previous_catalog_index = Some(catalog_index);
            parsed_overlays.push(overlay);
        }
        let stop = match stage.stop {
            seiche::spec::Stop::Rest => StageStop::Rest,
            seiche::spec::Stop::Frames(0) => {
                return Err("schedule frame stop must use a positive count".into());
            },
            seiche::spec::Stop::Frames(n) => StageStop::Frames(n),
            seiche::spec::Stop::LawDone => StageStop::LawDone,
        };
        Ok((
            StageDraft {
                law,
                stop,
                capture: stage.capture,
            },
            parsed_overlays,
        ))
    }

    let (first, overlays) = read_stage(&stages[0])?;
    let (second, second_overlays) = read_stage(&stages[1])?;
    if overlays != second_overlays {
        return Err("schedule stages must share the same overlay list".into());
    }
    Ok(Some(TwoStageSchedule {
        first,
        second,
        overlays,
    }))
}

/// Explicitly replace the root with `choice`'s single law, even when its law
/// and overlays equal the current flat view. Use [`apply_physics`] for edits
/// that should retain a running composition when only sources change.
pub fn apply_single_law(canvas: &mut Canvas, choice: &PhysicsChoice) -> Result<String, String> {
    let (choice, refused) = choice.clone().admitted();
    let mut spec = canvas.dynamics_spec()?;
    spec.channels = choice.sources().channels();
    spec.root = weighted_preset(choice.law, 1.0);
    if let Node::Preset { overlays, .. } = &mut spec.root {
        overlays.extend(
            choice
                .overlays
                .iter()
                .map(|overlay| Node::preset(overlay.id())),
        );
    }
    canvas
        .set_dynamics_spec(&spec)
        .map_err(|error| error.to_string())?;
    canvas.set_view_follow(true);
    let status = with_overlays(format!("Physics set to {}", choice.law.label()), canvas);
    Ok(match refused {
        Some(refusal) => format!("{status} · {}", refusal.reason),
        None => status,
    })
}

/// Set the target's default arrangement role through the canonical dynamics
/// record. The Canvas resolves a missing target from its current live state.
pub fn apply_role(canvas: &mut Canvas, role: Role) -> Result<String, String> {
    edit_role_target(canvas, |target| target.default_role = role)?;
    Ok(format!("Role set to {}", role.id()))
}

/// Read target group role overrides from the selected groups channel.
pub fn apply_role_group_source(
    canvas: &mut Canvas,
    source: PhysicsKindSource,
) -> Result<String, String> {
    edit_role_target(canvas, |target| {
        if target
            .groups
            .as_ref()
            .is_some_and(|groups| groups.channel == Channel::Groups(source).id())
        {
            return;
        }
        let roles = target
            .groups
            .take()
            .map(|groups| groups.roles)
            .unwrap_or_default();
        target.groups =
            (source != PhysicsKindSource::Site || !roles.is_empty()).then(|| GroupRoles {
                channel: Channel::Groups(source).id(),
                roles,
            });
    })?;
    Ok(format!("Group roles read from {}", source.id()))
}

fn edit_role_target(canvas: &mut Canvas, edit: impl FnOnce(&mut Target)) -> Result<(), String> {
    let mut spec = canvas.dynamics_spec()?;
    let mut target = spec
        .target
        .take()
        .unwrap_or_else(|| canvas.running_target());
    let resolved_before = target.clone();
    edit(&mut target);
    if target == resolved_before {
        return Ok(());
    }
    spec.target = Some(target);
    canvas
        .set_dynamics_spec(&spec)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn weighted_preset(law: PhysicsLaw, share: f64) -> Node {
    let mut node = Node::preset(law.id());
    if let Node::Preset { weight, .. } = &mut node {
        *weight = share;
    }
    node
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
        match ArrangementTransition::returning(canvas, &stop) {
            Ok(pending) => *transition = pending,
            Err(reason) => {
                *transition = None;
                return Some(format!("Arrangement cancelled: {reason}"));
            },
        }
    }
    if transition
        .as_ref()
        .is_some_and(|pending| !pending.source.is_current(canvas))
    {
        *transition = None;
        return Some("Arrangement cancelled: its graph or view changed".into());
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
    source: super::host_results::LayoutResultSource,
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
        if !stop.source.is_current(canvas) {
            return Err("Stop return belongs to a superseded graph or view".into());
        }
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
            source: super::host_results::LayoutResultSource::capture(canvas),
            schedule,
            clock: ProjectionClock::default(),
            node_of,
            start_positions,
            final_positions: final_positions.to_vec(),
            returning,
        }))
    }

    /// Sample positions at host time `host_ms`. This is only schedule sampling;
    /// hosts install through `advance_arrangement`, which revalidates the source.
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
    fn edit_physics_changes_the_running_view_without_following_the_layout() {
        let mut canvas = canvas();
        canvas.set_view_follow(false);
        let write = edit_physics(&mut canvas, |choice| choice.law = PhysicsLaw::Charge);
        assert_eq!(write, PhysicsWrite::default());
        assert_eq!(PhysicsChoice::live(&canvas).law, PhysicsLaw::Charge);
        assert!(!canvas.view_follows(), "an edit leaves the camera alone");
    }

    #[test]
    fn edit_physics_reports_the_overlays_the_law_refuses() {
        let mut canvas = canvas();
        let refused = PhysicsOverlay::ALL.into_iter().find_map(|overlay| {
            let mut choice = PhysicsChoice::live(&canvas);
            choice.overlays = vec![overlay];
            choice.admitted().1.map(|_| overlay)
        });
        let Some(overlay) = refused else { return };
        let write = edit_physics(&mut canvas, |choice| choice.overlays = vec![overlay]);
        assert!(write.refusal.is_some());
        assert!(write.error.is_none());
        assert!(!PhysicsChoice::live(&canvas).overlays.contains(&overlay));
    }

    #[test]
    fn mix_and_grouping_read_back_what_apply_wrote() {
        let mut canvas = canvas();
        assert_eq!(read_two_law_mix(&canvas), Ok(None));
        assert_eq!(read_two_law_grouping(&canvas), Ok(None));
        let mix = TwoLawMix {
            first: PhysicsLaw::Charge,
            second: PhysicsLaw::Springs,
            first_share: 0.75,
            second_share: 2.0,
        };
        apply_mix(&mut canvas, &mix).unwrap();
        assert_eq!(read_two_law_mix(&canvas), Ok(Some(mix)));
        assert_eq!(read_two_law_grouping(&canvas), Ok(None));
        let grouping = TwoLawGrouping {
            partition: PhysicsKindSource::ALL[0],
            outer: PhysicsLaw::Charge,
            inner: PhysicsLaw::Springs,
            outer_share: 1.5,
        };
        apply_grouping(&mut canvas, &grouping).unwrap();
        assert_eq!(read_two_law_grouping(&canvas), Ok(Some(grouping)));
        assert_eq!(read_two_law_mix(&canvas), Ok(None));
    }

    #[test]
    fn speed_presets_round_trip_through_the_canvas() {
        let mut canvas = canvas();
        for (index, (id, label)) in SPEED_PRESETS.iter().enumerate() {
            assert_eq!(
                apply_speed(&mut canvas, id).unwrap(),
                format!("Speed set to {label}")
            );
            assert_eq!(speed_preset(&canvas), index);
            assert_eq!(speed_label(canvas.physics_speed()), *label);
        }
        assert!(apply_speed(&mut canvas, "3").is_err());
        canvas.set_physics_speed(Speed::from_factor(3.7));
        assert_eq!(speed_label(canvas.physics_speed()), "3.7x");
    }

    #[test]
    fn typed_mix_is_canonical_and_source_edits_keep_it_until_explicit_replacement() {
        use crate::canvas::dynamics_spec::{Bar, Observable, Realization, Target};

        let mut subject = canvas();
        let mut original = subject.dynamics_spec().unwrap();
        original.seed = 73;
        original.bars = vec![Bar {
            observable: Observable::Overlaps,
            at_least: None,
            at_most: Some(3.0),
        }];
        original.realization = Realization::Integrate {
            damping: Some(0.35),
        };
        original
            .channels
            .insert("kind".into(), "kind.degree".into());
        // Keep a target on the record through the composition write.
        original.target = Some(Target {
            arrangement: "grid.default".into(),
            anchored_pull: 0.3,
            default_role: seiche::Role::Anchored,
            groups: None,
            items: Default::default(),
        });
        subject.set_dynamics_spec(&original).unwrap();

        apply_mix(
            &mut subject,
            &TwoLawMix {
                first: PhysicsLaw::Springs,
                second: PhysicsLaw::Charge,
                first_share: 0.25,
                second_share: 2.0,
            },
        )
        .unwrap();
        let mixed = subject.dynamics_spec().unwrap();
        let Node::Mix { parts, .. } = &mixed.root else {
            panic!("typed action writes a mix root")
        };
        assert_eq!(parts[0].weight(), 0.25);
        assert_eq!(parts[1].weight(), 2.0);
        assert_eq!(mixed.seed, original.seed);
        assert_eq!(mixed.bars, original.bars);
        assert_eq!(mixed.target, original.target);
        assert_eq!(mixed.realization, original.realization);
        assert_eq!(mixed.channels, original.channels);

        // A source-only edit through the existing picker keeps the mix.
        let mut choice = PhysicsChoice::live(&subject);
        choice.kind = PhysicsKindSource::Component;
        apply_physics(&mut subject, &choice);
        let edited = subject.dynamics_spec().unwrap();
        assert_eq!(edited.root, mixed.root);
        assert_eq!(edited.channels["kind"], "kind.component");

        // A save and reopen round trip preserves the authored spec.
        let mut reopened = canvas();
        reopened.set_dynamics_spec(&edited).unwrap();
        assert_eq!(reopened.dynamics_spec().unwrap(), edited);

        // The explicit single-law action clears a composition even though
        // the flat law picker still shows Springs.
        let choice = PhysicsChoice::live(&subject);
        apply_single_law(&mut subject, &choice).unwrap();
        assert!(matches!(
            subject.dynamics_spec().unwrap().root,
            Node::Preset { .. }
        ));
    }

    #[test]
    fn refused_mix_is_atomic_for_the_record_and_live_positions() {
        let mut canvas = canvas();
        let before = canvas.dynamics_spec().unwrap();
        let mut positions: Vec<_> = canvas.view.positions().collect();
        positions.sort_by_key(|(key, _)| key.index());
        let result = apply_mix(
            &mut canvas,
            &TwoLawMix {
                first: PhysicsLaw::Anneal,
                second: PhysicsLaw::Springs,
                first_share: 1.0,
                second_share: 1.0,
            },
        );
        let refusal = result.unwrap_err();
        assert!(
            refusal.contains("root.parts[0]"),
            "exact refusal path: {refusal}"
        );
        assert_eq!(canvas.dynamics_spec().unwrap(), before);
        let mut after: Vec<_> = canvas.view.positions().collect();
        after.sort_by_key(|(key, _)| key.index());
        assert_eq!(after, positions);

        for (first_share, second_share) in [
            (f64::NAN, 1.0),
            (1.0, f64::INFINITY),
            (1e300, 1.0),
            (1.0, 1e-300),
        ] {
            assert!(
                apply_mix(
                    &mut canvas,
                    &TwoLawMix {
                        first: PhysicsLaw::Springs,
                        second: PhysicsLaw::Charge,
                        first_share,
                        second_share,
                    }
                )
                .is_err()
            );
            assert_eq!(canvas.dynamics_spec().unwrap(), before);
        }
    }

    #[test]
    fn typed_grouping_is_canonical_and_keeps_its_partition_across_source_edits() {
        use crate::canvas::dynamics_spec::{Bar, GroupRoles, Observable, Realization, Target};
        use crate::canvas::{Channel, PhysicsOverlay, Role};
        use std::collections::BTreeMap;

        let mut subject = canvas();
        let mut original = subject.dynamics_spec().unwrap();
        original.seed = 91;
        original.bars = vec![Bar {
            observable: Observable::Overlaps,
            at_least: None,
            at_most: Some(4.0),
        }];
        original.realization = Realization::Integrate {
            damping: Some(0.27),
        };
        original
            .channels
            .insert("kind".into(), "kind.degree".into());
        original
            .channels
            .insert("groups".into(), "groups.degree".into());
        original.target = Some(Target {
            arrangement: "grid.default".into(),
            anchored_pull: 0.3,
            default_role: Role::Anchored,
            groups: Some(GroupRoles {
                channel: Channel::Groups(PhysicsKindSource::Site).id(),
                roles: BTreeMap::from([("groups.site#example".into(), Role::Pinned)]),
            }),
            items: BTreeMap::from([(
                "00000000-0000-0000-0000-000000000001".into(),
                Role::Anchored,
            )]),
        });
        original.root = Node::preset(PhysicsLaw::Springs.id());
        original
            .root
            .overlays_mut()
            .push(Node::preset(PhysicsOverlay::GravityLocus.id()));
        subject.set_dynamics_spec(&original).unwrap();

        let share = 0.100_000_000_000_000_02_f64;
        apply_grouping(
            &mut subject,
            &TwoLawGrouping {
                partition: PhysicsKindSource::Cluster,
                outer: PhysicsLaw::Charge,
                inner: PhysicsLaw::Springs,
                outer_share: share,
            },
        )
        .unwrap();

        let grouped = subject.dynamics_spec().unwrap();
        let Node::Grouped {
            partition,
            outer,
            inner,
            weight,
            overlays,
        } = &grouped.root
        else {
            panic!("typed action writes a flat grouped root")
        };
        assert_eq!(partition, &Channel::Groups(PhysicsKindSource::Cluster).id());
        assert_eq!(outer.weight(), share, "authored f64 share is retained");
        assert_eq!(inner.weight(), 1.0);
        assert_eq!(*weight, 1.0);
        assert_eq!(overlays, original.root.overlays());
        assert_eq!(grouped.seed, original.seed);
        assert_eq!(grouped.bars, original.bars);
        assert_eq!(grouped.target, original.target);
        assert_eq!(grouped.realization, original.realization);
        assert_eq!(grouped.channels, original.channels);

        // The group's partition, Group pull source and role-group source are
        // independent. Changing the latter two leaves the authored grouping
        // and role target intact.
        let mut choice = PhysicsChoice::live(&subject);
        choice.groups = PhysicsKindSource::Component;
        choice.kind = PhysicsKindSource::Component;
        apply_physics(&mut subject, &choice);
        let edited = subject.dynamics_spec().unwrap();
        assert_eq!(edited.root, grouped.root);
        assert_eq!(
            edited
                .target
                .as_ref()
                .unwrap()
                .groups
                .as_ref()
                .unwrap()
                .channel,
            Channel::Groups(PhysicsKindSource::Site).id()
        );
        assert_eq!(edited.channels["groups"], "groups.component");

        let mut reopened = canvas();
        reopened.set_dynamics_spec(&edited).unwrap();
        assert_eq!(reopened.dynamics_spec().unwrap(), edited);

        // The explicit single-law action clears a grouping, even though the
        // flat law picker continues to show its outer law.
        let choice = PhysicsChoice::live(&subject);
        apply_single_law(&mut subject, &choice).unwrap();
        assert!(matches!(
            subject.dynamics_spec().unwrap().root,
            Node::Preset { .. }
        ));
    }

    #[test]
    fn refused_grouping_is_atomic_for_the_record_and_live_positions() {
        let mut subject = canvas();
        let before = subject.dynamics_spec().unwrap();
        let mut positions: Vec<_> = subject.view.positions().collect();
        positions.sort_by_key(|(key, _)| key.index());

        let refusal = apply_grouping(
            &mut subject,
            &TwoLawGrouping {
                partition: PhysicsKindSource::Site,
                outer: PhysicsLaw::Anneal,
                inner: PhysicsLaw::Springs,
                outer_share: 16.0,
            },
        )
        .unwrap_err();
        assert!(
            refusal.contains("root.outer"),
            "exact refusal path: {refusal}"
        );
        assert_eq!(subject.dynamics_spec().unwrap(), before);

        let refusal = apply_grouping(
            &mut subject,
            &TwoLawGrouping {
                partition: PhysicsKindSource::Site,
                outer: PhysicsLaw::Charge,
                inner: PhysicsLaw::Anneal,
                outer_share: 16.0,
            },
        )
        .unwrap_err();
        assert!(
            refusal.contains("root.inner"),
            "exact refusal path: {refusal}"
        );
        assert_eq!(subject.dynamics_spec().unwrap(), before);

        let refusal = apply_grouping(
            &mut subject,
            &TwoLawGrouping {
                partition: PhysicsKindSource::Site,
                outer: PhysicsLaw::Charge,
                inner: PhysicsLaw::Springs,
                outer_share: -1.0,
            },
        )
        .unwrap_err();
        assert!(
            refusal.contains("root.outer"),
            "exact refusal path: {refusal}"
        );
        assert_eq!(subject.dynamics_spec().unwrap(), before);

        for outer_share in [f64::NAN, f64::INFINITY, 1e300, 1e-300] {
            let error = apply_grouping(
                &mut subject,
                &TwoLawGrouping {
                    partition: PhysicsKindSource::Site,
                    outer: PhysicsLaw::Charge,
                    inner: PhysicsLaw::Springs,
                    outer_share,
                },
            )
            .unwrap_err();
            assert!(error.contains("root.outer.weight"), "{error}");
            assert_eq!(subject.dynamics_spec().unwrap(), before);
        }

        let mut after: Vec<_> = subject.view.positions().collect();
        after.sort_by_key(|(key, _)| key.index());
        assert_eq!(after, positions);
    }

    #[test]
    fn role_edits_preserve_authored_root_and_every_other_spec_field() {
        use crate::canvas::dynamics_spec::{Bar, GroupRoles, Observable, Realization, Target};
        use crate::canvas::{Channel, Role};
        use std::collections::BTreeMap;

        let mut subject = canvas();
        let ids: Vec<_> = subject
            .graph()
            .nodes()
            .map(|(_, node)| node.id.to_string())
            .collect();
        let keys: Vec<_> = subject.graph().nodes().map(|(key, _)| key).collect();
        let absent_id = uuid::Uuid::from_u128(u128::MAX).to_string();
        assert!(
            !ids.contains(&absent_id),
            "absent override is not a fixture member"
        );
        let mut original = subject.dynamics_spec().unwrap();
        original.root = Node::Mix {
            parts: vec![
                weighted_preset(PhysicsLaw::Springs, 0.1),
                weighted_preset(PhysicsLaw::Charge, 0.3),
            ],
            weight: 1.0,
            overlays: vec![],
        };
        original.seed = 918;
        original.bars = vec![Bar {
            observable: Observable::Overlaps,
            at_least: Some(0.25),
            at_most: Some(4.5),
        }];
        original.realization = Realization::Integrate {
            damping: Some(0.375),
        };
        original.target = Some(Target {
            arrangement: "authored-layout".into(),
            anchored_pull: 0.271,
            default_role: Role::Seeded,
            groups: Some(GroupRoles {
                channel: Channel::Groups(PhysicsKindSource::Cluster).id(),
                roles: BTreeMap::from([("kept-label".into(), Role::Pinned)]),
            }),
            items: BTreeMap::from([(ids[0].clone(), Role::Seeded), (absent_id, Role::Pinned)]),
        });
        original
            .channels
            .insert("kind".into(), "kind.degree".into());
        subject.set_dynamics_spec(&original).unwrap();

        assert_eq!(
            apply_role(&mut subject, Role::Anchored).unwrap(),
            "Role set to anchored"
        );
        let edited = subject.dynamics_spec().unwrap();
        assert_eq!(edited.root, original.root, "f64 mix shares remain authored");
        assert_eq!(edited.seed, original.seed);
        assert_eq!(edited.bars, original.bars);
        assert_eq!(edited.realization, original.realization);
        assert_eq!(edited.channels, original.channels);
        let target = edited.target.unwrap();
        let old_target = original.target.unwrap();
        assert_eq!(target.arrangement, old_target.arrangement);
        assert_eq!(target.anchored_pull, old_target.anchored_pull);
        assert_eq!(target.default_role, Role::Anchored);
        assert_eq!(target.groups, old_target.groups);
        assert_eq!(target.items, old_target.items);
        assert_eq!(
            subject.arrangement_role_of(keys[0]),
            Role::Seeded,
            "item override wins"
        );
        assert_eq!(
            subject.arrangement_role_of(keys[1]),
            Role::Anchored,
            "unmatched group label falls through to the default"
        );

        apply_role_group_source(&mut subject, PhysicsKindSource::Component).unwrap();
        let source_edited = subject.dynamics_spec().unwrap();
        assert_eq!(source_edited.root, original.root);
        assert_eq!(source_edited.seed, original.seed);
        assert_eq!(source_edited.bars, original.bars);
        assert_eq!(source_edited.realization, original.realization);
        assert_eq!(source_edited.channels, original.channels);
        let source_target = source_edited.target.unwrap();
        assert_eq!(source_target.arrangement, "authored-layout");
        assert_eq!(source_target.anchored_pull, 0.271);
        assert_eq!(source_target.default_role, Role::Anchored);
        assert_eq!(
            source_target.groups.unwrap(),
            GroupRoles {
                channel: Channel::Groups(PhysicsKindSource::Component).id(),
                roles: old_target.groups.unwrap().roles,
            }
        );
        assert_eq!(source_target.items, old_target.items);
    }

    #[test]
    fn group_source_edits_materialize_resolved_target_and_keep_override_labels() {
        use crate::canvas::Channel;
        use crate::canvas::dynamics_spec::GroupRoles;

        let mut subject = canvas();
        // A missing authored target starts from the Canvas's resolved state
        // after the setter applies the target-less spec.
        let mut spec = subject.dynamics_spec().unwrap();
        spec.target = None;
        subject.set_dynamics_spec(&spec).unwrap();
        let resolved = subject.running_target();
        apply_role_group_source(&mut subject, PhysicsKindSource::Cluster).unwrap();
        let switched = subject.dynamics_spec().unwrap();
        assert_eq!(
            switched.target.as_ref().unwrap().default_role,
            resolved.default_role
        );
        assert_eq!(switched.target.as_ref().unwrap().items, resolved.items);
        assert_eq!(
            switched.target.unwrap().groups,
            Some(GroupRoles {
                channel: Channel::Groups(PhysicsKindSource::Cluster).id(),
                roles: resolved
                    .groups
                    .as_ref()
                    .map(|groups| groups.roles.clone())
                    .unwrap_or_default(),
            })
        );

        let mut no_op = canvas();
        let default_role = no_op.running_target().default_role;
        let mut targetless = no_op.dynamics_spec().unwrap();
        targetless.target = None;
        no_op.set_dynamics_spec(&targetless).unwrap();
        apply_role(&mut no_op, default_role).unwrap();
        assert!(no_op.dynamics_spec().unwrap().target.is_none());
        apply_role_group_source(&mut no_op, PhysicsKindSource::Site).unwrap();
        assert!(no_op.dynamics_spec().unwrap().target.is_none());

        // Empty site overrides keep the compact Site representation; every
        // other source must carry an explicit groups channel.
        let mut empty = canvas();
        assert!(
            empty
                .dynamics_spec()
                .unwrap()
                .target
                .unwrap()
                .groups
                .is_none()
        );
        apply_role_group_source(&mut empty, PhysicsKindSource::Cluster).unwrap();
        assert_eq!(
            empty
                .dynamics_spec()
                .unwrap()
                .target
                .unwrap()
                .groups
                .unwrap()
                .channel,
            Channel::Groups(PhysicsKindSource::Cluster).id()
        );
        apply_role_group_source(&mut empty, PhysicsKindSource::Site).unwrap();
        assert!(
            empty
                .dynamics_spec()
                .unwrap()
                .target
                .unwrap()
                .groups
                .is_none()
        );

        let mut explicit_site = empty.dynamics_spec().unwrap();
        explicit_site.target.as_mut().unwrap().groups = Some(GroupRoles {
            channel: Channel::Groups(PhysicsKindSource::Site).id(),
            roles: std::collections::BTreeMap::new(),
        });
        empty.set_dynamics_spec(&explicit_site).unwrap();
        let explicit_site = empty.dynamics_spec().unwrap();
        apply_role_group_source(&mut empty, PhysicsKindSource::Site).unwrap();
        assert_eq!(empty.dynamics_spec().unwrap(), explicit_site);
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
        let seeded_at = |geometry: &pandect::CartographyGeometry| {
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

    fn two_stage(overlays: Vec<PhysicsOverlay>) -> TwoStageSchedule {
        TwoStageSchedule {
            first: StageDraft {
                law: PhysicsLaw::Springs,
                stop: StageStop::Frames(12),
                capture: Some(Role::Anchored),
            },
            second: StageDraft {
                law: PhysicsLaw::Charge,
                stop: StageStop::LawDone,
                capture: None,
            },
            overlays,
        }
    }

    #[test]
    fn two_stage_schedule_replaces_only_the_root_and_reads_back() {
        use crate::canvas::dynamics_spec::{Bar, Observable, Realization};

        let mut subject = canvas();
        let mut original = subject.dynamics_spec().unwrap();
        original.seed = 73;
        original.bars = vec![Bar {
            observable: Observable::Overlaps,
            at_least: Some(1.0),
            at_most: Some(4.0),
        }];
        original.realization = Realization::Integrate {
            damping: Some(0.35),
        };
        original
            .channels
            .insert("kind".into(), "kind.degree".into());
        subject.set_dynamics_spec(&original).unwrap();

        let schedule = two_stage(vec![PhysicsOverlay::GravityLocus]);
        apply_two_stage_schedule(&mut subject, &schedule).unwrap();
        let applied = subject.dynamics_spec().unwrap();
        assert_eq!(applied.seed, original.seed);
        assert_eq!(applied.bars, original.bars);
        assert_eq!(applied.realization, original.realization);
        assert_eq!(applied.channels, original.channels);
        assert_eq!(applied.target, original.target);
        assert!(matches!(
            &applied.root,
            Node::Schedule {
                weight: 1.0,
                overlays,
                stages,
            } if overlays.is_empty()
                && stages.len() == 2
                && stages.iter().all(|stage| stage.node.overlays().len() == 1)
        ));
        assert_eq!(read_two_stage_schedule(&subject).unwrap(), Some(schedule));
    }

    #[test]
    fn refused_shared_overlay_is_atomic_for_spec_and_positions() {
        let mut subject = canvas();
        let before = subject.dynamics_spec().unwrap();
        let mut positions: Vec<_> = subject.view.positions().collect();
        positions.sort_by_key(|(key, _)| key.index());
        let mut schedule = two_stage(vec![PhysicsOverlay::GridSnap]);
        schedule.second.law = PhysicsLaw::Density;

        let error = apply_two_stage_schedule(&mut subject, &schedule).unwrap_err();
        assert!(
            error.contains("overlay") || error.contains("Density"),
            "the shared overlay refusal is surfaced: {error}"
        );
        assert_eq!(subject.dynamics_spec().unwrap(), before);
        let mut after: Vec<_> = subject.view.positions().collect();
        after.sort_by_key(|(key, _)| key.index());
        assert_eq!(after, positions);
    }

    #[test]
    fn editor_rejects_zero_frames_and_noncanonical_overlays_atomically() {
        let mut subject = canvas();
        subject.set_view_follow(false);
        subject.set_physics_paused(true);
        let before = subject.dynamics_spec().unwrap();
        let mut positions: Vec<_> = subject.view.positions().collect();
        positions.sort_by_key(|(key, _)| key.index());

        let mut zero_frames = two_stage(Vec::new());
        zero_frames.first.stop = StageStop::Frames(0);
        let duplicate = two_stage(vec![
            PhysicsOverlay::GravityLocus,
            PhysicsOverlay::GravityLocus,
        ]);
        let reversed = two_stage(vec![
            PhysicsOverlay::Skeleton,
            PhysicsOverlay::DegreeRepulsion,
        ]);
        for schedule in [zero_frames, duplicate, reversed] {
            assert!(apply_two_stage_schedule(&mut subject, &schedule).is_err());
            assert_eq!(subject.dynamics_spec().unwrap(), before);
            assert!(subject.physics_paused());
            assert!(!subject.view_follows());
            assert_eq!(subject.physics_schedule_stage(), None);
            let mut after: Vec<_> = subject.view.positions().collect();
            after.sort_by_key(|(key, _)| key.index());
            assert_eq!(after, positions);
        }
    }

    #[test]
    fn schedule_apply_starts_and_source_or_role_edits_keep_progress() {
        let mut subject = canvas();
        subject.set_physics_paused(true);
        let mut schedule = two_stage(Vec::new());
        schedule.first.stop = StageStop::Frames(1);
        schedule.second.stop = StageStop::Frames(10_000);
        apply_two_stage_schedule(&mut subject, &schedule).unwrap();
        assert_eq!(subject.physics_schedule_stage(), Some(0));
        assert!(!subject.physics_paused(), "Apply and run starts ticking");
        subject.step_layout();
        assert_eq!(subject.physics_schedule_stage(), Some(1));

        subject.set_physics_paused(true);
        let mut choice = PhysicsChoice::live(&subject);
        choice.kind = PhysicsKindSource::Degree;
        apply_physics(&mut subject, &choice);
        assert_eq!(subject.physics_schedule_stage(), Some(1));
        assert!(subject.physics_paused());
        apply_role(&mut subject, Role::Pinned).unwrap();
        assert_eq!(subject.physics_schedule_stage(), Some(1));
        assert!(subject.physics_paused());

        apply_two_stage_schedule(&mut subject, &schedule).unwrap();
        assert_eq!(subject.physics_schedule_stage(), Some(0));
        assert!(
            !subject.physics_paused(),
            "reapplying restarts the schedule"
        );
    }

    #[test]
    fn schedule_reader_refuses_richer_schedule_shapes_without_mutation() {
        use crate::canvas::dynamics_spec::{Node, Stage, Stop};

        let mut subject = canvas();
        let mut spec = subject.dynamics_spec().unwrap();
        spec.root = Node::Schedule {
            stages: vec![
                Stage {
                    node: Node::preset(PhysicsLaw::Springs.id()),
                    stop: Stop::Rest,
                    capture: None,
                },
                Stage {
                    node: Node::preset(PhysicsLaw::Charge.id()),
                    stop: Stop::Frames(12),
                    capture: None,
                },
                Stage {
                    node: Node::preset(PhysicsLaw::Stress.id()),
                    stop: Stop::LawDone,
                    capture: None,
                },
            ],
            weight: 1.0,
            overlays: Vec::new(),
        };
        subject.set_dynamics_spec(&spec).unwrap();
        let before = subject.dynamics_spec().unwrap();
        let error = read_two_stage_schedule(&subject).unwrap_err();
        assert!(error.contains("exactly two"), "{error}");
        assert_eq!(subject.dynamics_spec().unwrap(), before);
        assert_eq!(read_two_stage_schedule(&canvas()).unwrap(), None);
    }

    #[test]
    fn schedule_reader_rejects_zero_frames_and_non_catalog_overlay_order() {
        use crate::canvas::dynamics_spec::{Node, Stage, Stop};

        let mut subject = canvas();
        let make_root = |first_stop, overlay_ids: &[PhysicsOverlay]| Node::Schedule {
            stages: [PhysicsLaw::Springs, PhysicsLaw::Charge]
                .into_iter()
                .enumerate()
                .map(|(i, law)| {
                    let mut node = Node::preset(law.id());
                    node.overlays_mut()
                        .extend(overlay_ids.iter().map(|overlay| Node::preset(overlay.id())));
                    Stage {
                        node,
                        stop: if i == 0 { first_stop } else { Stop::Rest },
                        capture: None,
                    }
                })
                .collect(),
            weight: 1.0,
            overlays: Vec::new(),
        };
        for (root, expected_error) in [
            (make_root(Stop::Frames(0), &[]), "positive count"),
            (
                make_root(
                    Stop::Rest,
                    &[PhysicsOverlay::Skeleton, PhysicsOverlay::DegreeRepulsion],
                ),
                "catalog order",
            ),
        ] {
            let mut spec = subject.dynamics_spec().unwrap();
            spec.root = root;
            subject.set_dynamics_spec(&spec).unwrap();
            let before = subject.dynamics_spec().unwrap();
            let error = read_two_stage_schedule(&subject).unwrap_err();
            assert!(error.contains(expected_error), "{error}");
            assert_eq!(subject.dynamics_spec().unwrap(), before);
        }
    }
}
