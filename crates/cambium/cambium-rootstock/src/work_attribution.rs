// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Per-call wall/CPU attribution. A non-CPU residual is not driver/GPU time.

use crate::{Duration, Instant};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct NativeWorkSpan {
    pub wall_ns: u64,
    pub thread_cpu_ns: Option<u64>,
    pub non_cpu_residual_ns: Option<u64>,
    pub rejected_thread_cpu_ns: Option<u64>,
}
impl NativeWorkSpan {
    pub fn from_nanos(wall_ns: u64, thread_cpu_ns: Option<u64>) -> Self {
        let rejected_thread_cpu_ns = thread_cpu_ns.filter(|cpu| *cpu > wall_ns);
        let thread_cpu_ns = thread_cpu_ns.filter(|cpu| *cpu <= wall_ns);
        Self {
            wall_ns,
            thread_cpu_ns,
            non_cpu_residual_ns: thread_cpu_ns.and_then(|cpu| wall_ns.checked_sub(cpu)),
            rejected_thread_cpu_ns,
        }
    }
    pub fn from_durations(wall: Duration, thread_cpu: Option<Duration>) -> Option<Self> {
        Some(Self::from_nanos(
            wall.as_nanos().try_into().ok()?,
            thread_cpu.and_then(|cpu| cpu.as_nanos().try_into().ok()),
        ))
    }
    pub(crate) fn remainder(
        self,
        children: impl IntoIterator<Item = Option<Self>>,
    ) -> Option<Self> {
        let mut wall = 0_u64;
        let mut cpu = Some(0_u64);
        for child in children.into_iter().flatten() {
            wall = wall.checked_add(child.wall_ns)?;
            cpu = cpu
                .zip(child.thread_cpu_ns)
                .and_then(|(a, b)| a.checked_add(b));
        }
        Some(Self::from_nanos(
            self.wall_ns.checked_sub(wall)?,
            self.thread_cpu_ns
                .zip(cpu)
                .and_then(|(a, b)| a.checked_sub(b)),
        ))
    }
}

