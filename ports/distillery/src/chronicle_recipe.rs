// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Chronicle's authored projection recipe: the one definition its Distillery
//! and Djinn inputs share. The caller supplies source locators and
//! authority-emitted generations; this authoring seam never acquires either
//! authority. It lived in Graphshell's projection editor until the editor
//! became a stack crate, because the recipe is Distillery's.

use std::collections::BTreeMap;

use scenograph::{
    Appearance, Arrangement, AuthoredProjectionDefinition, Channel, Encoding, Interaction,
    PROJECTION_DEFINITION_VERSION, ProjectionInputBinding, ProjectionVariant, Provenance,
    PublicSourceRevision, Reading, RevisionEvidence,
};

/// Chronicle's stable authored definition id.
pub const CHRONICLE_DEFINITION_ID: &str = "distillery.chronicle";
/// The arrangement-owned guide toggle used by Chronicle.
pub const CHRONICLE_ERA_BANDS_OPTION: &str = "era_bands";

/// Build the one Chronicle recipe used for its named Distillery and Djinn
/// inputs. The caller supplies source locators and authority-emitted
/// generations; this authoring seam never acquires either authority.
pub fn chronicle_definition(
    sources: BTreeMap<String, ProjectionInputBinding>,
    author: impl Into<String>,
    definition_revision: impl Into<String>,
) -> AuthoredProjectionDefinition {
    AuthoredProjectionDefinition {
        dynamics: None,
        version: PROJECTION_DEFINITION_VERSION,
        id: CHRONICLE_DEFINITION_ID.to_owned(),
        label: "Job Chronicle".to_owned(),
        sources,
        reading: Reading {
            kind: "mesh.jobs".to_owned(),
            key: "job_id".to_owned(),
            value: Some("state".to_owned()),
        },
        encoding: Encoding {
            x: Channel::Field("observation_tick".to_owned()),
            y: Channel::Field("lease_epoch".to_owned()),
            color: Some(Channel::Field("state".to_owned())),
            label: Some(Channel::Field("job_id".to_owned())),
        },
        arrangement: Arrangement {
            kind: "timeline.default".to_owned(),
            direction: "observation_tick".to_owned(),
            spacing: 180,
            options: BTreeMap::from([(CHRONICLE_ERA_BANDS_OPTION.to_owned(), "true".to_owned())]),
        },
        interaction: Interaction::default(),
        appearance: Appearance {
            realization: "portable-card".to_owned(),
            title: "Job Chronicle".to_owned(),
            theme: "chronicle".to_owned(),
        },
        provenance: Provenance {
            author: author.into(),
            source_revision: Some(PublicSourceRevision::from(definition_revision.into())),
            revision_evidence: RevisionEvidence::PublicGeneration,
            note: "One read-only job-board recipe; source bindings carry authority generation expectations."
                .to_owned(),
        },
    }
}

