// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            runtime_input_keys: [GraphInputKey::new_source(); 4],
            inner: chartulary::Graph::new(),
            surface_node_lifetimes: HashMap::new(),
            resources: chartulary::Graph::new(),
            resource_facets: chartulary::FacetStore::new(),
            shown_resources: BTreeMap::new(),
            resource_surfaces: BTreeMap::new(),
            facets: chartulary::FacetStore::new(),
            url_to_nodes: HashMap::new(),
            import_records: Vec::new(),
            fields: BTreeMap::new(),
            couplings: BTreeMap::new(),
            nav: SharedNavigationMemory::empty(),
            revision: 0,
            url_grouping_revision: 0,
            content_revision: 0,
            visit_revision: 0,
            current_session: 0,
            write_author: Author::user(),
            recorder: capture::Recorder::default(),
            pending_link_state: PendingLinkState::default(),
            pending_derivation_suppressed: pending_links::Suppression::default(),
            semantic_observation_revision: 0,
            appearance_admission_revision: 0,
            known_coverage: CoverageNote::default(),
            staged_edges: Default::default(),
            handle_index: Default::default(),
        }
    }

    /// Send every delta this graph applies through `apply_graph_delta` to
    /// `recorder`, or stop with `None`. The per-graph twin of the thread's
    /// capture hook: a host holding many session graphs records each into its
    /// own journal. A clone of this graph, and a replay onto it, record nothing.
    pub fn set_recorder(&mut self, recorder: Option<DeltaRecorder>) {
        self.recorder.0 = recorder;
    }

    /// Whether this graph records its deltas.
    pub fn is_recording(&self) -> bool {
        self.recorder.0.is_some()
    }

    /// The recorder identity used when a writer has no attributed source.
    pub fn write_author(&self) -> &Author {
        &self.write_author
    }

    /// Run a write under the same author its journal will record.
    pub fn write_as<R>(&mut self, author: Author, write: impl FnOnce(&mut Self) -> R) -> R {
        let previous = std::mem::replace(&mut self.write_author, author);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| write(self)));
        self.write_author = previous;
        match result {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    /// Set the current app-launch session number (Alembic B5). The host calls this once,
    /// right after construction or snapshot-restore, with its persisted-and-incremented
    /// session counter; every [`navigate_node`](Self::navigate_node) call afterwards stamps
    /// the visited node's [`Node::last_session_visited`] with this value.
    pub fn set_current_session(&mut self, session: u64) {
        self.current_session = session;
    }

    /// The launch number [`set_current_session`](Self::set_current_session) installed.
    pub fn current_session(&self) -> u64 {
        self.current_session
    }

    /// The current structural revision (see [`revision`](Self::revision)). A consumer captures it
    /// alongside a derived result and recomputes only when it has advanced. (Graph signals.)
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The current URL-authority grouping revision. This advances only when a
    /// node's primary URL moves between site groups, letting site-grouped
    /// projections refresh without invalidating structural caches for an
    /// ordinary same-site navigation.
    pub fn url_grouping_revision(&self) -> u64 {
        self.url_grouping_revision
    }

    /// The current content revision (see the field's doc): it advances on a
    /// node's title or primary URL changing and on a node joining or leaving,
    /// never on an edge or relation change.
    pub fn content_revision(&self) -> u64 {
        self.content_revision
    }

    /// The current visit revision (see the field's doc): it advances when a
    /// node's visit history may have changed, never on structure or text.
    pub fn visit_revision(&self) -> u64 {
        self.visit_revision
    }

    /// The categorical site key used by URL-grouped projections: the authority
    /// after `://` and before a path, query, or fragment, or the whole hostless
    /// input. It deliberately retains ports and opaque values.
    pub fn url_grouping_key(url: &str) -> &str {
        let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
        after_scheme
            .split(['/', '?', '#'])
            .next()
            .unwrap_or(after_scheme)
    }

    /// Advance the structural revision. Called by the topology/relation mutators on a real change.
    /// (Graph signals — the local counter; cross-owner caches use structural_input_key.)
    pub(super) fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.runtime_input_keys[0].advance();
    }

    pub(super) fn bump_url_grouping_revision(&mut self) {
        self.url_grouping_revision = self.url_grouping_revision.wrapping_add(1);
        self.runtime_input_keys[2].advance();
    }

    pub(crate) fn bump_content_revision(&mut self) {
        self.content_revision = self.content_revision.wrapping_add(1);
        self.runtime_input_keys[1].advance();
    }

    pub(crate) fn bump_visit_revision(&mut self) {
        self.visit_revision = self.visit_revision.wrapping_add(1);
        self.runtime_input_keys[3].advance();
    }

    // Durable write boundaries. The Phase 6.5 GraphDelta migration covers
    // ordinary primitive mutations; assertion lifecycle and predicate
    // declaration APIs also expose public durable writes and emit captures
    // directly. A host must install a recorder and apply its admission policy
    // around either entry point; GraphDelta is not the sole public writer.
    // The writer classes:
    //   1. Ordinary primitive mutations (topology, relations, nav history, tags,
    //      titles, body, media, classifications, properties, predicates,
    //      fields/couplings, frame hints, import records) — `pub(crate)`,
    //      reached externally only via `GraphDelta`. Navigation has one primitive
    //      per operation (`navigate_node`, `branch_history`, `node_history_back`,
    //      `node_history_forward`) under both the live and the replay arm.
    //   2. Public native assertion/declaration writes: `assertion_write` and
    //      `predicate_declarations`, with their own capture emission.
    //   3. Compound kernel operations stay `pub`: `from_snapshot`,
    //      `cross_graph::copy_*` — kernel-authored multi-step ops over
    //      non-delta-able inputs (`&Graph` donors); the "trusted writers".
    //   4. Transient/runtime state stays `pub`, exempt by design:
    //      `set_node_position` / `set_node_projected_position` (physics/view —
    //      positions are not graph truth), `set_current_session` (per-launch
    //      host wiring). Webview runtime state left the kernel entirely
    //      (`BrowserNodeState` sidecar, boundary pass slice C).
    //   5. Test fixtures: the `fixtures` cargo feature re-exposes the raw
    //      structural mutators via `graph::fixtures::GraphFixtures` —
    //      dev-dependencies only, never a production enable. Navigation is not
    //      among them: a fixture graph navigates through `GraphDelta`, so it
    //      carries what a live navigation leaves.

    /// The fixed namespace for the deterministic node id (see
    /// [`node_namespace_id`](Self::node_namespace_id)). Fixed forever: changing it
    /// would renumber every node derived from it.
    const NODE_NAMESPACE: Uuid = Uuid::from_u128(0x6D65_7265_4E4F_4445_6E61_6D65_7370_6163);

    /// The deterministic node UUID for `url`: a name-based UUIDv5 under
    /// [`NODE_NAMESPACE`](Self::NODE_NAMESPACE). Two hosts that materialize the
    /// same `url` mint the same id, so a federated merge needs no identity
    /// reconciliation. The linked-data ingest layer uses this for a document's
    /// `@id`, where the URL *is* the identity. Raw [`add_node`] keeps a fresh
    /// random id, because the kernel treats a node's address as a property and
    /// lets several nodes share one (see `get_nodes_by_url`).
    pub fn node_namespace_id(url: &str) -> Uuid {
        Uuid::new_v5(&Self::NODE_NAMESPACE, url.as_bytes())
    }

    /// Add a new node to the graph.
    ///
    /// Not available on `wasm32`; use [`add_node_with_id`] with a host-provided
    /// UUID (e.g. from [`node_namespace_id`](Self::node_namespace_id)) instead.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn add_node(&mut self, url: String, position: Point2D<f32>) -> NodeKey {
        self.add_node_with_id(Uuid::new_v4(), url, position)
    }

    /// Add a node with a pre-existing UUID, or return the key of the node that
    /// already holds it ([`insert_node_if_missing`](Self::insert_node_if_missing)).
    pub(crate) fn add_node_with_id(
        &mut self,
        id: Uuid,
        url: String,
        position: Point2D<f32>,
    ) -> NodeKey {
        self.insert_node_if_missing(id, url, position).0
    }

    /// The insertion contract shared by live `AddNode` and replay: insert if
    /// the id is missing, else leave the existing node, its address and its URL
    /// index entries as they are. The flag reports whether a node was inserted.
    /// `_position` is the caller's initial placement hint; it is no longer
    /// stored on the node (position is not graph truth), so the host applies it
    /// to seiche / the cartography sidecar.
    pub(crate) fn insert_node_if_missing(
        &mut self,
        id: Uuid,
        url: String,
        _position: Point2D<f32>,
    ) -> (NodeKey, bool) {
        let primary_address = address_from_url(&url);
        let mut container = chartulary::Container::with_identity(id)
            .with_address_record(primary_address)
            .with_title(url.clone());
        container.media_type = detect_mime(&url, None);
        let (key, inserted) = self.insert_surface_node(Node {
            container,
            images: std::collections::BTreeMap::new(),
        });
        if inserted {
            self.set_node_last_visited_at_ms(key, Self::epoch_ms());
            self.url_to_nodes.entry(url).or_default().push(key);
            self.bump_revision();
            self.bump_content_revision();
        }
        (key, inserted)
    }

    /// Remove a node and all its connected edges
    pub(crate) fn remove_node(&mut self, key: NodeKey) -> bool {
        if let Some(node) = self.inner.remove(key) {
            // The node's navigation owner is intentionally *kept* in the shared
            // visit space: a node branched from it still anchors to its visits, so
            // erasing the lineage on node removal would orphan those anchors. The
            // owner is simply never queried once the node is gone (pruning belongs
            // to a future GC / `eidetic` scope, not graph removal).
            // Deregister every URL claim this node carried — Primary plus
            // any aliases. (Aliases are not yet wired into add paths but the
            // index handles them when they land.)
            for address in &node.addresses {
                self.remove_url_mapping(address.as_url_str(), key);
            }
            let node_id = node.id;
            self.surface_node_lifetimes.remove(&node_id);
            if let Some(resource) = self.shown_resources.remove(&node_id) {
                self.unlink_shown_surface(resource, node_id);
            }
            self.remove_facets_for_node(node_id);
            let removed_id = node_id.to_string();
            for record in &mut self.import_records {
                record
                    .memberships
                    .retain(|membership| membership.node_id != removed_id);
            }
            self.import_records
                .retain(|record| !record.memberships.is_empty());
            self.bump_revision();
            self.bump_content_revision();
            true
        } else {
            false
        }
    }

    /// Write a node's primary address, maintaining the url_to_nodes index.
    /// Returns the old URL, or None if the node doesn't exist.
    ///
    /// The one writer of the address. `SetNodeUrl` calls it alone, changing the
    /// address with no visit; [`navigate_node`](Self::navigate_node) calls it
    /// after recording the visit, and a history step after moving the cursor
    /// (`land_history_step`). The Stemma cursor is the browse trail's position,
    /// not the address; see the `history` module.
    ///
    /// Operates on the Primary claim only; aliases (when supported) stay
    /// attached. To mutate aliases, use dedicated alias methods (future).
    pub(crate) fn update_node_url(&mut self, key: NodeKey, new_url: String) -> Option<String> {
        let node = self.inner.node_mut(key)?;
        let old_url = node.primary_address().as_url_str().to_string();
        let host_changed = cached_host_from_url(&old_url) != cached_host_from_url(&new_url);
        let site_group_changed =
            Self::url_grouping_key(&old_url) != Self::url_grouping_key(&new_url);
        // A navigation to a different host invalidates the favicon (it was the old
        // site's icon); clear it so a stale favicon does not linger on the tile until
        // the new one loads. A same-host path change keeps it. (Favicon-on-tile.)
        if host_changed {
            node.clear_image(crate::types::ImageRole::Favicon);
        }
        // Primary-first is Container's address-role mapping. Replace index 0;
        // aliases (the rest) stay attached.
        let new_primary_address = address_from_url(&new_url);
        if let Some(primary) = node.addresses.first_mut() {
            *primary = new_primary_address;
        }
        let url_changed = old_url != new_url;
        self.remove_url_mapping(&old_url, key);
        self.url_to_nodes.entry(new_url).or_default().push(key);
        if site_group_changed {
            self.bump_url_grouping_revision();
        }
        if url_changed {
            self.bump_content_revision();
        }
        Some(old_url)
    }

    /// Establish the current page binding for a live write, retaining an
    /// already recorded binding. Legacy replay does not call this helper.
    pub(crate) fn ensure_surface_resource(&mut self, key: NodeKey) -> Option<Uuid> {
        self.shown_resource_id(key)
            .or_else(|| self.refresh_surface_resource(key))
    }

    /// Refresh a live surface's page binding after its surface mutation has
    /// been captured. The resource record precedes the shown association.
    pub(crate) fn refresh_surface_resource(&mut self, key: NodeKey) -> Option<Uuid> {
        let node = self.get_node(key)?;
        let surface_id = node.id;
        let resource = ResourceNode::new(node.primary_address().as_url_str());
        let resource_id = resource.id();
        if self.resource(resource_id).is_none() {
            let record = crate::persistence::PersistedResourceRecord {
                canonical_iri: resource.canonical_iri().to_string(),
                facets: Vec::new(),
            };
            if !self.set_resource_record(resource_id, Some(record.clone())) {
                return None;
            }
            self.record_delta(&CapturedDelta::ReplaySetResourceRecordById {
                resource_id: resource_id.to_string(),
                record: Some(record),
            });
        }
        if self.set_shown_resource(surface_id, Some(resource_id)) {
            self.record_delta(&CapturedDelta::ReplaySetShownResourceById {
                surface_id: surface_id.to_string(),
                resource_id: Some(resource_id.to_string()),
            });
        }
        Some(resource_id)
    }

    /// Navigate `key` in place to `url`: record the visit in the node's own
    /// browse history (a forward-fork if the cursor had stepped back), write
    /// `url` as its primary address, and stamp its recency at `at_ms` and its
    /// session. A visit moves the trail and the address together; `SetNodeUrl`
    /// moves the address alone. No new node and no edge: the node is a browsing
    /// surface whose content changes; the graph shape does not. Cross-node
    /// lineage (the navigated-from relation) is a separate, explicit edge.
    ///
    /// The one mutation under both the live and the replayed arm, which differ
    /// only in the stamp they pass. Returns the node's id, or `None` if absent.
    pub(crate) fn navigate_node(
        &mut self,
        key: NodeKey,
        url: &str,
        transition: NodeHistoryTransitionKind,
        at_ms: u64,
        last_session_visited: u64,
    ) -> Option<Uuid> {
        let id = self.inner.node(key)?.id;
        self.nav.record_visit(id, url, transition, at_ms);
        self.set_node_last_session_visited(key, last_session_visited);
        self.update_node_url(key, url.to_string());
        self.set_node_last_visited_at_ms(key, at_ms);
        Some(id)
    }

    /// Anchor a freshly-minted `child` node's history under `parent`'s current
    /// visit, the navigated-from anchor. Do this **before** the child's first
    /// [`navigate_node`](Self::navigate_node) so that first visit attaches there
    /// in the shared lineage tree (the (b) cross-node anchor; the branch-mint
    /// path). Returns the `(child, parent)` ids, or `None` if either is absent.
    pub(crate) fn branch_history(
        &mut self,
        child: NodeKey,
        parent: NodeKey,
    ) -> Option<(Uuid, Uuid)> {
        let child_id = self.inner.node(child)?.id;
        let parent_id = self.inner.node(parent)?.id;
        self.nav.spawn(child_id, parent_id);
        Some((child_id, parent_id))
    }

    /// Step `key` back one visit in its own history, updating its Primary URL to
    /// the revealed page. Returns the new URL, or `None` if already at the root.
    pub(crate) fn node_history_back(&mut self, key: NodeKey, at_ms: u64) -> Option<String> {
        let id = self.inner.node(key)?.id;
        let url = self.nav.back(id, at_ms)?;
        self.land_history_step(key, &url, at_ms);
        Some(url)
    }

    /// Step `key` forward one visit in its own history. Returns the new URL, or
    /// `None` if already at the tip.
    pub(crate) fn node_history_forward(&mut self, key: NodeKey, at_ms: u64) -> Option<String> {
        let id = self.inner.node(key)?.id;
        let url = self.nav.forward(id, at_ms)?;
        self.land_history_step(key, &url, at_ms);
        Some(url)
    }

    /// What a moved history cursor leaves on the node: the revealed page as its
    /// primary address, and the step as its latest visit. The one place history
    /// writes the address: the cursor is the trail's position, and a step lands
    /// its page on the address even where a `SetNodeUrl` has moved it since.
    pub(super) fn land_history_step(&mut self, key: NodeKey, url: &str, at_ms: u64) {
        self.update_node_url(key, url.to_string());
        self.set_node_last_visited_at_ms(key, at_ms);
    }

    /// Whether `key`'s within-node history can step back (toolbar gating).
    pub fn node_can_back(&self, key: NodeKey) -> bool {
        self.inner
            .node(key)
            .is_some_and(|n| self.nav.can_back(n.id))
    }

    /// Whether `key`'s within-node history can step forward (toolbar gating).
    pub fn node_can_forward(&self, key: NodeKey) -> bool {
        self.inner
            .node(key)
            .is_some_and(|n| self.nav.can_forward(n.id))
    }

    /// `key`'s linear-history projection (active path + cursor).
    pub fn node_history_projection(&self, key: NodeKey) -> NodeHistoryProjection {
        match self.inner.node(key) {
            Some(node) => self.nav.projection(node.id),
            None => NodeHistoryProjection {
                entries: Vec::new(),
                current_index: 0,
            },
        }
    }

    /// `key`'s branching-history projection (visit tree with alternates).
    pub fn node_history_branch_projection(&self, key: NodeKey) -> NodeHistoryBranchProjection {
        self.inner
            .node(key)
            .map(|node| self.nav.branch_projection(node.id))
            .unwrap_or_default()
    }

    /// A coarse semantic summary of `key`'s history.
    pub fn node_history_semantic_summary(&self, key: NodeKey) -> NodeHistorySemanticSummary {
        self.inner
            .node(key)
            .map(|node| self.nav.semantic_summary(node.id))
            .unwrap_or_default()
    }

    /// Milliseconds since the Unix epoch, for visit timestamps (live-app clock).
    pub(super) fn epoch_ms() -> u64 {
        crate::time::unix_epoch_millis()
    }
}
