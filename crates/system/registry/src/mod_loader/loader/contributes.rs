// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a mod adds to the GUI: declared in its manifest before it runs, and
//! checked against the records it returns when it activates.

use inker::PresentationDeclaration;
use inker::presentation::Presents;
use serde::{Deserialize, Serialize};

use super::types::{ModActivationError, ModExtensionRecord};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContributionKind {
    Presentation,
    PaneAdapter,
    Pane,
    Servitor,
}

/// One contribution, named by kind and id. A presentation carries inker's
/// declaration; the other kinds name only the id until they have a seam to
/// declare against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Contribution {
    Presentation(PresentationDeclaration),
    PaneAdapter { id: String },
    Pane { id: String },
    Servitor { id: String },
}

impl Contribution {
    pub fn kind(&self) -> ContributionKind {
        match self {
            Self::Presentation(_) => ContributionKind::Presentation,
            Self::PaneAdapter { .. } => ContributionKind::PaneAdapter,
            Self::Pane { .. } => ContributionKind::Pane,
            Self::Servitor { .. } => ContributionKind::Servitor,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Presentation(declaration) => &declaration.id,
            Self::PaneAdapter { id } | Self::Pane { id } | Self::Servitor { id } => id,
        }
    }
}

/// The first record in `records` that `contributes` does not declare.
///
/// Protocol schemes, content types and extensions must be presented by one of
/// the manifest's presentations, and a presentation's reach by that
/// presentation. Actions, index providers, lenses, themes and the wasm
/// runtime belong to none of the four kinds, so a manifest cannot declare
/// them and they pass.
pub(super) fn undeclared_record<'a>(
    contributes: &[Contribution],
    records: &'a [ModExtensionRecord],
) -> Option<&'a ModExtensionRecord> {
    let presentations = || {
        contributes
            .iter()
            .filter_map(|contribution| match contribution {
                Contribution::Presentation(declaration) => Some(declaration),
                _ => None,
            })
    };
    let presented = |wanted: &str, list: fn(&Presents) -> &Vec<String>| {
        presentations().any(|declaration| {
            list(&declaration.presents)
                .iter()
                .any(|have| have.eq_ignore_ascii_case(wanted))
        })
    };
    records.iter().find(|record| !match record {
        ModExtensionRecord::ProtocolScheme { scheme, .. } => {
            presented(scheme, |presents| &presents.schemes)
        },
        ModExtensionRecord::PresentationContentType { content_type, .. } => {
            presented(content_type, |presents| &presents.content_types)
        },
        ModExtensionRecord::PresentationExtension { extension, .. } => {
            presented(extension.trim_start_matches('.'), |presents| {
                &presents.extensions
            })
        },
        ModExtensionRecord::PresentationReaches {
            presentation_id, ..
        } => presentations().any(|declaration| declaration.id == *presentation_id),
        ModExtensionRecord::Action { .. }
        | ModExtensionRecord::IndexProvider { .. }
        | ModExtensionRecord::Lens { .. }
        | ModExtensionRecord::Theme { .. }
        | ModExtensionRecord::WasmRuntime { .. } => true,
    })
}

/// Refuse `records` that `contributes` does not declare. The refusal carries
/// all the records, so the caller rolls back everything the mod applied.
pub(super) fn verify_records(
    mod_id: &str,
    contributes: &[Contribution],
    records: Vec<ModExtensionRecord>,
) -> Result<Vec<ModExtensionRecord>, ModActivationError> {
    match undeclared_record(contributes, &records).cloned() {
        Some(record) => Err(ModActivationError::undeclared(mod_id, record, records)),
        None => Ok(records),
    }
}
