// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The furnace's graph passes: forgetting and consolidation over a mere.
//!
//! Moved from Athanor 2026-10-31 (distillery-extraction D12): these passes read
//! and write a mere's graph, codicils and content store, so they stay with the
//! graph when Distillery leaves Mere. Athanor keeps the workshop.
//!
//! The forgetting pass proposes which short-term cached content to evict and
//! applies an accepted proposal by dropping that content. It honours eidetic
//! **R0**: the `propose_*` half is pure (reads a snapshot, mutates nothing), and
//! the `apply_*` half drops only short-term **cached content** (`content_store`
//! blobs), never graph truth and never codicils.
//!
//! Consolidation relates graph codicils that are successive versions of the same
//! material (significant URL overlap) but carry no lineage link yet. Composing is
//! the only way eidetic links two saved manifests, so `apply_consolidation` reuses
//! [`graph_codicil::compose_graph_codicils`], which is content-addressed: re-linking
//! an already-consolidated pair is a no-op.

use std::collections::{HashMap, HashSet};

use eidetic::{ManifestId, ProvenanceOrigin, Result, Store, Timestamp};
use kernel::graph::Graph;
use kernel::persistence::GraphSnapshot;
use kernel::types::ImageRole;

use crate::memory_levels::{EvictionPolicy, evictable_short_term};
// The retention halves moved to `eidetic::memory` (engine split D45); their
// names stay here for callers.
pub use eidetic::memory::{
    ForgetProposal, RetirementProposal, apply_forgetting, apply_retirement, propose_retirement,
};
use crate::graph_codicil::{self, RedactionPolicy};
use crate::image_store;

/// How many of the most-recently-created codicils a consolidation pass considers.
/// Bounds the pairwise scan (thaw is per-candidate, comparison is per-pair) as the
/// store grows; revisit if codicils get numerous enough for this to matter (like B6).
const CONSOLIDATION_CANDIDATE_CAP: usize = 50;

/// The minimum URL-overlap fraction (of the smaller codicil's url set) for a pair to
/// count as "the same material" rather than coincidental overlap (e.g. two unrelated
/// sessions that both visited one common site). A first-cut heuristic; tune once
/// real consolidation proposals can be eyeballed.
const SAME_MATERIAL_OVERLAP: f64 = 0.5;

/// Propose forgetting: the urls of the short-term nodes `policy` would evict as of
/// `now_ms` / `current_session`, given per-node last-visit timing (`last_visit_ms`,
/// keyed by `node_id`, supplied by the host from the graph's navigation history) for
/// the by-time policies, or each node's own `last_session_visited` stamp for
/// [`KeepSessions`](EvictionPolicy::KeepSessions).
///
/// Pure: it reads `snapshot` and mutates nothing (R0). The eviction decision is
/// [`evictable_short_term`](crate::memory_levels::evictable_short_term) (short-term
/// only, dated-and-stale only, promoted exempt); this maps the evictable node ids to
/// their urls, since cached content is url-keyed.
pub fn propose_forgetting(
    graph: &Graph,
    last_visit_ms: &HashMap<String, u64>,
    policy: EvictionPolicy,
    now_ms: u64,
    current_session: u64,
) -> ForgetProposal {
    let evictable: HashSet<String> =
        evictable_short_term(graph, last_visit_ms, policy, now_ms, current_session)
            .into_iter()
            .collect();
    let urls = graph
        .nodes()
        .filter(|(_, node)| evictable.contains(&node.id.to_string()))
        .map(|(_, node)| node.url().to_string())
        .filter(|url| !url.is_empty())
        .collect();
    ForgetProposal { urls }
}

/// Stored image blobs not named by any live node.
///
/// This is a mark/sweep proposal rather than a per-node deletion: image blobs
/// are content-addressed and shared, so one node dropping a reference cannot
/// decide that the blob itself is dead.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImageGcProposal {
    pub hexes: Vec<String>,
}

impl ImageGcProposal {
    pub fn is_empty(&self) -> bool {
        self.hexes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.hexes.len()
    }
}

fn referenced_image_hexes(snapshot: &GraphSnapshot) -> HashSet<String> {
    snapshot
        .nodes
        .iter()
        .flat_map(|node| node.images.values())
        .map(|image| image.hex())
        .collect()
}

