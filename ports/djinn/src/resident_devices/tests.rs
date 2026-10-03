// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::sync::atomic::{AtomicU64, Ordering};

use personae::{IdentityProvider, InMemoryProvider};
use transport::PeerID;

use super::*;

fn peer(seed: u8) -> PeerID {
    PeerID::from_bytes(
        &InMemoryProvider::from_seed([seed; 32])
            .master_public_key()
            .to_bytes(),
    )
    .unwrap()
}

fn paired(peer: PeerID, hint: Option<String>) -> PairedDevice {
    PairedDevice {
        node_id: owner_settings::hex32(&peer.to_bytes()).to_ascii_uppercase(),
        root: Some("ab".repeat(32)),
        label: "thinkpad".into(),
        added_ms: 1_700_000_000_000,
        pairing_id: Some("pairing-1".into()),
        last_endpoint: hint,
        prekey: None,
    }
}

fn direct(addr: &str) -> PeerAddr {
    PeerAddr::Direct(addr.parse().unwrap())
}

#[test]
fn an_entry_reports_the_live_path_and_decodes_the_saved_hint() {
    let thinkpad = peer(0x51);
    let relay = PeerAddr::Relay("https://relay.example.test./".into());
    let hint =
        transport::encode_peer_ticket(thinkpad, &[direct("192.168.1.32:51234"), relay.clone()])
            .unwrap();
    let live = KnownPeer {
        peer: thinkpad,
        reachable: true,
        bootstrap: false,
        connected: true,
    };
    let paths = [
        PeerPath {
            addr: direct("192.168.1.32:51234"),
            active: false,
        },
        PeerPath {
            addr: direct("192.168.1.68:40000"),
            active: true,
        },
        PeerPath {
            addr: relay,
            active: false,
        },
    ];
    let entry = directory_entry(&paired(thinkpad, Some(hint)), Some(&live), &paths);

    assert_eq!(entry.node_id, owner_settings::hex32(&thinkpad.to_bytes()));
    assert_eq!(
        (entry.label.as_str(), entry.added_ms),
        ("thinkpad", 1_700_000_000_000)
    );
    assert_eq!(entry.pairing_id.as_deref(), Some("pairing-1"));
    assert!(entry.connected && entry.reachable);
    assert_eq!(
        entry.path,
        vec![
            PathAddrV1 {
                kind: AddrKindV1::Direct,
                addr: "192.168.1.32:51234".into(),
                active: false,
            },
            PathAddrV1 {
                kind: AddrKindV1::Direct,
                addr: "192.168.1.68:40000".into(),
                active: true,
            },
            PathAddrV1 {
                kind: AddrKindV1::Relay,
                addr: "https://relay.example.test./".into(),
                active: false,
            },
        ]
    );
    let hint = entry.hint.expect("a saved hint is reported");
    assert_eq!(hint.unreadable, None);
    assert!(hint.addrs.contains(&HintAddrV1 {
        kind: AddrKindV1::Direct,
        addr: "192.168.1.32:51234".into(),
    }));
    assert!(hint.addrs.contains(&HintAddrV1 {
        kind: AddrKindV1::Relay,
        addr: "https://relay.example.test./".into(),
    }));
}

#[test]
fn an_unlisted_device_is_not_connected_and_a_bad_hint_says_why() {
    let thinkpad = peer(0x52);
    let elsewhere = transport::encode_peer_ticket(peer(0x53), &[direct("10.0.0.1:1")]).unwrap();
    let entry = directory_entry(&paired(thinkpad, Some(elsewhere)), None, &[]);
    assert!(!entry.connected && !entry.reachable && entry.path.is_empty());
    let hint = entry.hint.unwrap();
    assert!(hint.addrs.is_empty());
    assert!(hint.unreadable.unwrap().contains("another node"));

    let garbled = directory_entry(&paired(thinkpad, Some("not a ticket".into())), None, &[]);
    assert!(garbled.hint.unwrap().unreadable.is_some());
    assert_eq!(
        directory_entry(&paired(thinkpad, None), None, &[]).hint,
        None
    );
}

/// Gossip and iroh's path can disagree either way, and the card says which:
/// connected with no path active, or not connected with one still active.
#[test]
fn the_card_says_when_gossip_and_the_path_disagree() {
    let thinkpad = peer(0x54);
    let live = KnownPeer {
        peer: thinkpad,
        reachable: true,
        bootstrap: false,
        connected: true,
    };
    let mut path = [PeerPath {
        addr: direct("192.168.1.32:51234"),
        active: false,
    }];
    let idle = directory_entry(&paired(thinkpad, None), Some(&live), &path);
    assert!(idle.connected);
    assert_eq!(
        card_value(&idle).value,
        "connected, no active path right now"
    );
    path[0].active = true;
    let busy = directory_entry(&paired(thinkpad, None), Some(&live), &path);
    assert_eq!(card_value(&busy).value, "connected via 192.168.1.32:51234");
    let gone = KnownPeer {
        connected: false,
        ..live
    };
    let stale = directory_entry(&paired(thinkpad, None), Some(&gone), &path);
    assert!(!stale.connected);
    assert_eq!(
        card_value(&stale).value,
        "not connected (a path is still marked active)"
    );
}

/// The directory moves under a reader; the snapshot it took stays whole.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_snapshot_keeps_its_resources_while_the_directory_moves() {
    let reads = Arc::new(AtomicU64::new(0));
    let counter = Arc::clone(&reads);
    let source = DeviceDirectorySource::from_fn(move || {
        let read = counter.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            Ok(DeviceDirectoryV1 {
                version: 1,
                local_node: format!("read-{read}"),
                devices: Vec::new(),
            })
        })
    });
    let mut endpoint = DeviceDirectoryEndpoint::new(source);
    let snapshot = endpoint
        .snapshot(DeviceDirectoryEndpoint::request())
        .unwrap();
    let offer = snapshot
        .presentation
        .offers
        .values()
        .flatten()
        .next()
        .unwrap();
    let card: PortableCardV1 = serde_json::from_slice(
        &endpoint
            .resource(ResourceRequest {
                session: snapshot.session.clone(),
                resource: offer.resource,
            })
            .unwrap()
            .bytes,
    )
    .unwrap();
    let directory: DeviceDirectoryV1 = serde_json::from_slice(
        &endpoint
            .resource(ResourceRequest {
                session: snapshot.session,
                resource: card.media[0],
            })
            .unwrap()
            .bytes,
    )
    .unwrap();
    assert_eq!(directory.local_node, "read-0");
    assert_eq!(reads.load(Ordering::SeqCst), 1, "resources do not re-read");
    assert!(matches!(
        endpoint.invoke(IntentInvocation {
            session: DeviceDirectoryEndpoint::session(),
            target: InstanceId(0),
            observed_epoch: SceneEpoch(1),
            observed_revision: Revision(1),
            intent: "anything".into(),
            payload: Vec::new(),
        }),
        Ok(IntentResult::Rejected { .. })
    ));
}
