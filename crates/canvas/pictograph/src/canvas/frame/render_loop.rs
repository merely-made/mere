// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::time::Duration;

use super::super::elapsed::FrameElapsed;
use super::*;

impl Canvas {
    pub(in super::super) fn frame_observed(
        &mut self,
        w: u32,
        h: u32,
        elapsed: Option<FrameElapsed>,
        observer: &mut impl super::super::frame_profile::Observer,
    ) -> (Scene, bool) {
        let (w, h) = (w.max(1), h.max(1));
        self.view_w = w;
        self.view_h = h;
        let viewport = DeviceIntSize::new(w as i32, h as i32);
        // Host time for this frame, or one tick when the host gives none.
        let dt = elapsed
            .as_ref()
            .map_or(seiche::TICK_DT, |elapsed| elapsed.physics.as_secs_f32());
        let presentation_dt = elapsed
            .as_ref()
            .map_or(Duration::from_secs_f32(seiche::TICK_DT), |elapsed| elapsed.presentation);

        self.apply_forme_region();
        self.sync_attention();
        self.tend_link_suggestions();
        // Advance physics (the in-thread tick, or the freshest actor snapshot)
        // into the read model, and learn whether the layout is still settling.
        // Everything below reprojects from the view — never the rapier world.
        let settling = if let Some(elapsed) = elapsed {
            let report = self
                .physics
                .advance_elapsed(&mut self.view, elapsed.physics, elapsed.config);
            self.elapsed_step = Some(report);
            report.settling
        } else {
            self.physics.advance_frame(&mut self.view)
        };
        observer.mark(0);
        // Advance the ambient backdrop sim (it paces itself internally - GoL accumulates toward its
        // generation interval, a continuous sim integrates). A fixed ~frame dt is fine for a
        // backdrop. (Physics scenes P5.)
        if let Some(sim) = self.ambient.as_mut() {
            sim.advance(1.0 / 60.0);
        }
        // A non-seiche layout strategy overrides the physics snapshot: write its buffered
        // positions into the view before anything reads it. (Layout picker.)
        self.apply_strategy_to_view();
        // A playing graph come to rest replaces Settled and sends anchored
        // items home (G7, F30, F45).
        self.advance_roles();
        self.apply_forme_region();
        // Pick up any finished off-thread community partition (a no-op when computing inline), so a
        // result dispatched on an earlier frame lands before the rings paint. (Graph signals — P3.)
        self.drain_community();
        // The community-ring overlay needs a fresh partition; recompute it only when the toggle is
        // on and the graph changed (generation-gated). (Graph signals — community to a ring.)
        if self.show_community_rings {
            self.ensure_community_fresh();
        }
        // The bridge-ring overlay needs the broker set, likewise revision-gated. (Graph signals.)
        if self.show_bridge_rings {
            self.ensure_bridges_fresh();
        }
        // Take a finished Meaning run, start one when the content moved, and hand a new snapshot
        // to Kinds, Group pull and the affinity signal, before the affinity sync reads it. (G2.)
        self.sync_meaning();
        // Keep the affinity-clustering force in step with the toggle + the current affinity signal
        // (installs / rebuilds / clears once per real change, with a settle so it takes; a no-op when
        // the toggle is off and no force is installed). (Graph signals — P4.)
        self.sync_affinity_force();
        // The gloss size-by-importance encoding reads the importance cache; keep it fresh (the
        // recompute is dirty-gated, so this is cheap when nothing changed). (Graph signals — P6c.)
        if self.gloss_size_by_importance {
            self.recompute_importance();
        }
        // Refresh the revision-gated weighted-edge memo (cache generalization C), so a per-frame
        // gloss redraw reads the collapsed edge list instead of re-deduping every frame. (Query memos.)
        self.refresh_weighted_edges();
        // A dragged node tracks the cursor with zero round-trip: re-pin it locally
        // over whatever the backend just reported (the actor lags a frame behind,
        // and the in-thread snapshot already agrees, so this is a no-op there).
        let dragging = self.drag.is_some_and(|d| d.moved);
        if let Some(d) = self.drag {
            if d.moved {
                self.view
                    .set_position(d.node, self.screen_to_world(self.cursor));
            }
        }
        // Pan inertia: glide + decay when not actively middle-dragging.
        let hand_panning = self.middle_drag.is_some() || self.left_panning();
        let gliding =
            !hand_panning && (self.pan_velocity.0.abs() > 0.05 || self.pan_velocity.1.abs() > 0.05);
        if gliding {
            self.camera.offset.0 += self.pan_velocity.0;
            self.camera.offset.1 += self.pan_velocity.1;
            self.pan_velocity.0 *= PAN_DECAY;
            self.pan_velocity.1 *= PAN_DECAY;
        } else if !hand_panning {
            self.pan_velocity = (0.0, 0.0);
        }
        // Following the layout: ease toward fit-to-content while physics plays.
        let following = self.follow_step(dt);
        self.generation = self.generation.wrapping_add(1);

        // Reproject the underlay from the view positions (a
        // node with no body falls back to its committed position in the producer).
        let positions: HashMap<NodeKey, PortablePoint> = self
            .view
            .positions()
            .map(|(k, p)| (k, PortablePoint::new(p.x, p.y)))
            .collect();
        // On-screen nodes (cull against the world-space viewport) become DOM
        // children; the rest demote to underlay rects, so no node double-draws.
        let mut on_screen: HashSet<NodeKey> = self
            .view
            .cull_aabb(self.world_viewport())
            .into_iter()
            .collect();
        // The scope lens (curated canvas): when set, render only the scoped nodes —
        // filter the on-screen DOM set here, project the underlay through a curated
        // arrangement of just the scope (below), and hide non-scoped DOM nodes (the
        // gnode loop). `None` shows the whole graph. (Curated canvas.)
        let scoped: Option<HashSet<NodeKey>> =
            self.scope.as_ref().map(|s| s.iter().copied().collect());
        let folds = self.fold_projections();
        let folded_members: HashSet<NodeKey> =
            folds.iter().flat_map(FoldProjection::hidden).collect();
        let fold_members: HashSet<_> = folds
            .iter()
            .flat_map(|fold| fold.members.iter().copied())
            .collect();
        let node_visible = |key: NodeKey| {
            scoped.as_ref().is_none_or(|scope| scope.contains(&key))
                && !folded_members.contains(&key)
        };
        on_screen.retain(|key| node_visible(*key));
        let visible_keys: Vec<NodeKey> = self
            .graph
            .nodes()
            .map(|(key, _)| key)
            .filter(|key| node_visible(*key))
            .collect();

        // The graph grammar, when the host presents by it: who is loud, who is
        // labelled, each node's category.
        let grammar = self.grammar_frame();

        // Route the underlay through the canvas's forme arrangement — the full
        // read-through Identity arrangement, or a curated arrangement of just the
        // scope. Either way the canvas renders as a Cartography projection of an
        // arrangement (the spine's "two projections of one arrangement"); a scoped
        // arrangement is exactly the shape a stored/compare arrangement would take.
        let arrangement = match (&self.scope, folds.is_empty()) {
            (None, true) => identity_arrangement(&self.graph),
            _ => crate::canvas::underlay::arrangement_of_keys(&self.graph, &visible_keys),
        };
        observer.mark(1);
        let mut underlay = canvas_paint_list_demoted_from_arrangement(
            &self.graph,
            &arrangement,
            |k| positions.get(&k).copied(),
            |k| !on_screen.contains(&k),
            // Link strokes are projected below from canonical LinkReading values.
            // Suppress Platen's pair fallback so it cannot double-draw a link.
            |_| false,
            // Each node's face radius (node_size / 2) so straight edges trim to the
            // face and demoted underlay rects draw at the node's true size. (Node-rep
            // Decision 5 — size drives the face geometry.)
            |k| Some(self.node_size(k) / 2.0),
            viewport,
            self.camera,
            &self.style,
            self.generation,
        );
        // Placed field regions paint *under* the edges + demoted nodes (a background
        // the graph sits within), spliced at the bottom of the same camera transform.
        // The dashed extent box shows only for the hovered field (box-on-interaction);
        // hidden fields are skipped. (Field regions — disk-in-box + box-on-interaction.)
        underlay.splice_world_underlay(field_overlay(
            &self.graph,
            self.active_field(),
            self.hidden_field_ids(),
        ));
        underlay.splice_world_underlay(self.forme_overlay());
        underlay.splice_world_overlays(link_overlay_from_segments(
            self.rendered_link_segments(),
            &self.selected_links,
            |key| node_visible(key) && !fold_members.contains(&key),
            |from, to| super::super::grammar_view::link_tone(grammar.as_ref(), from, to),
        ));
        underlay.splice_world_overlays(self.fold_overlay(&folds, &positions));
        // Community rings: a halo per node in its community's colour, spliced into the same
        // world-space transform. (Graph signals — community to a ring.)
        if self.show_community_rings
            && let Some(community) = self.channels.community_held()
        {
            let rings = community_ring_overlay(&self.view, community, |k| self.node_size(k) / 2.0);
            underlay.splice_world_overlays(rings);
        }
        // Bridge rings: a bold ring on the high-betweenness brokers, over the community rings so the
        // connectors stand out. (Graph signals — bridges.)
        if self.show_bridge_rings
            && let Some(bridges) = self.channels.bridges_held(self.bridge_metric)
        {
            let rings = bridge_ring_overlay(&self.view, bridges, |k| self.node_size(k) / 2.0);
            underlay.splice_world_overlays(rings);
        }

        observer.mark(2);
        // The node-children layer is one retained Livery document. Compute the
        // host-owned placements first, then publish one DOM mutation batch so
        // Livery can classify and retain the frame coherently.
        // The gnode pool is positioned as upright billboards (each gnode carries its
        // own screen-space transform via `Camera::to_screen` below), so the `.stage`
        // container is identity: the camera's foreshorten lives in each node's anchor,
        // not a shear on the whole stage. (Isometric camera P1 — billboards.)
        // When the host renders these gnodes as chrome DOM elements (canvas-as-element)
        // the in-scene gnode layer is dropped below, so skip the per-gnode transform/class
        // updates: an empty set makes the loop a no-op, the costliest part of the frame on
        // a big graph. (Phase 2 — perf / cleaning.)
        let gnodes: Vec<(NodeKey, DomNodeId)> = if self.render_gnodes_as_dom {
            Vec::new()
        } else {
            self.gnode_of.iter().map(|(&k, &g)| (k, g)).collect()
        };
        let mut gnode_updates = Vec::with_capacity(gnodes.len());
        for (key, gnode) in gnodes {
            // Scope lens: hide a non-scoped node's DOM child (the underlay already
            // excludes it), so a scoped canvas shows only its subset. (Curated canvas.)
            if !node_visible(key) {
                gnode_updates.push((gnode, "display: none;".to_string(), None));
                continue;
            }
            let pos = positions.get(&key).copied().unwrap_or_default();
            let Some(body) = self.node_billboard_at(key, pos) else {
                gnode_updates.push((gnode, "display: none;".to_string(), None));
                continue;
            };
            let bounds = body.screen_rect();
            let mut style = format!(
                "transform: translate({}px, {}px) scale({}); z-index: {}; width: {}px; height: {}px; border-radius: {}px;",
                bounds.min.x,
                bounds.min.y,
                body.zoom,
                body.depth as i32,
                body.local_side,
                body.local_side,
                body.corner_radius,
            );
            if let Some(frame) = &grammar {
                style.push_str(&self.grammar_fill(key, frame));
            }
            // Selection wins (orange); otherwise color by activation state —
            // green open, red closed, blue idle (the default for an unset node).
            let state_class = if self.selected.contains(&key) {
                "gnode-selected"
            } else {
                match self.node_states.get(&key) {
                    Some(NodeState::Open) => "gnode-open",
                    Some(NodeState::Closed) => "gnode-closed",
                    _ => "gnode-idle",
                }
            };
            // Shape rides as a second class (border-radius only) so it merges with
            // the color class; square is the default (no shape class).
            let shape_class = match self.node_shapes.get(&key) {
                Some(NodeShape::Rounded) => " gnode-rounded",
                Some(NodeShape::Circle) => " gnode-circle",
                _ => "",
            };
            // The representation rung changes how the same measured footprint
            // is realized, not how much space it occupies. Glyph suppresses
            // the caption and rounds the body; Card retains the ordinary Canvas
            // face; LivePane receives a framed focus treatment. It remains a
            // face request here, not a claim that interactive content has been
            // embedded. The open representation tail falls back to Card.
            // (Projection proofs — P3b renderer consumption.)
            let representation_class = self.node_representation_class(key);
            let class = match &grammar {
                Some(frame) => format!("{}{representation_class}", self.grammar_class(key, frame)),
                None => format!("{state_class}{shape_class}{representation_class}"),
            };
            gnode_updates.push((gnode, style, Some(class)));
        }
        // Each fold's "+N", over the node layer it belongs to: none when the
        // host draws the gnodes itself (it reads `fold_badges` instead).
        let badges: Vec<((f32, f32), String)> = if self.render_gnodes_as_dom {
            Vec::new()
        } else {
            folds
                .iter()
                .filter_map(|fold| {
                    let at = self.fold_badge_anchor(fold, |key| positions.get(&key).copied())?;
                    Some((at, fold.badge()))
                })
                .collect()
        };
        let stage_node = self.stage_node;
        let stage_style = self.stage_style.as_str();
        let mut badge_nodes = std::mem::take(&mut self.fold_badge_nodes);
        observer.mark(3);
        let (_, restyle) = self.node_document.mutate_dom(|dom| {
            set_style(dom, stage_node, stage_style);
            for (gnode, style, class) in &gnode_updates {
                set_style(dom, *gnode, style);
                if let Some(class) = class {
                    set_class(dom, *gnode, class);
                }
            }
            place_fold_badges(dom, stage_node, &mut badge_nodes, &badges);
        });
        self.fold_badge_nodes = badge_nodes;
        tracing::debug!(?restyle, "canvas pool: Livery mutation batch");
        observer.mark(4);
        let nodes_plist = self
            .node_document
            .frame(w, h)
            .expect("canvas Livery/Buckram node frame");
        observer.mark(5);

        // Face layer: either a palette-decoded derived vector or a resolved favicon over each
        // on-screen tile, above the colored body and below the marquee.
        let (face_cmds, face_images) = self.face_layer(&on_screen, &positions);
        observer.mark(6);

        // P3 fake height: a stem from each raised node's ground anchor up to its
        // floating gnode. (Isometric camera P3.)
        let stem_cmds = self.height_stem_cmds(&on_screen, &positions);

        // A screen-space layer for the marquee rubber-band, when active.
        let marquee_cmds = self
            .marquee
            .map(|origin| marquee_rect_cmds(origin, self.cursor));
        // The canvas ground fills the viewport. The content wash sits just above
        // it, under territory and the other scene layers.
        let bg_cmds = background_cmds(w, h, self.backdrop);
        let (wash_cmds, wash_pending) =
            self.content_backdrop_commands(presentation_dt, scoped, folded_members);
        let territory_cmds = self.territory_commands();
        let projection_backdrop_cmds = self.projection_backdrop_commands();
        let strategy_region_cmds = self.strategy_region_commands();
        // Ambient backdrop: the sim painted as the bottom layer (above the bg fill, below the scene),
        // in its tincture, stretched across the viewport. The sim owns its look (GoL = run-merged
        // cell rects; a continuous sim = dots). (Physics scenes P5.)
        let ambient_cmds: Vec<PaintCmd> = self
            .ambient
            .as_ref()
            .map(|sim| sim.paint(w as f32, h as f32, self.ambient_tincture))
            .unwrap_or_default();
        // Living backdrop: the scene bodies as soft orbs / polygons, plus the
        // textured sprite props in their own layer above them. (Physics scenes
        // P1, P4b; scene-prop sprites.)
        let (scene_cmds, scene_sprite_cmds, scene_sprite_images) = self.scene_body_layers();

        // Liquid pool: the PBF particles as soft watery orbs, above the backdrop
        // scene and below the graph. (Physics scenes P4c.)
        let fluid_cmds = self.fluid_cmds();
        observer.mark(7);

        let viewport_bounds =
            LayoutRect::new(LayoutPoint::zero(), LayoutPoint::new(w as f32, h as f32));
        let underlay_commands =
            super::super::cull::visible_commands(underlay.commands(), viewport_bounds);
        let node_commands =
            super::super::cull::visible_commands(nodes_plist.commands(), viewport_bounds);
        observer.mark(8);
        let mut layers = vec![CompositeLayer::commands_only(&bg_cmds)];
        if !wash_cmds.is_empty() {
            layers.push(CompositeLayer::commands_only(&wash_cmds));
        }
        if !territory_cmds.is_empty() {
            layers.push(CompositeLayer::commands_only(&territory_cmds));
        }
        if !projection_backdrop_cmds.is_empty() {
            layers.push(CompositeLayer::commands_only(&projection_backdrop_cmds));
        }
        if !strategy_region_cmds.is_empty() {
            layers.push(CompositeLayer::commands_only(&strategy_region_cmds));
        }
        if !ambient_cmds.is_empty() {
            layers.push(CompositeLayer::commands_only(&ambient_cmds));
        }
        if !scene_cmds.is_empty() {
            layers.push(CompositeLayer::commands_only(&scene_cmds));
        }
        if !scene_sprite_cmds.is_empty() {
            layers.push(CompositeLayer {
                commands: &scene_sprite_cmds,
                fonts: &[],
                images: &scene_sprite_images,
            });
        }
        if !fluid_cmds.is_empty() {
            layers.push(CompositeLayer::commands_only(&fluid_cmds));
        }
        layers.push(CompositeLayer::commands_only(&underlay_commands));
        // The on-screen gnode + face layers, unless the host renders these gnodes as
        // chrome DOM elements instead (canvas-as-element); then only edges + demoted
        // dots remain as the underlay. (Canvas-as-element — Phase 2.)
        if !self.render_gnodes_as_dom {
            // Height stems under the gnodes (P3): the floating gnode paints over its stem.
            if !stem_cmds.is_empty() {
                layers.push(CompositeLayer::commands_only(&stem_cmds));
            }
            layers.push(CompositeLayer {
                commands: &node_commands,
                fonts: nodes_plist.fonts(),
                images: nodes_plist.images(),
            });
            if !face_cmds.is_empty() {
                layers.push(CompositeLayer {
                    commands: &face_cmds,
                    fonts: &[],
                    images: &face_images,
                });
            }
        }
        if let Some(cmds) = marquee_cmds.as_ref() {
            layers.push(CompositeLayer::commands_only(cmds));
        }
        let scene = match composite_paint_layers(viewport, &layers) {
            Ok(translated) => {
                self.last_translation_error = None;
                translated.scene
            },
            Err(error) => {
                self.last_translation_error = Some(error);
                Scene::new(w, h)
            },
        };

        let after_cull = layers
            .iter()
            .map(|layer| layer.commands.len())
            .sum::<usize>();
        let culled = underlay.commands().len() - underlay_commands.len()
            + if self.render_gnodes_as_dom {
                0
            } else {
                nodes_plist.commands().len() - node_commands.len()
            };
        observer.counts(
            self.graph.node_count(),
            on_screen.len(),
            after_cull + culled,
            after_cull,
        );

        // A sliced Meaning run advances once a frame, so it asks for the next
        // one until it lands. (Dynamics grammar plan, G2, F35.)
        let needs_redraw = settling
            || gliding
            || dragging
            || following
            || self.ambient.is_some()
            || self.meaning_pending()
            || wash_pending;
        observer.mark(9);
        (scene, needs_redraw)
    }
}
