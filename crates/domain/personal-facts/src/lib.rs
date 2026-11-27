// Copyright 2026 Mark AB (markik)
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Content-addressed personal resources admitted by the resident's mere.
//! This crate transforms candidates and validates complete graphs. Djinn owns
//! files, grants, sessions, the journal and archives. References grant no access.
//!
//! Each record is one facet of the resource at its content-addressed IRI
//! (engine split D216, D247), written through the recorded resource-record
//! write. Nothing here places a surface node: a host that shows a record binds
//! one itself. Records held as surface-node facets are refused (D246). Every
//! function takes any [`SessionGraph`] (engine split D261).

use dramatis::{
    birth::BirthFacts,
    resources::{ResourceRef, SubjectResourceAssociation},
    subject::SubjectRef,
};
use gaz::{Contact, LocalId, PersonaScope};
use mere::model::{Author, PersistedResourceFacet, PersistedResourceRecord, ResourceNode};
use mere::session::SessionGraph;
use portolan::FacetId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const DOMAIN: &str = "personal-facts";
pub const BIRTH_FACET: &str = "dramatis.birth-facts/v1";
pub const ASSOCIATION_FACET: &str = "dramatis.subject-resource-association/v1";
pub const CONTACT_FACET: &str = "dramatis.contact-resource/v1";
const RECORD_FACETS: [&str; 3] = [BIRTH_FACET, CONTACT_FACET, ASSOCIATION_FACET];
const PREFIX: &str = "mere://personal-facts/";
const BIRTH_PREFIX: &str = "mere://personal-facts/birth/";
const CONTACT_PREFIX: &str = "mere://personal-facts/contact/";
const ASSOCIATION_PREFIX: &str = "mere://personal-facts/association/";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthResource {
    pub resource: ResourceRef,
    pub facts: BirthFacts,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactResource {
    pub subject: SubjectRef,
    pub contact: Contact,
}

pub fn digest<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(blake3::hash(&serde_json::to_vec(value).map_err(err)?)
        .to_hex()
        .to_string())
}
pub fn birth_resource(facts: &BirthFacts) -> Result<BirthResource, String> {
    facts.validate().map_err(err)?;
    validate_subject(&facts.subject)?;
    let revision = digest(facts)?;
    Ok(BirthResource {
        resource: ResourceRef {
            domain: DOMAIN.into(),
            id: format!("{BIRTH_PREFIX}{revision}"),
            revision,
        },
        facts: facts.clone(),
    })
}

/// Explicitly create a local Gaz contact in this persona's shared data domain.
/// No key/login is minted and no other contact is merged or copied by name.
pub fn create_contact<G: SessionGraph>(
    graph: &mut G,
    scope: &str,
    id: LocalId,
    label: &str,
) -> Result<SubjectRef, String> {
    validate(graph)?;
    if scope.trim().is_empty() || label.trim().is_empty() {
        return Err("contact scope and name are required".into());
    }
    let contact = Contact::new_local(label.trim(), id);
    let subject = SubjectRef::Contact {
        scope: PersonaScope::new(scope),
        anchor: contact.anchor().clone(),
    };
    let record = ContactResource {
        subject: subject.clone(),
        contact,
    };
    let revision = digest(&record)?;
    let resource = ResourceRef {
        domain: DOMAIN.into(),
        id: format!("{CONTACT_PREFIX}{revision}"),
        revision,
    };
    put(graph, &resource.id, CONTACT_FACET, &record)?;
    put_association(
        graph,
        &SubjectResourceAssociation::about(subject.clone(), resource),
    )?;
    validate(graph)?;
    Ok(subject)
}

/// A correction creates a new revision. Existing records are unchanged.
pub fn save_birth<G: SessionGraph>(
    graph: &mut G,
    facts: &BirthFacts,
) -> Result<BirthResource, String> {
    validate(graph)?;
    let value = birth_resource(facts)?;
    require_contact(graph, &facts.subject)?;
    put(graph, &value.resource.id, BIRTH_FACET, &value.facts)?;
    put_association(
        graph,
        &SubjectResourceAssociation::about(facts.subject.clone(), value.resource.clone()),
    )?;
    validate(graph)?;
    Ok(value)
}
pub fn save_association<G: SessionGraph>(
    graph: &mut G,
    value: &SubjectResourceAssociation,
) -> Result<(), String> {
    validate(graph)?;
    value.validate().map_err(err)?;
    validate_subject(&value.subject)?;
    check_binding(graph, value)?;
    put_association(graph, value)?;
    validate(graph)
}
pub fn births<G: SessionGraph>(graph: &G) -> Result<Vec<BirthResource>, String> {
    validate(graph)?;
    let mut result: Vec<_> = read_values::<BirthFacts, _>(graph, BIRTH_FACET)?
        .iter()
        .map(birth_resource)
        .collect::<Result<_, _>>()?;
    result.sort_by(|a, b| a.resource.id.cmp(&b.resource.id));
    Ok(result)
}
pub fn contacts<G: SessionGraph>(graph: &G) -> Result<Vec<ContactResource>, String> {
    validate(graph)?;
    read_values(graph, CONTACT_FACET)
}
pub fn associations<G: SessionGraph>(graph: &G) -> Result<Vec<SubjectResourceAssociation>, String> {
    validate(graph)?;
    read_values(graph, ASSOCIATION_FACET)
}

