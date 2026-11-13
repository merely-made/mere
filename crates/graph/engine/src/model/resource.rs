// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resource identity beneath browsing surfaces. Moved from the kernel's
//! `graph::resource` (engine split P5 L3).

use super::{FrozenLink, FrozenSelection};
use portolan::{Address, Addressed, Identified};
use uuid::Uuid;

/// A tagging assertion points from the tagged resource to its concept resource.
pub const TAGGED_WITH_IRI: &str = "https://mere.computer/ns/rel#taggedWith";

/// One resource identified by a canonical IRI.
///
/// Identity and address are immutable together. Content records will be
/// attached through the kernel's recorded write path.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceNode {
    container: portolan::Container<Uuid>,
    nested_selection: Option<FrozenSelection>,
    nested_link: Option<FrozenLink>,
}

impl ResourceNode {
    /// Identify a resource using the common canonicalizer and UUID namespace.
    pub fn new(iri: &str) -> Self {
        let canonical = portolan::canonical_url(iri);
        Self::from_canonical_iri(&canonical)
    }

    /// Identify a vocabulary term by its exact IRI, including fragment and case.
    /// Page resources use [`Self::new`] instead.
    pub fn for_term(iri: &str) -> Self {
        Self::from_canonical_iri(iri)
    }

    /// Identify a resource by an already canonical IRI.
    pub fn from_canonical_iri(canonical: &str) -> Self {
        Self {
            nested_selection: None,
            nested_link: None,
            container: portolan::Container::with_identity(
                portolan::resource_id_from_canonical_iri(canonical),
            )
            .with_address_record(Address::new(canonical)),
        }
    }

    /// The same resource bearing these frozen readings (decoded from its record).
    pub fn with_frozen(
        mut self,
        selection: Option<FrozenSelection>,
        link: Option<FrozenLink>,
    ) -> Self {
        self.nested_selection = selection;
        self.nested_link = link;
        self
    }

    /// Immutable nested selection borne by this Resource, when frozen.
    pub fn nested_selection(&self) -> Option<&FrozenSelection> {
        self.nested_selection.as_ref()
    }

    /// Immutable link reading borne by this Resource, when frozen.
    pub fn nested_link(&self) -> Option<&FrozenLink> {
        self.nested_link.as_ref()
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
