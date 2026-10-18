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

#[derive(Clone, Debug)]
pub struct LinkPathResponseReport {
    pub state: LinkPathState,
    pub failure: Option<LinkPathFailure>,
    pub settings: LinkPathResponse,
    pub paths: Vec<EvaluatedLinkPath>,
    pub responsive_paths: usize,
    pub ineligible_self_links: usize,
    pub excluded_endpoints: usize,
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
        let (segments, readings) = edge_cells::visible_link_geometry_with_assertion_hides(
            &self.graph,
            &self.view,
            &self.hidden_links,
            &self.hidden_edges,
            &self.hidden_assertions,
            |key| self.node_size(key),
        );
        evaluate_canvas(self, &segments, &readings).0
    }

    /// Current evaluated paths for host inspection. This always reads the
    /// current graph, view positions, and hide state; no paths are cached.
    pub fn visible_link_paths(&self) -> Vec<EvaluatedLinkPath> {
        self.link_path_response_report().paths
    }

    pub(crate) fn rendered_link_segments(&self) -> Vec<edge_cells::LinkAppearanceSegment> {
        if !self.link_path_response.field_response && !self.link_path_response.compatible_attraction
        {
            return edge_cells::visible_link_segments_with_assertion_hides(
                &self.graph,
                &self.view,
                &self.hidden_links,
                &self.hidden_edges,
                &self.hidden_assertions,
                |key| self.node_size(key),
            );
        }
        let (mut segments, readings) = edge_cells::visible_link_geometry_with_assertion_hides(
            &self.graph,
            &self.view,
            &self.hidden_links,
            &self.hidden_edges,
            &self.hidden_assertions,
            |key| self.node_size(key),
        );
        let (_, paths) = evaluate_canvas(self, &segments, &readings);
        for segment in &mut segments {
            let key = (
                segment.appearance.key.clone(),
                segment.appearance.display_from,
                segment.appearance.display_to,
            );
            if let Some(points) = paths.get(&key) {
                segment.points = points
                    .iter()
                    .map(|&(x, y)| euclid::default::Point2D::new(x, y))
                    .collect();
                if let Some(first) = segment.points.first() {
                    segment.from = *first;
                }
                if let Some(last) = segment.points.last() {
                    segment.to = *last;
                }
            }
        }
        segments
    }
}

pub(crate) type AppearancePathMap = HashMap<(LinkKey, NodeKey, NodeKey), Vec<(f32, f32)>>;

