// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use dramatis::{
    roster::{Roster, RosterEntry},
    view::VaultLockView,
};
use dramatis_app::{Reply, Request, Snapshot, State};
use personae::ProfileId;
#[test]
fn a_failed_refresh_clears_the_previous_live_surface() {
    let mut s = State::default();
    s.snapshot = Some(Snapshot {
        lock: VaultLockView::Unlocked,
        roster: Roster {
            entries: vec![],
            chosen: ProfileId("family".into()),
            description: String::new(),
        },
        cards: vec![],
        unassigned: vec![],
    });
    s.queue(Request::Refresh);
    s.adopt(Reply {
        snapshot: None,
        notice: "Djinn absent".into(),
    });
    assert!(s.snapshot.is_none());
    assert!(!s.busy);
    assert_eq!(s.notice, "Djinn absent");
}
#[test]
fn one_outstanding_request_cannot_be_overwritten_by_another_click() {
    let mut s = State::default();
    s.queue(Request::Select(ProfileId("family".into())));
    s.queue(Request::Select(ProfileId("work".into())));
    assert!(matches!(s.request,Some(Request::Select(ProfileId(ref p))) if p=="family"));
}
#[test]
fn the_choice_comes_from_djinn_and_is_not_remembered_in_app_storage() {
    let mut s = State::default();
    s.adopt(Reply {
        snapshot: Some(Snapshot {
            lock: VaultLockView::Locked,
            roster: Roster {
                entries: vec![RosterEntry {
                    id: ProfileId("family".into()),
                    display_name: "Family".into(),
                    slot_count: 0,
                    chosen: true,
                }],
                chosen: ProfileId("family".into()),
                description: String::new(),
            },
            cards: vec![],
            unassigned: vec![],
        }),
        notice: String::new(),
    });
    assert_eq!(s.selected_profile(), Some(ProfileId("family".into())));
}
#[tokio::test]
async fn an_absent_custodian_leaves_the_app_pending() {
    #[cfg(windows)]
    let endpoint = format!(r"\\.\pipe\dramatis-absent-{}", std::process::id());
    #[cfg(not(windows))]
    let endpoint = format!("/tmp/dramatis-absent-{}.sock", std::process::id());
    let reply = dramatis_app::execute_at(&endpoint, Request::Refresh).await;
    assert!(reply.snapshot.is_none());
    assert!(reply.notice.contains("pending"));
}