/// This additional owner-bound check prevents cross-persona contact insertion.
/// Structural validation requires every contact association to resolve to one
/// of these retained scoped contacts, including links to external domains.
pub fn validate_for_persona<G: SessionGraph>(graph: &G, persona: &str) -> Result<(), String> {
    validate(graph)?;
    for record in read_values::<ContactResource, _>(graph, CONTACT_FACET)? {
        if let SubjectRef::Contact { scope, .. } = record.subject {
            if scope.as_str() != persona {
                return Err("contact resource belongs to a different reservoir persona".into());
            }
        }
    }
    Ok(())
}

/// Check first introduction of profile subjects at a resident admission boundary.
/// Both complete graph shapes are checked before consulting the caller's lookup.
/// Only profile IDs absent from `before` and present in `after` need a positive
/// lookup, once per ID across birth and association facets. Existing references
/// may survive profile removal or lookup unavailability, including new revisions
/// concerning those same historical subjects. This neither mutates graphs nor
/// performs credential operations; lookup authority belongs to the resident.
/// Pure cold replay must use structural validation, not introduce an empty
/// `before` graph here as a substitute for its already admitted history.
pub fn validate_new_profile_subjects<G: SessionGraph>(
    before: &G,
    after: &G,
    lookup: impl Fn(&personae::ProfileId) -> Result<bool, String>,
) -> Result<(), String> {
    validate(before)?;
    validate(after)?;
    let existing = profile_subjects(before)?;
    let proposed = profile_subjects(after)?;
    for id in proposed.difference(&existing) {
        if !lookup(id)? {
            return Err(format!("new profile subject is unavailable: {}", id.0));
        }
    }
    Ok(())
}

fn profile_subjects<G: SessionGraph>(graph: &G) -> Result<BTreeSet<personae::ProfileId>, String> {
    let mut subjects = BTreeSet::new();
    for subject in read_values::<BirthFacts, _>(graph, BIRTH_FACET)?
        .into_iter()
        .map(|facts| facts.subject)
        .chain(
            read_values::<SubjectResourceAssociation, _>(graph, ASSOCIATION_FACET)?
                .into_iter()
                .map(|association| association.subject),
        )
    {
        if let SubjectRef::Profile { id } = subject {
            subjects.insert(id);
        }
    }
    Ok(subjects)
}