/// Diff the image store's inventory against every live node reference. Pure:
/// the caller supplies the inventory and this mutates neither graph nor store.
pub fn propose_image_gc(
    snapshot: &GraphSnapshot,
    stored_hexes: impl IntoIterator<Item = String>,
) -> ImageGcProposal {
    let referenced = referenced_image_hexes(snapshot);
    let mut hexes: Vec<String> = stored_hexes
        .into_iter()
        .filter(|hex| !referenced.contains(hex))
        .collect();
    hexes.sort();
    hexes.dedup();
    ImageGcProposal { hexes }
}

/// Apply an accepted image-GC proposal, re-checking the current snapshot
/// before each delete so a reference added after proposal creation wins.
pub async fn apply_image_gc(
    store: &mut dyn Store,
    snapshot: &GraphSnapshot,
    proposal: &ImageGcProposal,
) -> Result<usize> {
    let referenced = referenced_image_hexes(snapshot);
    let mut dropped = 0;
    for hex in &proposal.hexes {
        if !referenced.contains(hex) && image_store::delete_image_hex(store, hex).await? {
            dropped += 1;
        }
    }
    Ok(dropped)
}

/// A role-specific reference Athanor proposes forgetting from a stale
/// short-term node. The digest is included so apply can refuse a stale
/// proposal after a newer capture replaces the role.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageReferenceDrop {
    pub node_id: String,
    pub role: ImageRole,
    pub hex: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImageReferenceForgetProposal {
    pub drops: Vec<ImageReferenceDrop>,
}

/// Propose disposable image references for the same stale short-term nodes as
/// ordinary content forgetting. Favicons are re-fetchable; preview/snapshot
/// roles are retained because they may be the only offline representation.
/// Promoted nodes never enter `evictable_short_term`.
pub fn propose_image_reference_forgetting(
    graph: &Graph,
    last_visit_ms: &HashMap<String, u64>,
    policy: EvictionPolicy,
    now_ms: u64,
    current_session: u64,
) -> ImageReferenceForgetProposal {
    let evictable: HashSet<String> =
        evictable_short_term(graph, last_visit_ms, policy, now_ms, current_session)
            .into_iter()
            .collect();
    let drops = graph
        .nodes()
        .filter(|(_, node)| evictable.contains(&node.id.to_string()))
        .filter_map(|node| {
            let (_, node) = node;
            node.images
                .get(&ImageRole::Favicon)
                .map(|image| ImageReferenceDrop {
                    node_id: node.id.to_string(),
                    role: ImageRole::Favicon,
                    hex: image.hex(),
                })
        })
        .collect();
    ImageReferenceForgetProposal { drops }
}

/// Apply role-specific reference forgetting to a snapshot before it
/// materializes, re-checking both role and digest. Returns references removed.
/// The now-unreferenced blob is reclaimed by the separate GC pass.
pub fn apply_image_reference_forgetting(
    snapshot: &mut GraphSnapshot,
    proposal: &ImageReferenceForgetProposal,
) -> usize {
    let mut dropped = 0;
    for proposal in &proposal.drops {
        let Some(node) = snapshot
            .nodes
            .iter_mut()
            .find(|node| node.node_id == proposal.node_id)
        else {
            continue;
        };
        let still_same = node
            .images
            .get(&proposal.role)
            .is_some_and(|image| image.hex() == proposal.hex);
        if still_same {
            node.images.remove(&proposal.role);
            dropped += 1;
        }
    }
    dropped
}

/// A proposal to consolidate: pairs of graph codicils that look like successive
/// versions of the same material but carry no lineage link yet.
///
/// Athanor emits this; the host decides whether to apply it (the R0 propose/apply
/// split). Applying a pair composes it (the only linking mechanism eidetic offers),
/// so `apply_consolidation` names it explicitly rather than leaving it implicit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConsolidationProposal {
    /// Each pair to relate. Composing a pair links them: the new codicil's
    /// `upstream` names both.
    pub pairs: Vec<(ManifestId, ManifestId)>,
}

impl ConsolidationProposal {
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }
}

