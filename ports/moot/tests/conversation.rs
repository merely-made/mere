// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! CPU-only port receipt over two encrypted stores and actual Gemot authority.
//! Operations cross the existing accept boundary; no transport is qualified.
#![cfg(feature = "commons-chat")]

use commons_spine::chat::{Channel, ChatEvent, ChatReplica, Message, chat_write_capability};
use commons_spine::{AuthorityState, CommonsAuthority, GemotAuthorityView};
use gemot::moot::constitution::{CapabilityGrant, ConstitutionRules};
use gemot::moot::{MOOT_ACT_ACTION, MOOT_DELEGATION_DOMAIN, MootAuthority, MootDelegations};
use insigne::delegation::{
    CapabilityScope, DelegationCertificate, DelegationParent, DelegationRevocation,
    SignedDelegationCertificate, SignedDelegationRevocation,
};
use moot::conversation::{Pane, conversation_id, snapshot};
use moot::coop::{Access, LifecycleReport, Membership, Reading, Verdict};
use personae::delegation::Issue;
use personae::{IdentityProvider, InMemoryProvider};
use servitor::{Mode, Subject, cap_path};
use stickleback::DataKeyring;

const SPACE: [u8; 32] = [0xc1; 32];
const MOOT: [u8; 32] = [0xc2; 32];
const ROOT: [u8; 32] = [0xc3; 32];

struct Authority {
    rules: ConstitutionRules,
    grants: MootDelegations,
}

impl Authority {
    fn new(founder: &InMemoryProvider, member: &InMemoryProvider, revoked: bool) -> Self {
        let founder_id = founder.master_public_key().to_bytes();
        let needed = cap_path(&chat_write_capability(SPACE));
        let mut rules = ConstitutionRules::founder_only(founder_id);
        rules.grant(CapabilityGrant {
            id: ROOT,
            subject: founder_id,
            path_prefix: needed.clone(),
            not_before_ms: 10,
            expires_at_ms: Some(1_000),
            delegation_depth: 2,
        });
        let certificate = SignedDelegationCertificate::issue(
            founder,
            DelegationCertificate::new(
                DelegationParent::Root(ROOT),
                founder_id,
                member.master_public_key().to_bytes(),
                CapabilityScope {
                    domain: MOOT_DELEGATION_DOMAIN.into(),
                    resource: MOOT.to_vec(),
                    path_prefix: needed,
                    actions: [MOOT_ACT_ACTION.to_string()].into_iter().collect(),
                },
                15,
                20,
                Some(900),
                0,
                [1; 32],
            ),
        )
        .unwrap();
        let id = certificate.certificate.id();
        let scope = certificate.certificate.scope.clone();
        let mut grants = MootDelegations::new();
        grants
            .accept_certificate(MOOT, &rules, certificate)
            .unwrap();
        if revoked {
            let revocation = SignedDelegationRevocation::issue(
                founder,
                DelegationRevocation::new(id, founder_id, scope, 60, [2; 32]),
            )
            .unwrap();
            grants.accept_revocation(revocation).unwrap();
        }
        Self { rules, grants }
    }

    fn view(&self, now_ms: u64) -> GemotAuthorityView<'_> {
        GemotAuthorityView {
            authority: MootAuthority {
                delegations: &self.grants,
                rules: &self.rules,
                moot_id: MOOT,
                now_ms,
            },
        }
    }
}

fn report(verdict: Verdict, now_ms: u64) -> LifecycleReport {
    let mut report = LifecycleReport::new(
        verdict,
        (verdict != Verdict::Joined).then(|| format!("consumer: {}", verdict.name())),
        now_ms,
    );
    report.membership = Some(Membership {
        members: 2,
        access: Access::Write,
    });
    report.reading = Reading::Continues;
    report
}

fn replicas() -> (
    InMemoryProvider,
    InMemoryProvider,
    ChatReplica<muniment::MemoryBackend>,
    ChatReplica<muniment::MemoryBackend>,
) {
    let alice = InMemoryProvider::from_seed([0xa1; 32]);
    let bob = InMemoryProvider::from_seed([0xb1; 32]);
    let mut keys = DataKeyring::new();
    let secret = keys.rotate_random().unwrap();
    let mut peer_keys = DataKeyring::new();
    peer_keys.install(secret);
    let a = ChatReplica::in_memory_for_identity(SPACE, &alice, keys).unwrap();
    let b = ChatReplica::in_memory_for_identity(SPACE, &bob, peer_keys).unwrap();
    (alice, bob, a, b)
}

