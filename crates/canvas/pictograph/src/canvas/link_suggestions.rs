// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Suggestions from the links a page carries that the graph does not yet hold
//! (graph grammar, *Suggestions and derived content*). Each admitted link is a
//! **derived member**: a session-only node, linked from its page, that the
//! host never persists. Selecting one keeps it: it stops being offered and the
//! host takes it through [`Canvas::take_kept_links`] to write it as the
//! person's own. Only as many links are minted as the suggestion settings
//! admit, so turning suggestions off removes them. A host that reinstalls the
//! graph drops derived members; the next frame mints them again.

use std::collections::{BTreeMap, HashSet};

use kernel::geometry::PortablePoint;
use kernel::graph::NodeKey;

use euclid::default::Point2D;
use kernel::graph::apply::{self as graph_apply, GraphDelta, apply_graph_delta};

use super::Canvas;
use super::build::hyperlink;

/// The suggestion source name links offer under.
pub const LINK_SOURCE: &str = "links";
/// How far from its page a derived member is seeded.
const SEED_RADIUS: f32 = 90.0;

/// A derived member the person selected, for the host to write as their own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeptLink {
    /// The canvas's derived member, replaced once the host's write lands.
    pub member: uuid::Uuid,
    pub url: String,
    /// The page the link was found on.
    pub origin: uuid::Uuid,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct LinkSuggestions {
    /// (page, link) in the order the pages offered them.
    offers: Vec<(uuid::Uuid, String)>,
    /// Pages that have offered (even nothing), so a host reads each once.
    pages: std::collections::BTreeSet<uuid::Uuid>,
    /// Derived members by id: (page, link, kept).
    derived: BTreeMap<uuid::Uuid, (uuid::Uuid, String, bool)>,
    kept: Vec<KeptLink>,
}

impl LinkSuggestions {
    pub(crate) fn is_derived(&self, member: uuid::Uuid) -> bool {
        self.derived.contains_key(&member)
    }

    /// Members on offer (derived and not yet kept).
    fn offered(&self) -> impl Iterator<Item = uuid::Uuid> + '_ {
        self.derived
            .iter()
            .filter(|(_, (_, _, kept))| !kept)
            .map(|(id, _)| *id)
    }
}

/// A link's member id: the same link mints the same id, every session.
fn link_member(url: &str) -> uuid::Uuid {
    let hash = |salt: u64| {
        url.bytes().fold(0xcbf2_9ce4_8422_2325 ^ salt, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
        })
    };
    let mut bytes = [0; 16];
    bytes[..8].copy_from_slice(&hash(0x6d65_7265).to_be_bytes());
    bytes[8..].copy_from_slice(&hash(0x6c69_6e6b).to_be_bytes());
    uuid::Builder::from_custom_bytes(bytes).into_uuid()
}

impl Canvas {
    /// Offer the links `page` carries, replacing what it offered before. Links
    /// the graph already holds are skipped; the settings decide how many show.
    pub fn offer_page_links(&mut self, page: uuid::Uuid, links: impl IntoIterator<Item = String>) {
        self.grammar.links.pages.insert(page);
        let held = self.held_urls();
        let mut fresh: Vec<(uuid::Uuid, String)> = Vec::new();
        for url in links {
            if !held.contains(&url) && !fresh.iter().any(|(_, u)| *u == url) {
                fresh.push((page, url));
            }
        }
        let offers = &mut self.grammar.links.offers;
        let before: Vec<_> = offers.iter().filter(|(p, _)| *p == page).cloned().collect();
        if before == fresh {
            return;
        }
        offers.retain(|(p, _)| *p != page);
        offers.extend(fresh);
        self.refresh_link_suggestions();
    }

    /// Whether `page` has offered its links already.
    pub fn has_offered_links(&self, page: uuid::Uuid) -> bool {
        self.grammar.links.pages.contains(&page)
    }

    /// Whether `member` is a derived member (session-only, never persisted).
    pub fn is_derived_member(&self, member: uuid::Uuid) -> bool {
        self.grammar.links.is_derived(member)
    }

    pub fn derived_members(&self) -> Vec<uuid::Uuid> {
        self.grammar.links.derived.keys().copied().collect()
    }

    /// The links kept since the last call, oldest first.
    pub fn take_kept_links(&mut self) -> Vec<KeptLink> {
        std::mem::take(&mut self.grammar.links.kept)
    }

