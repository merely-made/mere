// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Canvas {
    /// A new canvas over an **empty** session graph. The host grows it with
    /// [`visit`](Canvas::visit) as the user navigates (the graph-rooted browse
    /// loop). For an isolated demo / the standalone bin, use
    /// [`with_sample_graph`](Canvas::with_sample_graph).
    pub fn new() -> Self {
        Self::from_graph(Graph::new())
    }

    /// A new canvas over a small built-in sample graph (a ring + spokes), seeded
    /// into a tight central spiral so the first settle is visible. The standalone
    /// `canvas-host` bin and the canvas tests use this; meerkat uses
    /// [`new`](Canvas::new) and drives the graph through [`visit`](Canvas::visit).
    pub fn with_sample_graph() -> Self {
        Self::from_graph(sample_graph())
    }

    /// Build an canvas over a **restored** session `graph`: park every node at the
    /// origin and halt (do not auto-settle), so the host can then apply the saved
    /// layout from its cartography sidecar via [`view`](Canvas::view) / physics
    /// seeding without the sim re-scrambling first. Positions are no longer graph
    /// truth (S2), so they no longer ride the graph snapshot. (Persistence host seam, S3.)
    pub fn with_graph(graph: Graph) -> Self {
        Self::with_graph_on(graph, None)
    }

    /// [`with_graph`](Canvas::with_graph) with the simulation on the far side of
    /// `transport` (a Web Worker) and no engine built here: every physics call
    /// the canvas makes becomes a data command, and the layout comes back as
    /// updates, folded when the host advances a frame. Until the far side is
    /// ready the canvas draws its seed and the commands queue in order.
    pub fn with_graph_remote(graph: Graph, transport: Box<dyn seiche::Transport>) -> Self {
        Self::with_graph_on(graph, Some(transport))
    }

    fn with_graph_on(graph: Graph, transport: Option<Box<dyn seiche::Transport>>) -> Self {
        let mut canvas = Self::from_graph_on(graph, transport);
        let positions: Vec<(NodeKey, Point2D<f32>)> = canvas
            .graph
            .nodes()
            .map(|(key, _node)| (key, Point2D::zero()))
            .collect();
        for &(key, pos) in &positions {
            canvas.view.set_position(key, pos);
        }
        canvas.physics.seed(positions);
        canvas.physics.halt();
        canvas
    }

    /// Re-point this canvas at a different session `graph` **in place**, keeping the
    /// (possibly offloaded) physics actor and the node-children pool alive. The
    /// graph is replaced wholesale, every derived view is reconciled to it
    /// (departed nodes drop, the spring topology and node pool rebuild), per-graph
    /// interaction state (selection, hidden edges, drags, pushed node states/shapes)
    /// is cleared, and every node is parked at the origin and halted so the host can
    /// re-apply the switched-to session's saved layout from its cartography sidecar
    /// without the sim re-scrambling first (positions are no longer graph truth, S2).
    /// This is the Model-A graph swap the multi-graph switch drives. (Multi-graph MG2.)
    pub fn set_graph(&mut self, graph: Graph) {
        // Carry a transient scope's intent by member UUID across an explicit
        // switch, never by the old graph's reusable slots.
        let scope_members: Option<Vec<_>> = self.scope.as_ref().map(|scope| {
            scope
                .iter()
                .filter_map(|key| self.graph.get_node(*key).map(|node| node.id))
                .collect()
        });
        // An explicit graph switch ends every runtime cache life, even if the
        // incoming graph is a clone with the same kernel witnesses.
        let retired = std::mem::take(&mut self.cache_memberships);
        self.retire_member_caches(&retired);
        self.physics
            .sync_nodes_retiring(Vec::new(), retired.keys().copied().collect());
        // Node keys can be reused by another graph; release this view's local
        // holds before syncing bodies under the replacement graph.
        for key in self.forme_held.drain() {
            self.physics.unpin(key);
        }
        self.graph = graph;
        self.wash = Default::default();
        // A graph switch ends every child life, even if member UUIDs recur.
        self.described_keys = Default::default();
        // NodeKeys do not survive a graph swap; an exact scope resolves its
        // authored IDs again against the replacement.
        if let Some(roster) = self.exact_scope.clone() {
            self.scope = Some(self.resolve_exact_scope(&roster));
        } else if let Some(members) = scope_members {
            self.scope = Some(
                members
                    .into_iter()
                    .filter_map(|id| self.graph.get_node_key_by_id(id))
                    .collect(),
            );
        }
        self.forget_graph_models();
        self.last_strategy_inputs = None;
        *self
            .strategy_graph_memo
            .get_mut()
            .expect("strategy input cache") = None;
        self.forme_region = None;
        self.forme_guides = None;
        self.forme_held.clear();
        self.selected.clear();
        self.selected_links.clear();
        self.selected_link_appearances.clear();
        self.hidden_edges.clear();
        self.hidden_links.clear();
        self.hidden_assertions.clear();
        self.link_path_response = Default::default();
        self.active_field = None;
        self.hidden_fields.clear();
        self.node_states.clear();
        self.node_shapes.clear();
        self.node_faces.clear();
        self.derived_face_cache.clear();
        self.node_sizes.clear();
        self.node_sprites.clear();
        self.node_sprite_hulls.clear();
        self.node_materials.clear();
        self.pending_image_requests.clear();
        self.requested_images.clear();
        self.projection_score = None;
        self.projection_representations.clear();
        self.node_rule_faces.clear();
        self.graph_view_epoch = self
            .graph_view_epoch
            .checked_add(1)
            .expect("Canvas epoch exhausted");
        self.restored_score_hold = None;
        self.strategy_positions = None;
        self.paused_positions = None;
        self.channels = crate::signals::ChannelRegistry::new();
        self.drag = None;
        self.pinned_nodes.clear();
        // Attention and seeds are keyed by node; the host's suggestion settings
        // and the pages' link offers (keyed by member) outlive the swap.
        let suggestions = std::mem::take(&mut self.grammar.suggestions);
        let links = std::mem::take(&mut self.grammar.links);
        self.grammar = Default::default();
        self.grammar.suggestions = suggestions;
        self.grammar.links = links;
        self.strategy_regions.clear();
        self.mosaic = Default::default();
        self.field_drag = None;
        self.marquee = None;
        self.middle_drag = None;
        self.folds.clear();
        self.fold_press = None;
        self.fold_undo.clear();
        self.fold_redo.clear();
        self.reconcile_derived();
        self.sync_arrangement_roles();
        let positions: Vec<(NodeKey, Point2D<f32>)> = self
            .graph
            .nodes()
            .map(|(key, _node)| (key, Point2D::zero()))
            .collect();
        for &(key, pos) in &positions {
            self.view.set_position(key, pos);
        }
        self.reset_frame_time();
        self.physics.reset_field_runtime(positions);
        self.reset_frame_time();
        self.physics.halt();
        self.generation += 1;
    }

    /// Build an canvas over `graph`: its [`build_simulation`], the node-children
    /// pool, and a default camera. Shared by [`new`](Canvas::new) (empty),
    /// [`with_sample_graph`](Canvas::with_sample_graph), and
    /// [`with_graph`](Canvas::with_graph).
    pub(crate) fn from_graph(graph: Graph) -> Self {
        Self::from_graph_on(graph, None)
    }

    /// [`from_graph`](Canvas::from_graph) on the backend `transport` names: in
    /// process when `None`.
    fn from_graph_on(graph: Graph, transport: Option<Box<dyn seiche::Transport>>) -> Self {
        let (physics, view) = match transport {
            #[cfg(feature = "physics-engine")]
            None => {
                let sim = build_simulation(&graph);
                let view = sim.view();
                (Physics::inline(sim, SETTLE_TICKS), view)
            },
            // No engine in this build and no transport named: the physics is
            // unavailable, and the canvas keeps its seed layout and says so.
            #[cfg(not(feature = "physics-engine"))]
            None => crate::canvas::seiche_bridge::remote_physics(
                &graph,
                Box::new(crate::canvas::seiche_bridge::UnavailableTransport::default()),
                SETTLE_TICKS,
            ),
            Some(transport) => {
                crate::canvas::seiche_bridge::remote_physics(&graph, transport, SETTLE_TICKS)
            },
        };
        let (node_dom, gnode_of, stage_node) = build_pool_dom(&graph);
        let node_document = genet_livery::LiveryDocument::new(
            node_dom,
            genet_livery::StyleSet::cambium(&crate::canvas::build::NODE_SHEET),
            genet_livery::Device::screen(1.0, 1.0),
        );
        let mut canvas = Self {
            graph,
            described_keys: Default::default(),
            cache_memberships: Default::default(),
            result_owner: Arc::new(()),
            physics,
            #[cfg(feature = "gpu")]
            physics_device: None,
            frame_timestamp: None,
            elapsed_step: None,
            physics_paused: false,
            view,
            node_document,
            gnode_of,
            stage_node,
            stage_style: crate::canvas::build::stage_style(None),
            camera: Camera::default(),
            style: dark_scene_style(),
            backdrop: surface_bg(),
            content_backdrop: false,
            wash: Default::default(),
            projection_backdrops: Vec::new(),
            generation: 0,
            last_translation_error: None,
            cursor: (0.0, 0.0),
            pan_velocity: (0.0, 0.0),
            follow: false,
            middle_drag: None,
            orbit_drag: None,
            drag: None,
            pinned_nodes: HashSet::new(),
            grammar: Default::default(),
            strategy_regions: Vec::new(),
            mosaic: Default::default(),
            territory_field: false,
            graph_grammar: false,
            withdraw_policy: None,
            category_colors: Vec::new(),
            field_drag: None,
            forme_region: None,
            forme_guides: None,
            forme_held: HashSet::new(),
            selected: HashSet::new(),
            selected_links: HashSet::new(),
            selected_link_appearances: HashMap::new(),
            hidden_edges: HashSet::new(),
            hidden_links: HashSet::new(),
            hidden_assertions: HashSet::new(),
            link_path_response: Default::default(),
            occurrence_edges: None,
            physics_view_revision: 0,
            active_field: None,
            hidden_fields: HashSet::new(),
            node_states: HashMap::new(),
            node_shapes: HashMap::new(),
            node_faces: HashMap::new(),
            derived_face_palette: DerivedFacePalette::default(),
            derived_face_cache: HashMap::new(),
            node_sizes: HashMap::new(),
            node_sprites: HashMap::new(),
            node_sprite_hulls: HashMap::new(),
            scene_sprite_textures: HashMap::new(),
            resolved_images: resolved_image_cache::ResolvedImageCache::default(),
            pending_image_requests: BTreeMap::new(),
            requested_images: HashSet::new(),
            ambient: None,
            ambient_tincture: ColorF::new(0.0, 0.0, 0.0, 0.0),
            node_materials: HashMap::new(),
            size_by_degree: false,
            size_by_importance: false,
            size_by_recency: false,
            importance_metric: crate::signals::ImportanceMetric::Degree,
            channels: crate::signals::ChannelRegistry::new(),
            last_strategy_inputs: None,
            strategy_graph_memo: std::sync::Mutex::new(None),
            strategy_footprint_revision: 0,
            strategy_footprints: HashMap::new(),
            show_community_rings: false,
            offthread_wake: None,
            community_actor: None,
            show_bridge_rings: false,
            bridge_metric: crate::signals::BridgeMetric::default(),
            cluster_by_affinity: false,
            installed_affinity_revision: None,
            content_affinity: None,
            content_affinity_dirty: false,
            affinity_force_installed: false,
            affinity_blend: AffinityBlend::default(),
            gloss_strategy: None,
            gloss_positions: None,
            gloss_cache_inputs: None,
            gloss_overlays: Vec::new(),
            gloss_scope_selection: false,
            gloss_size_by_importance: false,
            weighted_edges_cache: None,
            weighted_edges_rebuilds: 0,
            height_by_degree: false,
            marquee: None,
            empty_press: None,
            self_link_press: None,
            context_request: None,
            ctrl: false,
            shift: false,
            alt: false,
            view_w: 1024,
            view_h: 600,
            active_strategy: None,
            strategy_positions: None,
            paused_positions: None,
            projection_score: None,
            projection_representations: HashMap::new(),
            graph_view_epoch: 0,
            node_rule_faces: HashMap::new(),
            roles: Default::default(),
            physics_law: crate::canvas::PhysicsLaw::Springs,
            physics_overlays: Vec::new(),
            physics_kind_source: crate::canvas::PhysicsKindSource::Site,
            physics_group_source: crate::canvas::PhysicsKindSource::Site,
            meaning: Default::default(),
            physics_mass_source: crate::canvas::PhysicsMassSource::Degree,
            physics_depth_source: crate::canvas::PhysicsDepthSource::Roots,
            physics_composition: None,
            schedule: None,
            dynamics: Default::default(),
            #[cfg(test)]
            law_rebuilds: 0,
            restored_score_hold: None,
            scope: None,
            exact_scope: None,
            folds: Vec::new(),
            fold_press: None,
            fold_badge_nodes: Vec::new(),
            fold_undo: Vec::new(),
            fold_redo: Vec::new(),
            render_gnodes_as_dom: false,
        };
        canvas.channels.bind_graph(&canvas.graph);
        canvas.described_keys.reconcile(&canvas.graph);
        canvas.reconcile_member_lifetimes();
        // Construction establishes the footprint baseline before any layout is cached.
        canvas.strategy_footprints = canvas
            .graph
            .nodes()
            .map(|(key, _)| (key, canvas.node_size(key)))
            .collect();
        canvas
    }
}
