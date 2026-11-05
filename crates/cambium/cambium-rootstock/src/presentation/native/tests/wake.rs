// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native notifications collect existing work without an OS-redraw roundtrip.

use super::*;

fn collect_all(hosts: &mut [&mut Fixture]) {
    let deadline = Instant::now() + Duration::from_secs(6);
    while hosts.iter().any(|f| f.host.s.pending_native.is_some()) {
        assert!(
            Instant::now() < deadline,
            "native notification did not retire its candidate"
        );
        let mut attempted = false;
        for f in hosts.iter_mut() {
            attempted = f.host.collect_presentation_wake() || attempted;
        }
        if !attempted {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

#[test]
#[ignore = "actual GPU; execute the serialized native wake collection gate"]
fn pending_redraws_coalesce_without_hooks_and_completion_collects_without_redraw() {
    let mut f = fixture();
    let out = captures(&mut f.host);
    f.host.redraw();
    assert!(f.host.s.pending_native.is_some());
    let requested = f.requests.get();
    f.model.set(9);
    for _ in 0..32 {
        assert!(f.host.coalesce_presentation_redraw());
    }
    assert!(f.host.s.native_dirty);
    assert!(f.host.s.pending_native.is_some());
    assert_eq!(f.requests.get(), requested);
    assert_eq!(f.producer.borrow().attempts, 1);
    assert_eq!(f.prepared.get(), 1);
    assert_eq!(f.acquisitions.get(), 0);
    assert!(out.borrow().is_empty());
    assert_unpublished(&f);
    collect_all(&mut [&mut f]);
    assert_eq!(
        f.requests.get(),
        requested,
        "unavailable completion cannot schedule another picture"
    );
    assert_eq!(f.producer.borrow().attempts, 1);
    assert_eq!(f.prepared.get(), 1);
    assert_eq!(f.application_wakes.get(), 0);
    assert_eq!(f.acquisitions.get(), 1);
    assert_once(&out);
    assert_unpublished(&f);
    assert!(!f.host.s.native_completion_pending.load(Ordering::Acquire));
    assert!(!f.host.collect_presentation_wake());
    assert!(!f.host.coalesce_presentation_redraw());
    assert_eq!(f.requests.get(), requested);
    wait_lease_release(&f.core);
    assert_eq!(f.core.try_reuse_native_raster(|| 23).unwrap(), 23);
}

#[test]
#[ignore = "actual GPU; execute the serialized shared-Core native wake gate"]
fn completion_pass_visits_both_hosts_and_rearms_each_notification_across_cycles() {
    // Independent receivers, one actual Core. Shared-forest routing and the
    // platform event-source pass are qualified by their separate host gates.
    let mut a = fixture();
    let mut b = fixture_with_core(a.core.clone());
    assert!(Arc::ptr_eq(&a.core, &b.core));
    for cycle in 1..=2 {
        let outcome_a = captures(&mut a.host);
        let outcome_b = captures(&mut b.host);
        a.host.redraw();
        assert!(a.host.s.pending_native.is_some());
        let before = b.prepared.get();
        let requested = b.requests.get();
        b.host.redraw();
        assert!(
            b.host.s.pending_native.is_none(),
            "one Core admits one batch"
        );
        assert_eq!(b.prepared.get(), before);
        assert_eq!(
            b.requests.get(),
            requested,
            "busy admission waits for resource availability"
        );
        collect_all(&mut [&mut a, &mut b]);
        assert_once(&outcome_a);
        assert_eq!(
            b.prepared.get(),
            before,
            "completion did not prepare another host"
        );
        assert_eq!(
            b.requests.get(),
            requested + 1,
            "one availability wake requests the owed picture"
        );
        b.host.redraw();
        assert!(b.host.s.pending_native.is_some());
        collect_all(&mut [&mut a, &mut b]);
        assert_once(&outcome_b);
        for f in [&mut a, &mut b] {
            assert_eq!(f.prepared.get(), cycle);
            assert_eq!(f.producer.borrow().attempts, cycle);
            assert_eq!(f.application_wakes.get(), 0);
            assert!(!f.host.s.native_completion_pending.load(Ordering::Acquire));
            assert!(!f.host.collect_presentation_wake());
            assert_unpublished(f);
        }
        wait_lease_release(&a.core);
        assert_eq!(a.core.try_reuse_native_raster(|| 24).unwrap(), 24);
    }
}
