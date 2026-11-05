// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn zero_cpu_and_an_unavailable_clock_do_not_mean_the_same_thing() {
    let zero = NativeWorkSpan::from_nanos(40, Some(0));
    let absent = NativeWorkSpan::from_nanos(40, None);
    assert_eq!(zero.thread_cpu_ns, Some(0));
    assert_eq!(zero.non_cpu_residual_ns, Some(40));
    assert_eq!(absent.thread_cpu_ns, None);
    assert_eq!(absent.non_cpu_residual_ns, None);
    assert_eq!(zero.wall_ns, absent.wall_ns);
}

#[test]
fn inconsistent_cpu_and_overlapping_child_walls_never_become_zero_waits() {
    let invalid = NativeWorkSpan::from_nanos(40, Some(41));
    assert_eq!(invalid.thread_cpu_ns, None);
    assert_eq!(invalid.rejected_thread_cpu_ns, Some(41));
    assert_eq!(invalid.non_cpu_residual_ns, None);
    assert!(
        NativeWorkSpan::from_nanos(40, Some(20))
            .remainder([Some(NativeWorkSpan::from_nanos(41, Some(10)))])
            .is_none()
    );
}

#[test]
fn named_children_and_unscoped_wall_telescope_without_repeating_the_envelope() {
    let total = NativeWorkSpan::from_nanos(100, Some(60));
    let a = NativeWorkSpan::from_nanos(20, Some(5));
    let b = NativeWorkSpan::from_nanos(30, Some(20));
    let other = total.remainder([Some(a), Some(b)]).unwrap();
    assert_eq!(other.wall_ns, 50);
    assert_eq!(other.thread_cpu_ns, Some(35));
    assert_eq!(other.non_cpu_residual_ns, Some(15));
    assert_eq!(a.wall_ns + b.wall_ns + other.wall_ns, total.wall_ns);
    assert_eq!(
        a.thread_cpu_ns.unwrap() + b.thread_cpu_ns.unwrap() + other.thread_cpu_ns.unwrap(),
        total.thread_cpu_ns.unwrap()
    );
    let unavailable = total
        .remainder([Some(a), Some(NativeWorkSpan::from_nanos(30, None))])
        .unwrap();
    assert_eq!(unavailable.wall_ns, 50);
    assert_eq!(unavailable.thread_cpu_ns, None);
    assert_eq!(unavailable.non_cpu_residual_ns, None);
}

#[test]
fn missing_native_work_and_duration_overflow_are_unavailable() {
    assert!(!NativeWorkAttribution::default().available());
    assert!(NativeWorkAttribution::default().worker.is_none());
    assert!(NativeWorkSpan::from_durations(Duration::MAX, Some(Duration::ZERO)).is_none());
}

#[test]
fn a_worker_result_cannot_follow_a_reused_presentation_sequence_to_another_batch() {
    let mut work = NativeWorkAttribution {
        packet: Some(NativeWorkPacket {
            host: 1,
            generation: 2,
            batch_id: 3,
            proposed_sequence: 4,
        }),
        ..Default::default()
    };
    let worker = NativeWorkerWork {
        batch_id: 3,
        core_generation: 2,
        gpu: NativeGpuWork {
            batch_id: 3,
            ..Default::default()
        },
        ..Default::default()
    };
    work.attach_worker(3, worker);
    assert!(work.worker.is_some());
    work.packet.as_mut().unwrap().batch_id = 5;
    work.attach_worker(3, worker);
    assert!(work.worker.is_none());
    assert_eq!(work.worker_rejected, Some("batch_or_generation_mismatch"));
    assert_eq!(work.packet.unwrap().proposed_sequence, 4);
}