pub fn validate<G: SessionGraph>(graph: &G) -> Result<(), String> {
    // The node-facet form is refused outright, with no legacy reader (D246).
    for (_, facets) in graph.facets().iter() {
        if facets.iter().any(|(name, _)| is_personal(name.as_str())) {
            return Err("personal records are resource facets, not node facets".into());
        }
    }
    let mut required = Vec::new();
    let links = read_values::<SubjectResourceAssociation, _>(graph, ASSOCIATION_FACET)?;
    for resource in graph.resources().resource_nodes() {
        let iri = resource.canonical_iri();
        let names: Vec<&str> = graph
            .resources()
            .facets()
            .facets_of(&resource.id())
            .map(|facets| facets.iter().map(|(name, _)| name.as_str()).collect())
            .unwrap_or_default();
        if names
            .iter()
            .any(|name| is_personal(name) && !RECORD_FACETS.contains(name))
        {
            return Err("unsupported personal facet".into());
        }
        let records = names
            .iter()
            .filter(|name| RECORD_FACETS.contains(name))
            .count();
        if !iri.starts_with(PREFIX) {
            if records != 0 {
                return Err("personal record has a noncanonical address".into());
            }
            continue;
        }
        if records != 1 {
            return Err("personal record needs one supported facet".into());
        }
        if let Some(value) = facet(graph, iri, BIRTH_FACET) {
            let facts: BirthFacts = serde_json::from_value(value.clone()).map_err(err)?;
            let birth = birth_resource(&facts)?;
            if iri != birth.resource.id {
                return Err("birth revision differs from its inputs".into());
            }
            require_contact(graph, &facts.subject)?;
            required.push((facts.subject, birth.resource));
        }
        if let Some(value) = facet(graph, iri, CONTACT_FACET) {
            let record: ContactResource = serde_json::from_value(value.clone()).map_err(err)?;
            validate_subject(&record.subject)?;
            let SubjectRef::Contact { anchor, .. } = &record.subject else {
                return Err("contact record must name a scoped contact".into());
            };
            let revision = digest(&record)?;
            if record.contact.petname.trim().is_empty()
                || anchor != record.contact.anchor()
                || iri != format!("{CONTACT_PREFIX}{revision}")
            {
                return Err("contact identity/revision differs from its record".into());
            }
            required.push((
                record.subject,
                ResourceRef {
                    domain: DOMAIN.into(),
                    id: iri.into(),
                    revision,
                },
            ));
        }
        if let Some(value) = facet(graph, iri, ASSOCIATION_FACET) {
            let value: SubjectResourceAssociation =
                serde_json::from_value(value.clone()).map_err(err)?;
            value.validate().map_err(err)?;
            validate_subject(&value.subject)?;
            if iri != format!("{ASSOCIATION_PREFIX}{}", digest(&value)?) {
                return Err("association revision differs from its inputs".into());
            }
            check_binding(graph, &value)?;
        }
    }
    for (subject, resource) in required {
        if !links
            .iter()
            .any(|v| v.subject == subject && v.resource == resource)
        {
            return Err("personal resource requires its matching subject association".into());
        }
    }
    Ok(())
}
fn is_personal(facet: &str) -> bool {
    facet.starts_with("dramatis.")
}
fn require_contact<G: SessionGraph>(graph: &G, subject: &SubjectRef) -> Result<(), String> {
    if matches!(subject, SubjectRef::Contact { .. })
        && !read_values::<ContactResource, _>(graph, CONTACT_FACET)?
            .iter()
            .any(|r| &r.subject == subject)
    {
        return Err("personal records require an existing scoped contact resource".into());
    }
    Ok(())
}
fn validate_subject(subject: &SubjectRef) -> Result<(), String> {
    match subject {
        SubjectRef::Profile { id } if id.0.trim().is_empty() => {
            Err("profile reference is empty".into())
        },
        SubjectRef::Contact { scope, .. } if scope.as_str().trim().is_empty() => {
            Err("contact scope is empty".into())
        },
        _ => Ok(()),
    }
}
fn check_binding<G: SessionGraph>(
    graph: &G,
    value: &SubjectResourceAssociation,
) -> Result<(), String> {
    // Subject validity is independent of the target's owner. External links
    // skip local target binding, never retained-contact existence/scope checks.
    require_contact(graph, &value.subject)?;
    if value.resource.domain != DOMAIN {
        return Ok(());
    }
    let id = value.resource.id.as_str();
    let (subject, revision) = if let Some(facts) = facet(graph, id, BIRTH_FACET) {
        let facts: BirthFacts = serde_json::from_value(facts.clone()).map_err(err)?;
        (facts.subject.clone(), digest(&facts)?)
    } else if let Some(contact) = facet(graph, id, CONTACT_FACET) {
        let contact: ContactResource = serde_json::from_value(contact.clone()).map_err(err)?;
        (contact.subject.clone(), digest(&contact)?)
    } else if graph
        .resources()
        .resource(ResourceNode::for_term(id).id())
        .is_some()
    {
        return Err("unsupported personal association target".into());
    } else {
        return Err("associated personal resource is absent".into());
    };
    if subject != value.subject || revision != value.resource.revision {
        return Err("association subject/revision differs from its target".into());
    }
    Ok(())
}
fn read_values<T: serde::de::DeserializeOwned, G: SessionGraph>(
    graph: &G,
    name: &str,
) -> Result<Vec<T>, String> {
    graph
        .resources()
        .resource_nodes()
        .filter_map(|resource| facet(graph, resource.canonical_iri(), name))
        .map(|v| serde_json::from_value(v.clone()).map_err(err))
        .collect()
}
fn put_association<G: SessionGraph>(
    graph: &mut G,
    value: &SubjectResourceAssociation,
) -> Result<(), String> {
    put(
        graph,
        &format!("{ASSOCIATION_PREFIX}{}", digest(value)?),
        ASSOCIATION_FACET,
        value,
    )
}
/// The `name` facet of the resource at `iri`.
fn facet<'a, G: SessionGraph>(
    graph: &'a G,
    iri: &str,
    name: &str,
) -> Option<&'a serde_json::Value> {
    graph
        .resources()
        .facets()
        .get(&ResourceNode::for_term(iri).id(), &FacetId::new(name))
}
/// Writes `value` as the `name` facet of the resource at `address`, through
/// the recorded resource-record write (the `page_claims` path).
fn put<G: SessionGraph, T: Serialize>(
    graph: &mut G,
    address: &str,
    name: &str,
    value: &T,
) -> Result<(), String> {
    let value = serde_json::to_value(value).map_err(err)?;
    if let Some(old) = facet(graph, address, name) {
        return if old == &value {
            Ok(())
        } else {
            Err("immutable personal resource cannot be overwritten".into())
        };
    }
    let resource = ResourceNode::for_term(address);
    let mut record = graph
        .resources()
        .resource_record(resource.id())
        .unwrap_or_else(|| PersistedResourceRecord {
            canonical_iri: resource.canonical_iri().to_owned(),
            facets: Vec::new(),
        });
    record.facets.push(PersistedResourceFacet {
        facet: name.to_owned(),
        value_json: value.to_string(),
    });
    // A record write carries no attribution of its own: the session that
    // journals it attributes the change.
    let _ = graph.write_resource_record(&Author::user(), resource.id(), Some(record));
    if facet(graph, address, name) != Some(&value) {
        return Err("personal resource could not be written".into());
    }
    Ok(())
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[cfg(test)]
mod tests;
