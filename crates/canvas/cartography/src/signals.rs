// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Narrow contract for signals from intelligence layers.
//!
//! Cartography does not depend on `embed`. Producers
//! of these signals construct an [`IntelligenceSignals`] value and
//! hand it to cartography; the signal-producer crate's internal shapes
//! never leak through this type.
//!
//! Signals are keyed by channel id, `family.option` (dynamics grammar plan,
//! F32 and F55), each a variant of one [`Signal`] type (F86, "Keyed
//! signals"). The host's channel registry computes them; cartography
//! computes none. An adapter names the ids it reads and reports one that is
//! missing or of the wrong kind ([`SignalFault`]). View configuration (a
//! host's axes, the extents) stays on [`crate::ViewIntent`].

use std::collections::BTreeMap;

use kernel::graph::NodeKey;
use serde::{Deserialize, Serialize};

/// The enumeration order every score's ordinal follows (F84): the order is
/// computed once, by the host's registry, and travels in the request.
pub const ORDER_TIMELINE: &str = "order.timeline";
/// The graph Laplacian's coordinates, which Spectral places by.
pub const COORDS_SPECTRAL: &str = "coords.spectral";
/// Coordinates a host's own projection produced (UMAP, t-SNE, PCA), which
/// the semantic-embedding strategy places by (*Reading, not ruled*: the id,
/// G2b's question 4 (a), ruled as F55's pattern).
pub const COORDS_HOST: &str = "coords.host";
/// Degree plus one, which Radial's weighted policy spreads its rings by.
pub const WEIGHT_DEGREE: &str = "weight.degree";
/// Breadth-first rings from the view's focus, Radial's.
pub const RINGS_FOCUS: &str = "rings.focus";

/// Signals from intelligence layers that strategies can consume, keyed by
/// channel id.
///
/// A strategy reads the ids it needs and ignores the rest; a host supplies
/// the ids its strategies read. Serialized as a map from id to signal.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IntelligenceSignals {
    signals: BTreeMap<String, Signal>,
}

/// One channel's values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Signal {
    /// A partition into named clusters (`groups.cluster`).
    Groups(ClusterSet),
    /// Weighted node pairs (`pairs.*`).
    Pairs(AffinityScores),
    /// A set of nodes (`groups.bridges`).
    Nodes(BridgeNodes),
    /// A weight per node (`weight.degree`, `importance.*`).
    Weights(ImportanceWeights),
    /// A coordinate pair per node (`coords.spectral`, `coords.host`).
    Coords(NodeEmbeddings),
    /// A ring index per node (`rings.focus`).
    Rings(NodeRings),
    /// Every node, in order (`order.timeline`).
    Order(NodeOrder),
}

/// Which variant a [`Signal`] is, for a fault's report.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalKind {
    Groups,
    Pairs,
    Nodes,
    Weights,
    Coords,
    Rings,
    Order,
}

impl Signal {
    pub fn kind(&self) -> SignalKind {
        match self {
            Signal::Groups(_) => SignalKind::Groups,
            Signal::Pairs(_) => SignalKind::Pairs,
            Signal::Nodes(_) => SignalKind::Nodes,
            Signal::Weights(_) => SignalKind::Weights,
            Signal::Coords(_) => SignalKind::Coords,
            Signal::Rings(_) => SignalKind::Rings,
            Signal::Order(_) => SignalKind::Order,
        }
    }
}

/// Why a strategy could not read a channel: absent, or present as another
/// kind. Reported on the projection's metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalFault {
    Missing {
        id: String,
    },
    Mistyped {
        id: String,
        expected: SignalKind,
        found: SignalKind,
    },
}

impl std::fmt::Display for SignalFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalFault::Missing { id } => write!(f, "channel {id} is missing"),
            SignalFault::Mistyped {
                id,
                expected,
                found,
            } => write!(f, "channel {id} is {found:?}, not {expected:?}"),
        }
    }
}

