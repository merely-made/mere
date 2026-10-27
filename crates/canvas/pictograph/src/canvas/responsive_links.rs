// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//!
//! View-local responsive link-path inputs for the ordinary graph canvas.
//!
//! Source readings remain canonical graph readings. Numen supplies only
//! temporary sampled geometry; it does not change graph identity or physics.

use std::collections::HashMap;
use std::sync::Arc;

use kernel::graph::{FieldDefinition, FieldExtent, FieldId, NodeKey};
use scenograph::link::{LinkKey, LinkPairReading, LinkSource, LinkStratum};

use super::responsive_link_adapter::{
    AppearanceCompatibility, FieldRegistry, PathField, ResponsivePathConfig, SourceBucketRef,
    evaluate_link_appearances,
};
#[path = "responsive_links/fields.rs"]
mod fields;
#[path = "responsive_links/geometry.rs"]
mod geometry;
#[path = "responsive_links/report.rs"]
mod report;
use super::{Canvas, edge_cells};

#[derive(Clone, Debug)]
pub struct LinkPathResponse {
    pub field_response: bool,
    pub compatible_attraction: bool,
    pub selected_fields: Vec<FieldId>,
    pub compatible_links: Vec<LinkKey>,
    pub field_strength: f32,
    /// An explicitly view-local field AST. Its label and construction belong
    /// to the host; this never becomes graph truth.
    pub view_field: Option<Arc<numen::VectorField>>,
    pub config: ResponsivePathConfig,
}

