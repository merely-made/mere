// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use mere_signalman::DjinnStationAuthority;
#[test]
#[ignore = "run scripts/dr_c_receipts.py against the isolated receipt keeper"]
fn dr_c_signalman_identity_stays_pending() {
    let mode = std::env::var("DR_C_RECEIPT_MODE").expect("receipt mode");
    let authority = DjinnStationAuthority::at(std::env::var("GRAPHSHELL_APP_ENDPOINT").unwrap());
    if mode == "locked" {
        let status = authority.status().unwrap();
        assert_eq!(status.lock, graphshell::identity::VaultLockView::Locked);
        assert!(status.wallet_public_key.is_none());
        let mut client = graphshell::native::custody_client::BlockingCustodyClient::open(
            graphshell::native::app_admission::AppId::new("signalman")).unwrap();
        assert!(!client.roster().unwrap().entries.is_empty(), "public persona roster remains readable");
    } else {
        assert_eq!(mode, "absent");
    }
    assert!(
        matches!(authority.release_station(pandect::DeviceId::new()), Err(error) if error.is_pending()),
        "Signalman must not derive a fallback station"
    );
    assert!(
        matches!(authority.release_storage(b"receipt".to_vec()), Err(error) if error.is_pending()),
        "no storage key"
    );
    assert!(matches!(authority.release_controller(b"receipt"), Err(error) if error.is_pending()),
        "no fallback controller identity");
}
