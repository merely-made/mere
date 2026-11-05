// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Real batches and an unavailable surface; no fabricated swapchain picture.

use super::*;
use crate::NativeWorkCallKind;

#[test]
#[ignore = "actual GPU; serialized per-packet UI/worker attribution control"]
fn preparation_acquisition_and_worker_observations_follow_only_their_own_batch() {
    let mut f = fixture();
    f.core
        .enable_native_worker_diagnostics()
        .expect("Classic diagnostic fixture");
    let mut previous: Option<crate::NativeWorkPacket> = None;
    for _ in 0..2 {
        let outcomes = captures(&mut f.host);
        f.host.redraw();
        let submitted = f.host.s.last_frame_profile.unwrap().native_work;
        let packet = submitted.packet.unwrap();
        assert_eq!(packet.host, f.host.s.presentation_host);
        assert_eq!(packet.generation, f.core.generation());
        assert_eq!(
            submitted.current_call.unwrap().kind,
            NativeWorkCallKind::Submit
        );
        assert!(submitted.submission_call.is_some());
        assert!(
            submitted.worker.is_none(),
            "submission cannot borrow an earlier result"
        );
        assert!(submitted.acquire.is_none());
        let prep = submitted.preparation.unwrap();
        let parts = [
            prep.validation,
            prep.application_hook,
            prep.frame_setup,
            prep.model_layout,
            prep.producers,
            prep.leaf_fragments,
            prep.ime,
            prep.emit_scene,
            prep.packet_binding,
            prep.unscoped,
        ];
        assert_eq!(
            parts.into_iter().flatten().map(|s| s.wall_ns).sum::<u64>(),
            prep.total.unwrap().wall_ns
        );
        if let Some(prior) = previous {
            assert_eq!(
                packet.proposed_sequence, prior.proposed_sequence,
                "unavailable pictures do not consume a presentation sequence"
            );
            assert!(
                packet.batch_id > prior.batch_id,
                "diagnostic identity cannot alias a retry"
            );
        }
        collect_on_wakes(&mut f);
        let collected = f.host.s.last_frame_profile.unwrap().native_work;
        assert_eq!(collected.packet, Some(packet));
        assert_eq!(
            collected.current_call.unwrap().kind,
            NativeWorkCallKind::Collect
        );
        assert_eq!(
            collected.submission_call, submitted.submission_call,
            "the original submission observation survives pending storage"
        );
        let worker = collected
            .worker
            .expect("worker CPU is independent of GPU timestamps");
        assert_eq!(worker.batch_id, packet.batch_id);
        assert_eq!(worker.core_generation, packet.generation);
        assert_eq!(worker.gpu.batch_id, packet.batch_id);
        assert!(worker.total.is_some());
        assert!(collected.worker_rejected.is_none());
        assert!(collected.acquire.is_some());
        assert!(
            collected.clear.is_none() && collected.compose.is_none() && collected.present.is_none()
        );
        let gpu_enabled = f
            .core
            .device()
            .features()
            .contains(genet_render_host::RenderCore::native_worker_diagnostic_features());
        assert_eq!(worker.gpu.device_enabled, Some(gpu_enabled));
        if !gpu_enabled {
            assert!(worker.gpu.recording_commands_ns.is_none());
            assert_eq!(worker.gpu.status, Some("device_unsupported"));
        }
        assert_once(&outcomes);
        assert_unpublished(&f);
        wait_lease_release(&f.core);
        previous = Some(packet);
    }
    assert_eq!(f.acquisitions.get(), 2);
}
