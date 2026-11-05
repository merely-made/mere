// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn valid_draft() -> ProjectionDraft {
    ProjectionDraft {
        dynamics: None,
        version: PROJECTION_DEFINITION_VERSION,
        id: "notes-by-topic".into(),
        label: "Notes by topic".into(),
        source: SourceBinding {
            authority: "local-device".into(),
            domain: "notes".into(),
            resource: "graph:notes".into(),
        },
        reading: Reading {
            kind: "nodes".into(),
            key: "topic".into(),
            value: None,
        },
        encoding: Encoding {
            x: Channel::Field("topic_x".into()),
            y: Channel::Field("topic_y".into()),
            color: Some(Channel::Field("kind".into())),
            label: Some(Channel::Field("title".into())),
        },
        arrangement: Arrangement::default(),
        interaction: Interaction::default(),
        appearance: Appearance {
            realization: "canvas".into(),
            title: "Notes by topic".into(),
            theme: "light".into(),
        },
        provenance: Provenance {
            author: "mark".into(),
            source_revision: Some("rev-7".into()),
            revision_evidence: RevisionEvidence::PublicGeneration,
            note: "fixture".into(),
        },
    }
}

fn sources() -> BTreeMap<String, ProjectionInputBinding> {
    BTreeMap::from([
        (
            "personal".to_owned(),
            ProjectionInputBinding {
                source: valid_draft().source,
                expects_generation: Some("generation:personal".into()),
                revision_evidence: RevisionEvidence::PublicGeneration,
            },
        ),
        (
            "shared".to_owned(),
            ProjectionInputBinding {
                source: SourceBinding {
                    authority: "community".into(),
                    domain: "notes".into(),
                    resource: "graph:shared-notes".into(),
                },
                expects_generation: Some("generation:shared".into()),
                revision_evidence: RevisionEvidence::PublicGeneration,
            },
        ),
    ])
}

fn authored() -> AuthoredProjectionDefinition {
    let draft = valid_draft();
    AuthoredProjectionDefinition {
        dynamics: None,
        version: draft.version,
        id: draft.id,
        label: draft.label,
        sources: sources(),
        reading: draft.reading,
        encoding: draft.encoding,
        arrangement: draft.arrangement,
        interaction: draft.interaction,
        appearance: draft.appearance,
        provenance: draft.provenance,
    }
}

#[test]
fn validation_reports_useful_fields() {
    let mut draft = valid_draft();
    draft.source.domain.clear();
    draft.arrangement.spacing = 0;
    draft
        .provenance
        .source_revision
        .as_mut()
        .expect("fixture has source revision")
        .clear();
    let issues = draft.validate().expect_err("invalid draft");
    assert!(issues.iter().any(|issue| issue.field == "source.domain"));
    assert!(
        issues
            .iter()
            .any(|issue| issue.field == "arrangement.spacing")
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.field == "provenance.source_revision")
    );
}

