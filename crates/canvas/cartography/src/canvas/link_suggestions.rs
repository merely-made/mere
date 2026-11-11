// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Suggestions from the links a page carries that the graph does not yet hold
//! (graph grammar, *Suggestions and derived content*). Each admitted link is a
//! **derived member**: a session-only node, linked from every page that offers
//! it, that the host never persists. Selecting one keeps it: it stops being
//! offered and the host takes it through [`Canvas::take_kept_links`] to write
//! it as the person's own. Which links are minted is the canvas's one admitted
//! set (`suggestions.rs`), so turning suggestions off removes them. A host that
//! reinstalls the graph drops derived members; the next frame mints them again.

use std::collections::{BTreeMap, HashMap, HashSet};

use kernel::geometry::PortablePoint;
use kernel::graph::NodeKey;

use euclid::default::Point2D;
use kernel::graph::apply::{self as graph_apply, GraphDelta, apply_graph_delta};
use kernel::graph::{RelationSelector, SemanticSubKind};

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
    /// A page that still held the link when it was kept (the least id when
    /// several did); `None` when no page did.
    pub origin: Option<uuid::Uuid>,
}

/// A link on offer: its member id and the live pages that offer it, in the
/// order they offered. Pages only dedupe the member; the graph relation from
/// each page is what says where a link was found.
pub(crate) struct LinkOffer {
    pub(crate) member: uuid::Uuid,
    url: String,
    pages: Vec<uuid::Uuid>,
}