pub(crate) fn evaluate_canvas(
    canvas: &Canvas,
    segments: &[edge_cells::LinkAppearanceSegment],
    readings: &[LinkPairReading],
) -> (LinkPathResponseReport, AppearancePathMap) {
    let settings = canvas.link_path_response.clone();
    let mut report = LinkPathResponseReport {
        state: LinkPathState::Disabled,
        failure: None,
        settings: settings.clone(),
        paths: report::base_paths(canvas, segments, readings),
        responsive_paths: 0,
        ineligible_self_links: segments
            .iter()
            .filter(|segment| segment.appearance.display_from == segment.appearance.display_to)
            .count(),
        excluded_endpoints: 0,
    };
    let mut path_map = HashMap::new();
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
    report.excluded_endpoints = segments
        .iter()
        .filter(|segment| {
            !node_eligible(segment.appearance.display_from)
                || !node_eligible(segment.appearance.display_to)
        })
        .count();
    if !settings.field_response && !settings.compatible_attraction {
        return (report, path_map);
    }

    let endpoint_visible = |id: &str| {
        id.parse::<uuid::Uuid>()
            .ok()
            .and_then(|id| canvas.graph.get_node_key_by_id(id))
            .is_some_and(node_eligible)
    };
    let mut eligible_readings = readings.to_vec();
    for reading in &mut eligible_readings {
        reading.links.retain(|link| {
            let Some(bucket) = link.buckets.first() else {
                return false;
            };
            if bucket.displayed_from == bucket.displayed_to {
                return false;
            }
            let endpoints = [
                bucket.displayed_from.parse::<uuid::Uuid>(),
                bucket.displayed_to.parse::<uuid::Uuid>(),
            ];
            let visible = endpoints.iter().all(|id| {
                id.as_ref()
                    .is_ok_and(|id| endpoint_visible(&id.to_string()))
            });
            visible
        });
    }
    let eligible_segments: Vec<_> = segments
        .iter()
        .filter(|segment| {
            canvas
                .graph
                .get_node(segment.appearance.display_from)
                .is_some_and(|node| endpoint_visible(&node.id.to_string()))
                && canvas
                    .graph
                    .get_node(segment.appearance.display_to)
                    .is_some_and(|node| endpoint_visible(&node.id.to_string()))
        })
        .collect();
    report.ineligible_self_links = segments
        .iter()
        .filter(|segment| segment.appearance.display_from == segment.appearance.display_to)
        .count();
    let mut selected_vectors = Vec::new();
    for id in settings
        .selected_fields
        .iter()
        .filter(|_| settings.field_response)
    {
        let Some(field) = canvas.graph.field(*id) else {
            return refused(report, LinkPathFailure::MissingField(*id));
        };
        if !field.is_active() {
            return refused(report, LinkPathFailure::RetiredField(*id));
        }
        if !matches!(field.extent, FieldExtent::Global) {
            return refused(report, LinkPathFailure::UnsupportedExtent(*id));
        }
        let FieldDefinition::Vector(vector) = &field.definition else {
            return refused(report, LinkPathFailure::ScalarField(*id));
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
            Err(failure) => return refused(report, failure),
        }
    } else {
        Vec::new()
    };
    if settings.field_response && !settings.field_strength.is_finite() {
        return refused(report, LinkPathFailure::InvalidFieldStrength);
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
                return refused(report, LinkPathFailure::StaleCompatibleLink(key.clone()));
            }
        }
        if !settings.field_response {
            for reading in &mut eligible_readings {
                reading
                    .links
                    .retain(|link| settings.compatible_links.contains(&link.key));
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
            report,
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
        Err(error) => return refused(report, LinkPathFailure::Adapter(format!("{error:?}"))),
    };

    let mut responsive_by_key = HashMap::new();
    for appearance in &batch.appearances {
        if let Some(path) = batch
            .paths
            .iter()
            .find(|path| path.id == appearance.evaluator_id)
        {
            responsive_by_key.insert(
                (appearance.reading_index, appearance.link_index),
                path.points.clone(),
            );
        }
    }
    let segment_index: HashMap<_, _> = eligible_segments
        .iter()
        .map(|s| {
            (
                (
                    s.appearance.key.clone(),
                    s.appearance.display_from,
                    s.appearance.display_to,
                ),
                *s,
            )
        })
        .collect();
    for appearance in &batch.appearances {
        let Some(points) =
            responsive_by_key.get(&(appearance.reading_index, appearance.link_index))
        else {
            continue;
        };
        let Some(reading) = eligible_readings.get(appearance.reading_index) else {
            continue;
        };
        let Some(link) = reading.links.get(appearance.link_index) else {
            continue;
        };
        let Ok(first_id) = appearance.displayed_start.parse::<uuid::Uuid>() else {
            continue;
        };
        let Ok(second_id) = appearance.displayed_end.parse::<uuid::Uuid>() else {
            continue;
        };
        let (Some(first), Some(second)) = (
            canvas.graph.get_node_key_by_id(first_id),
            canvas.graph.get_node_key_by_id(second_id),
        ) else {
            continue;
        };
        let (Some(first_node), Some(second_node)) =
            (canvas.graph.get_node(first), canvas.graph.get_node(second))
        else {
            continue;
        };
        if first == second {
            continue;
        }
        let pair_key = if first <= second {
            (first, second)
        } else {
            (second, first)
        };
        let Some(segment) = segment_index.get(&(link.key.clone(), pair_key.0, pair_key.1)) else {
            continue;
        };
        let (route_from, route_to) = if first == pair_key.0 {
            (first_node, second_node)
        } else {
            (second_node, first_node)
        };
        let oriented: Vec<_> = if first == pair_key.0 {
            points.clone()
        } else {
            points.iter().rev().copied().collect()
        };
        let shifted = offset_and_trim(
            &oriented,
            segment,
            canvas.node_size(pair_key.0),
            canvas.node_size(pair_key.1),
        );
        path_map.insert((link.key.clone(), pair_key.0, pair_key.1), shifted.clone());
        if let Some(path) = report.paths.iter_mut().find(|path| {
            path.key == link.key
                && path.displayed_from == route_from.id
                && path.displayed_to == route_to.id
        }) {
            path.points = shifted;
            path.responsive = true;
            report.responsive_paths += 1;
        }
    }
    report.state = LinkPathState::Applied;
    (report, path_map)
}

fn refused(
    mut report: LinkPathResponseReport,
    failure: LinkPathFailure,
) -> (LinkPathResponseReport, AppearancePathMap) {
    report.state = LinkPathState::Refused;
    report.failure = Some(failure);
    (report, HashMap::new())
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

fn offset_and_trim(
    points: &[(f32, f32)],
    segment: &edge_cells::LinkAppearanceSegment,
    from_size: f32,
    to_size: f32,
) -> Vec<(f32, f32)> {
    if points.len() < 2 {
        return segment.points.iter().map(|p| (p.x, p.y)).collect();
    }
    let dx = points[points.len() - 1].0 - points[0].0;
    let dy = points[points.len() - 1].1 - points[0].1;
    let len = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    let mut out = points.to_vec();
    // Retain the existing stratum separation by translating every sample by
    // the resting segment's endpoint offset from the unshifted chord.
    let ox = segment.from.x - points[0].0;
    let oy = segment.from.y - points[0].1;
    for point in &mut out {
        point.0 += ox;
        point.1 += oy;
    }
    let ux = dx / len;
    let uy = dy / len;
    out[0].0 += ux * from_size.max(0.0) * 0.5;
    out[0].1 += uy * from_size.max(0.0) * 0.5;
    let last = out.len() - 1;
    out[last].0 -= ux * to_size.max(0.0) * 0.5;
    out[last].1 -= uy * to_size.max(0.0) * 0.5;
    out
}

#[cfg(test)]
#[path = "responsive_links_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "responsive_links_controls_tests.rs"]
mod controls_tests;
