// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Dramatis: public persona and quarantine management, backed by Djinn.
use dramatis::roster::Roster;
use dramatis::view::VaultLockView;
use graphshell::native::app_admission::{AppId, configured_app_endpoint};
use graphshell::native::custody_client::CustodyClient;
use graphshell::native::quarantine::{
    AssignImportRequest, QuarantineCard, ReviewAction, ReviewRequest, UnassignedImport,
};
use personae::ProfileId;

pub mod views;
pub mod worker;

#[derive(Clone, Debug)]
pub enum Request {
    Refresh,
    Select(ProfileId),
    Review(ReviewRequest),
    Assign(AssignImportRequest),
    Unlock,
    Lock,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub lock: VaultLockView,
    pub roster: Roster,
    pub cards: Vec<QuarantineCard>,
    pub unassigned: Vec<UnassignedImport>,
}
#[derive(Clone, Debug)]
pub struct Reply {
    pub snapshot: Option<Snapshot>,
    pub notice: String,
}
#[derive(Default)]
pub struct State {
    pub snapshot: Option<Snapshot>,
    pub busy: bool,
    pub notice: String,
    pub request: Option<Request>,
}
impl State {
    pub fn queue(&mut self, request: Request) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.request = Some(request);
    }
    pub fn adopt(&mut self, reply: Reply) {
        self.snapshot = reply.snapshot;
        self.notice = reply.notice;
        self.busy = false;
    }
    pub fn selected_profile(&self) -> Option<ProfileId> {
        self.snapshot
            .as_ref()?
            .roster
            .entries
            .iter()
            .find(|p| p.chosen)
            .map(|p| p.id.clone())
    }
    pub fn review(&mut self, card: &QuarantineCard, action: ReviewAction) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        if snapshot.lock == VaultLockView::Locked {
            return;
        }
        if action == ReviewAction::Accept && card.accept_blocked.is_some() {
            return;
        }
        let Some(profile) = self.selected_profile() else {
            return;
        };
        if !snapshot.cards.contains(card) {
            return;
        }
        self.queue(Request::Review(ReviewRequest {
            store: card.store.clone(),
            profile,
            persona: card.persona,
            item: card.item.id,
            revision: card.revision.clone(),
            action,
        }));
    }
}
/// There is no filesystem, seed or passphrase operation in the application.
pub async fn execute_at(endpoint: &str, request: Request) -> Reply {
    let mut client = match CustodyClient::open_at(endpoint, AppId::new("dramatis")).await {
        Ok(client) => client,
        Err(_) => {
            return Reply {
                snapshot: None,
                notice: "Djinn is unavailable. Identity is pending. Start Djinn, then reconnect."
                    .into(),
            };
        },
    };
    let outcome = match request {
        Request::Refresh => Ok(()),
        Request::Select(profile) => client.choose_profile(profile).await,
        Request::Review(request) => client.review_quarantine(request).await,
        Request::Assign(request) => client.assign_import(request).await,
        Request::Unlock => client.unlock_native().await,
        Request::Lock => client.lock_vault().await,
    };
    let notice = outcome.err().map(|e| e.to_string()).unwrap_or_default();
    let read = async {
        let status = client.status().await?;
        let roster = client.roster().await?;
        let cards = if status.lock == VaultLockView::Unlocked {
            match roster.entries.iter().find(|p| p.chosen) {
                Some(profile) => client.quarantine(profile.id.clone()).await?,
                None => Vec::new(),
            }
        } else {
            Vec::new()
        };
        let unassigned = if status.lock == VaultLockView::Unlocked {
            client.unassigned_imports().await?
        } else {
            vec![]
        };
        Ok::<_, graphshell::native::custody_client::CustodyClientError>(Snapshot {
            lock: status.lock,
            roster,
            cards,
            unassigned,
        })
    }
    .await;
    match read {
        Ok(snapshot) => Reply {
            snapshot: Some(snapshot),
            notice,
        },
        Err(error) => Reply {
            snapshot: None,
            notice: format!("Identity is pending or unavailable: {error}"),
        },
    }
}
pub fn endpoint() -> String {
    configured_app_endpoint()
}