    /// URLs the graph holds as the person's own (not derived).
    fn held_urls(&self) -> HashSet<String> {
        self.graph
            .nodes()
            .filter(|(_, node)| !self.grammar.links.is_derived(node.id))
            .map(|(_, node)| node.url().to_string())
            .collect()
    }

    /// Mint the admitted links as derived members and remove the rest.
    pub(crate) fn refresh_link_suggestions(&mut self) {
        let graph = &self.graph;
        self.grammar
            .links
            .derived
            .retain(|id, _| graph.get_node_key_by_id(*id).is_some());
        let held = self.held_urls();
        let others = self.grammar.offered.len();
        let derived = self.grammar.links.derived.len();
        let kept_now = self.graph.nodes().count().saturating_sub(others + derived);
        let budget = self
            .grammar
            .suggestions
            .budget(kept_now, self.plate_fill())
            .saturating_sub(others);
        let sources = &self.grammar.suggestions.sources;
        let allowed = sources.is_empty() || sources.iter().any(|s| s == LINK_SOURCE);
        let mut seen = HashSet::new();
        let wanted: Vec<(uuid::Uuid, String)> = self
            .grammar
            .links
            .offers
            .iter()
            .filter(|(page, url)| {
                allowed
                    && !held.contains(url)
                    && self.graph.get_node_key_by_id(*page).is_some()
                    && seen.insert(url.clone())
            })
            .take(budget)
            .cloned()
            .collect();
        let mut changed = false;
        let stale: Vec<uuid::Uuid> = self
            .grammar
            .links
            .derived
            .iter()
            .filter(|(_, (_, url, kept))| !kept && !wanted.iter().any(|(_, u)| u == url))
            .map(|(id, _)| *id)
            .collect();
        for id in stale {
            if let Some(key) = self.graph.get_node_key_by_id(id) {
                let _ = apply_graph_delta(&mut self.graph, GraphDelta::RemoveNode { key });
            }
            self.grammar.links.derived.remove(&id);
            changed = true;
        }
        let mut seeds = Vec::new();
        for (page, url) in wanted {
            let id = link_member(&url);
            if self.grammar.links.derived.contains_key(&id) {
                continue;
            }
            let Some(origin) = self.graph.get_node_key_by_id(page) else {
                continue;
            };
            let center = self
                .view
                .position_of(origin)
                .map_or(sceno::Vec2::ZERO, |p| sceno::Vec2::new(p.x, p.y));
            let at = scenograph::seed::identity_seed(&url, center, SEED_RADIUS);
            let key = graph_apply::add_node(
                &mut self.graph,
                Some(id),
                url.clone(),
                PortablePoint::new(at.x, at.y),
            );
            let _ = graph_apply::assert_relation(&mut self.graph, origin, key, hyperlink());
            self.grammar.links.derived.insert(id, (page, url, false));
            seeds.push((key, Point2D::new(at.x, at.y)));
            changed = true;
        }
        if changed {
            self.reconcile_derived();
            for (key, at) in &seeds {
                self.view.set_position(*key, *at);
            }
            self.physics.seed(seeds);
        }
        self.admit_suggestions();
    }

    /// Per frame: keep a selected derived member, and mint again after a host
    /// reinstalled the graph.
    pub(crate) fn tend_link_suggestions(&mut self) {
        let selected: Vec<NodeKey> = self.selected.iter().copied().collect();
        for key in selected {
            let Some(id) = self.graph.get_node(key).map(|node| node.id) else {
                continue;
            };
            if let Some((origin, url, kept)) = self.grammar.links.derived.get_mut(&id)
                && !*kept
            {
                *kept = true;
                let (origin, url) = (*origin, url.clone());
                self.grammar.links.offers.retain(|(_, u)| *u != url);
                self.grammar.links.kept.push(KeptLink {
                    member: id,
                    url,
                    origin,
                });
                self.admit_suggestions();
            }
        }
        let graph = &self.graph;
        let lost = self
            .grammar
            .links
            .derived
            .keys()
            .any(|id| graph.get_node_key_by_id(*id).is_none());
        if lost {
            self.refresh_link_suggestions();
        }
    }

    pub(crate) fn link_suggestion_members(&self) -> Vec<(NodeKey, String)> {
        self.grammar
            .links
            .offered()
            .filter_map(|id| Some((self.graph.get_node_key_by_id(id)?, LINK_SOURCE.to_string())))
            .collect()
    }
}