#[derive(Clone, Debug)]
struct Derived {
    url: String,
    kept: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct LinkSuggestions {
    /// (page, link) in the order the pages offered them.
    offers: Vec<(uuid::Uuid, String)>,
    /// Pages that have offered (even nothing), so a host reads each once.
    pages: std::collections::BTreeSet<uuid::Uuid>,
    /// Derived members by id.
    derived: BTreeMap<uuid::Uuid, Derived>,
    kept: Vec<KeptLink>,
}

impl LinkSuggestions {
    pub(crate) fn is_derived(&self, member: uuid::Uuid) -> bool {
        self.derived.contains_key(&member)
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
        self.admit_suggestions();
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

    /// Forget derived members the graph no longer holds.
    pub(crate) fn prune_derived(&mut self) {
        let graph = &self.graph;
        self.grammar
            .links
            .derived
            .retain(|id, _| graph.get_node_key_by_id(*id).is_some());
    }

    /// The links that could be minted: offered by a live page, not held and
    /// not already kept, one per URL in the order first offered.
    pub(crate) fn link_candidates(&self) -> Vec<LinkOffer> {
        let held = self.held_urls();
        let links = &self.grammar.links;
        let kept: HashSet<&str> = links
            .derived
            .values()
            .filter(|d| d.kept)
            .map(|d| d.url.as_str())
            .collect();
        let mut index: HashMap<&str, usize> = HashMap::new();
        let mut out: Vec<LinkOffer> = Vec::new();
        for (page, url) in &links.offers {
            if held.contains(url)
                || kept.contains(url.as_str())
                || self.graph.get_node_key_by_id(*page).is_none()
            {
                continue;
            }
            let at = *index.entry(url).or_insert_with(|| {
                out.push(LinkOffer {
                    member: link_member(url),
                    url: url.clone(),
                    pages: Vec::new(),
                });
                out.len() - 1
            });
            if !out[at].pages.contains(page) {
                out[at].pages.push(*page);
            }
        }
        out
    }

    /// Make the derived members the admitted ones: remove those no longer
    /// admitted, mint those that are new and keep each one's hyperlink from
    /// every page that offers it (and from no page that stopped).
    pub(crate) fn sync_link_members(&mut self, offers: &[LinkOffer], admitted: &[uuid::Uuid]) {
        let admitted: HashSet<uuid::Uuid> = admitted.iter().copied().collect();
        let offers: Vec<&LinkOffer> = offers
            .iter()
            .filter(|offer| admitted.contains(&offer.member))
            .collect();
        let live: HashSet<uuid::Uuid> = offers.iter().map(|offer| offer.member).collect();
        let stale: Vec<uuid::Uuid> = self
            .grammar
            .links
            .derived
            .iter()
            .filter(|(id, d)| !d.kept && !live.contains(*id))
            .map(|(id, _)| *id)
            .collect();
        let mut changed = !stale.is_empty();
        for id in stale {
            if let Some(key) = self.graph.get_node_key_by_id(id) {
                let _ = apply_graph_delta(&mut self.graph, GraphDelta::RemoveNode { key });
            }
            self.grammar.links.derived.remove(&id);
        }
        let mut seeds = Vec::new();
        for offer in offers {
            let pages: Vec<NodeKey> = offer
                .pages
                .iter()
                .filter_map(|page| self.graph.get_node_key_by_id(*page))
                .collect();
            let Some(first) = pages.first().copied() else {
                continue;
            };
            let key = match self.graph.get_node_key_by_id(offer.member) {
                Some(key) if self.grammar.links.is_derived(offer.member) => key,
                _ => {
                    let (key, at) = self.mint_link(offer, first);
                    seeds.push((key, at));
                    changed = true;
                    key
                },
            };
            changed |= self.sync_link_relations(key, &pages);
        }
        if changed {
            self.reconcile_derived();
            for (key, at) in &seeds {
                self.view.set_position(*key, *at);
            }
            self.physics.seed(seeds);
        }
    }

    /// Add `offer`'s derived member, seeded beside the page `first` that
    /// offered it first.
    fn mint_link(&mut self, offer: &LinkOffer, first: NodeKey) -> (NodeKey, Point2D<f32>) {
        let center = self
            .view
            .position_of(first)
            .map_or(sceno::Vec2::ZERO, |p| sceno::Vec2::new(p.x, p.y));
        let at = scenograph::seed::identity_seed(&offer.url, center, SEED_RADIUS);
        let key = graph_apply::add_node(
            &mut self.graph,
            Some(offer.member),
            offer.url.clone(),
            PortablePoint::new(at.x, at.y),
        );
        let derived = Derived {
            url: offer.url.clone(),
            kept: false,
        };
        self.grammar.links.derived.insert(offer.member, derived);
        (key, Point2D::new(at.x, at.y))
    }

    /// Hold a hyperlink from each of `pages` to `member` and withdraw the ones
    /// from pages that offered links before but no longer offer this one.
    /// Returns whether the graph changed.
    fn sync_link_relations(&mut self, member: NodeKey, pages: &[NodeKey]) -> bool {
        let mut holders = self.link_holders(member);
        let mut changed = false;
        for page in pages {
            if !holders.contains(page) {
                changed |=
                    graph_apply::assert_relation(&mut self.graph, *page, member, hyperlink())
                        .is_some();
            }
        }
        holders.retain(|holder| !pages.contains(holder));
        for holder in holders {
            let offered = self
                .graph
                .get_node(holder)
                .is_some_and(|node| self.grammar.links.pages.contains(&node.id));
            if offered {
                let selector = RelationSelector::Semantic(SemanticSubKind::Hyperlink);
                let _ = apply_graph_delta(
                    &mut self.graph,
                    GraphDelta::RetractRelations {
                        from: holder,
                        to: member,
                        selector,
                    },
                );
                changed = true;
            }
        }
        changed
    }

    /// The nodes holding a hyperlink to `member`, whichever stratum holds it.
    fn link_holders(&self, member: NodeKey) -> Vec<NodeKey> {
        let hyperlink = RelationSelector::Semantic(SemanticSubKind::Hyperlink);
        self.graph
            .projected_incoming_relations(member)
            .filter(|(_, _, payload)| payload.has_relation(hyperlink))
            .map(|(from, _, _)| from)
            .collect()
    }

    /// Per frame: keep a selected derived member, and mint again after a host
    /// reinstalled the graph.
    pub(crate) fn tend_link_suggestions(&mut self) {
        let selected: Vec<NodeKey> = self.selected.iter().copied().collect();
        let mut kept_any = false;
        for key in selected {
            let Some(id) = self.graph.get_node(key).map(|node| node.id) else {
                continue;
            };
            let url = match self.grammar.links.derived.get(&id) {
                Some(derived) if !derived.kept => derived.url.clone(),
                _ => continue,
            };
            let origin = self.link_origin(key);
            if let Some(derived) = self.grammar.links.derived.get_mut(&id) {
                derived.kept = true;
            }
            self.grammar.links.offers.retain(|(_, u)| *u != url);
            self.grammar.links.kept.push(KeptLink {
                member: id,
                url,
                origin,
            });
            kept_any = true;
        }
        let graph = &self.graph;
        let lost = self
            .grammar
            .links
            .derived
            .keys()
            .any(|id| graph.get_node_key_by_id(*id).is_none());
        if kept_any || lost {
            self.admit_suggestions();
        }
    }

    /// The page `member` was found on, read from the graph at keep time: the
    /// least id among the pages still holding a hyperlink to it.
    fn link_origin(&self, member: NodeKey) -> Option<uuid::Uuid> {
        self.link_holders(member)
            .into_iter()
            .filter_map(|page| self.graph.get_node(page).map(|node| node.id))
            .min()
    }
}