/// The Chronicle variant that hides lease-epoch guide bands while preserving
/// the base definition and every non-arrangement field.
pub fn chronicle_era_bands_off_variant() -> ProjectionVariant {
    ProjectionVariant {
        dynamics: None,
        definition_id: CHRONICLE_DEFINITION_ID.to_owned(),
        id: "era-bands-off".to_owned(),
        arrangement_options: BTreeMap::from([(
            CHRONICLE_ERA_BANDS_OPTION.to_owned(),
            "false".to_owned(),
        )]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scenograph::{
        AuthoredDefinitionError, AuthoredProjectionDefinition, ProjectionInputBinding,
        PublicSourceRevision, RevisionEvidence, SourceBinding,
    };

    fn chronicle_sources() -> BTreeMap<String, ProjectionInputBinding> {
        BTreeMap::from([
            (
                "distillery".to_owned(),
                ProjectionInputBinding {
                    source: SourceBinding {
                        authority: "distillery.endpoint".to_owned(),
                        domain: "mesh.job-board".to_owned(),
                        resource: "distillery.chronicle".to_owned(),
                    },
                    expects_generation: Some("generation:distillery-fixture".into()),
                    revision_evidence: RevisionEvidence::PublicGeneration,
                },
            ),
            (
                "djinn".to_owned(),
                ProjectionInputBinding {
                    source: SourceBinding {
                        authority: "djinn.resident".to_owned(),
                        domain: "mesh.job-board".to_owned(),
                        resource: "distillery.chronicle".to_owned(),
                    },
                    expects_generation: Some("generation:djinn-fixture".into()),
                    revision_evidence: RevisionEvidence::PublicGeneration,
                },
            ),
        ])
    }

    #[test]
    fn chronicle_is_one_recipe_bound_to_two_checkable_sources() {
        let authored = chronicle_definition(
            chronicle_sources(),
            "Distillery W2",
            "chronicle-definition-v1",
        );
        authored.validate().expect("complete authored definition");
        let distillery = authored.bind("distillery", None).expect("distillery bind");
        let djinn = authored.bind("djinn", None).expect("djinn bind");

        assert_eq!(authored.id, CHRONICLE_DEFINITION_ID);
        assert_eq!(distillery.id, authored.id);
        assert_eq!(djinn.id, authored.id);
        assert_ne!(distillery.source, djinn.source);
        assert_eq!(distillery.reading, djinn.reading);
        assert_eq!(distillery.encoding, djinn.encoding);
        assert_eq!(distillery.arrangement, djinn.arrangement);
        assert_eq!(distillery.interaction, djinn.interaction);
        assert_eq!(distillery.appearance, djinn.appearance);
        assert_eq!(
            distillery.provenance.source_revision,
            Some(PublicSourceRevision::from("generation:distillery-fixture"))
        );
        assert_eq!(
            djinn.provenance.source_revision,
            Some(PublicSourceRevision::from("generation:djinn-fixture"))
        );

        let first = authored.to_json_bytes().expect("authoring bytes");
        let second = authored.to_json_bytes().expect("authoring bytes");
        assert_eq!(first, second);
        assert_eq!(
            serde_json::from_slice::<AuthoredProjectionDefinition>(&first).expect("authoring wire"),
            authored
        );
    }

    #[test]
    fn chronicle_era_bands_off_is_a_variant_not_another_recipe() {
        let authored = chronicle_definition(
            chronicle_sources(),
            "Distillery W2",
            "chronicle-definition-v1",
        );
        let base = authored.bind("distillery", None).expect("base bind");
        let variant = chronicle_era_bands_off_variant();
        let without_bands = authored
            .bind("distillery", Some(&variant))
            .expect("variant bind");

        assert_eq!(variant.definition_id, authored.id);
        assert_eq!(base.id, without_bands.id);
        assert_eq!(base.source, without_bands.source);
        assert_eq!(base.reading, without_bands.reading);
        assert_eq!(base.encoding, without_bands.encoding);
        assert_eq!(base.interaction, without_bands.interaction);
        assert_eq!(base.appearance, without_bands.appearance);
        assert_eq!(base.provenance, without_bands.provenance);
        assert_eq!(
            base.arrangement.options.get(CHRONICLE_ERA_BANDS_OPTION),
            Some(&"true".to_owned())
        );
        assert_eq!(
            without_bands
                .arrangement
                .options
                .get(CHRONICLE_ERA_BANDS_OPTION),
            Some(&"false".to_owned())
        );
    }

    #[test]
    fn authored_definition_refuses_a_variant_for_another_recipe() {
        let authored = chronicle_definition(
            chronicle_sources(),
            "Distillery W2",
            "chronicle-definition-v1",
        );
        let variant = ProjectionVariant {
            dynamics: None,
            definition_id: "other.recipe".to_owned(),
            id: "era-bands-off".to_owned(),
            arrangement_options: BTreeMap::new(),
        };
        assert_eq!(
            authored.bind("distillery", Some(&variant)),
            Err(AuthoredDefinitionError::VariantForAnotherDefinition(
                "other.recipe".to_owned()
            ))
        );
    }
}