fn message(channel: &str, body: &str, sent_at_ms: u64, reply_to: Option<[u8; 32]>) -> ChatEvent {
    ChatEvent::Message(Message {
        channel: channel.into(),
        body: body.into(),
        sent_at_ms,
        reply_to,
    })
}

#[tokio::test]
async fn real_encrypted_replicas_preserve_stable_authors_causal_order_edits_and_retractions() {
    let (alice, bob, mut a, mut b) = replicas();
    let authority = Authority::new(&alice, &bob, false);
    let channel = a
        .author(ChatEvent::Channel(Channel {
            id: "hall".into(),
            title: "Hall".into(),
        }))
        .await
        .unwrap();
    b.accept(&channel).await.unwrap();
    let first = a
        .author(message("hall", "first, with a later clock", 2000, None))
        .await
        .unwrap();
    b.accept(&first).await.unwrap();
    let reply = b
        .author(message(
            "hall",
            "reply, with an earlier clock",
            1,
            Some(first.hash.into()),
        ))
        .await
        .unwrap();
    a.accept(&reply).await.unwrap();
    let edit = a
        .edit_message(first.hash.into(), "edited first".into(), 3)
        .await
        .unwrap();
    b.accept(&edit).await.unwrap();
    let av = snapshot(&a, &authority.view(50), report(Verdict::Joined, 50))
        .await
        .unwrap();
    let bv = snapshot(&b, &authority.view(50), report(Verdict::Joined, 50))
        .await
        .unwrap();
    assert_eq!(
        av.threads[0]
            .messages
            .iter()
            .map(|m| &m.id)
            .collect::<Vec<_>>(),
        bv.threads[0]
            .messages
            .iter()
            .map(|m| &m.id)
            .collect::<Vec<_>>()
    );
    let thread = &av.threads[0];
    assert_eq!(thread.messages[0].body.text(), "edited first");
    assert_eq!(
        thread.messages[1].body.text(),
        "reply, with an earlier clock"
    );
    assert_eq!(
        thread.messages[0].author.address,
        hex::encode(alice.master_public_key().to_bytes())
    );
    assert_eq!(
        thread.messages[1].author.address,
        hex::encode(bob.master_public_key().to_bytes())
    );
    assert_eq!(
        thread.context[0].latest_edit.as_ref().unwrap().0,
        hex::encode(edit.hash.as_bytes())
    );
    assert_eq!(
        thread.context[1].reply_to,
        Some(thread.messages[0].id.clone())
    );
    assert_eq!(thread.messages[0].direction, comms::Direction::Outgoing);
    assert_eq!(
        bv.threads[0].messages[0].direction,
        comms::Direction::Incoming
    );
    assert!(
        thread.conversation.participants.is_empty(),
        "authors do not establish a roster or presence"
    );
    let deletion = b.delete_message(reply.hash.into(), 4).await.unwrap();
    a.accept(&deletion).await.unwrap();
    let deleted = snapshot(&a, &authority.view(50), report(Verdict::Joined, 50))
        .await
        .unwrap();
    assert_eq!(deleted.threads[0].messages.len(), 1);
    assert_eq!(deleted.withheld.retracted, 1);
    assert_eq!(a.sync_store().operation_count().await.unwrap(), 5);
}