macro_rules! typed_read {
    ($(#[$doc:meta])* $name:ident, $variant:ident, $ty:ty) => {
        $(#[$doc])*
        pub fn $name(&self, id: &str) -> Result<&$ty, SignalFault> {
            match self.signals.get(id) {
                Some(Signal::$variant(value)) => Ok(value),
                Some(other) => Err(SignalFault::Mistyped {
                    id: id.to_string(),
                    expected: SignalKind::$variant,
                    found: other.kind(),
                }),
                None => Err(SignalFault::Missing { id: id.to_string() }),
            }
        }
    };
}

impl IntelligenceSignals {
    pub fn new() -> Self {
        Self::default()
    }

    /// `signal` under `id`, replacing any signal there.
    pub fn insert(&mut self, id: impl Into<String>, signal: Signal) {
        self.signals.insert(id.into(), signal);
    }

    /// With `signal` under `id`.
    pub fn with(mut self, id: impl Into<String>, signal: Signal) -> Self {
        self.insert(id, signal);
        self
    }

    pub fn get(&self, id: &str) -> Option<&Signal> {
        self.signals.get(id)
    }

    /// Every id held, in order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.signals.keys().map(String::as_str)
    }

    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }

    typed_read!(
        /// The partition under `id`.
        groups, Groups, ClusterSet
    );
    typed_read!(
        /// The pairs under `id`.
        pairs, Pairs, AffinityScores
    );
    typed_read!(
        /// The node set under `id`.
        nodes, Nodes, BridgeNodes
    );
    typed_read!(
        /// The weights under `id`.
        weights, Weights, ImportanceWeights
    );
    typed_read!(
        /// The coordinates under `id`.
        coords, Coords, NodeEmbeddings
    );
    typed_read!(
        /// The rings under `id`.
        rings, Rings, NodeRings
    );
    typed_read!(
        /// The order under `id`.
        order, Order, NodeOrder
    );
}

/// A ring index per node, from a focus.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeRings {
    pub rings: Vec<(NodeKey, u32)>,
}

/// Every node, in an order.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeOrder {
    pub order: Vec<NodeKey>,
}

/// A partition of nodes into named clusters with confidence scores.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ClusterSet {
    pub clusters: Vec<Cluster>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cluster {
    pub id: String,
    pub label: Option<String>,
    pub members: Vec<NodeKey>,
    pub confidence: f32,
}

/// Pairwise affinity scores (e.g. embedding cosine similarity).
///
/// `pairs[(a, b)]` is the affinity from node `a` to node `b`; not
/// necessarily symmetric in general (an embedding model may produce
/// non-symmetric scores when query/document roles differ). Use
/// [`Self::lookup`] to canonicalize when symmetry is expected.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AffinityScores {
    pub pairs: Vec<((NodeKey, NodeKey), f32)>,
}

impl AffinityScores {
    /// Look up the score for an ordered pair, falling back to the
    /// reverse pair if the forward pair isn't recorded.
    pub fn lookup(&self, from: NodeKey, to: NodeKey) -> Option<f32> {
        self.pairs
            .iter()
            .find(|((a, b), _)| (*a == from && *b == to) || (*a == to && *b == from))
            .map(|(_, score)| *score)
    }
}

/// Nodes that bridge communities. Detected by embed
/// or by graph-structural betweenness.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BridgeNodes {
    pub bridges: Vec<NodeKey>,
}

/// Per-node importance weight (normalized 0..=1).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ImportanceWeights {
    pub weights: Vec<(NodeKey, f32)>,
}

impl ImportanceWeights {
    pub fn lookup(&self, node: NodeKey) -> Option<f32> {
        self.weights
            .iter()
            .find(|(n, _)| *n == node)
            .map(|(_, w)| *w)
    }
}

/// Per-node 2D embedding coordinates from an external projection
/// (UMAP / t-SNE / PCA / similar). Hosts that have a projection
/// pipeline populate this; the
/// [`crate::adapters::SemanticEmbeddingAdapter`]
/// strategy reads it to place nodes at embedded positions directly.
///
/// Coordinates are typically in a host-chosen range (`[-1, 1]` or
/// `[0, 1]`); strategies apply their own `scale` and `origin` config
/// to map them to world-space layout positions. Stored as `(f32, f32)`
/// tuples rather than a geometry type so this contract stays
/// dependency-light.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeEmbeddings {
    pub coords: Vec<(NodeKey, (f32, f32))>,
}

