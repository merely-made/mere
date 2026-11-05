// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Admitting an accepted tag suggestion as a personal `taggedWith` assertion.
//!
//! Eidetic produces a `PersonalTagAssertionDraftV1` when a person accepts a
//! generated suggestion, and leaves admission to the owning application. This
//! module is that application funnel on the graph side. The kernel does not
//! depend on Eidetic: the host converts the draft into [`AcceptedTagSuggestion`]
//! (concept owner and id, asserter, `cites_suggestion` as lowercase hex) and
//! names the resource itself. Mapping the draft's retained source to a resource
//! is a host decision; nothing here invents one.
//!
//! The admitted statement is an ordinary personal tag: `provenance_iri` stays
//! the asserter, as in every other tag writer, so `remove_shown_tag` and
//! per-asserter reads treat it like any hand-made tag. The suggestion is cited
//! in the statement's [`AssertionBasis`]: method
//! [`ACCEPTED_TAG_SUGGESTION_METHOD`] and one evidence IRI from
//! [`tag_suggestion_iri`].

use super::SemanticStatement;
use super::resource::{ResourceNode, TAGGED_WITH_IRI};
use super::resource_content::ContentError;
use super::resource_tags::TagConcept;
use crate::types::{AssertionBasis, AssertionMethod, GraphScope};
use uuid::Uuid;

/// Method IRI recorded on a statement admitted from an accepted suggestion.
pub const ACCEPTED_TAG_SUGGESTION_METHOD: &str = "urn:mere:method:accepted-tag-suggestion";

const SUGGESTION_IRI_PREFIX: &str = "urn:eidetic:tag-suggestion:";

/// The kernel-native form of an accepted suggestion's assertion draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedTagSuggestion {
    /// The concept's declared owner (the draft's `concept.owner`).
    pub concept_owner_iri: String,
    /// The concept's IRI (the draft's `concept.concept`).
    pub concept_iri: String,
    /// Label for a concept this graph does not hold yet. When the concept
    /// resource exists its label is used, and a different label here is refused.
    pub concept_label: Option<String>,
    /// Absolute IRI of the person who accepted. The host maps the draft's
    /// reviewer string to an agent IRI; a non-IRI is refused.
    pub asserter_iri: String,
    /// The suggestion's content hash (`cites_suggestion`) as 64 lowercase hex.
    pub suggestion_hash_hex: String,
}

/// What an admission changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagAdmission {
    /// A new personal statement citing the suggestion was recorded.
    Added,
    /// The asserter already holds this tag on this resource, whether from this
    /// draft or otherwise. Nothing was written and the held statement is kept.
    AlreadyHeld,
}

/// The stable evidence IRI citing one suggestion.
///
/// # Errors
///
/// Refuses anything but 64 lowercase hex digits.
pub fn tag_suggestion_iri(suggestion_hash_hex: &str) -> Result<String, ContentError> {
    if suggestion_hash_hex.len() != 64
        || !suggestion_hash_hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(ContentError::InvalidFacet(
            "suggestion hash must be 64 lowercase hex digits".into(),
        ));
    }
    Ok(format!("{SUGGESTION_IRI_PREFIX}{suggestion_hash_hex}"))
}

fn is_absolute_iri(value: &str) -> bool {
    !value.chars().any(|c| c.is_whitespace() || c.is_control()) && url::Url::parse(value).is_ok()
}

impl super::Graph {
    /// Admit an accepted suggestion as `asserter`'s personal tag on `resource_id`.
    ///
    /// Idempotent per asserter, resource and concept: a repeat (or a prior
    /// hand-made tag by the same asserter) returns [`TagAdmission::AlreadyHeld`]
    /// and writes nothing. Other asserters' tags on the same pair, and every
    /// other tag, are untouched.
    ///
    /// # Errors
    ///
    /// Refuses an invalid concept, owner, asserter or hash, a missing
    /// resource, a concept with no known label, and a label that differs from
    /// the held concept's.
    pub fn admit_accepted_tag_suggestion(
        &mut self,
        resource_id: Uuid,
        accepted: &AcceptedTagSuggestion,
    ) -> Result<TagAdmission, ContentError> {
        let evidence = tag_suggestion_iri(&accepted.suggestion_hash_hex)?;
        if !is_absolute_iri(&accepted.asserter_iri) {
            return Err(ContentError::InvalidFacet(
                "asserter must be an absolute IRI".into(),
            ));
        }
        let held_label = self
            .resource_tag_concept(ResourceNode::for_term(&accepted.concept_iri).id())
            .map(|concept| concept.label);
        let label = accepted
            .concept_label
            .clone()
            .or(held_label)
            .ok_or_else(|| ContentError::InvalidFacet("concept label unknown".into()))?;
        let concept = TagConcept {
            owner_iri: accepted.concept_owner_iri.clone(),
            label,
        };
        let mut statement = SemanticStatement::new(
            TAGGED_WITH_IRI.into(),
            None,
            None,
            GraphScope::Default,
            Some(accepted.asserter_iri.clone()),
            Some(crate::time::unix_epoch_millis()),
        );
        statement.basis = AssertionBasis {
            method: Some(AssertionMethod {
                iri: ACCEPTED_TAG_SUGGESTION_METHOD.into(),
                version: Some("1".into()),
            }),
            evidence: vec![evidence],
            ..Default::default()
        };
        let added = self.tag_resource_with_statement(
            resource_id,
            &accepted.concept_iri,
            concept,
            statement,
        )?;
        Ok(if added {
            TagAdmission::Added
        } else {
            TagAdmission::AlreadyHeld
        })
    }
}

#[cfg(test)]
#[path = "resource_tag_admission_tests.rs"]
mod tests;