#[tokio::test]
async fn current_revocation_and_expiry_withhold_content_without_erasing_stores_or_drafts() {
    let (alice, bob, mut a, mut b) = replicas();
    let granted = Authority::new(&alice, &bob, false);
    let channel = b
        .author(ChatEvent::Channel(Channel {
            id: "hall".into(),
            title: "Bob's title".into(),
        }))
        .await
        .unwrap();
    a.accept(&channel).await.unwrap();
    let first = a
        .author(message("hall", "Alice survives", 1, None))
        .await
        .unwrap();
    b.accept(&first).await.unwrap();
    let reply = b
        .author(message("hall", "Bob withheld", 2, Some(first.hash.into())))
        .await
        .unwrap();
    a.accept(&reply).await.unwrap();
    let initial = snapshot(&b, &granted.view(50), report(Verdict::Joined, 50))
        .await
        .unwrap();
    let mut pane = Pane::new(initial.clone()).unwrap();
    let mut other = Pane::new(initial).unwrap();
    let id = conversation_id(SPACE, "hall");
    pane.select(&id).unwrap();
    other.select(&id).unwrap();
    pane.comms.set_draft_body("unsent local work");
    other.comms.set_draft_body("independent reading");
    let withdrawn = Authority::new(&alice, &bob, true);
    let filtered = snapshot(&b, &withdrawn.view(100), report(Verdict::Revoked, 100))
        .await
        .unwrap();
    assert_eq!(filtered.withheld.revoked, 2);
    assert_eq!(
        filtered.threads[0].conversation.title, "hall",
        "withheld title must not leak"
    );
    assert_eq!(filtered.threads[0].messages.len(), 1);
    pane.refresh(filtered).unwrap();
    assert_eq!(pane.comms.draft.body, "unsent local work");
    assert_eq!(other.comms.draft.body, "independent reading");
    assert_eq!(
        other.comms.thread.len(),
        2,
        "another pane refresh is explicit"
    );
    let retained = b.sync_store().operation_count().await.unwrap();
    assert_eq!(retained, 3);
    let expired = snapshot(&b, &granted.view(901), report(Verdict::Expired, 901))
        .await
        .unwrap();
    assert_eq!(expired.withheld.authority, 2);
    assert_eq!(b.sync_store().operation_count().await.unwrap(), retained);
    // A prepared intention or stale joined report cannot authorize a write.
    let submitted = pane.prepare_send().unwrap();
    assert_eq!(submitted.space, SPACE);
    assert_eq!(
        withdrawn.view(100).classify(
            Subject(b.stable_author()),
            &chat_write_capability(SPACE),
            Mode::Write
        ),
        AuthorityState::Revoked
    );
    assert_eq!(
        granted.view(901).classify(
            Subject(b.stable_author()),
            &chat_write_capability(SPACE),
            Mode::Write
        ),
        AuthorityState::Pending
    );
    assert_eq!(
        pane.comms.draft.body, "unsent local work",
        "owner refusal preserves draft"
    );
}

#[tokio::test]
async fn pane_rejects_foreign_reads_and_invalid_reports_and_preserves_attention_edits() {
    let (alice, bob, mut a, _) = replicas();
    let authority = Authority::new(&alice, &bob, false);
    for id in ["hall", "work"] {
        a.author(ChatEvent::Channel(Channel {
            id: id.into(),
            title: id.into(),
        }))
        .await
        .unwrap();
    }
    let read = snapshot(&a, &authority.view(50), report(Verdict::Joined, 50))
        .await
        .unwrap();
    let mut pane = Pane::new(read.clone()).unwrap();
    let hall = conversation_id(SPACE, "hall");
    pane.select(&hall).unwrap();
    pane.comms.set_draft_body("preserve");
    pane.comms.clear_selection();
    pane.select(&hall).unwrap();
    assert_eq!(pane.comms.draft.body, "preserve");
    assert!(pane.select(&conversation_id([99; 32], "hall")).is_err());
    let before = pane.clone();
    let mut foreign = read.clone();
    foreign.space = [99; 32];
    for thread in &mut foreign.threads {
        thread.conversation.id = conversation_id(foreign.space, &thread.channel);
    }
    assert!(pane.refresh(foreign).is_err());
    assert_eq!(pane, before);
    let mut invalid = read.clone();
    invalid.lifecycle.version = 99;
    assert!(pane.refresh(invalid).is_err());
    assert_eq!(pane, before);
    let mut invalid = read;
    invalid.lifecycle.reason = Some("invalid joined reason".into());
    assert!(Pane::new(invalid).is_err());
    pane.comms.draft.subject = Some("unsupported subject".into());
    assert!(pane.prepare_send().is_err());
    assert_eq!(pane.comms.draft.body, "preserve");
}