impl NodeEmbeddings {
    pub fn lookup(&self, node: NodeKey) -> Option<(f32, f32)> {
        self.coords
            .iter()
            .find(|(n, _)| *n == node)
            .map(|(_, xy)| *xy)
    }

    pub fn is_empty(&self) -> bool {
        self.coords.is_empty()
    }

    pub fn len(&self) -> usize {
        self.coords.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intelligence_signals_default_holds_no_channel() {
        let s = IntelligenceSignals::default();
        assert!(s.is_empty());
        assert_eq!(
            s.coords(COORDS_SPECTRAL),
            Err(SignalFault::Missing {
                id: COORDS_SPECTRAL.into()
            })
        );
    }

    /// F86: a channel is read by id and kind; an absent id and a present id
    /// of another kind are each reported, and the right kind reads back.
    #[test]
    fn a_keyed_signal_reads_by_id_and_reports_a_missing_or_mistyped_channel() {
        let a = NodeKey::new(0);
        let signals = IntelligenceSignals::new()
            .with(ORDER_TIMELINE, Signal::Order(NodeOrder { order: vec![a] }))
            .with(
                WEIGHT_DEGREE,
                Signal::Weights(ImportanceWeights {
                    weights: vec![(a, 2.0)],
                }),
            );
        assert_eq!(signals.order(ORDER_TIMELINE).unwrap().order, vec![a]);
        assert_eq!(signals.weights(WEIGHT_DEGREE).unwrap().lookup(a), Some(2.0));
        assert_eq!(
            signals.rings(ORDER_TIMELINE),
            Err(SignalFault::Mistyped {
                id: ORDER_TIMELINE.into(),
                expected: SignalKind::Rings,
                found: SignalKind::Order,
            })
        );
        assert_eq!(
            signals.rings(RINGS_FOCUS),
            Err(SignalFault::Missing {
                id: RINGS_FOCUS.into()
            })
        );
        assert_eq!(
            signals.ids().collect::<Vec<_>>(),
            vec![ORDER_TIMELINE, WEIGHT_DEGREE]
        );
    }

    /// The keyed shape on the wire: a map from id to signal, which reads
    /// back whole.
    #[test]
    fn keyed_signals_round_trip_as_a_map_by_id() {
        let a = NodeKey::new(3);
        let signals = IntelligenceSignals::new().with(
            RINGS_FOCUS,
            Signal::Rings(NodeRings {
                rings: vec![(a, 1)],
            }),
        );
        let json = serde_json::to_string(&signals).unwrap();
        assert!(json.starts_with("{\"rings.focus\":"), "{json}");
        let back: IntelligenceSignals = serde_json::from_str(&json).unwrap();
        assert_eq!(back, signals);
    }

    #[test]
    fn node_embeddings_lookup_returns_recorded_coord() {
        let a = NodeKey::new(0);
        let b = NodeKey::new(1);
        let embeddings = NodeEmbeddings {
            coords: vec![(a, (0.5, -0.3))],
        };
        assert_eq!(embeddings.lookup(a), Some((0.5, -0.3)));
        assert_eq!(embeddings.lookup(b), None);
        assert!(!embeddings.is_empty());
        assert_eq!(embeddings.len(), 1);
    }

    #[test]
    fn affinity_scores_lookup_returns_reverse_pair_when_forward_missing() {
        let a = NodeKey::new(0);
        let b = NodeKey::new(1);
        let scores = AffinityScores {
            pairs: vec![((a, b), 0.75)],
        };
        assert_eq!(scores.lookup(a, b), Some(0.75));
        assert_eq!(scores.lookup(b, a), Some(0.75));
    }

    #[test]
    fn affinity_scores_lookup_returns_none_for_unrecorded_pair() {
        let a = NodeKey::new(0);
        let b = NodeKey::new(1);
        let c = NodeKey::new(2);
        let scores = AffinityScores {
            pairs: vec![((a, b), 0.75)],
        };
        assert_eq!(scores.lookup(a, c), None);
    }

    #[test]
    fn importance_weights_lookup_returns_recorded_weight() {
        let a = NodeKey::new(0);
        let weights = ImportanceWeights {
            weights: vec![(a, 0.9)],
        };
        assert_eq!(weights.lookup(a), Some(0.9));
        assert_eq!(weights.lookup(NodeKey::new(1)), None);
    }
}