pub(crate) struct WorkTimer {
    wall: Instant,
    #[cfg(not(target_arch = "wasm32"))]
    cpu: genet_render_host::NativeThreadCpuStamp,
}
impl WorkTimer {
    pub(crate) fn start() -> Self {
        // Bracket CPU sampling overhead within the wall observation.
        let wall = Instant::now();
        Self {
            wall,
            #[cfg(not(target_arch = "wasm32"))]
            cpu: genet_render_host::NativeThreadCpuStamp::now(),
        }
    }
    pub(crate) fn finish(self) -> Option<NativeWorkSpan> {
        #[cfg(not(target_arch = "wasm32"))]
        let cpu = genet_render_host::NativeThreadCpuStamp::now().elapsed_since(self.cpu);
        #[cfg(target_arch = "wasm32")]
        let cpu = None;
        let wall = Instant::now().checked_duration_since(self.wall)?;
        NativeWorkSpan::from_durations(wall, cpu)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct NativePreparationWork {
    pub total: Option<NativeWorkSpan>,
    pub validation: Option<NativeWorkSpan>,
    pub application_hook: Option<NativeWorkSpan>,
    pub frame_setup: Option<NativeWorkSpan>,
    pub model_layout: Option<NativeWorkSpan>,
    pub producers: Option<NativeWorkSpan>,
    pub leaf_fragments: Option<NativeWorkSpan>,
    pub ime: Option<NativeWorkSpan>,
    pub emit_scene: Option<NativeWorkSpan>,
    pub packet_binding: Option<NativeWorkSpan>,
    pub unscoped: Option<NativeWorkSpan>,
}
impl NativePreparationWork {
    pub(crate) fn finish(&mut self, total: Option<NativeWorkSpan>) {
        self.total = total;
        self.unscoped = total.and_then(|total| {
            total.remainder([
                self.validation,
                self.application_hook,
                self.frame_setup,
                self.model_layout,
                self.producers,
                self.leaf_fragments,
                self.ime,
                self.emit_scene,
                self.packet_binding,
            ])
        });
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct NativeWorkPacket {
    pub host: u64,
    pub generation: u64,
    pub batch_id: u64,
    pub proposed_sequence: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeWorkCallKind {
    Submit,
    Collect,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct NativeWorkCall {
    pub kind: NativeWorkCallKind,
    pub span: NativeWorkSpan,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct NativeGpuWork {
    pub batch_id: u64,
    pub selected_adapter_supported: Option<bool>,
    pub device_enabled: Option<bool>,
    pub status: Option<&'static str>,
    pub scope: Option<&'static str>,
    pub recordings: u64,
    pub completed: u64,
    pub pending: u64,
    pub failed: u64,
    pub first_recording_id: Option<u64>,
    pub last_recording_id: Option<u64>,
    pub timestamp_period_bits: Option<u32>,
    pub recording_commands_ns: Option<u64>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct NativeWorkerWork {
    pub batch_id: u64,
    pub core_generation: u64,
    pub total: Option<NativeWorkSpan>,
    pub poll: Option<NativeWorkSpan>,
    pub wait: Option<NativeWorkSpan>,
    pub scopes: Option<NativeWorkSpan>,
    pub quality_map: Option<NativeWorkSpan>,
    pub quality_lock: Option<NativeWorkSpan>,
    pub gpu_readback: Option<NativeWorkSpan>,
    pub yielded: Option<NativeWorkSpan>,
    pub other: Option<NativeWorkSpan>,
    /// Overlaps poll/map; this is not another partition component.
    pub quality_total: Option<NativeWorkSpan>,
    /// Overlaps wait; this is not another partition component.
    pub scope_wait: Option<NativeWorkSpan>,
    /// Request to callback wall readiness, not a cross-thread CPU observation.
    pub map_readiness_ns: Option<u64>,
    pub poll_calls: u64,
    pub wait_calls: u64,
    pub scope_polls: u64,
    pub quality_polls: u64,
    pub quality_map_calls: u64,
    pub quality_lock_calls: u64,
    pub gpu_readback_calls: u64,
    pub yield_calls: u64,
    pub scope_wait_calls: u64,
    pub mapped_buffers: u64,
    pub mapped_bytes: u64,
    pub map_callbacks: u64,
    pub cpu_partition_rejected: bool,
    pub gpu: NativeGpuWork,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct NativeWorkAttribution {
    pub packet: Option<NativeWorkPacket>,
    /// Only this callback's UI work; never a CPU delta across worker latency.
    pub current_call: Option<NativeWorkCall>,
    pub submission_call: Option<NativeWorkSpan>,
    pub preparation: Option<NativePreparationWork>,
    pub observer_capture: Option<NativeWorkSpan>,
    pub input_capture: Option<NativeWorkSpan>,
    pub encode: Option<NativeWorkSpan>,
    pub seal: Option<NativeWorkSpan>,
    pub submission_unscoped: Option<NativeWorkSpan>,
    pub collection_call: Option<NativeWorkSpan>,
    pub acquire: Option<NativeWorkSpan>,
    pub clear: Option<NativeWorkSpan>,
    pub compose: Option<NativeWorkSpan>,
    pub present: Option<NativeWorkSpan>,
    pub collection_unscoped: Option<NativeWorkSpan>,
    pub worker: Option<NativeWorkerWork>,
    pub worker_rejected: Option<&'static str>,
}
impl NativeWorkAttribution {
    pub fn available(&self) -> bool {
        self.current_call.is_some() || self.packet.is_some()
    }
    pub(crate) fn attach_worker(&mut self, batch_id: u64, worker: NativeWorkerWork) {
        if self.packet.is_none_or(|packet| {
            packet.batch_id != batch_id
                || packet.batch_id != worker.batch_id
                || packet.generation != worker.core_generation
                || worker.gpu.batch_id != packet.batch_id
        }) {
            self.worker_rejected = Some("batch_or_generation_mismatch");
            self.worker = None;
        } else {
            self.worker = Some(worker);
            self.worker_rejected = None;
        }
    }
    pub fn summary(&self) -> String {
        format!(
            "batch={:?} call={:?} wall-ns={:?} cpu-ns={:?} residual-ns={:?} worker-wall-ns={:?} worker-cpu-ns={:?} gpu-ns={:?}",
            self.packet.map(|p| (p.host, p.generation, p.batch_id)),
            self.current_call.map(|c| c.kind),
            self.current_call.map(|c| c.span.wall_ns),
            self.current_call.and_then(|c| c.span.thread_cpu_ns),
            self.current_call.and_then(|c| c.span.non_cpu_residual_ns),
            self.worker.and_then(|w| w.total).map(|s| s.wall_ns),
            self.worker
                .and_then(|w| w.total)
                .and_then(|s| s.thread_cpu_ns),
            self.worker.and_then(|w| w.gpu.recording_commands_ns)
        )
    }
    pub(crate) fn finish_call(&mut self, kind: NativeWorkCallKind, span: Option<NativeWorkSpan>) {
        self.current_call = span.map(|span| NativeWorkCall { kind, span });
        match kind {
            NativeWorkCallKind::Submit => {
                self.submission_call = span;
                self.submission_unscoped = span.and_then(|total| {
                    total.remainder([
                        self.preparation.and_then(|p| p.total),
                        self.observer_capture,
                        self.input_capture,
                        self.encode,
                        self.seal,
                    ])
                });
            },
            NativeWorkCallKind::Collect => {
                self.collection_call = span;
                self.collection_unscoped = span.and_then(|total| {
                    total.remainder([self.acquire, self.clear, self.compose, self.present])
                });
            },
        }
    }
}

#[cfg(test)]
mod tests;