#[test]
fn dynamics_survives_authored_binding_and_variant_replacement() {
    let mut recipe = authored();
    let original =
        DynamicsSlot::from_json(1, r#"{"version":2,"root":{"future":"preserved"}}"#).unwrap();
    recipe.dynamics = Some(original.clone());
    let inherited = recipe.bind("personal", None).unwrap();
    assert_eq!(inherited.dynamics, Some(original.clone()));
    assert_eq!(
        serde_json::from_slice::<ProjectionDefinition>(&inherited.to_json_bytes().unwrap())
            .unwrap(),
        inherited
    );
    let mut variant = ProjectionVariant {
        definition_id: recipe.id.clone(),
        id: "another-law".into(),
        arrangement_options: BTreeMap::new(),
        dynamics: Some(DynamicsVariant::Preset { id: "law".into() }),
    };
    assert!(matches!(
        recipe.bind("personal", Some(&variant)),
        Err(AuthoredDefinitionError::Dynamics(_))
    ));
    let replacement = DynamicsSlot::from_json(1, r#"{"root":"replacement"}"#).unwrap();
    let bound = recipe
        .bind_with_dynamics("personal", Some(&variant), |id| {
            assert_eq!(id, "law");
            Ok(replacement.clone())
        })
        .unwrap();
    assert_eq!(bound.dynamics, Some(replacement.clone()));
    variant.dynamics = Some(DynamicsVariant::Spec {
        slot: replacement.clone(),
    });
    assert_eq!(
        recipe.bind("shared", Some(&variant)).unwrap().dynamics,
        Some(replacement)
    );
    assert_eq!(recipe.dynamics, Some(original));
}

#[test]
fn definition_json_is_repeatable_and_round_trips() {
    let definition = valid_draft().to_definition().expect("valid definition");
    let first = definition.to_json_bytes().expect("definition serializes");
    let second = definition.to_json_bytes().expect("definition serializes");
    assert_eq!(first, second);
    assert_eq!(
        first,
        br#"{"version":1,"id":"notes-by-topic","label":"Notes by topic","source":{"authority":"local-device","domain":"notes","resource":"graph:notes"},"reading":{"kind":"nodes","key":"topic","value":null},"encoding":{"x":{"kind":"field","value":"topic_x"},"y":{"kind":"field","value":"topic_y"},"color":{"kind":"field","value":"kind"},"label":{"kind":"field","value":"title"}},"arrangement":{"kind":"grid","direction":"horizontal","spacing":16},"interaction":{"selection":"single","pan":true,"zoom":true},"appearance":{"realization":"canvas","title":"Notes by topic","theme":"light"},"provenance":{"author":"mark","source_revision":"rev-7","note":"fixture"}}"#
    );
    assert_eq!(
        serde_json::from_slice::<ProjectionDefinition>(&first).expect("definition wire"),
        definition
    );
}

#[test]
fn authored_definition_binds_each_named_source_without_changing_recipe() {
    let authored = authored();
    authored.validate().expect("complete authored definition");
    let personal = authored.bind("personal", None).expect("personal bind");
    let shared = authored.bind("shared", None).expect("shared bind");

    assert_eq!(personal.id, authored.id);
    assert_eq!(shared.id, authored.id);
    assert_ne!(personal.source, shared.source);
    assert_eq!(personal.reading, shared.reading);
    assert_eq!(
        personal.provenance.source_revision,
        Some(PublicSourceRevision::from("generation:personal"))
    );
    assert_eq!(
        shared.provenance.source_revision,
        Some(PublicSourceRevision::from("generation:shared"))
    );

    let bytes = authored.to_json_bytes().expect("authoring bytes");
    assert_eq!(
        serde_json::to_vec(authored.sources.get("personal").expect("personal source"))
            .expect("public source binding bytes"),
        br#"{"source":{"authority":"local-device","domain":"notes","resource":"graph:notes"},"expects_generation":"generation:personal"}"#
    );
    assert!(
        !String::from_utf8(bytes.clone())
            .expect("authoring json")
            .contains("revision_evidence")
    );
    assert_eq!(
        serde_json::from_slice::<AuthoredProjectionDefinition>(&bytes).expect("authoring wire"),
        authored
    );
}

#[test]
fn runtime_witness_stays_out_of_the_durable_definition() {
    struct OpaqueWitness(&'static str);

    let mut authored = authored();
    let source = {
        let declared = authored
            .sources
            .get_mut("personal")
            .expect("personal source");
        declared.expects_generation = None;
        declared.revision_evidence = RevisionEvidence::RuntimeVerified;
        declared.source.clone()
    };
    let bound = authored
        .bind_runtime(
            "personal",
            RuntimeSourceBinding::new(source, OpaqueWitness("private-base-token")),
            None,
        )
        .expect("matching runtime source binds");

    assert_eq!(
        bound.with_witness(|witness| witness.0),
        "private-base-token"
    );
    assert_eq!(bound.definition().provenance.source_revision, None);
    assert_eq!(
        bound.definition().provenance.revision_evidence,
        RevisionEvidence::RuntimeVerified
    );
    let recorded_json = String::from_utf8(
        bound
            .into_recorded_definition()
            .to_json_bytes()
            .expect("definition bytes"),
    )
    .expect("definition json");
    assert!(recorded_json.contains("\"revision_evidence\":\"runtime_verified\""));
    assert!(!recorded_json.contains("source_revision"));
    assert!(!recorded_json.contains("private-base-token"));
}

#[test]
fn runtime_binding_requires_runtime_evidence() {
    let authored = authored();
    let declared = authored.sources.get("personal").expect("personal source");
    let result = authored.bind_runtime(
        "personal",
        RuntimeSourceBinding::new(declared.source.clone(), ()),
        None,
    );
    match result {
        Err(error) => assert_eq!(
            error,
            AuthoredDefinitionError::RuntimeEvidenceRequired("personal".to_owned())
        ),
        Ok(_) => panic!("public-generation binding cannot accept a runtime witness"),
    }
}

#[test]
fn ordinary_binding_cannot_bypass_a_runtime_witness() {
    let mut authored = authored();
    {
        let declared = authored
            .sources
            .get_mut("personal")
            .expect("personal source");
        declared.expects_generation = None;
        declared.revision_evidence = RevisionEvidence::RuntimeVerified;
    }

    assert_eq!(
        authored.bind("personal", None),
        Err(AuthoredDefinitionError::RuntimeBindingRequiresWitness(
            "personal".to_owned()
        ))
    );
}

#[test]
fn variant_changes_arrangement_options_without_creating_another_recipe() {
    let authored = authored();
    let variant = ProjectionVariant {
        dynamics: None,
        definition_id: authored.id.clone(),
        id: "compact".to_owned(),
        arrangement_options: BTreeMap::from([("guides".to_owned(), "false".to_owned())]),
    };
    let base = authored.bind("personal", None).expect("base bind");
    let compact = authored
        .bind("personal", Some(&variant))
        .expect("variant bind");

    assert_eq!(base.id, compact.id);
    assert_eq!(base.source, compact.source);
    assert_eq!(base.reading, compact.reading);
    assert_eq!(base.encoding, compact.encoding);
    assert_eq!(base.interaction, compact.interaction);
    assert_eq!(base.appearance, compact.appearance);
    assert_eq!(base.provenance, compact.provenance);
    assert_eq!(
        compact.arrangement.options.get("guides"),
        Some(&"false".to_owned())
    );
}

#[test]
fn variant_for_another_definition_is_refused() {
    let authored = authored();
    let variant = ProjectionVariant {
        dynamics: None,
        definition_id: "other.recipe".to_owned(),
        id: "compact".to_owned(),
        arrangement_options: BTreeMap::new(),
    };
    assert_eq!(
        authored.bind("personal", Some(&variant)),
        Err(AuthoredDefinitionError::VariantForAnotherDefinition(
            "other.recipe".to_owned()
        ))
    );
}
