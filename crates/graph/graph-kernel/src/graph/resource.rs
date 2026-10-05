// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resource identity beneath browsing surfaces.

use chartulary::{Address, Addressed, Identified};
use uuid::Uuid;

/// One resource identified by a canonical IRI.
///
/// Identity and address are immutable together. Content records will be
/// attached through the kernel's recorded write path.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceNode {
    container: chartulary::Container<Uuid>,
}

impl ResourceNode {
    /// Identify a resource using the common canonicalizer and UUID namespace.
    pub fn new(iri: &str) -> Self {
        let canonical = chartulary::canonical_url(iri);
        Self {
            container: chartulary::Container::with_identity(chartulary::resource_id(&canonical))
                .with_address_record(Address::new(canonical)),
        }
    }

    /// The stable identity, shared by every surface showing this resource.
    pub fn id(&self) -> Uuid {
        self.container.id
    }

    /// The canonical IRI from which the stable identity was derived.
    pub fn canonical_iri(&self) -> &str {
        self.container.addresses[0].as_str()
    }
}

impl Identified for ResourceNode {
    type Id = Uuid;

    fn id(&self) -> &Uuid {
        &self.container.id
    }
}

impl Addressed for ResourceNode {
    fn addresses(&self) -> Vec<Address> {
        self.container.addresses.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Graph, SurfaceNode};

    #[test]
    fn resource_identity_is_common_and_canonical_without_renumbering_surfaces() {
        let iri = "https://example.com/page?id=7";
        let alias = "https://EXAMPLE.COM:443/page?utm_source=news&id=7#part";
        let resource = ResourceNode::new(alias);
        assert_eq!(resource.canonical_iri(), iri);
        assert_eq!(resource.id(), chartulary::resource_id(iri));
        assert_eq!(*Identified::id(&resource), resource.id());
        assert_eq!(resource.primary_address(), Some(Address::new(iri)));
        assert_eq!(resource, ResourceNode::new(iri));
        assert_ne!(
            resource.id(),
            ResourceNode::new("https://example.com/page?id=8").id()
        );
        assert_ne!(resource.id(), Graph::node_namespace_id(iri));

        let first = SurfaceNode::test_stub(iri);
        let second = SurfaceNode::test_stub(alias);
        assert_ne!(first.id, second.id);
        assert_eq!(
            ResourceNode::new(first.url()),
            ResourceNode::new(second.url())
        );
    }

    #[test]
    fn chartulary_indexes_resources_by_the_shared_identity() {
        let mut graph = chartulary::Graph::<ResourceNode, ()>::new();
        let first = graph.insert(ResourceNode::new("https://example.com/page"));
        let alias = graph.insert(ResourceNode::new("https://EXAMPLE.COM/page#part"));
        assert_eq!(first, alias);
        assert_eq!(graph.node_count(), 1);
        let other = graph.insert(ResourceNode::new("https://example.com/other"));
        assert_ne!(first, other);
        assert_eq!(graph.node_count(), 2);
        let clone = graph.clone();
        assert_eq!(
            clone.get(&chartulary::resource_id("https://example.com/page")),
            graph.node(first)
        );
        assert_eq!(
            clone.get(&chartulary::resource_id("https://example.com/other")),
            graph.node(other)
        );
    }
}
