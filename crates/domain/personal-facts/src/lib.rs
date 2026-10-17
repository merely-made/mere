// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MPL-2.0

//! Content-addressed personal resources admitted by the resident's mere.
//! This crate transforms candidates and validates complete graphs. Djinn owns
//! files, grants, sessions, the journal and archives. References grant no access.

use chartulary::FacetId;
use dramatis::{
    birth::BirthFacts,
    resources::{ResourceRef, SubjectResourceAssociation},
    subject::SubjectRef,
};
use gaz::{Contact, LocalId, PersonaScope};
use kernel::{
    geometry::PortablePoint,
    graph::{
        Graph, NodeKey,
        apply::{GraphDelta, add_node, apply_graph_delta},
    },
};
use serde::{Deserialize, Serialize};

pub const DOMAIN: &str = "personal-facts";
pub const BIRTH_FACET: &str = "dramatis.birth-facts/v1";
pub const ASSOCIATION_FACET: &str = "dramatis.subject-resource-association/v1";
pub const CONTACT_FACET: &str = "dramatis.contact-resource/v1";
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
pub fn create_contact(
    graph: &mut Graph,
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
    put(graph, &resource.id, label.trim(), CONTACT_FACET, &record)?;
    put_association(
        graph,
        &SubjectResourceAssociation::about(subject.clone(), resource),
    )?;
    validate(graph)?;
    Ok(subject)
}

/// A correction creates a new revision. Existing records are unchanged.
pub fn save_birth(graph: &mut Graph, facts: &BirthFacts) -> Result<BirthResource, String> {
    validate(graph)?;
    let value = birth_resource(facts)?;
    require_contact(graph, &facts.subject)?;
    put(
        graph,
        &value.resource.id,
        "Birth facts",
        BIRTH_FACET,
        &value.facts,
    )?;
    put_association(
        graph,
        &SubjectResourceAssociation::about(facts.subject.clone(), value.resource.clone()),
    )?;
    validate(graph)?;
    Ok(value)
}
pub fn save_association(
    graph: &mut Graph,
    value: &SubjectResourceAssociation,
) -> Result<(), String> {
    validate(graph)?;
    value.validate().map_err(err)?;
    validate_subject(&value.subject)?;
    check_binding(graph, value)?;
    put_association(graph, value)?;
    validate(graph)
}
pub fn births(graph: &Graph) -> Result<Vec<BirthResource>, String> {
    validate(graph)?;
    let mut result: Vec<_> = read_values::<BirthFacts>(graph, BIRTH_FACET)?
        .iter()
        .map(birth_resource)
        .collect::<Result<_, _>>()?;
    result.sort_by(|a, b| a.resource.id.cmp(&b.resource.id));
    Ok(result)
}
pub fn contacts(graph: &Graph) -> Result<Vec<ContactResource>, String> {
    validate(graph)?;
    read_values(graph, CONTACT_FACET)
}
pub fn associations(graph: &Graph) -> Result<Vec<SubjectResourceAssociation>, String> {
    validate(graph)?;
    read_values(graph, ASSOCIATION_FACET)
}

