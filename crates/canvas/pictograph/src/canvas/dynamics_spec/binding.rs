// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// A root or a stage: a law, or a mix or grouping, with overlays.
pub(super) fn bind_node(
    node: &Node,
    place: &str,
) -> Result<(PhysicsLaw, Option<PhysicsComposition>, Vec<PhysicsOverlay>), BindError> {
    let overlays = bound_overlays(node, place)?;
    match node {
        Node::Preset { .. } => Ok((preset_law(node, place)?.0, None, overlays)),
        Node::Mix { parts, .. } => {
            let laws = parts
                .iter()
                .enumerate()
                .map(|(i, part)| {
                    let part_place = format!("{place}.parts[{i}]");
                    let (law, weight) = preset_law(part, &part_place)?;
                    Ok((law, runtime_f32(weight, &format!("{part_place}.weight"))?))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((laws[0].0, Some(PhysicsComposition::Mix(laws)), overlays))
        },
        Node::Grouped {
            partition,
            outer,
            inner,
            ..
        } => {
            let groups = groups_source(partition, &format!("{place}.partition"))?;
            let (outer, outer_weight) = preset_law(outer, &format!("{place}.outer"))?;
            let outer_weight = runtime_f32(outer_weight, &format!("{place}.outer.weight"))?;
            let (inner, _) = preset_law(inner, &format!("{place}.inner"))?;
            Ok((
                outer,
                Some(PhysicsComposition::Grouped(PhysicsGrouping {
                    groups: GroupSource::Channel(groups),
                    outer,
                    inner,
                    outer_weight,
                })),
                overlays,
            ))
        },
        Node::Raw { .. } | Node::Schedule { .. } => Err(unbound(
            place,
            "the canvas runs a law, a mix or a grouping here",
        )),
    }
}

pub(super) fn bind_stop(stop: Stop) -> StageStop {
    match stop {
        Stop::Rest => StageStop::Rest,
        Stop::Frames(n) => StageStop::Frames(n),
        Stop::LawDone => StageStop::LawDone,
    }
}

pub(super) fn spec_stop(stop: StageStop) -> Stop {
    match stop {
        StageStop::Rest => Stop::Rest,
        StageStop::Frames(n) => Stop::Frames(n),
        StageStop::LawDone => Stop::LawDone,
    }
}

pub(super) fn bind_target(target: &Target) -> Result<BoundTarget, BindError> {
    let (group_source, groups) = match &target.groups {
        Some(groups) => (
            groups_source(&groups.channel, "target.groups.channel")?,
            groups.roles.clone(),
        ),
        None => (PhysicsKindSource::Site, BTreeMap::new()),
    };
    let items = target
        .items
        .iter()
        .map(|(id, role)| {
            uuid::Uuid::parse_str(id)
                .map(|id| (id, *role))
                .map_err(|_| {
                    unbound(
                        format!("target.items.{id}"),
                        format!("{id} is not an item id"),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let anchored_pull = runtime_f32(target.anchored_pull, "target.anchored_pull")?;
    Ok(BoundTarget {
        arrangement: target.arrangement.clone(),
        anchored_pull,
        default_role: target.default_role,
        group_source,
        groups,
        items,
    })
}

/// Read `spec` into what the canvas runs (see the module docs); graph-free,
/// so a host checks a scene's spec before it opens anything.
pub fn bind(spec: &DynamicsSpec) -> Result<Bound, BindError> {
    validate_runtime_fields(spec)?;
    derive(spec).map_err(BindError::Refused)?;
    bind_shape(spec)
}

/// [`bind`] past derive: the channels, the root's shape and the target.
pub(crate) fn bind_shape(spec: &DynamicsSpec) -> Result<Bound, BindError> {
    validate_runtime_fields(spec)?;
    let sources = resolve_channels(&spec.channels)?;
    let root = match &spec.root {
        Node::Schedule { stages, .. } => BoundRoot::Schedule(
            stages
                .iter()
                .enumerate()
                .map(|(i, stage)| {
                    let (law, composition, overlays) =
                        bind_node(&stage.node, &format!("root.stages[{i}].node"))?;
                    Ok(PhysicsStage {
                        law,
                        overlays,
                        composition,
                        stop: bind_stop(stage.stop),
                        capture: stage.capture,
                    })
                })
                .collect::<Result<_, BindError>>()?,
        ),
        node => match bind_node(node, "root")? {
            (law, None, overlays) => BoundRoot::Law { law, overlays },
            (law, Some(composition), overlays) => BoundRoot::Composed {
                law,
                composition,
                overlays,
            },
        },
    };
    let Realization::Integrate { damping } = spec.realization;
    Ok(Bound {
        sources,
        seed: spec.seed,
        root,
        target: spec.target.as_ref().map(bind_target).transpose()?,
        damping,
    })
}
