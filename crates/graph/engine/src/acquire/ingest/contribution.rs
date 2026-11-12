// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Group a stream of RDF quads into a [`GraphContribution`] — shared by the
/// network-free and bundled-context parsers. `namespace` scopes blank-node
/// skolemization to the source document.
pub(super) fn collect_contribution<E: std::fmt::Display>(
    quads: impl Iterator<Item = Result<Quad, E>>,
    namespace: &str,
) -> Result<ImportEnvelope, IngestError> {
    let mut nodes: BTreeMap<String, NodeContribution> = BTreeMap::new();
    let mut edges: Vec<EdgeContribution> = Vec::new();

    // Pass A: materialize the stream and lift out RDF 1.2 reifiers. A subject
    // that `rdf:reifies` a triple term is statement METADATA, not a node: its
    // reified (s, p, o, g) names the fact, its other quads carry the fact's
    // label / provenance / assertion time, and a legacy or encoded Mere
    // IRI carries the fact handle itself. Everything else flows to pass B.
    let mut plain: Vec<Quad> = Vec::new();
    let mut reified: BTreeMap<String, ReifiedStatement> = BTreeMap::new();
    let mut reifier_meta: Vec<Quad> = Vec::new();
    let mut activity_meta: Vec<Quad> = Vec::new();
    let quads = quads
        .map(|quad| quad.map_err(|error| IngestError::Parse(error.to_string())))
        .collect::<Result<Vec<_>, _>>()?;
    // Reserved assertion identifiers have the same validation in both formats,
    // including descriptions that otherwise stay ordinary RDF.
    let mut classic_reifiers = std::collections::HashSet::new();
    for quad in &quads {
        if quad.predicate.as_str() == RDF_TYPE
            && matches!(&quad.object, Term::NamedNode(node)
                if node.as_str() == crate::formats::jsonld::RDF_STATEMENT)
        {
            let reifier = subject_iri(&quad.subject, namespace);
            statement_id_from_reifier(&reifier).map_err(|error| {
                IngestError::Parse(format!("invalid assertion ID reifier {reifier}: {error}"))
            })?;
            classic_reifiers.insert(reifier);
        }
    }
    let quads = crate::formats::jsonld::bridge_classic_reification(quads);
    let descriptions: std::collections::HashSet<_> = quads
        .iter()
        .filter(|quad| {
            quad.predicate.as_str() == RDF_TYPE
                && matches!(&quad.object, Term::NamedNode(node)
                    if node.as_str() == crate::formats::jsonld::RDF_STATEMENT)
        })
        .map(|quad| subject_iri(&quad.subject, namespace))
        .collect();
    classic_reifiers.retain(|reifier| !descriptions.contains(reifier));
    for quad in quads {
        if quad.predicate.as_str() == RDF_REIFIES
            && let Term::Triple(triple) = &quad.object
            && !descriptions.contains(&subject_iri(&quad.subject, namespace))
        {
            let reifier = subject_iri(&quad.subject, namespace);
            let statement_id = statement_id_from_reifier(&reifier).map_err(|error| {
                IngestError::Parse(format!("invalid assertion ID reifier {reifier}: {error}"))
            })?;
            let object = match &triple.object {
                Term::NamedNode(node) => ReifiedObject::Resource(node.as_str().to_string()),
                Term::BlankNode(node) => {
                    ReifiedObject::Resource(skolemize(namespace, node.as_str()))
                },
                Term::Literal(literal) => ReifiedObject::Literal {
                    value: literal.value().to_string(),
                    datatype: literal.datatype().as_str().to_string(),
                    lang: literal.language().map(str::to_string),
                },
                Term::Triple(_) => continue, // nested triple terms: out of profile
            };
            let candidate = ReifiedStatement {
                statement_id,
                subject: subject_iri(&triple.subject.clone(), namespace),
                predicate: normalize_schema_org(triple.predicate.as_str()),
                object,
                graph_scope: scope_from_graph_name(&quad.graph_name, namespace),
                label: None,
                provenance_iri: None,
                first_asserted_ms: None,
                last_asserted_ms: None,
                lifecycle_version: false,
                method_iri: None,
                method_version: None,
                evidence: Vec::new(),
                withdrawal_activity: None,
                withdrawal_actor: None,
                withdrawal_at_ms: None,
                asserted_graph_kind: None,
                asserted_graph_name: None,
            };
            if let Some(existing) = reified.get(&reifier) {
                if existing.statement_id != candidate.statement_id
                    || existing.subject != candidate.subject
                    || existing.predicate != candidate.predicate
                    || existing.object != candidate.object
                    || existing.graph_scope != candidate.graph_scope
                {
                    return Err(IngestError::Parse(format!(
                        "reifier {reifier} identifies conflicting RDF statements"
                    )));
                }
            } else {
                reified.insert(reifier, candidate);
            }
            continue;
        }
        plain.push(quad);
    }
    if !reified.is_empty() {
        // Split off the reifier-subject metadata quads; the rest stay plain.
        let mut rest = Vec::with_capacity(plain.len());
        for quad in plain {
            let reifier = subject_iri(&quad.subject, namespace);
            if reified.get(&reifier).is_some_and(|statement| {
                (!classic_reifiers.contains(&reifier)
                    || statement.graph_scope == scope_from_graph_name(&quad.graph_name, namespace))
                    && (crate::formats::jsonld::representable_metadata(
                        quad.predicate.as_str(),
                        &quad.object,
                    ) || (!classic_reifiers.contains(&reifier)
                        && quad.predicate.as_str() == RDFS_LABEL
                        && matches!(&quad.object, Term::Literal(_)))
                        || (!classic_reifiers.contains(&reifier)
                            && quad.predicate.as_str() == PROV_GENERATED_AT_TIME
                            && matches!(&quad.object, Term::Literal(literal)
                                if parse_xsd_datetime_ms(literal.value()).is_some())))
            }) {
                reifier_meta.push(quad);
            } else if is_withdrawal_activity_metadata(&quad, namespace) {
                activity_meta.push(quad);
            } else {
                rest.push(quad);
            }
        }
        plain = rest;
        let mut activities = BTreeMap::<String, (bool, Option<String>)>::new();
        for quad in &activity_meta {
            let activity = subject_iri(&quad.subject, namespace);
            let entry = activities.entry(activity).or_default();
            match (quad.predicate.as_str(), &quad.object) {
                (RDF_TYPE, Term::NamedNode(kind)) if kind.as_str() == PROV_ACTIVITY => {
                    entry.0 = true;
                },
                (PROV_WAS_ASSOCIATED_WITH, Term::NamedNode(actor)) => {
                    if entry
                        .1
                        .as_ref()
                        .is_some_and(|existing| existing != actor.as_str())
                    {
                        return Err(IngestError::Parse(
                            "withdrawal activity has multiple associated actors".into(),
                        ));
                    }
                    entry.1 = Some(actor.as_str().to_string());
                },
                _ => {},
            }
        }
        for quad in &reifier_meta {
            let reifier = subject_iri(&quad.subject, namespace);
            let Some(statement) = reified.get_mut(&reifier) else {
                continue;
            };
            match (quad.predicate.as_str(), &quad.object) {
                (RDFS_LABEL, Term::Literal(literal)) => {
                    statement.label = Some(literal.value().to_string());
                },
                (PROV_WAS_ATTRIBUTED_TO, Term::NamedNode(agent)) => {
                    set_once_or_equal(
                        &mut statement.provenance_iri,
                        agent.as_str().to_string(),
                        "asserted provenance attribution",
                    )?;
                },
                (PROV_GENERATED_AT_TIME, Term::Literal(literal)) => {
                    set_once_or_equal(
                        &mut statement.first_asserted_ms,
                        parse_metadata_time(literal.value())?,
                        "first assertion time",
                    )?;
                },
                (MERE_LIFECYCLE_VERSION, Term::Literal(literal)) => {
                    if literal.value() != "1" {
                        return Err(IngestError::Parse(format!(
                            "unsupported assertion lifecycle version {}",
                            literal.value()
                        )));
                    }
                    statement.lifecycle_version = true;
                },
                (MERE_LAST_ASSERTED_AT, Term::Literal(literal)) => {
                    set_once_or_equal(
                        &mut statement.last_asserted_ms,
                        parse_metadata_time(literal.value())?,
                        "last assertion time",
                    )?;
                },
                (PROV_WAS_GENERATED_BY, Term::NamedNode(method)) => {
                    set_once_or_equal(
                        &mut statement.method_iri,
                        method.as_str().to_string(),
                        "method IRI",
                    )?;
                },
                (MERE_METHOD_VERSION, Term::Literal(version)) => {
                    set_once_or_equal(
                        &mut statement.method_version,
                        version.value().to_string(),
                        "method version",
                    )?;
                },
                (PROV_WAS_DERIVED_FROM, Term::NamedNode(evidence)) => {
                    statement.evidence.push(evidence.as_str().to_string());
                },
                (PROV_WAS_INVALIDATED_BY, Term::NamedNode(activity)) => {
                    set_once_or_equal(
                        &mut statement.withdrawal_activity,
                        activity.as_str().to_string(),
                        "withdrawal activity",
                    )?;
                },
                (PROV_INVALIDATED_AT_TIME, Term::Literal(literal)) => {
                    set_once_or_equal(
                        &mut statement.withdrawal_at_ms,
                        parse_metadata_time(literal.value())?,
                        "withdrawal time",
                    )?;
                },
                (MERE_ASSERTED_GRAPH_KIND, Term::Literal(kind)) => {
                    set_once_or_equal(
                        &mut statement.asserted_graph_kind,
                        kind.value().to_string(),
                        "original assertion graph kind",
                    )?;
                },
                (MERE_ASSERTED_GRAPH_NAME, Term::Literal(name)) => {
                    set_once_or_equal(
                        &mut statement.asserted_graph_name,
                        name.value().to_string(),
                        "original assertion graph name",
                    )?;
                },
                _ => {},
            }
        }
        for statement in reified.values_mut() {
            if statement.asserted_graph_kind.is_some() || statement.asserted_graph_name.is_some() {
                if statement.graph_scope
                    != GraphScope::Custom(crate::ASSERTION_HISTORY_GRAPH_IRI.into())
                {
                    return Err(IngestError::Parse(
                        "original assertion scope markers are valid only in the history graph"
                            .into(),
                    ));
                }
                statement.graph_scope = restore_graph_scope(
                    statement.asserted_graph_kind.as_deref(),
                    statement.asserted_graph_name.as_deref(),
                )?;
            }
            if let Some(activity) = &statement.withdrawal_activity {
                let Some((is_activity, actor)) = activities.get(activity) else {
                    return Err(IngestError::Parse(
                        "withdrawal reference does not identify an activity".into(),
                    ));
                };
                if !is_activity {
                    return Err(IngestError::Parse(
                        "withdrawal reference target is not a prov:Activity".into(),
                    ));
                }
                if let Some(id) = &statement.statement_id
                    && parse_withdrawal_activity_id(activity).as_deref() != Some(id.as_str())
                {
                    return Err(IngestError::Parse(
                        "withdrawal activity ID does not match its assertion".into(),
                    ));
                }
                // The activity association is retained separately until edge construction.
                statement.withdrawal_actor = actor.clone();
            }
            if statement.method_version.is_some() && statement.method_iri.is_none() {
                return Err(IngestError::Parse(
                    "method version metadata requires a method IRI".into(),
                ));
            }
            if !statement.lifecycle_version && statement.last_asserted_ms.is_none() {
                statement.last_asserted_ms = statement.first_asserted_ms;
            }
        }
    }

    let mut evidence = envelope::ImportEvidence::from_quads(&plain, namespace);
    for quad in plain {
        let subject = subject_iri(&quad.subject, namespace);
        let predicate_norm = normalize_schema_org(quad.predicate.as_str());
        let predicate = predicate_norm.as_str();
        let graph_scope = scope_from_graph_name(&quad.graph_name, namespace);
        nodes
            .entry(subject.clone())
            .or_insert_with(|| NodeContribution::new(&subject));

        match &quad.object {
            Term::Literal(literal) => {
                let node = nodes.get_mut(&subject).expect("subject just inserted");
                let value = literal.value().to_string();
                if graph_scope == GraphScope::Default && is_title_predicate(predicate) {
                    node.title = Some(value);
                } else if graph_scope == GraphScope::Default && is_tags_predicate(predicate) {
                    node.tags.push(value);
                } else {
                    // Any other literal goes into the open property bag.
                    let lang = literal.language().map(str::to_string);
                    let datatype = if lang.is_some() {
                        None
                    } else {
                        let datatype = literal.datatype().as_str();
                        (datatype != XSD_STRING).then(|| normalize_schema_org(datatype))
                    };
                    let mut property = NodeProperty::new(predicate.to_string(), value)
                        .with_graph_scope(graph_scope.clone());
                    property.datatype = datatype;
                    property.lang = lang;
                    evidence.plain_property(&subject, &property);
                    node.properties.push(property);
                }
            },
            Term::NamedNode(object) => route_resource(
                &mut nodes,
                &mut edges,
                &subject,
                predicate,
                object.as_str().to_string(),
                graph_scope,
            ),
            Term::BlankNode(object) => route_resource(
                &mut nodes,
                &mut edges,
                &subject,
                predicate,
                skolemize(namespace, object.as_str()),
                graph_scope,
            ),
            Term::Triple(_) => {},
        }
    }
    for (index, edge) in edges.iter().enumerate() {
        evidence.plain_edge(index, edge);
    }

    // Attach the lifted reifier metadata to the matching contributions:
    // resource-object statements to edges (by subject + predicate + object +
    // scope), literal-object statements to node properties (by predicate +
    // value + typing + scope). A Mere reifier also hands over the exact fact
    // id, keeping the round trip id-stable; a foreign reifier
    // leaves the id to the kernel's minter.
    //
    // Both matches are indexed up front rather than linear-scanned per reifier,
    // which is what turns this pass from O(R x E) into O(R + E). Each key maps
    // to the FIRST position that carries it, preserving `find`'s semantics when
    // a document repeats a triple.
    type EdgeMatchKey = (String, String, String, GraphScope);
    type PropertyMatchKey = (
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        GraphScope,
    );
    let mut edge_index: std::collections::HashMap<EdgeMatchKey, usize> =
        std::collections::HashMap::new();
    let mut property_index: std::collections::HashMap<PropertyMatchKey, usize> =
        std::collections::HashMap::new();
    if !reified.is_empty() {
        for (position, edge) in edges.iter().enumerate() {
            edge_index
                .entry((
                    edge.subject.clone(),
                    edge.predicate.clone(),
                    edge.object.clone(),
                    edge.graph_scope.clone(),
                ))
                .or_insert(position);
        }
        for (subject, node) in &nodes {
            for (position, property) in node.properties.iter().enumerate() {
                property_index
                    .entry((
                        subject.clone(),
                        property.predicate.clone(),
                        property.value.clone(),
                        property.datatype.clone(),
                        property.lang.clone(),
                        property.graph_scope.clone(),
                    ))
                    .or_insert(position);
            }
        }
    }
    let asserted_literals: std::collections::HashSet<_> = property_index.keys().cloned().collect();
    for statement in reified.into_values() {
        let basis = AssertionBasis {
            note: statement.label.clone(),
            method: statement
                .method_iri
                .clone()
                .map(|iri| crate::model::AssertionMethod {
                    iri,
                    version: statement.method_version.clone(),
                }),
            evidence: statement.evidence.clone(),
        };
        let withdrawn = (statement.withdrawal_activity.is_some()
            || statement.withdrawal_at_ms.is_some())
        .then(|| AssertionWithdrawal {
            at_ms: statement.withdrawal_at_ms,
            by: statement.withdrawal_actor.clone(),
        });
        let statement_id = statement.statement_id;
        match &statement.object {
            ReifiedObject::Resource(object) => {
                let match_key = (
                    statement.subject.clone(),
                    statement.predicate.clone(),
                    object.clone(),
                    statement.graph_scope.clone(),
                );
                // A base triple describes the shared claim, not another
                // assertion. Consume its slot once; each further reifier
                // creates its own contribution instead of overwriting it.
                if let Some(position) = edge_index.remove(&match_key) {
                    evidence.reified_edge(position);
                    let edge = &mut edges[position];
                    edge.statement_id = statement_id;
                    edge.label = statement.label;
                    edge.provenance_iri = statement.provenance_iri;
                    edge.first_asserted_ms = statement.first_asserted_ms;
                    edge.asserted_at_ms = statement.last_asserted_ms;
                    edge.withdrawn = withdrawn;
                    edge.basis = basis;
                } else {
                    // A reifier names a fact even when its base triple is not
                    // asserted; history rows use this form by design.
                    nodes
                        .entry(statement.subject.clone())
                        .or_insert_with(|| NodeContribution::new(&statement.subject));
                    edges.push(EdgeContribution {
                        subject: statement.subject,
                        predicate: statement.predicate,
                        object: object.clone(),
                        graph_scope: statement.graph_scope,
                        statement_id,
                        label: statement.label,
                        provenance_iri: statement.provenance_iri,
                        first_asserted_ms: statement.first_asserted_ms,
                        asserted_at_ms: statement.last_asserted_ms,
                        withdrawn,
                        basis,
                    });
                }
            },
            ReifiedObject::Literal {
                value,
                datatype,
                lang,
            } => {
                let wanted_datatype = if lang.is_some() {
                    None
                } else {
                    (datatype != XSD_STRING).then(|| normalize_schema_org(datatype))
                };
                let match_key = (
                    statement.subject.clone(),
                    statement.predicate.clone(),
                    value.clone(),
                    wanted_datatype,
                    lang.clone(),
                    statement.graph_scope.clone(),
                );
                if !asserted_literals.contains(&match_key) {
                    continue;
                }
                let Some(node) = nodes.get_mut(&statement.subject) else {
                    continue;
                };
                let position = match property_index.remove(&match_key) {
                    Some(position) => position,
                    None => {
                        // The shared base literal is consumed once, but each
                        // attributable reifier keeps its own assertion record.
                        let mut property =
                            NodeProperty::new(statement.predicate.clone(), value.clone())
                                .with_graph_scope(statement.graph_scope.clone());
                        property.datatype = match_key.3.clone();
                        property.lang = lang.clone();
                        node.properties.push(property);
                        node.properties.len() - 1
                    },
                };
                let property = &mut node.properties[position];
                evidence.reified_property(&statement.subject, &property.statement_id);
                if let Some(id) = &statement_id {
                    property.statement_id = id.clone();
                }
                property.provenance_iri = statement.provenance_iri;
                property.asserted_at_ms = statement.last_asserted_ms;
                if statement_id.is_some() {
                    evidence.carried_property(&statement.subject, property);
                }
            },
        }
    }

    // Make the contribution self-contained: every edge endpoint is a node.
    for edge in &edges {
        nodes
            .entry(edge.object.clone())
            .or_insert_with(|| NodeContribution::new(&edge.object));
    }

    let nodes = nodes
        .into_values()
        .map(|mut node| {
            node.types.sort();
            node.types.dedup();
            node.tags.sort();
            node.tags.dedup();
            node.properties.sort_by(|a, b| {
                (
                    a.predicate.as_str(),
                    a.value.as_str(),
                    a.datatype.as_deref(),
                    a.lang.as_deref(),
                    &a.graph_scope,
                    &a.provenance_iri,
                    &a.statement_id,
                )
                    .cmp(&(
                        b.predicate.as_str(),
                        b.value.as_str(),
                        b.datatype.as_deref(),
                        b.lang.as_deref(),
                        &b.graph_scope,
                        &b.provenance_iri,
                        &b.statement_id,
                    ))
            });
            node.properties.dedup_by(|a, b| {
                a.content_eq(b)
                    && (a.statement_id == b.statement_id
                        || (!evidence.is_carried_property(&node.id, a)
                            && !evidence.is_carried_property(&node.id, b)))
            });
            node
        })
        .collect();

    Ok(ImportEnvelope {
        contribution: GraphContribution { nodes, edges },
        evidence,
    })
}
