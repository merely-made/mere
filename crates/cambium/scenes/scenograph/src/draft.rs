// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl ProjectionDraft {
    /// Start an empty draft with safe enum and layout defaults.
    pub fn new() -> Self {
        Self {
            version: PROJECTION_DEFINITION_VERSION,
            id: String::new(),
            label: String::new(),
            source: SourceBinding::default(),
            reading: Reading::default(),
            encoding: Encoding::default(),
            arrangement: Arrangement::default(),
            dynamics: None,
            interaction: Interaction::default(),
            appearance: Appearance::default(),
            provenance: Provenance::default(),
        }
    }

    /// Validate and promote this draft to the durable definition.
    pub fn to_definition(&self) -> Result<ProjectionDefinition, Vec<ValidationIssue>> {
        self.validate()?;
        Ok(ProjectionDefinition {
            dynamics: self.dynamics.clone(),
            version: self.version,
            id: self.id.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            reading: self.reading.clone(),
            encoding: self.encoding.clone(),
            arrangement: self.arrangement.clone(),
            interaction: self.interaction.clone(),
            appearance: self.appearance.clone(),
            provenance: self.provenance.clone(),
        })
    }

    /// Check all required fields without contacting a host or authority.
    pub fn validate(&self) -> Result<(), Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        if let Some(slot) = &self.dynamics {
            if let Err(message) = slot.validate() {
                issues.push(ValidationIssue::error("dynamics", &message));
            }
        }
        if self.version != PROJECTION_DEFINITION_VERSION {
            issues.push(ValidationIssue::error(
                "version",
                "unsupported projection definition version",
            ));
        }
        required(&mut issues, "id", &self.id, "an id is required");
        required(&mut issues, "label", &self.label, "a label is required");
        required(
            &mut issues,
            "source.authority",
            &self.source.authority,
            "an authority binding is required",
        );
        required(
            &mut issues,
            "source.domain",
            &self.source.domain,
            "a domain binding is required",
        );
        required(
            &mut issues,
            "source.resource",
            &self.source.resource,
            "a source resource is required",
        );
        required(
            &mut issues,
            "reading.kind",
            &self.reading.kind,
            "a registered reading id is required",
        );
        required(
            &mut issues,
            "reading.key",
            &self.reading.key,
            "a reading key is required",
        );
        if self.reading.kind == "values" {
            match self.reading.value.as_deref() {
                Some(value) if !value.trim().is_empty() => {},
                _ => issues.push(ValidationIssue::error(
                    "reading.value",
                    "a value field is required for a values reading",
                )),
            }
        }
        channel_required(&mut issues, "encoding.x", &self.encoding.x);
        channel_required(&mut issues, "encoding.y", &self.encoding.y);
        required(
            &mut issues,
            "arrangement.kind",
            &self.arrangement.kind,
            "a registered arrangement id is required",
        );
        required(
            &mut issues,
            "arrangement.direction",
            &self.arrangement.direction,
            "an arrangement direction or coordinate mode is required",
        );
        if self.arrangement.spacing == 0 {
            issues.push(ValidationIssue::error(
                "arrangement.spacing",
                "spacing must be greater than zero",
            ));
        }
        required(
            &mut issues,
            "appearance.realization",
            &self.appearance.realization,
            "a registered realization id is required",
        );
        required(
            &mut issues,
            "appearance.title",
            &self.appearance.title,
            "a preview title is required",
        );
        required(
            &mut issues,
            "provenance.author",
            &self.provenance.author,
            "an author is required",
        );
        match self.provenance.revision_evidence {
            RevisionEvidence::PublicGeneration => match &self.provenance.source_revision {
                Some(revision) => required(
                    &mut issues,
                    "provenance.source_revision",
                    revision.as_str(),
                    "the source revision is required for reproducibility",
                ),
                None => issues.push(ValidationIssue::error(
                    "provenance.source_revision",
                    "the source revision is required for reproducibility",
                )),
            },
            RevisionEvidence::RuntimeVerified => {
                if self.provenance.source_revision.is_some() {
                    issues.push(ValidationIssue::error(
                        "provenance.source_revision",
                        "runtime-verified sources cannot claim a public source revision",
                    ));
                }
            },
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}