impl Default for LinkPathResponse {
    fn default() -> Self {
        Self {
            field_response: false,
            compatible_attraction: false,
            selected_fields: Vec::new(),
            compatible_links: Vec::new(),
            field_strength: 1.0,
            view_field: None,
            config: ResponsivePathConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkPathState {
    Disabled,
    Applied,
    Refused,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkPathFailure {
    MissingField(FieldId),
    RetiredField(FieldId),
    ScalarField(FieldId),
    UnsupportedExtent(FieldId),
    FieldSampleKindMismatch(FieldId),
    FieldDependencyTooLarge,
    StaleCompatibleLink(LinkKey),
    InvalidFieldStrength,
    InvalidProjection,
    Adapter(String),
    HiddenEndpoint,
}

#[derive(Clone, Debug)]
pub struct LinkPathSourceRef {
    pub displayed_from: String,
    pub displayed_to: String,
    pub assertion_ids: Vec<String>,
    pub legacy_members: Vec<String>,
    pub bucket: SourceBucketRef,
}

#[derive(Clone, Debug)]
pub struct EvaluatedLinkPath {
    pub key: LinkKey,
    pub displayed_from: uuid::Uuid,
    pub displayed_to: uuid::Uuid,
    pub stratum: LinkStratum,
    pub points: Vec<(f32, f32)>,
    /// False for resting geometry (including self-link arches and refusals).
    pub responsive: bool,
    pub source: LinkSource,
    pub buckets: Vec<LinkPathSourceRef>,
}

impl EvaluatedLinkPath {
    /// Maximum perpendicular distance from this emitted path's endpoint chord,
    /// in world units. A geometry observation, independent of endpoint forces.
    pub fn max_bend(&self) -> f32 {
        let points = &self.points;
        let (Some(first), Some(last)) = (points.first(), points.last()) else {
            return 0.0;
        };
        let dx = last.0 - first.0;
        let dy = last.1 - first.1;
        let length = dx.hypot(dy).max(f32::EPSILON);
        points
            .iter()
            .map(|point| ((dx * (first.1 - point.1) - (first.0 - point.0) * dy).abs()) / length)
            .fold(0.0_f32, f32::max)
    }
}

#[derive(Clone, Debug)]
pub struct LinkPathResponseReport {
    pub state: LinkPathState,
    pub failure: Option<LinkPathFailure>,
    pub settings: LinkPathResponse,
    pub paths: Vec<EvaluatedLinkPath>,
    pub responsive_paths: usize,
    pub ineligible_self_links: usize,
    pub excluded_endpoints: usize,
    /// Evaluated paths kept at rest, or self arches reduced to a finite stem
    /// foot, because their bodies cannot be safely attached.
    pub presentation_fallbacks: usize,
}

impl Canvas {
    pub fn link_path_response(&self) -> &LinkPathResponse {
        &self.link_path_response
    }

    pub fn set_link_path_response(&mut self, mut response: LinkPathResponse) {
        response.selected_fields.sort_by_key(|id| id.as_uuid());
        response.selected_fields.dedup_by_key(|id| id.as_uuid());
        response.compatible_links.sort();
        response.compatible_links.dedup();
        self.link_path_response = response;
    }

    pub fn link_path_response_report(&self) -> LinkPathResponseReport {
        let (mut segments, readings) = self.link_geometry();
        let outcome = evaluate_canvas(self, &mut segments, &readings);
        LinkPathResponseReport {
            state: outcome.state,
            failure: outcome.failure,
            settings: self.link_path_response.clone(),
            responsive_paths: segments.iter().filter(|segment| segment.responsive).count(),
            paths: report::base_paths(self, &segments, &readings),
            ineligible_self_links: outcome.ineligible_self_links,
            excluded_endpoints: outcome.excluded_endpoints,
            presentation_fallbacks: outcome.presentation_fallbacks,
        }
    }

    fn link_geometry(&self) -> (Vec<edge_cells::LinkAppearanceSegment>, Vec<LinkPairReading>) {
        edge_cells::visible_link_geometry_with_assertion_hides(
            &self.graph,
            &self.view,
            &self.hidden_links,
            &self.hidden_edges,
            &self.hidden_assertions,
            |key| self.node_size(key),
        )
    }

    /// Current evaluated paths for host inspection. This always reads the
    /// current graph, view positions, and hide state; no paths are cached.
    pub fn visible_link_paths(&self) -> Vec<EvaluatedLinkPath> {
        self.link_path_response_report().paths
    }

    pub(crate) fn rendered_link_segments(&self) -> Vec<edge_cells::LinkAppearanceSegment> {
        if !self.link_path_response.field_response && !self.link_path_response.compatible_attraction
        {
            let mut segments = edge_cells::visible_link_segments_with_assertion_hides(
                &self.graph,
                &self.view,
                &self.hidden_links,
                &self.hidden_edges,
                &self.hidden_assertions,
                |key| self.node_size(key),
            );
            edge_cells::attach_self_link_geometry(&mut segments, &self.camera, |key| {
                self.node_billboard(key)
            });
            return segments;
        }
        let (mut segments, readings) = self.link_geometry();
        evaluate_canvas(self, &mut segments, &readings);
        segments
    }
}

struct EvaluationOutcome {
    state: LinkPathState,
    failure: Option<LinkPathFailure>,
    ineligible_self_links: usize,
    excluded_endpoints: usize,
    presentation_fallbacks: usize,
}

fn evaluate_canvas(
    canvas: &Canvas,
    segments: &mut [edge_cells::LinkAppearanceSegment],
    readings: &[LinkPairReading],
) -> EvaluationOutcome {
    let settings = &canvas.link_path_response;
    let self_fallbacks = edge_cells::attach_self_link_geometry(segments, &canvas.camera, |key| {
        canvas.node_billboard(key)
    });
    let mut outcome = EvaluationOutcome {
        state: LinkPathState::Disabled,
        failure: None,
        ineligible_self_links: segments
            .iter()
            .filter(|segment| segment.appearance.display_from == segment.appearance.display_to)
            .count(),
        excluded_endpoints: 0,
        presentation_fallbacks: self_fallbacks,
    };
    if (settings.field_response || settings.compatible_attraction) && !canvas.camera.is_valid() {
        outcome.presentation_fallbacks += segments.len() - outcome.ineligible_self_links;
        return refused(outcome, LinkPathFailure::InvalidProjection);
    }
    let on_screen: std::collections::HashSet<_> = if canvas.generation == 0 {
        canvas.graph.nodes().map(|(key, _)| key).collect()
    } else {
        canvas
            .view
            .cull_aabb(canvas.world_viewport())
            .into_iter()
            .collect()
    };
    let node_eligible =
        |key: NodeKey| canvas.node_visible_in_canvas(key) && on_screen.contains(&key);
    outcome.excluded_endpoints = segments
        .iter()
        .filter(|segment| {
            !node_eligible(segment.appearance.display_from)
                || !node_eligible(segment.appearance.display_to)
        })
        .count();
    if !settings.field_response && !settings.compatible_attraction {
        return outcome;
    }

    let endpoint_visible = |id: &str| {
        id.parse::<uuid::Uuid>()
            .ok()
            .and_then(|id| canvas.graph.get_node_key_by_id(id))
            .is_some_and(node_eligible)
    };
    // Filter before cloning provenance: offscreen/self appearances are not
    // evaluator inputs, and empty pairs must not consume its reading budget.
    let eligible_readings: Vec<_> = readings
        .iter()
        .filter_map(|reading| {
            let links: Vec<_> = reading
                .links
                .iter()
                .filter(|link| {
                    let Some(bucket) = link.buckets.first() else {
                        return false;
                    };
                    bucket.displayed_from != bucket.displayed_to
                        && endpoint_visible(&bucket.displayed_from)
                        && endpoint_visible(&bucket.displayed_to)
                        && (settings.field_response
                            || settings.compatible_links.contains(&link.key))
                })
                .cloned()
                .collect();
            (!links.is_empty()).then(|| LinkPairReading {
                source: reading.source.clone(),
                links,
                coverage: reading.coverage.clone(),
            })
        })
        .collect();
    let mut selected_vectors = Vec::new();
    for id in settings
        .selected_fields
        .iter()
        .filter(|_| settings.field_response)
    {
        let Some(field) = canvas.graph.field(*id) else {
            return refused(outcome, LinkPathFailure::MissingField(*id));
        };
        if !field.is_active() {
            return refused(outcome, LinkPathFailure::RetiredField(*id));
        }
        if !matches!(field.extent, FieldExtent::Global) {
            return refused(outcome, LinkPathFailure::UnsupportedExtent(*id));
        }
        let FieldDefinition::Vector(vector) = &field.definition else {
            return refused(outcome, LinkPathFailure::ScalarField(*id));
        };
        selected_vectors.push(vector);
    }
    // Sample references are dependencies, not implicit global inputs. Validate
    // them with Numen's AST limits before building its registry.
    let dependency_ids = if settings.field_response {
        let mut roots = selected_vectors.clone();
        if let Some(vector) = settings.view_field.as_deref() {
            roots.push(vector);
        }
        match fields::validate_dependencies(&canvas.graph, &roots) {
            Ok(ids) => ids,
            Err(failure) => return refused(outcome, failure),
        }
    } else {
        Vec::new()
    };
    if settings.field_response && !settings.field_strength.is_finite() {
        return refused(outcome, LinkPathFailure::InvalidFieldStrength);
    }
    let mut registry = FieldRegistry::new();
    for id in dependency_ids {
        if let Some(field) = canvas.graph.field(id).filter(|field| field.is_active()) {
            registry.insert_with_id(id, field.definition.clone());
        }
    }
    let mut fields: Vec<PathField<'_>> = selected_vectors
        .iter()
        .map(|field| PathField {
            field,
            strength: settings.field_strength,
        })
        .collect();
    if let Some(field) = settings.view_field.as_deref() {
        fields.push(PathField {
            field,
            strength: settings.field_strength,
        });
    }

    let mut compat = Vec::new();
    if settings.compatible_attraction {
        let current_keys = canonical_keys(canvas);
        for key in &settings.compatible_links {
            if !current_keys.contains(key) {
                return refused(outcome, LinkPathFailure::StaleCompatibleLink(key.clone()));
            }
        }
        for (reading_index, reading) in eligible_readings.iter().enumerate() {
            for link in &reading.links {
                if !settings.compatible_links.contains(&link.key) {
                    continue;
                }
                let Some(bucket) = link.buckets.first() else {
                    continue;
                };
                compat.push(AppearanceCompatibility {
                    reading_index,
                    link_key: &link.key,
                    displayed_first: &bucket.displayed_from,
                    displayed_second: &bucket.displayed_to,
                    compatibility: "view:explicit-compatible-links",
                });
            }
        }
    }
    if settings.field_response && fields.is_empty() {
        return refused(
            outcome,
            LinkPathFailure::Adapter(
                "field response is enabled without a selected graph field or view field".into(),
            ),
        );
    }

    let batch = match evaluate_link_appearances(
        &eligible_readings,
        &compat,
        |id| {
            let uuid = id.parse::<uuid::Uuid>().ok()?;
            let key = canvas.graph.get_node_key_by_id(uuid)?;
            canvas.view.position_of(key).map(|p| (p.x, p.y))
        },
        &fields,
        &registry,
        ResponsivePathConfig {
            field_response: settings.field_response,
            compatible_attraction: settings.compatible_attraction,
            ..settings.config
        },
    ) {
        Ok(batch) => batch,
        Err(error) => return refused(outcome, LinkPathFailure::Adapter(format!("{error:?}"))),
    };

    // Numen preserves input order and the adapter creates these handles together.
    // Resolve each appearance once and move its final geometry into the segment
    // used by paint, picking and (only when requested) the source report.
    let segment_index: HashMap<_, _> = segments
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            (
                (
                    segment.appearance.key.clone(),
                    segment.appearance.display_from,
                    segment.appearance.display_to,
                ),
                index,
            )
        })
        .collect();
    for (appearance, mut path) in batch.appearances.iter().zip(batch.paths) {
        debug_assert_eq!(appearance.evaluator_id, path.id);
        let (Ok(first_id), Ok(second_id)) = (
            appearance.displayed_start.parse::<uuid::Uuid>(),
            appearance.displayed_end.parse::<uuid::Uuid>(),
        ) else {
            continue;
        };
        let (Some(first), Some(second)) = (
            canvas.graph.get_node_key_by_id(first_id),
            canvas.graph.get_node_key_by_id(second_id),
        ) else {
            continue;
        };
        let pair = if first <= second {
            (first, second)
        } else {
            (second, first)
        };
        let Some(&index) = segment_index.get(&(appearance.link_key.clone(), pair.0, pair.1)) else {
            continue;
        };
        if first != pair.0 {
            path.points.reverse();
        }
        let segment = &mut segments[index];
        let bodies = canvas
            .node_billboard(pair.0)
            .zip(canvas.node_billboard(pair.1));
        if let Some(points) = bodies.and_then(|(from, to)| {
            geometry::responsive_path_geometry(&path.points, segment, canvas.camera, from, to)
        }) {
            segment.from = points[0];
            segment.to = points[points.len() - 1];
            segment.points = points;
            segment.responsive = true;
        } else {
            outcome.presentation_fallbacks += 1;
        }
    }
    outcome.state = LinkPathState::Applied;
    outcome
}

fn refused(mut outcome: EvaluationOutcome, failure: LinkPathFailure) -> EvaluationOutcome {
    outcome.state = LinkPathState::Refused;
    outcome.failure = Some(failure);
    outcome
}

fn canonical_keys(canvas: &Canvas) -> std::collections::BTreeSet<LinkKey> {
    let source = LinkSource::Unavailable {
        reason: "Pictograph graph read for compatibility identity validation".into(),
    };
    canvas
        .graph
        .projected_relations()
        .filter_map(|(_, relation)| {
            let (a, b) = (
                canvas.graph.get_node(relation.from)?,
                canvas.graph.get_node(relation.to)?,
            );
            super::link_reading::read_link(&canvas.graph, a.id, b.id, source.clone()).ok()
        })
        .flat_map(|pair| pair.links.into_iter().map(|link| link.key))
        .collect()
}

#[cfg(test)]
#[path = "responsive_links_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "responsive_links_controls_tests.rs"]
mod controls_tests;
