// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Bounded native-host CPU wall-time receipts. No GPU-duration claims.
//!
//! Product-neutral: the per-frame totals, eligibility and populated flags
//! arrive as a [`CostObservation`] the product computes from its own scenes.
use cambium_rootstock::FrameProfile;
use serde::Serialize;
use std::collections::BTreeMap;
const MAX_SAMPLES: usize = 4096;
const MAX_PHASES: usize = 32;

#[derive(Clone, Copy, Default)]
pub struct Totals {
    pub redraws: u64,
    pub mesh_bytes: u64,
    pub instance_bytes: u64,
}
/// One frame's product-side reading for the cost receipt.
#[derive(Clone, Copy)]
pub struct CostObservation {
    /// Cumulative redraw and upload counters across every scene the product owns.
    pub totals: Totals,
    /// Whether no active producer reported an error this frame.
    pub valid: bool,
    /// Whether any scene drew body geometry this frame.
    pub populated: bool,
}

impl Default for CostObservation {
    /// An eligible, empty, unpopulated frame: a product that reports nothing
    /// still gets honest frame timings rather than a receipt of invalid frames.
    fn default() -> Self {
        Self {
            totals: Totals::default(),
            valid: true,
            populated: false,
        }
    }
}

#[derive(Clone, Serialize)]
struct Sample {
    attempt: u64,
    presented: bool,
    native_pending: bool,
    native_timing_available: bool,
    native_work: Option<cambium_rootstock::NativeWorkAttribution>,
    /// Immutable product snapshot, taken outside the timed producer stages.
    #[serde(skip_serializing_if = "Option::is_none")]
    product_work: Option<serde_json::Value>,
    frame: u64,
    capture: bool,
    valid: bool,
    times_us: BTreeMap<&'static str, u64>,
    counts: BTreeMap<&'static str, u64>,
}
#[derive(Clone, Serialize)]
struct ProductWorkSnapshot {
    attempt: u64,
    product_work: Option<serde_json::Value>,
}
#[derive(Default)]
pub struct Costs {
    previous: Totals,
    previous_product_work: Option<ProductWorkSnapshot>,
    attempts: u64,
    native_pending_attempts: u64,
    unpresented_attempts: u64,
    ui_thread_us: u64,
    first_document: Option<Sample>,
    first_populated: Option<Sample>,
    active: Option<Phase>,
    phases: Vec<Phase>,
    retained: usize,
    failed: bool,
}
struct Phase {
    name: String,
    samples: Vec<Sample>,
    product_work_before_phase: Option<ProductWorkSnapshot>,
}
#[derive(Debug, Serialize)]
struct Distribution {
    count: usize,
    median: f64,
    p95: u64,
    max: u64,
}
#[derive(Serialize)]
struct PhaseReport<'a> {
    name: &'a str,
    completed: bool,
    samples: &'a [Sample],
    product_work_before_phase: Option<&'a ProductWorkSnapshot>,
    eligible_frames: usize,
    pending_attempts: usize,
    unpresented_attempts: usize,
    observed_ui_thread_us: u64,
    capture_frames: usize,
    invalid_frames: usize,
    native_timing_frames: usize,
    first_frame: Option<&'a Sample>,
    all_frames_us: BTreeMap<&'static str, Distribution>,
    steady_after_first_us: BTreeMap<&'static str, Distribution>,
    eligible_counter_totals: BTreeMap<&'static str, u64>,
    all_attempt_counter_totals: BTreeMap<&'static str, u64>,
}
fn distribution(values: impl IntoIterator<Item = u64>) -> Option<Distribution> {
    let mut values: Vec<_> = values.into_iter().collect();
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let count = values.len();
    let median = if count % 2 == 0 {
        values[count / 2 - 1] as f64 / 2. + values[count / 2] as f64 / 2.
    } else {
        values[count / 2] as f64
    };
    Some(Distribution {
        count,
        median,
        p95: values[(count * 95).div_ceil(100) - 1],
        max: values[count - 1],
    })
}
impl Phase {
    fn report(&self, completed: bool) -> PhaseReport<'_> {
        let eligible: Vec<_> = self
            .samples
            .iter()
            .filter(|s| s.valid && s.presented && !s.native_pending && !s.capture)
            .collect();
        let mut all = BTreeMap::new();
        let mut steady = BTreeMap::new();
        let mut counters = BTreeMap::<_, u64>::new();
        if let Some(first) = eligible.first() {
            for &key in first.times_us.keys() {
                let measured =
                    |s: &&Sample| !native_partition_key(key) || s.native_timing_available;
                if let Some(d) = distribution(
                    eligible
                        .iter()
                        .copied()
                        .filter(measured)
                        .map(|s| s.times_us[key]),
                ) {
                    all.insert(key, d);
                }
                if let Some(d) = distribution(
                    eligible
                        .iter()
                        .skip(1)
                        .copied()
                        .filter(measured)
                        .map(|s| s.times_us[key]),
                ) {
                    steady.insert(key, d);
                }
            }
        }
        for sample in &eligible {
            for (&key, &value) in &sample.counts {
                let n = counters.entry(key).or_default();
                *n = n.saturating_add(value);
            }
        }
        let mut all_counters = BTreeMap::<_, u64>::new();
        for sample in &self.samples {
            for (&key, &value) in &sample.counts {
                // Host structural counters can be repeated on the final
                // accepted profile. Only product cumulative deltas are unique.
                if ![
                    "actual_scene_redraws",
                    "mesh_upload_bytes",
                    "instance_upload_bytes",
                ]
                .contains(&key)
                {
                    continue;
                }
                let count = all_counters.entry(key).or_default();
                *count = count.saturating_add(value);
            }
        }
        PhaseReport {
            name: &self.name,
            completed,
            samples: &self.samples,
            product_work_before_phase: self.product_work_before_phase.as_ref(),
            eligible_frames: eligible.len(),
            pending_attempts: self.samples.iter().filter(|s| s.native_pending).count(),
            observed_ui_thread_us: self.samples.iter().fold(0u64, |sum, s| {
                sum.saturating_add(s.times_us["ui_attempt_us"])
            }),
            unpresented_attempts: self.samples.iter().filter(|s| !s.presented).count(),
            capture_frames: self.samples.iter().filter(|s| s.capture).count(),
            invalid_frames: self.samples.iter().filter(|s| !s.valid).count(),
            native_timing_frames: eligible
                .iter()
                .filter(|s| s.native_timing_available)
                .count(),
            first_frame: eligible.first().copied(),
            all_frames_us: all,
            steady_after_first_us: steady,
            eligible_counter_totals: counters,
            all_attempt_counter_totals: all_counters,
        }
    }
}
impl Costs {
    pub fn begin(&mut self, name: &str) -> Result<(), String> {
        if self.failed {
            return Err("cost sample budget was exceeded".into());
        }
        if self.active.is_some() {
            return Err("end the active cost phase first".into());
        }
        if self.phases.len() >= MAX_PHASES {
            return Err("at most 32 cost phases are admitted".into());
        }
        if name.is_empty()
            || name.len() > 64
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("cost phase names require 1..64 ASCII letters, digits, - or _".into());
        }
        if self.phases.iter().any(|p| p.name == name) {
            return Err(format!("duplicate cost phase {name}"));
        }
        self.active = Some(Phase {
            name: name.into(),
            samples: Vec::new(),
            product_work_before_phase: self.previous_product_work.clone(),
        });
        Ok(())
    }
    pub fn end(&mut self) -> Result<(), String> {
        let phase = self.active.take().ok_or("no cost phase is active")?;
        let valid = phase
            .samples
            .iter()
            .any(|s| s.valid && s.presented && !s.native_pending && !s.capture);
        self.phases.push(phase);
        if !valid {
            return Err("cost phase has no eligible frame profiles; use settle between cost-begin and cost-end".into());
        }
        Ok(())
    }
    pub fn finish(&self) -> Result<(), String> {
        if self.active.is_some() {
            Err("cost phase was not ended".into())
        } else {
            Ok(())
        }
    }
    pub fn observe(
        &mut self,
        frame: u64,
        profile: Option<FrameProfile>,
        totals: Totals,
        capture: bool,
        valid: bool,
        populated: bool,
    ) -> Result<(), String> {
        self.observe_attempt(frame, profile, totals, capture, valid, populated, true)
    }
    /// Every redraw attempt, including submissions and polls that have not
    /// published an image. Only accepted, non-pending samples are eligible.
    pub fn observe_attempt(
        &mut self,
        frame: u64,
        profile: Option<FrameProfile>,
        totals: Totals,
        capture: bool,
        valid: bool,
        populated: bool,
        presented: bool,
    ) -> Result<(), String> {
        self.observe_attempt_with_details(
            frame, profile, totals, capture, valid, populated, presented, None,
        )
    }
    /// As `observe_attempt`, preserving optional product-owned scalar readings.
    /// Cumulative fields are not added here: their epochs and meaning belong
    /// to the product. Snapshot acquisition is outside the UI stage clocks.
    pub fn observe_attempt_with_details(
        &mut self,
        frame: u64,
        profile: Option<FrameProfile>,
        totals: Totals,
        capture: bool,
        valid: bool,
        populated: bool,
        presented: bool,
        product_work: Option<serde_json::Value>,
    ) -> Result<(), String> {
        self.attempts = self.attempts.saturating_add(1);
        self.previous_product_work = Some(ProductWorkSnapshot {
            attempt: self.attempts,
            product_work: product_work.clone(),
        });
        self.native_pending_attempts += u64::from(profile.is_some_and(|p| p.native_pending));
        self.unpresented_attempts += u64::from(!presented);
        let delta = Totals {
            redraws: totals.redraws.saturating_sub(self.previous.redraws),
            mesh_bytes: totals.mesh_bytes.saturating_sub(self.previous.mesh_bytes),
            instance_bytes: totals
                .instance_bytes
                .saturating_sub(self.previous.instance_bytes),
        };
        self.previous = totals;
        if self.failed {
            return Ok(());
        }
        let mut sample = sample(
            self.attempts,
            frame,
            profile,
            delta,
            capture,
            valid,
            presented,
        );
        sample.product_work = product_work;
        self.ui_thread_us = self
            .ui_thread_us
            .saturating_add(sample.times_us["ui_attempt_us"]);
        if self.first_document.is_none() && sample.presented && !sample.native_pending {
            self.first_document = Some(sample.clone());
        }
        if self.first_populated.is_none()
            && populated
            && totals.redraws > 0
            && sample.valid
            && sample.presented
            && !sample.native_pending
        {
            self.first_populated = Some(sample.clone());
        }
        if let Some(phase) = &mut self.active {
            if self.retained >= MAX_SAMPLES {
                self.failed = true;
                return Err(
                    "cost receipt exceeds 4096 samples; split the run into bounded phases".into(),
                );
            }
            phase.samples.push(sample);
            self.retained += 1;
        }
        Ok(())
    }
    pub fn report(&self) -> serde_json::Value {
        serde_json::json!({
            "schema":1,"timing":"Microseconds: UI stages are host-thread wall time; native queue/worker/ready/latency are wall intervals, not GPU durations",
            "scope":"every scene the product owns; actual redraw and upload deltas distinguish retained producer calls",
            "eligibility":"profile available, accepted publication or explicit windowless test turn, not native-pending, no active producer error, no capture pending; publication does not prove physical display or GPU duration",
            "summary_policy":"capture, invalid, unpresented and native-pending attempts excluded from eligible timings; raw samples and all_attempt_counter_totals retain their work; steady excludes first eligible; p95 nearest rank; overlapping stages must not be summed; native_latency_us is wall latency, not UI CPU or GPU duration",
            "first_frame_policy":"first_document is the first accepted publication or explicit windowless turn; first_populated is the first such populated frame after recorded scene work; neither measures process startup",
            "observed_ui_thread_us":self.ui_thread_us,
            "attempt_timing_policy":"pending native total_us is submission or collection UI work; accepted native UI attempt is native_collect_us plus a11y_us since total_us repeats submit; legacy uses total_us; missing profiles contribute zero and remain invalid; all_attempt_counter_totals contains only unique cumulative product deltas",
            "native_partition_policy":"native_preseal_us+native_queue_us+native_worker_us+native_ready_to_collect_us+native_postcollect_us=native_timeline_us=native_latency_us at rounded cumulative boundaries, before host A11y; measured only when native_timing_available; queue/worker/ready intervals are wall time, not UI CPU or GPU durations; ready_to_collect includes notification/dispatch/scheduling; other stage spans overlap this partition",
            "native_work_policy":"Raw nanoseconds on the identified host/generation/batch/proposed sequence. current_call is only this submit/collect UI call, never CPU across pending latency; saved submission components repeated on collection are provenance and must not be counted twice. Envelopes overlap their children; unscoped remainders are checked. CPU sampling is inside wall boundaries; null means unavailable/not executed, measured zero remains zero, rejected CPU counters are retained separately. wall-minus-CPU includes scheduling/descheduling/locks/possible driver waits and is not driver or GPU time. Worker quality_total and map readiness overlap the base worker partition. GPU duration covers recording commands/barriers/stalls, not pure shader time. Old R5 timing and AT-only-turn exclusions are unchanged.",
            "product_work_policy":"Optional immutable bounded scalar product snapshots acquired in AfterFrame outside producer/UI stage clocks; their acquisition/serialization overhead is unprofiled here. Repeated cumulative readings are not additional work; use product epochs and monotonic deltas. product_work_before_phase retains the immediate preceding actual attempt, including an unavailable payload, so initial deltas need not discard first-sample work. Missing payload is unavailable, never measured zero; nested spans must not be added twice. Eligibility and existing counter/timing summaries are unchanged.",
            "redraw_attempts":self.attempts,"native_pending_attempts":self.native_pending_attempts,"unpresented_attempts":self.unpresented_attempts,
            "limits":{"samples":MAX_SAMPLES,"phases":MAX_PHASES},"budget_exceeded":self.failed,
            "first_document":self.first_document,"first_populated":self.first_populated,
            "phases":self.phases.iter().map(|p|p.report(true)).collect::<Vec<_>>(),
            "unfinished":self.active.as_ref().map(|p|p.report(false)),
        })
    }
}
fn sample(
    attempt: u64,
    frame: u64,
    profile: Option<FrameProfile>,
    delta: Totals,
    capture: bool,
    valid: bool,
    presented: bool,
) -> Sample {
    let available = profile.is_some();
    let p = profile.unwrap_or_default();
    let native_timing =
        p.native_submit_us != 0 || p.native_collect_us != 0 || p.native_latency_us != 0;
    let ui_attempt_us = if !p.native_pending && native_timing {
        p.native_collect_us.saturating_add(p.a11y_us)
    } else {
        p.total_us
    };
    let mut times = BTreeMap::from([("ui_attempt_us", ui_attempt_us)]);
    macro_rules! timings {($($field:ident),*)=>{$(times.insert(stringify!($field),p.$field);)*};}
    timings!(
        total_us,
        native_submit_us,
        native_collect_us,
        native_latency_us,
        native_preseal_us,
        native_queue_us,
        native_worker_us,
        native_ready_to_collect_us,
        native_postcollect_us,
        native_timeline_us,
        frame_hook_us,
        relayout_us,
        layout_update_us,
        layout_tick_us,
        layout_apply_us,
        layout_rebuild_us,
        style_resolve_us,
        layout_with_text_us,
        content_extent_us,
        leaf_boxes_us,
        leaf_render_us,
        leaf_fragments_us,
        ime_us,
        emit_scene_us,
        shadows_us,
        raster_us,
        acquire_us,
        clear_us,
        compose_us,
        present_us,
        capture_us,
        pointer_us,
        a11y_us,
        a11y_publish_us,
        a11y_actions_us,
        raster_total_us,
        tile_invalidate_us,
        dirty_tile_rebuild_us,
        master_compose_us,
        vello_render_us
    );
    times.insert("producer_render_us", p.producers.render_us);
    times.insert("producer_stage_us", p.producers.stage_us);
    times.insert(
        "excluding_acquire_present_capture_us",
        p.total_us
            .saturating_sub(p.acquire_us)
            .saturating_sub(p.present_us)
            .saturating_sub(p.capture_us),
    );
    let counts = BTreeMap::from([
        ("producer_calls", p.producers.render_calls),
        ("producer_stages", p.producers.stages),
        ("producer_suspensions", p.producers.suspensions),
        ("producer_retirements", p.producers.retirements),
        ("producer_invalid_frames", p.producers.invalid_frames),
        ("actual_scene_redraws", delta.redraws),
        ("mesh_upload_bytes", delta.mesh_bytes),
        ("instance_upload_bytes", delta.instance_bytes),
        ("layout_mutations", p.layout_mutations),
        ("layout_rebuilt", u64::from(p.layout_rebuilt)),
        ("leaf_repaints", p.leaf_repaints),
        ("dirty_tiles", p.dirty_tiles),
        ("raster_reuse_hits", u64::from(p.raster_reused)),
    ]);
    Sample {
        attempt,
        presented,
        native_pending: p.native_pending,
        native_timing_available: p.native_timing_available,
        native_work: p.native_work.available().then_some(p.native_work),
        product_work: None,
        frame,
        capture,
        valid: valid && available && p.producers.invalid_frames == 0,
        times_us: times,
        counts,
    }
}
fn native_partition_key(key: &str) -> bool {
    matches!(
        key,
        "native_preseal_us"
            | "native_queue_us"
            | "native_worker_us"
            | "native_ready_to_collect_us"
            | "native_postcollect_us"
            | "native_timeline_us"
    )
}
#[cfg(test)]
mod product_tests;
#[cfg(test)]
#[path = "cost/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "cost/timing_tests.rs"]
mod timing_tests;
#[cfg(test)]
mod work_tests;