/// Propose consolidation: pairs of graph codicils whose node-url sets overlap enough
/// to be "the same material" at different points in time, and that no existing
/// codicil already relates (no manifest's `upstream` already names both).
///
/// Reads the store — lists manifests (cheap; `provenance.upstream` rides on the
/// manifest, no thaw needed to check existing links) and thaws each of the newest
/// [`CONSOLIDATION_CANDIDATE_CAP`] *originally-generated* codicils once to compare url
/// sets — but writes nothing (R0). Only `Generated`-origin codicils are candidates:
/// composing a `Derived` (already-consolidated) codicil into another would keep
/// re-merging composites rather than relating fresh version chains.
pub async fn propose_consolidation(store: &mut dyn Store) -> Result<ConsolidationProposal> {
    let mut manifests = graph_codicil::list_graph_codicils(store).await?;
    manifests.sort_by_key(|m| std::cmp::Reverse(m.created_at.0));

    let already_linked = |a: ManifestId, b: ManifestId| {
        manifests
            .iter()
            .any(|m| m.provenance.upstream.contains(&a) && m.provenance.upstream.contains(&b))
    };

    let mut candidates: Vec<(ManifestId, HashSet<String>)> = Vec::new();
    for manifest in manifests
        .iter()
        .filter(|m| m.provenance.origin == ProvenanceOrigin::Generated)
        .take(CONSOLIDATION_CANDIDATE_CAP)
    {
        let Some(codicil) = graph_codicil::load_graph_codicil(store, manifest.id).await? else {
            continue;
        };
        let urls: HashSet<String> = codicil
            .snapshot
            .nodes
            .iter()
            .map(|n| n.url.clone())
            .filter(|u| !u.is_empty())
            .collect();
        candidates.push((manifest.id, urls));
    }

    let mut pairs = Vec::new();
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            let (id_a, urls_a) = &candidates[i];
            let (id_b, urls_b) = &candidates[j];
            let smaller = urls_a.len().min(urls_b.len());
            if smaller == 0 || already_linked(*id_a, *id_b) {
                continue;
            }
            let overlap = urls_a.intersection(urls_b).count();
            if (overlap as f64 / smaller as f64) >= SAME_MATERIAL_OVERLAP {
                pairs.push((*id_a, *id_b));
            }
        }
    }
    Ok(ConsolidationProposal { pairs })
}

