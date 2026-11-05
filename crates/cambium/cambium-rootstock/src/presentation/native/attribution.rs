// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Copy only this accepted batch's observations; never read global last stats.

use crate::{FrameProfile, NativeGpuWork, NativeWorkSpan, NativeWorkerWork};

fn span(s: netrender::DiagnosticWorkSpan) -> Option<NativeWorkSpan> {
    (s.calls > 0)
        .then(|| NativeWorkSpan::from_durations(s.wall, s.thread_cpu))
        .flatten()
}
fn nanos(d: crate::Duration) -> Option<u64> {
    d.as_nanos().try_into().ok()
}

pub(super) fn capture(
    profile: &mut FrameProfile,
    batch_id: u64,
    data: Option<genet_render_host::NativeWorkerAttribution>,
) {
    profile.native_work.worker = None;
    profile.native_work.worker_rejected = None;
    let Some(d) = data else {
        return;
    };
    let status = match d.gpu.status {
        netrender::GpuStatus::Disabled => "disabled",
        netrender::GpuStatus::Unsupported => "device_unsupported",
        netrender::GpuStatus::NoRecording => "no_recording",
        netrender::GpuStatus::Pending => "pending",
        netrender::GpuStatus::Available => "available",
        netrender::GpuStatus::Failed => "failed",
        netrender::GpuStatus::CapacityExceeded => "capacity_exceeded",
        netrender::GpuStatus::HistoryExpired => "history_expired",
    };
    profile.native_work.attach_worker(
        batch_id,
        NativeWorkerWork {
            batch_id: d.batch_id,
            core_generation: d.core_generation,
            total: span(d.total),
            poll: span(d.poll),
            wait: span(d.wait),
            scopes: span(d.scopes),
            quality_map: span(d.quality_map),
            quality_lock: span(d.quality_lock),
            gpu_readback: span(d.gpu_readback),
            yielded: span(d.yielded),
            other: d.other.and_then(span),
            quality_total: span(d.quality_total),
            scope_wait: span(d.scope_wait),
            map_readiness_ns: d.map_ready_wall.and_then(nanos),
            poll_calls: d.poll.calls,
            wait_calls: d.wait.calls,
            scope_polls: d.scopes.calls,
            quality_polls: d.quality_polls,
            quality_map_calls: d.quality_map.calls,
            quality_lock_calls: d.quality_lock.calls,
            gpu_readback_calls: d.gpu_readback.calls,
            yield_calls: d.yielded.calls,
            scope_wait_calls: d.scope_wait.calls,
            mapped_buffers: d.mapped_buffers,
            mapped_bytes: d.mapped_bytes,
            map_callbacks: d.map_callbacks,
            cpu_partition_rejected: d.cpu_partition_rejected,
            gpu: NativeGpuWork {
                batch_id: d.gpu.batch_id,
                selected_adapter_supported: Some(d.gpu_timestamp_supported),
                device_enabled: Some(d.gpu_timestamp_enabled),
                status: Some(status),
                scope: Some(d.gpu.scope),
                recordings: d.gpu.recordings,
                completed: d.gpu.completed,
                pending: d.gpu.pending,
                failed: d.gpu.failed,
                first_recording_id: d.gpu.first_recording_id,
                last_recording_id: d.gpu.last_recording_id,
                timestamp_period_bits: d.gpu.timestamp_period_bits,
                recording_commands_ns: d.gpu.elapsed_ns,
            },
        },
    );
}
