// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The identity snapshot's public JSON, pinned byte for byte (dramatis repo
//! plan, ruling D32). DR-A moves these types out of castellan; the snapshots
//! here are built through castellan's old paths, so after the move they also
//! prove the re-exports serve the same bytes.
//!
//! Two forms are pinned: pretty, from `to_public_json`, and compact, as
//! graphshell's identity endpoint writes it (`serde_json::to_vec`), whose
//! revision counter depends on those bytes. `CASTELLAN_BLESS_GOLDEN=1`
//! rewrites the files, and is for a deliberate change only.

use std::path::PathBuf;

use castellan::view::{
    AgentListenerView, CarryView, DeviceGrantView, DeviceView, IdentitySurfaceSnapshot,
    ProfileView, SshKeyView, VaultLockView, VaultProtectionView, VaultView,
};
use personae::signing::{
    ApprovalSource, PendingSigningRequest, SigningFailureCode, SigningPolicy, SigningRecord,
    SigningRecordResult, SigningRequest,
};
use uuid::Uuid;

fn request(n: u128, operation: &str, at_ms: u64) -> SigningRequest {
    SigningRequest {
        request_id: Uuid::from_u128(n),
        profile: "work".into(),
        public_key_fingerprint: "SHA256:fixedfingerprint".into(),
        operation: operation.into(),
        payload_digest: format!("blake3:{n:064x}"),
        adapter: "ssh-agent".into(),
        requested_at_ms: at_ms,
        authenticated_requester: Some("git".into()),
        authenticated_process: None,
        authenticated_target: Some("github.com".into()),
        session_binding: Some("session-1".into()),
        related_object: None,
    }
}

fn record(
    n: u128,
    policy: SigningPolicy,
    source: Option<ApprovalSource>,
    result: SigningRecordResult,
) -> SigningRecord {
    SigningRecord {
        request: request(n, "sign", 1_700_000_000_000 + n as u64),
        policy,
        approval_source: source,
        result,
        completed_at_ms: 1_700_000_000_500 + n as u64,
    }
}

fn carry() -> CarryView {
    CarryView {
        recovery_policy: Some("two-of-three".into()),
        personas: vec!["work".into(), "home".into()],
        devices: vec![
            DeviceView {
                device_id: "device-a".into(),
                label: "ThinkPad".into(),
                mode: "full".into(),
                exposure: "trusted".into(),
                public_key_fingerprint: "SHA256:devicea".into(),
                revoked: false,
                grant_ref: Some("grant-a".into()),
            },
            DeviceView {
                device_id: "device-b".into(),
                label: "Old phone".into(),
                mode: "limited".into(),
                exposure: "exposed".into(),
                public_key_fingerprint: "SHA256:deviceb".into(),
                revoked: true,
                grant_ref: None,
            },
        ],
        grants: vec![DeviceGrantView {
            device_id: "device-a".into(),
            grant_ref: Some("grant-a".into()),
            signature_valid: Some(true),
            issued_at_ms: 1_690_000_000_000,
            expires_at_ms: Some(1_790_000_000_000),
            personas: vec!["work".into()],
            scopes: vec!["sign".into(), "sync".into()],
            attenuations: vec!["no-export".into()],
            wrapped_epoch_count: 2,
        }],
        unavailable: vec![],
    }
}

