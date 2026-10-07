// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Exact visible graph inputs for the analytic layout cache.
//! Walk-dependent inputs must extend this stamp when those consumers change.

use super::*;

#[derive(PartialEq, Eq)]
struct StrategyGraphInputs {
    nodes: Vec<(NodeKey, uuid::Uuid)>,
    relations: Vec<(kernel::graph::RelationKey, kernel::graph::RelationView)>,
    payloads: Vec<(kernel::graph::RelationKey, kernel::graph::EdgePayload)>,
}

pub(super) struct StrategyGraphMemo {
    revision: u64,
    stamp: u64,
    inputs: StrategyGraphInputs,
}

impl Canvas {
    pub(super) fn strategy_graph_stamp(&self) -> u64 {
        let revision = self.graph.revision();
        let mut memo = self
            .strategy_graph_memo
            .lock()
            .expect("strategy input cache");
        if let Some(memo) = memo.as_ref()
            && memo.revision == revision
        {
            return memo.stamp;
        }
        let relations: Vec<_> = self.graph.projected_relations().collect();
        let mut seen = HashSet::new();
        let payloads = relations
            .iter()
            .filter_map(|(key, _)| {
                seen.insert(*key)
                    .then(|| (*key, self.graph.get_relation(*key).unwrap().clone()))
            })
            .collect();
        let inputs = StrategyGraphInputs {
            nodes: self
                .graph
                .nodes()
                .map(|(key, node)| (key, node.id))
                .collect(),
            relations,
            payloads,
        };
        let stamp = match memo.as_ref() {
            Some(previous) if previous.inputs == inputs => previous.stamp,
            Some(previous) => previous.stamp.wrapping_add(1),
            None => 1,
        };
        *memo = Some(StrategyGraphMemo {
            revision,
            stamp,
            inputs,
        });
        stamp
    }
}
