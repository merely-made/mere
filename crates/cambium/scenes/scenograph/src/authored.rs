// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl AuthoredProjectionDefinition {
    /// Validate the recipe and every source it declares without contacting an
    /// authority. A bad dormant source is still a bad durable definition.
    pub fn validate(&self) -> Result<(), AuthoredDefinitionError> {
        let mut issues = Vec::new();
        if let Some(slot) = &self.dynamics {
            slot.validate().map_err(AuthoredDefinitionError::Dynamics)?;
        }
        required(
            &mut issues,
            "provenance.author",
            &self.provenance.author,
            "an author is required",
        );
        required(
            &mut issues,
            "provenance.source_revision",
            self.provenance
                .source_revision
                .as_ref()
                .map(PublicSourceRevision::as_str)
                .unwrap_or_default(),
            "an authored definition revision is required",
        );
        for (name, value) in &self.arrangement.options {
            if name.trim().is_empty() || value.trim().is_empty() {
                return Err(AuthoredDefinitionError::EmptyArrangementOption(
                    name.clone(),
                ));
            }
        }
        if !issues.is_empty() {
            return Err(AuthoredDefinitionError::Invalid(issues));
        }
        if self.sources.is_empty() {
            return Err(AuthoredDefinitionError::Invalid(vec![
                ValidationIssue::error("sources", "at least one source binding is required"),
            ]));
        }
        for (name, binding) in &self.sources {
            if name.trim().is_empty() {
                return Err(AuthoredDefinitionError::Invalid(vec![
                    ValidationIssue::error("sources", "a source binding name is required"),
                ]));
            }
            match binding.revision_evidence {
                RevisionEvidence::PublicGeneration => match &binding.expects_generation {
                    Some(generation) if !generation.as_str().trim().is_empty() => {},
                    _ => {
                        return Err(AuthoredDefinitionError::EmptyExpectedGeneration(
                            name.clone(),
                        ));
                    },
                },
                RevisionEvidence::RuntimeVerified if binding.expects_generation.is_some() => {
                    return Err(AuthoredDefinitionError::RuntimeBindingHasPublicGeneration(
                        name.clone(),
                    ));
                },
                RevisionEvidence::RuntimeVerified => {},
            }
            self.draft_for(binding, self.arrangement.clone())
                .validate()
                .map_err(AuthoredDefinitionError::Invalid)?;
        }
        Ok(())
    }

    /// Select one named source and optionally apply an authored arrangement
    /// variant. The result retains the recipe id and records the selected input
    /// public generation for the executable host path.
    pub fn bind(
        &self,
        source_name: &str,
        variant: Option<&ProjectionVariant>,
    ) -> Result<ProjectionDefinition, AuthoredDefinitionError> {
        self.bind_with_dynamics(source_name, variant, |id| {
            Err(format!(
                "variant.dynamics.id: preset {id:?} requires a host catalog"
            ))
        })
    }

    /// Bind with the host's preset catalog. The portable recipe never
    /// interprets or silently substitutes a physics preset.
    pub fn bind_with_dynamics(
        &self,
        source_name: &str,
        variant: Option<&ProjectionVariant>,
        resolve: impl FnOnce(&str) -> Result<DynamicsSlot, String>,
    ) -> Result<ProjectionDefinition, AuthoredDefinitionError> {
        self.validate()?;
        let binding = self
            .sources
            .get(source_name)
            .ok_or_else(|| AuthoredDefinitionError::UnknownSource(source_name.to_owned()))?;
        if binding.revision_evidence == RevisionEvidence::RuntimeVerified {
            return Err(AuthoredDefinitionError::RuntimeBindingRequiresWitness(
                source_name.to_owned(),
            ));
        }
        let arrangement = self.arrangement_with_variant(variant)?;
        let mut draft = self.draft_for(binding, arrangement);
        draft.dynamics = self.dynamics_with_variant(variant, resolve)?;
        draft
            .to_definition()
            .map_err(AuthoredDefinitionError::Invalid)
    }

    /// Bind a host-selected source whose only freshness evidence is an opaque
    /// runtime witness. This does not verify source authority or interpret the
    /// witness; both remain host-owned.
    pub fn bind_runtime<W>(
        &self,
        source_name: &str,
        runtime_source: RuntimeSourceBinding<W>,
        variant: Option<&ProjectionVariant>,
    ) -> Result<RuntimeProjectionBinding<W>, AuthoredDefinitionError> {
        self.bind_runtime_with_dynamics(source_name, runtime_source, variant, |id| {
            Err(format!(
                "variant.dynamics.id: preset {id:?} requires a host catalog"
            ))
        })
    }

    pub fn bind_runtime_with_dynamics<W>(
        &self,
        source_name: &str,
        runtime_source: RuntimeSourceBinding<W>,
        variant: Option<&ProjectionVariant>,
        resolve: impl FnOnce(&str) -> Result<DynamicsSlot, String>,
    ) -> Result<RuntimeProjectionBinding<W>, AuthoredDefinitionError> {
        self.validate()?;
        let declared = self
            .sources
            .get(source_name)
            .ok_or_else(|| AuthoredDefinitionError::UnknownSource(source_name.to_owned()))?;
        if declared.source != runtime_source.source {
            return Err(AuthoredDefinitionError::RuntimeSourceMismatch(
                source_name.to_owned(),
            ));
        }
        if declared.revision_evidence != RevisionEvidence::RuntimeVerified {
            return Err(AuthoredDefinitionError::RuntimeEvidenceRequired(
                source_name.to_owned(),
            ));
        }
        let arrangement = self.arrangement_with_variant(variant)?;
        let mut draft = self.draft_for(declared, arrangement);
        draft.dynamics = self.dynamics_with_variant(variant, resolve)?;
        let definition = draft
            .to_definition()
            .map_err(AuthoredDefinitionError::Invalid)?;
        Ok(RuntimeProjectionBinding {
            definition,
            source: runtime_source,
        })
    }

    /// Serialize in declaration and sorted-map order for a host-readable
    /// receipt or a durable authoring store.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub(super) fn arrangement_with_variant(
        &self,
        variant: Option<&ProjectionVariant>,
    ) -> Result<Arrangement, AuthoredDefinitionError> {
        let Some(variant) = variant else {
            return Ok(self.arrangement.clone());
        };
        if variant.definition_id != self.id {
            return Err(AuthoredDefinitionError::VariantForAnotherDefinition(
                variant.definition_id.clone(),
            ));
        }
        if variant.id.trim().is_empty() {
            return Err(AuthoredDefinitionError::EmptyVariantId);
        }
        let mut arrangement = self.arrangement.clone();
        for (name, value) in &variant.arrangement_options {
            if name.trim().is_empty() || value.trim().is_empty() {
                return Err(AuthoredDefinitionError::EmptyArrangementOption(
                    name.clone(),
                ));
            }
            arrangement.options.insert(name.clone(), value.clone());
        }
        Ok(arrangement)
    }

    pub(super) fn dynamics_with_variant(
        &self,
        variant: Option<&ProjectionVariant>,
        resolve: impl FnOnce(&str) -> Result<DynamicsSlot, String>,
    ) -> Result<Option<DynamicsSlot>, AuthoredDefinitionError> {
        match variant.and_then(|variant| variant.dynamics.as_ref()) {
            None => Ok(self.dynamics.clone()),
            Some(DynamicsVariant::Spec { slot }) => Ok(Some(slot.clone())),
            Some(DynamicsVariant::Preset { id }) => resolve(id)
                .map(Some)
                .map_err(AuthoredDefinitionError::Dynamics),
        }
    }

    pub(super) fn draft_for(
        &self,
        binding: &ProjectionInputBinding,
        arrangement: Arrangement,
    ) -> ProjectionDraft {
        ProjectionDraft {
            dynamics: self.dynamics.clone(),
            version: self.version,
            id: self.id.clone(),
            label: self.label.clone(),
            source: binding.source.clone(),
            reading: self.reading.clone(),
            encoding: self.encoding.clone(),
            arrangement,
            interaction: self.interaction.clone(),
            appearance: self.appearance.clone(),
            provenance: Provenance {
                author: self.provenance.author.clone(),
                source_revision: binding.expects_generation.clone(),
                revision_evidence: binding.revision_evidence,
                note: self.provenance.note.clone(),
            },
        }
    }
}