/// Unlocked, with pending signing and a history covering every result.
fn unlocked() -> IdentitySurfaceSnapshot {
    IdentitySurfaceSnapshot {
        vault: VaultView {
            protection: VaultProtectionView::OsProtected,
            lock: VaultLockView::Unlocked,
            agent: AgentListenerView::StandaloneRetained,
        },
        profiles: vec![
            ProfileView {
                id: "work".into(),
                display_name: "Work".into(),
                selected: true,
                slot_count: 3,
                master_public_fingerprint: "SHA256:masterwork".into(),
            },
            ProfileView {
                id: "home".into(),
                display_name: "Home".into(),
                selected: false,
                slot_count: 0,
                master_public_fingerprint: "SHA256:masterhome".into(),
            },
        ],
        ssh_keys: vec![SshKeyView {
            profile: "work".into(),
            fingerprint: "SHA256:sshkey".into(),
            comment: "work laptop".into(),
            public_openssh: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFixedPublicKeyBytes work".into(),
            lineage: "LocallyGenerated".into(),
            device_loss_note: "Re-add on a new device.".into(),
            unlock_policy: "45s idle".into(),
        }],
        carry: carry(),
        pending_signing: vec![PendingSigningRequest {
            request: request(1, "sign", 1_700_000_000_001),
            policy: SigningPolicy::ShortTtl { idle_seconds: 45 },
            expires_at_ms: 1_700_000_060_001,
        }],
        signing_history: vec![
            record(
                2,
                SigningPolicy::Session,
                Some(ApprovalSource::SessionPolicy),
                SigningRecordResult::Signed {
                    signature_ref: "sig-2".into(),
                },
            ),
            record(
                3,
                SigningPolicy::ShortTtl { idle_seconds: 45 },
                Some(ApprovalSource::CachedShortTtl),
                SigningRecordResult::Signed {
                    signature_ref: "sig-3".into(),
                },
            ),
            record(
                4,
                SigningPolicy::PerUse,
                Some(ApprovalSource::UserOnce),
                SigningRecordResult::Denied,
            ),
            record(
                5,
                SigningPolicy::PerUse,
                Some(ApprovalSource::UserUntilIdle),
                SigningRecordResult::TimedOut,
            ),
            record(
                6,
                SigningPolicy::PerUse,
                None,
                SigningRecordResult::Failed {
                    code: SigningFailureCode::ApprovalChannelClosed,
                },
            ),
            record(
                7,
                SigningPolicy::Session,
                None,
                SigningRecordResult::Failed {
                    code: SigningFailureCode::AdapterFailure,
                },
            ),
        ],
    }
}

/// Locked, as the kept snapshot shows it: profiles and keys, nothing pending.
fn locked_kept() -> IdentitySurfaceSnapshot {
    let mut snapshot = unlocked();
    snapshot.vault = VaultView {
        protection: VaultProtectionView::Passphrase,
        lock: VaultLockView::Locked,
        agent: AgentListenerView::ReceiptEndpoint {
            endpoint: r"\\.\pipe\receipt-agent".into(),
        },
    };
    snapshot.pending_signing.clear();
    snapshot.carry = CarryView {
        unavailable: vec!["identity wallet manifest is absent".into()],
        ..CarryView::default()
    };
    snapshot
}

/// The remaining variants: an ephemeral vault on the standard endpoint.
fn ephemeral() -> IdentitySurfaceSnapshot {
    IdentitySurfaceSnapshot {
        vault: VaultView {
            protection: VaultProtectionView::Ephemeral,
            lock: VaultLockView::Unlocked,
            agent: AgentListenerView::StandardEndpoint {
                endpoint: "/run/user/1000/djinn/agent.sock".into(),
            },
        },
        profiles: vec![],
        ssh_keys: vec![],
        carry: CarryView::default(),
        pending_signing: vec![],
        signing_history: vec![],
    }
}

fn golden(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(name)
}

fn check(name: &str, snapshot: &IdentitySurfaceSnapshot) {
    let pretty = snapshot.to_public_json().unwrap().into_bytes();
    let compact = serde_json::to_vec(snapshot).unwrap();
    for (form, bytes) in [("pretty", pretty), ("compact", compact)] {
        let path = golden(&format!("snapshot_{name}.{form}.json"));
        if std::env::var_os("CASTELLAN_BLESS_GOLDEN").is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &bytes).unwrap();
            continue;
        }
        let pinned = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert!(
            pinned == bytes,
            "{name} ({form}) differs from {}:\n{}",
            path.display(),
            String::from_utf8_lossy(&bytes)
        );
    }
}

#[test]
fn the_unlocked_snapshot_is_byte_identical() {
    check("unlocked", &unlocked());
}

#[test]
fn the_locked_kept_snapshot_is_byte_identical() {
    check("locked_kept", &locked_kept());
}

#[test]
fn the_ephemeral_snapshot_is_byte_identical() {
    check("ephemeral", &ephemeral());
}