/// This additional owner-bound check prevents cross-persona contact insertion.
pub fn validate_for_persona(graph: &Graph, persona: &str) -> Result<(), String> {
    validate(graph)?;
    for record in read_values::<ContactResource>(graph, CONTACT_FACET)? {
        if let SubjectRef::Contact { scope, .. } = record.subject {
            if scope.as_str() != persona {
                return Err("contact resource belongs to a different reservoir persona".into());
            }
        }
    }
    Ok(())
}
pub fn validate(graph: &Graph) -> Result<(), String> {
    let mut required = Vec::new();
    let links = read_values::<SubjectResourceAssociation>(graph, ASSOCIATION_FACET)?;
    for (key, node) in graph.nodes() {
        let records = [BIRTH_FACET, CONTACT_FACET, ASSOCIATION_FACET]
            .iter()
            .filter(|name| facet(graph, key, name).is_some())
            .count();
        if !node.url().starts_with(PREFIX) {
            if records != 0 {
                return Err("personal record has a noncanonical address".into());
            }
            continue;
        }
        if records != 1 || node.id != Graph::node_namespace_id(node.url()) {
            return Err("personal record needs canonical identity and one supported facet".into());
        }
        if let Some(value) = facet(graph, key, BIRTH_FACET) {
            let facts: BirthFacts = serde_json::from_value(value.clone()).map_err(err)?;
            let resource = birth_resource(&facts)?;
            if node.url() != resource.resource.id {
                return Err("birth revision differs from its inputs".into());
            }
            require_contact(graph, &facts.subject)?;
            required.push((facts.subject, resource.resource));
        }
        if let Some(value) = facet(graph, key, CONTACT_FACET) {
            let record: ContactResource = serde_json::from_value(value.clone()).map_err(err)?;
            validate_subject(&record.subject)?;
            let SubjectRef::Contact { anchor, .. } = &record.subject else {
                return Err("contact record must name a scoped contact".into());
            };
            let revision = digest(&record)?;
            if record.contact.petname.trim().is_empty()
                || anchor != record.contact.anchor()
                || node.url() != format!("{CONTACT_PREFIX}{revision}")
            {
                return Err("contact identity/revision differs from its record".into());
            }
            required.push((
                record.subject,
                ResourceRef {
                    domain: DOMAIN.into(),
                    id: node.url().into(),
                    revision,
                },
            ));
        }
        if let Some(value) = facet(graph, key, ASSOCIATION_FACET) {
            let value: SubjectResourceAssociation =
                serde_json::from_value(value.clone()).map_err(err)?;
            value.validate().map_err(err)?;
            validate_subject(&value.subject)?;
            if node.url() != format!("{ASSOCIATION_PREFIX}{}", digest(&value)?) {
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
    for (id, facets) in graph.facets().iter() {
        for (name, _) in facets.iter() {
            if name.as_str().starts_with("dramatis.")
                && (![BIRTH_FACET, CONTACT_FACET, ASSOCIATION_FACET].contains(&name.as_str())
                    || !graph.nodes().any(|(_, n)| &n.id == id))
            {
                return Err("unsupported or orphan personal facet".into());
            }
        }
    }
    Ok(())
}
fn require_contact(graph: &Graph, subject: &SubjectRef) -> Result<(), String> {
    if matches!(subject, SubjectRef::Contact { .. })
        && !read_values::<ContactResource>(graph, CONTACT_FACET)?
            .iter()
            .any(|r| &r.subject == subject)
    {
        return Err("birth facts require an existing scoped contact resource".into());
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
fn check_binding(graph: &Graph, value: &SubjectResourceAssociation) -> Result<(), String> {
    if value.resource.domain != DOMAIN {
        return Ok(());
    }
    let (key, _) = graph
        .get_node_by_url(&value.resource.id)
        .ok_or("associated personal resource is absent")?;
    let (subject, revision) = if let Some(facts) = facet(graph, key, BIRTH_FACET) {
        let facts: BirthFacts = serde_json::from_value(facts.clone()).map_err(err)?;
        (facts.subject.clone(), digest(&facts)?)
    } else if let Some(contact) = facet(graph, key, CONTACT_FACET) {
        let contact: ContactResource = serde_json::from_value(contact.clone()).map_err(err)?;
        (contact.subject.clone(), digest(&contact)?)
    } else {
        return Err("unsupported personal association target".into());
    };
    if subject != value.subject || revision != value.resource.revision {
        return Err("association subject/revision differs from its target".into());
    }
    Ok(())
}
fn read_values<T: serde::de::DeserializeOwned>(
    graph: &Graph,
    name: &str,
) -> Result<Vec<T>, String> {
    graph
        .nodes()
        .filter_map(|(key, _)| facet(graph, key, name))
        .map(|v| serde_json::from_value(v.clone()).map_err(err))
        .collect()
}
fn put_association(graph: &mut Graph, value: &SubjectResourceAssociation) -> Result<(), String> {
    put(
        graph,
        &format!("{ASSOCIATION_PREFIX}{}", digest(value)?),
        "Subject resource",
        ASSOCIATION_FACET,
        value,
    )
}
fn facet<'a>(graph: &'a Graph, key: NodeKey, name: &str) -> Option<&'a serde_json::Value> {
    let node = graph.get_node(key)?;
    graph.facets().get(&node.id, &FacetId::new(name))
}
fn put<T: Serialize>(
    graph: &mut Graph,
    address: &str,
    title: &str,
    name: &str,
    value: &T,
) -> Result<(), String> {
    let key = graph
        .get_node_by_url(address)
        .map(|(key, _)| key)
        .unwrap_or_else(|| {
            add_node(
                graph,
                Some(Graph::node_namespace_id(address)),
                address.into(),
                PortablePoint::new(0., 0.),
            )
        });
    let value = serde_json::to_value(value).map_err(err)?;
    if let Some(old) = facet(graph, key, name) {
        return if old == &value {
            Ok(())
        } else {
            Err("immutable personal resource cannot be overwritten".into())
        };
    }
    apply_graph_delta(
        graph,
        GraphDelta::SetNodeTitle {
            key,
            title: title.into(),
        },
    );
    apply_graph_delta(
        graph,
        GraphDelta::SetNodeFacet {
            key,
            facet: name.into(),
            value,
        },
    );
    Ok(())
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[cfg(test)]
mod tests;
