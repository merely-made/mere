// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

fn notified(f: &Fixture) {
    let deadline = Instant::now() + Duration::from_secs(6);
    while !f.host.s.native_completion_pending.load(Ordering::Acquire) {
        assert!(
            Instant::now() < deadline,
            "native completion never notified"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
#[ignore = "actual GPU; native terminal and reconfiguration demand gate"]
fn unavailable_surface_releases_the_candidate_without_creating_retry_demand() {
    let mut f = fixture();
    let out = captures(&mut f.host);
    f.host.redraw();
    notified(&f);
    let before = f.requests.get();
    assert!(f.host.collect_presentation_wake());
    assert_eq!(
        f.requests.get(),
        before,
        "unavailable surface must wait for fresh demand"
    );
    assert!(f.host.s.pending_native.is_none());
    assert_unpublished(&f);
    assert_once(&out);
    assert_eq!(f.acquisitions.get(), 1);
    wait_lease_release(&f.core);
    assert!(!f.host.collect_presentation_wake());
    assert_eq!(f.requests.get(), before);
}

#[test]
#[ignore = "actual GPU; native terminal and reconfiguration demand gate"]
fn a_reconfigured_surface_creates_one_retry_without_publishing_the_old_packet() {
    let mut f = fixture();
    let out = captures(&mut f.host);
    f.reconfigured.set(true);
    f.host.redraw();
    notified(&f);
    let before = f.requests.get();
    assert!(f.host.collect_presentation_wake());
    assert_eq!(f.requests.get(), before + 1);
    assert_unpublished(&f);
    assert_once(&out);
    assert!(f.host.s.pending_native.is_none());
    wait_lease_release(&f.core);
    assert!(!f.host.collect_presentation_wake());
    assert_eq!(f.requests.get(), before + 1);
}

#[test]
#[ignore = "actual GPU; close must revoke pending native presentation"]
fn closing_rejects_pending_work_before_acquisition_and_never_prepares_another_frame() {
    let mut f = fixture();
    let out = captures(&mut f.host);
    f.host.redraw();
    f.host.s.close_requested = true;
    notified(&f);
    let before = f.requests.get();
    assert!(f.host.collect_presentation_wake());
    assert_eq!(f.requests.get(), before);
    assert_unpublished(&f);
    assert_eq!(f.acquisitions.get(), 0);
    assert_once(&out);
    wait_lease_release(&f.core);
    f.host.redraw();
    assert!(f.host.s.pending_native.is_none());
    assert_eq!(f.prepared.get(), 1);
    assert_eq!(f.requests.get(), before);
}