/// Uses only the app API against the separately launched synthetic Djinn host.
#[tokio::test]
#[ignore = "requires dramatis-review-host and DRAMATIS_RECEIPT_ENDPOINT"]
async fn app_wire_receipt() {
    let endpoint = std::env::var("DRAMATIS_RECEIPT_ENDPOINT").unwrap();
    let mode = std::env::var("DRAMATIS_RECEIPT_MODE").unwrap();
    let reply = dramatis_app::execute_at(&endpoint, Request::Refresh).await;
    if mode == "absent" {
        assert!(reply.snapshot.is_none());
        assert!(reply.notice.contains("pending"));
        return;
    }
    let snapshot = reply.snapshot.unwrap();
    assert_eq!(snapshot.roster.entries.len(), 2);
    if mode == "locked" {
        assert_eq!(snapshot.lock, VaultLockView::Locked);
        assert!(snapshot.cards.is_empty());
        let reply =
            dramatis_app::execute_at(&endpoint, Request::Select(ProfileId("work".into()))).await;
        assert!(!reply.notice.is_empty());
        assert_eq!(
            reply.snapshot.unwrap().roster.chosen,
            ProfileId("family".into())
        );
        return;
    }
    assert_eq!(snapshot.cards.len(), 3);
    let encoded = serde_json::to_string(&snapshot.cards).unwrap();
    assert!(!encoded.contains("4111111111111111"));
    assert!(!encoded.contains("synthetic-never-render-this"));
    let reply =
        dramatis_app::execute_at(&endpoint, Request::Select(ProfileId("work".into()))).await;
    assert!(reply.notice.is_empty());
    let work = reply.snapshot.unwrap();
    assert_eq!(work.roster.chosen, ProfileId("work".into()));
    assert!(work.cards.is_empty());
    let locked = dramatis_app::execute_at(&endpoint, Request::Lock)
        .await
        .snapshot
        .unwrap();
    assert_eq!(locked.lock, VaultLockView::Locked);
    let unlocked = dramatis_app::execute_at(&endpoint, Request::Unlock).await;
    assert!(unlocked.notice.is_empty(), "{}", unlocked.notice);
    assert_eq!(
        unlocked.snapshot.unwrap().roster.chosen,
        ProfileId("work".into())
    );
    let reply =
        dramatis_app::execute_at(&endpoint, Request::Select(ProfileId("family".into()))).await;
    assert!(reply.notice.is_empty());
    let family = reply.snapshot.unwrap();
    let card = family
        .cards
        .iter()
        .find(|c| c.accept_blocked.is_none())
        .unwrap();
    let request = graphshell_endpoint::door::quarantine::ReviewRequest {
        store: card.store.clone(),
        profile: ProfileId("family".into()),
        persona: card.persona,
        item: card.item.id,
        revision: card.revision.clone(),
        action: graphshell_endpoint::door::quarantine::ReviewAction::Delete,
    };
    let reply = dramatis_app::execute_at(&endpoint, Request::Review(request)).await;
    assert!(reply.notice.contains("cancelled"));
    assert_eq!(reply.snapshot.unwrap().cards, family.cards);
    // Same app flows lock and unlock through the resident; fixture UI supplies
    // its synthetic passphrase, while the app handles no secret input.
    let reply = dramatis_app::execute_at(&endpoint, Request::Lock).await;
    let locked = reply.snapshot.unwrap();
    assert_eq!(locked.lock, VaultLockView::Locked);
    assert!(locked.cards.is_empty());
    let reply = dramatis_app::execute_at(&endpoint, Request::Unlock).await;
    assert!(reply.notice.is_empty(), "{}", reply.notice);
    assert_eq!(reply.snapshot.unwrap().cards, family.cards);
}

#[tokio::test]
#[ignore = "requires synthetic native Djinn UI and resident denial"]
async fn native_consent_receipt() {
    let endpoint = std::env::var("DRAMATIS_RECEIPT_ENDPOINT").unwrap();
    let snapshot = dramatis_app::execute_at(&endpoint, Request::Refresh)
        .await
        .snapshot
        .unwrap();
    let card = snapshot
        .cards
        .iter()
        .find(|c| c.accept_blocked.is_none())
        .unwrap();
    let request = graphshell_endpoint::door::quarantine::ReviewRequest {
        store: card.store.clone(),
        profile: ProfileId("family".into()),
        persona: card.persona,
        item: card.item.id,
        revision: card.revision.clone(),
        action: graphshell_endpoint::door::quarantine::ReviewAction::Delete,
    };
    let reply = dramatis_app::execute_at(&endpoint, Request::Review(request)).await;
    if std::env::var("DRAMATIS_NATIVE_RESPONSE").as_deref() == Ok("approve") {
        assert!(reply.notice.is_empty(), "{}", reply.notice);
        let remaining = reply.snapshot.unwrap().cards;
        assert_eq!(remaining.len(), snapshot.cards.len() - 1);
        assert!(!remaining.iter().any(|c| c.item.id == card.item.id));
    } else {
        assert!(reply.notice.contains("cancelled"));
        assert_eq!(reply.snapshot.unwrap().cards, snapshot.cards);
    }
}

#[test]
fn an_acknowledged_action_survives_background_refresh_but_not_loss_of_the_resident() {
    let mut state = State::default();
    let snapshot = Snapshot {
        lock: VaultLockView::Unlocked,
        roster: Roster {
            entries: vec![],
            chosen: ProfileId("family".into()),
            description: String::new(),
        },
        cards: vec![],
        unassigned: vec![],
    };
    state.adopt(Reply {
        snapshot: Some(snapshot.clone()),
        notice: "Passphrase changed.".into(),
    });
    state.adopt_refresh(Reply {
        snapshot: Some(snapshot),
        notice: String::new(),
    });
    assert_eq!(state.notice, "Passphrase changed.");
    state.adopt_refresh(Reply {
        snapshot: None,
        notice: "Djinn unavailable".into(),
    });
    assert!(state.snapshot.is_none());
    assert_eq!(state.notice, "Djinn unavailable");
}
