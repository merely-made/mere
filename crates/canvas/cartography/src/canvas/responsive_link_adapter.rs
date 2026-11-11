// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Bounded adapter from P1 source readings to Numen's temporary path evaluator.
//!
//! Evaluator handles are batch-local ordinals, never source identities. Every
//! handle reverses through `ResponsivePathBatch` to the borrowed LinkPairReading,
//! its full LinkKey, displayed endpoint IDs, and all directed source buckets.
//! Compatibility is supplied explicitly by the caller; this module does not
//! infer it from predicates or create persistent pair records.

use std::collections::BTreeMap;

use numen::{ResponsiveLink, evaluate_responsive_paths};
use scenograph::link::{LinkKey, LinkPairReading};

pub use numen::{
    FieldDef, FieldId, FieldRegistry, MAX_COMPATIBILITY_KEY_BYTES, MAX_CONTROL_POINTS,
    MAX_FIELD_AST_DEPTH, MAX_FIELD_AST_NODES, MAX_FIELDS, MAX_ITERATIONS, MAX_LINKS, PathField,
    ResponsivePath, ResponsivePathConfig, ResponsivePathError, ScalarField, VectorField,
    eval_vector,
};

/// Bound the number of source appearances accepted by one view-local step.
pub const MAX_RESPONSIVE_APPEARANCES: usize = numen::MAX_LINKS;
/// Bound provenance traversal and caller-side copying per evaluation.
pub const MAX_SOURCE_BUCKETS: usize = 1024;
/// Reject unusually large portable identities before repeated comparison.
pub const MAX_SOURCE_ID_BYTES: usize = 4096;

/// An explicit compatibility assignment for one appearance in the input
/// cohort. Endpoint strings are unordered and must match the displayed pair.
#[derive(Clone, Copy, Debug)]
pub struct AppearanceCompatibility<'a> {
    pub reading_index: usize,
    pub link_key: &'a LinkKey,
    pub displayed_first: &'a str,
    pub displayed_second: &'a str,
    pub compatibility: &'a str,
}

/// Reversible reference to one directed source bucket in the caller's input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceBucketRef {
    pub reading_index: usize,
    pub link_index: usize,
    pub bucket_index: usize,
}

/// One visible appearance, distinct even when another reading has the same
/// resource LinkKey and displayed endpoint pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponsiveAppearance<'a> {
    /// Opaque ordinal passed to Numen; meaningful only in this returned batch.
    pub evaluator_id: u64,
    /// Numen's endpoint handles reverse through `ResponsivePathBatch.endpoints`.
    pub start_endpoint_handle: u64,
    pub end_endpoint_handle: u64,
    pub reading_index: usize,
    pub link_index: usize,
    pub link_key: &'a LinkKey,
    /// Canonically ordered displayed surface identities; source direction is
    /// retained in the referenced buckets.
    pub displayed_start: &'a str,
    pub displayed_end: &'a str,
    pub compatibility: Option<String>,
    pub buckets: Vec<SourceBucketRef>,
}

/// An evaluator endpoint handle and its exact displayed source identity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EndpointHandle<'a> {
    pub evaluator_id: u64,
    pub displayed_id: &'a str,
    pub position: (f32, f32),
}

/// Geometry and its source correspondence for one evaluation. Source readings
/// remain borrowed and unchanged, preserving source bindings, revisions,
/// assertion IDs, bucket directions, and coverage without duplicating them.
#[derive(Clone, Debug)]
pub struct ResponsivePathBatch<'a> {
    pub paths: Vec<ResponsivePath>,
    pub appearances: Vec<ResponsiveAppearance<'a>>,
    pub endpoints: Vec<EndpointHandle<'a>>,
    pub readings: &'a [LinkPairReading],
    /// The exact declared controls used to generate `paths`.
    pub settings: ResponsivePathConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponsiveLinkAdapterError {
    TooManyReadings,
    TooManyAppearances,
    TooManySourceBuckets,
    SourceMismatch,
    EmptyAppearance,
    InconsistentDisplayedEndpoints,
    MissingDisplayedPosition,
    DuplicateCompatibility,
    StaleCompatibility,
    CompatibilityKeyTooLong,
    SourceIdentityTooLong,
    Evaluator(ResponsivePathError),
}