/// Apply a consolidation proposal: compose each pair, linking them (R0: an accepted
/// proposal the host hands back). Returns how many pairs composed successfully —
/// genuine feedback, never a placebo, even though a pair already consolidated on a
/// prior pass re-composes to the same content-addressed id at no real cost.
pub async fn apply_consolidation(
    store: &mut dyn Store,
    proposal: &ConsolidationProposal,
    redaction: RedactionPolicy,
    created_at: Timestamp,
) -> Result<usize> {
    let mut linked = 0;
    for &(a, b) in &proposal.pairs {
        if graph_codicil::compose_graph_codicils(store, &[a, b], redaction, created_at)
            .await?
            .is_some()
        {
            linked += 1;
        }
    }
    Ok(linked)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use euclid::default::Point2D;
    use kernel::graph::Graph;
    use kernel::graph::fixtures::GraphFixtures;

    const DAY_MS: u64 = 86_400_000;

    /// A graph with three test nodes (stale / fresh / kept), built through
    /// the real graph API so node ids and urls are valid. "kept" is tagged (long-term).
    fn sample_graph() -> Graph {
        let mut graph = Graph::new();
        graph.add_node("https://stale.example/".to_string(), Point2D::new(0.0, 0.0));
        graph.add_node("https://fresh.example/".to_string(), Point2D::new(0.0, 0.0));
        let kept = graph.add_node("https://kept.example/".to_string(), Point2D::new(0.0, 0.0));
        graph.insert_node_tag(kept, "keep".to_string());
        graph
    }

    fn id_of(graph: &Graph, fragment: &str) -> String {
        graph
            .nodes()
            .find(|(_, node)| node.url().contains(fragment))
            .expect("a node with that url fragment")
            .1
            .id
            .to_string()
    }

    #[test]
    fn proposes_only_stale_short_term_urls() {
        let now = 100 * DAY_MS;
        let graph = sample_graph();
        let mut times = HashMap::new();
        times.insert(id_of(&graph, "stale"), now - 60 * DAY_MS); // stale -> propose
        times.insert(id_of(&graph, "fresh"), now - 2 * DAY_MS); // fresh -> keep
        times.insert(id_of(&graph, "kept"), now - 90 * DAY_MS); // stale but tagged -> exempt

        let proposal = propose_forgetting(&graph, &times, EvictionPolicy::KeepDays(30), now, 0);
        assert_eq!(proposal.len(), 1, "only the stale short-term node");
        assert!(proposal.urls[0].contains("stale"));
    }

    #[test]
    fn proposes_only_stale_short_term_urls_by_session() {
        let mut graph = sample_graph();
        for (fragment, session) in [("stale", 2), ("fresh", 9), ("kept", 1)] {
            let key = graph
                .nodes()
                .find(|(_, node)| node.url().contains(fragment))
                .unwrap()
                .0;
            let id = graph.get_node(key).unwrap().id;
            graph
                .facets_mut()
                .set(
                    id,
                    chartulary::FacetId::new(kernel::graph::node_facets::VISIT_HISTORY),
                    serde_json::to_value(kernel::graph::VisitHistoryFacet {
                        last_visited_ms: None,
                        last_session_visited: session,
                    })
                    .unwrap(),
                    &chartulary::AcceptAll,
                )
                .unwrap();
        }

        let proposal = propose_forgetting(
            &graph,
            &HashMap::new(),
            EvictionPolicy::KeepSessions(3),
            0,
            10,
        );
        assert_eq!(proposal.len(), 1, "only the stale short-term node");
        assert!(proposal.urls[0].contains("stale"));
    }

    // The in-memory test store is muniment's (2026-07-12): the
    // hand-rolled one was the same map behind the same seam.
    use muniment::MemoryBackend as MemStore;

    #[test]
    fn image_gc_marks_only_orphans_and_apply_rechecks_live_references() {
        pollster::block_on(async {
            let mut store = MemStore::default();
            let live = image_store::save_image(&mut store, b"live", 1, 1)
                .await
                .unwrap();
            let orphan = image_store::save_image(&mut store, b"orphan", 1, 1)
                .await
                .unwrap();
            let raced = image_store::save_image(&mut store, b"raced", 1, 1)
                .await
                .unwrap();
            let mut snapshot = sample_graph().to_snapshot();
            snapshot.nodes[0].images.insert(ImageRole::Favicon, live);

            let proposal = propose_image_gc(
                &snapshot,
                image_store::stored_image_hexes(&mut store).await.unwrap(),
            );
            assert_eq!(proposal.len(), 2);
            assert!(!proposal.hexes.contains(&live.hex()));
            assert!(proposal.hexes.contains(&orphan.hex()));
            assert!(proposal.hexes.contains(&raced.hex()));

            // A reference that appears after proposal creation wins at apply.
            snapshot.nodes[1].images.insert(ImageRole::Preview, raced);
            assert_eq!(
                apply_image_gc(&mut store, &snapshot, &proposal)
                    .await
                    .unwrap(),
                1
            );
            assert!(
                image_store::load_image(&mut store, &live)
                    .await
                    .unwrap()
                    .is_some()
            );
            assert!(
                image_store::load_image(&mut store, &raced)
                    .await
                    .unwrap()
                    .is_some()
            );
            assert!(
                image_store::load_image(&mut store, &orphan)
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(
                apply_image_gc(&mut store, &snapshot, &proposal)
                    .await
                    .unwrap(),
                0,
                "re-running the accepted proposal is a no-op"
            );
        });
    }

    #[test]
    fn stale_short_term_drops_favicon_but_keeps_precious_images() {
        let now = 100 * DAY_MS;
        let mut graph = sample_graph();
        let stale_id = id_of(&graph, "stale");
        let kept_id = id_of(&graph, "kept");
        let favicon = kernel::types::ImageRef::new([1; 32], 1, 1);
        let snapshot_image = kernel::types::ImageRef::new([2; 32], 1, 1);
        for (_, node) in graph
            .nodes()
            .map(|(key, node)| (key, node.id))
            .collect::<Vec<_>>()
        {
            if node.to_string() == stale_id || node.to_string() == kept_id {
                let key = graph.get_node_key_by_id(node).unwrap();
                let node = graph.get_node_mut(key).unwrap();
                node.images.insert(ImageRole::Favicon, favicon);
                node.images.insert(ImageRole::Snapshot, snapshot_image);
            }
        }
        let times = HashMap::from([
            (stale_id.clone(), now - 60 * DAY_MS),
            (kept_id.clone(), now - 60 * DAY_MS),
        ]);

        let proposal = propose_image_reference_forgetting(
            &graph,
            &times,
            EvictionPolicy::KeepDays(30),
            now,
            0,
        );
        assert_eq!(proposal.drops.len(), 1, "the promoted node is exempt");
        assert_eq!(proposal.drops[0].node_id, stale_id);
        let mut snapshot = graph.to_snapshot();
        assert_eq!(
            apply_image_reference_forgetting(&mut snapshot, &proposal),
            1
        );

        let stale = snapshot
            .nodes
            .iter()
            .find(|node| node.node_id == stale_id)
            .unwrap();
        assert!(!stale.images.contains_key(&ImageRole::Favicon));
        assert!(stale.images.contains_key(&ImageRole::Snapshot));
        let kept = snapshot
            .nodes
            .iter()
            .find(|node| node.node_id == kept_id)
            .unwrap();
        assert!(kept.images.contains_key(&ImageRole::Favicon));
        assert!(kept.images.contains_key(&ImageRole::Snapshot));
        assert_eq!(
            apply_image_reference_forgetting(&mut snapshot, &proposal),
            0,
            "reference forgetting is idempotent"
        );
    }

    /// Save a graph codicil over exactly `urls`, oldest-first by node insertion.
    async fn save_codicil(store: &mut MemStore, urls: &[&str], at_ms: u64) -> ManifestId {
        let mut graph = Graph::new();
        for (i, url) in urls.iter().enumerate() {
            graph.add_node(url.to_string(), Point2D::new(i as f32, 0.0));
        }
        graph_codicil::save_graph_codicil(
            store,
            &graph,
            RedactionPolicy::default(),
            Timestamp(at_ms),
        )
        .await
        .expect("save")
    }

    #[test]
    fn propose_consolidation_finds_overlapping_unlinked_codicils() {
        pollster::block_on(async {
            let mut store = MemStore::default();
            let a = save_codicil(&mut store, &["https://x.example", "https://y.example"], 1).await;
            let b = save_codicil(&mut store, &["https://y.example", "https://z.example"], 2).await;
            save_codicil(&mut store, &["https://q.example"], 3).await; // unrelated

            let proposal = propose_consolidation(&mut store).await.expect("propose");
            assert_eq!(proposal.len(), 1, "only the overlapping pair is proposed");
            let (p_a, p_b) = proposal.pairs[0];
            assert!(
                (p_a == a && p_b == b) || (p_a == b && p_b == a),
                "the proposed pair is a and b",
            );
        });
    }

    #[test]
    fn propose_consolidation_ignores_low_overlap_pairs() {
        pollster::block_on(async {
            let mut store = MemStore::default();
            // a and b both have 4 urls, sharing only 1 -> 1/4 = 0.25 of the smaller
            // (here, either) set, below the 0.5 threshold.
            save_codicil(
                &mut store,
                &[
                    "https://1.example",
                    "https://2.example",
                    "https://3.example",
                    "https://4.example",
                ],
                1,
            )
            .await;
            save_codicil(
                &mut store,
                &[
                    "https://4.example",
                    "https://5.example",
                    "https://6.example",
                    "https://7.example",
                ],
                2,
            )
            .await;

            let proposal = propose_consolidation(&mut store).await.expect("propose");
            assert!(
                proposal.is_empty(),
                "a lone shared url is not enough overlap"
            );
        });
    }

    #[test]
    fn apply_consolidation_composes_and_records_the_upstream_link() {
        pollster::block_on(async {
            let mut store = MemStore::default();
            let a = save_codicil(&mut store, &["https://x.example", "https://y.example"], 1).await;
            let b = save_codicil(&mut store, &["https://y.example", "https://z.example"], 2).await;

            let proposal = propose_consolidation(&mut store).await.expect("propose");
            assert_eq!(proposal.len(), 1);

            let linked = apply_consolidation(
                &mut store,
                &proposal,
                RedactionPolicy::default(),
                Timestamp(3),
            )
            .await
            .expect("apply");
            assert_eq!(linked, 1, "one pair composed");

            let manifests = graph_codicil::list_graph_codicils(&mut store)
                .await
                .expect("list");
            assert!(
                manifests
                    .iter()
                    .any(|m| m.provenance.upstream.contains(&a)
                        && m.provenance.upstream.contains(&b)),
                "a new codicil links a and b via upstream",
            );
        });
    }

    #[test]
    fn propose_consolidation_skips_a_pair_already_linked_by_a_prior_pass() {
        pollster::block_on(async {
            let mut store = MemStore::default();
            save_codicil(&mut store, &["https://x.example", "https://y.example"], 1).await;
            save_codicil(&mut store, &["https://y.example", "https://z.example"], 2).await;

            let first = propose_consolidation(&mut store).await.expect("propose");
            apply_consolidation(&mut store, &first, RedactionPolicy::default(), Timestamp(3))
                .await
                .expect("apply");

            // The two source codicils are still Generated (compose never mutates its
            // sources) and still overlap exactly as before, but a linking codicil
            // now names both -- a second pass must not re-propose them.
            let second = propose_consolidation(&mut store)
                .await
                .expect("propose again");
            assert!(
                second.is_empty(),
                "already-linked pairs are not re-proposed: {:?}",
                second.pairs,
            );
        });
    }
}