/// Adapt and evaluate a deterministic cohort of real P1 readings.
///
/// One `LinkReading` becomes one path appearance. Separate input readings are
/// never merged, even if their resource LinkKeys and displayed endpoints are
/// equal. `position_of` resolves displayed endpoint IDs only; source endpoint
/// IDs remain available through each bucket reference. Input and field order
/// are passed through to Numen unchanged.
pub fn evaluate_link_appearances<'source, 'compat>(
    readings: &'source [LinkPairReading],
    compatibility: &[AppearanceCompatibility<'compat>],
    mut position_of: impl FnMut(&str) -> Option<(f32, f32)>,
    fields: &[PathField<'_>],
    registry: &FieldRegistry,
    settings: ResponsivePathConfig,
) -> Result<ResponsivePathBatch<'source>, ResponsiveLinkAdapterError> {
    if readings.len() > MAX_RESPONSIVE_APPEARANCES {
        return Err(ResponsiveLinkAdapterError::TooManyReadings);
    }
    if compatibility.len() > MAX_RESPONSIVE_APPEARANCES {
        return Err(ResponsiveLinkAdapterError::TooManyAppearances);
    }
    if compatibility
        .iter()
        .any(|declaration| declaration.compatibility.len() > MAX_COMPATIBILITY_KEY_BYTES)
    {
        return Err(ResponsiveLinkAdapterError::CompatibilityKeyTooLong);
    }
    if compatibility.iter().any(|declaration| {
        declaration.link_key.first.len() > MAX_SOURCE_ID_BYTES
            || declaration.link_key.second.len() > MAX_SOURCE_ID_BYTES
            || declaration.displayed_first.len() > MAX_SOURCE_ID_BYTES
            || declaration.displayed_second.len() > MAX_SOURCE_ID_BYTES
    }) {
        return Err(ResponsiveLinkAdapterError::SourceIdentityTooLong);
    }
    if let Some(first) = readings.first() {
        if readings
            .iter()
            .any(|reading| reading.source != first.source)
        {
            return Err(ResponsiveLinkAdapterError::SourceMismatch);
        }
    }

    let mut total_appearances = 0usize;
    let mut total_buckets = 0usize;
    for reading in readings {
        total_appearances = total_appearances.saturating_add(reading.links.len());
        if total_appearances > MAX_RESPONSIVE_APPEARANCES {
            return Err(ResponsiveLinkAdapterError::TooManyAppearances);
        }
        for link in &reading.links {
            if link.key.first.len() > MAX_SOURCE_ID_BYTES
                || link.key.second.len() > MAX_SOURCE_ID_BYTES
            {
                return Err(ResponsiveLinkAdapterError::SourceIdentityTooLong);
            }
            if link.buckets.is_empty() {
                return Err(ResponsiveLinkAdapterError::EmptyAppearance);
            }
            total_buckets = total_buckets.saturating_add(link.buckets.len());
            if total_buckets > MAX_SOURCE_BUCKETS {
                return Err(ResponsiveLinkAdapterError::TooManySourceBuckets);
            }
            if link
                .buckets
                .iter()
                .any(|bucket| bucket.assertions.is_empty() && bucket.legacy_members.is_empty())
            {
                return Err(ResponsiveLinkAdapterError::EmptyAppearance);
            }
        }
    }

    let mut endpoint_ids = BTreeMap::<&str, u64>::new();
    let mut endpoints = Vec::<EndpointHandle<'source>>::new();
    let mut appearances = Vec::with_capacity(total_appearances);
    let mut evaluator_links = Vec::with_capacity(total_appearances);
    let mut consumed_compatibility = vec![false; compatibility.len()];

    for (reading_index, reading) in readings.iter().enumerate() {
        for (link_index, link) in reading.links.iter().enumerate() {
            let first_bucket = &link.buckets[0];
            let (displayed_start, displayed_end) =
                if first_bucket.displayed_from <= first_bucket.displayed_to {
                    (
                        first_bucket.displayed_from.as_str(),
                        first_bucket.displayed_to.as_str(),
                    )
                } else {
                    (
                        first_bucket.displayed_to.as_str(),
                        first_bucket.displayed_from.as_str(),
                    )
                };
            for bucket in &link.buckets {
                if bucket.displayed_from.len() > MAX_SOURCE_ID_BYTES
                    || bucket.displayed_to.len() > MAX_SOURCE_ID_BYTES
                {
                    return Err(ResponsiveLinkAdapterError::SourceIdentityTooLong);
                }
                let pair_matches = (bucket.displayed_from == displayed_start
                    && bucket.displayed_to == displayed_end)
                    || (bucket.displayed_from == displayed_end
                        && bucket.displayed_to == displayed_start);
                if !pair_matches {
                    return Err(ResponsiveLinkAdapterError::InconsistentDisplayedEndpoints);
                }
            }

            let mut compatibility_key = None;
            for (index, declaration) in compatibility.iter().enumerate() {
                if declaration.reading_index != reading_index || declaration.link_key != &link.key {
                    continue;
                }
                let matches_pair = (declaration.displayed_first == displayed_start
                    && declaration.displayed_second == displayed_end)
                    || (declaration.displayed_first == displayed_end
                        && declaration.displayed_second == displayed_start);
                if matches_pair {
                    if consumed_compatibility[index] || compatibility_key.is_some() {
                        return Err(ResponsiveLinkAdapterError::DuplicateCompatibility);
                    }
                    consumed_compatibility[index] = true;
                    compatibility_key = Some(declaration.compatibility);
                }
            }

            let (start_endpoint_handle, start) = endpoint_handle(
                displayed_start,
                &mut endpoint_ids,
                &mut endpoints,
                &mut position_of,
            )?;
            let (end_endpoint_handle, end) = endpoint_handle(
                displayed_end,
                &mut endpoint_ids,
                &mut endpoints,
                &mut position_of,
            )?;
            let evaluator_id = u64::try_from(appearances.len())
                .map_err(|_| ResponsiveLinkAdapterError::TooManyAppearances)?;
            evaluator_links.push(ResponsiveLink {
                id: evaluator_id,
                start_endpoint_id: start_endpoint_handle,
                end_endpoint_id: end_endpoint_handle,
                start,
                end,
                compatibility: compatibility_key,
            });
            appearances.push(ResponsiveAppearance {
                evaluator_id,
                start_endpoint_handle,
                end_endpoint_handle,
                reading_index,
                link_index,
                link_key: &link.key,
                displayed_start,
                displayed_end,
                compatibility: compatibility_key.map(str::to_owned),
                buckets: (0..link.buckets.len())
                    .map(|bucket_index| SourceBucketRef {
                        reading_index,
                        link_index,
                        bucket_index,
                    })
                    .collect(),
            });
        }
    }
    if consumed_compatibility.iter().any(|used| !used) {
        return Err(ResponsiveLinkAdapterError::StaleCompatibility);
    }

    let paths = evaluate_responsive_paths(&evaluator_links, fields, registry, settings)
        .map_err(ResponsiveLinkAdapterError::Evaluator)?;
    Ok(ResponsivePathBatch {
        paths,
        appearances,
        endpoints,
        readings,
        settings,
    })
}

fn endpoint_handle<'a>(
    displayed_id: &'a str,
    endpoint_ids: &mut BTreeMap<&'a str, u64>,
    endpoints: &mut Vec<EndpointHandle<'a>>,
    position_of: &mut impl FnMut(&str) -> Option<(f32, f32)>,
) -> Result<(u64, (f32, f32)), ResponsiveLinkAdapterError> {
    if let Some(&handle) = endpoint_ids.get(displayed_id) {
        let index =
            usize::try_from(handle).map_err(|_| ResponsiveLinkAdapterError::TooManyAppearances)?;
        return Ok((handle, endpoints[index].position));
    }
    let handle = u64::try_from(endpoints.len())
        .map_err(|_| ResponsiveLinkAdapterError::TooManyAppearances)?;
    let position =
        position_of(displayed_id).ok_or(ResponsiveLinkAdapterError::MissingDisplayedPosition)?;
    endpoint_ids.insert(displayed_id, handle);
    endpoints.push(EndpointHandle {
        evaluator_id: handle,
        displayed_id,
        position,
    });
    Ok((handle, position))
}

#[cfg(test)]
#[path = "responsive_link_adapter_tests.rs"]
mod tests;
