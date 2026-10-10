// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A Moot's conversation view over retained Commons chat and the consumer's
//! coop report. Each pane owns attention and drafts; Gemot and Commons retain
//! authority and history. No live presence, transport, or durable state here.
//!
//! With `commons-chat`, [`snapshot`] reads a real encrypted replica through
//! `projection_with_authority`. Its channel messages keep Commons' causal order,
//! stable Personae authors and original operation identities. Withheld records
//! contribute counts only. The default port can render a snapshot on Wasm.

use comms::{CommsPane, Conversation, ConversationId, Draft, Inbox, Message, MessageId};

use crate::coop::LifecycleReport;

/// Original identities and immutable mutation facts accompanying a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageContext {
    pub id: MessageId,
    pub reply_to: Option<MessageId>,
    pub latest_edit: Option<MessageId>,
    pub edited_at_ms: Option<u64>,
}

/// One channel in a read of the currently authorized Commons projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thread {
    pub channel: String,
    pub conversation: Conversation,
    pub messages: Vec<Message>,
    pub context: Vec<MessageContext>,
}

/// Counts of retained facts omitted from presentation, never their content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Withheld {
    pub causal: usize,
    pub authority: usize,
    pub revoked: usize,
    pub retracted: usize,
}

/// A consumer's current read. This value is never an authorization token or a
/// wire protocol; the consumer supplies its own consistent lifecycle report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub space: [u8; 32],
    pub lifecycle: LifecycleReport,
    pub threads: Vec<Thread>,
    pub withheld: Withheld,
}

impl Snapshot {
    fn check(&self) -> Result<(), String> {
        if self.lifecycle.version != crate::coop::VERSION {
            return Err(format!(
                "unsupported coop report version {}",
                self.lifecycle.version
            ));
        }
        self.lifecycle.check_invariants()?;
        let mut ids = std::collections::HashSet::new();
        for thread in &self.threads {
            if thread.conversation.id != conversation_id(self.space, &thread.channel) {
                return Err(
                    "conversation does not belong to this Commons space and channel".into(),
                );
            }
            if !ids.insert(&thread.conversation.id) {
                return Err("duplicate channel in conversation snapshot".into());
            }
        }
        Ok(())
    }
}

/// Independent attention and editing state for one Moot conversation pane.
/// Construct another pane for another reading; neither edits the shared graph.
#[derive(Clone, Debug, PartialEq)]
pub struct Pane {
    pub comms: CommsPane,
    snapshot: Snapshot,
}

impl Pane {
    pub fn new(snapshot: Snapshot) -> Result<Self, String> {
        snapshot.check()?;
        let mut pane = Self {
            comms: CommsPane::new(),
            snapshot,
        };
        pane.reload();
        Ok(pane)
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    /// Replace the owner's read, keeping this pane's edits and attention where
    /// its channel still exists. A different space requires a separate pane.
    pub fn refresh(&mut self, snapshot: Snapshot) -> Result<(), String> {
        snapshot.check()?;
        if snapshot.space != self.snapshot.space {
            return Err("conversation refresh belongs to another Commons space".into());
        }
        self.snapshot = snapshot;
        self.reload();
        Ok(())
    }

    pub fn select(&mut self, id: &ConversationId) -> Result<(), String> {
        if !self
            .snapshot
            .threads
            .iter()
            .any(|t| &t.conversation.id == id)
        {
            return Err("conversation is absent from this Moot read".into());
        }
        self.comms.select(id.clone());
        self.load_selected();
        Ok(())
    }

    fn reload(&mut self) {
        self.comms.set_inbox(Inbox {
            conversations: self
                .snapshot
                .threads
                .iter()
                .map(|t| t.conversation.clone())
                .collect(),
            failures: Vec::new(),
        });
        self.load_selected();
    }

    fn load_selected(&mut self) {
        let Some(request) = self.comms.begin_thread_load() else {
            return;
        };
        if let Some(thread) = self
            .snapshot
            .threads
            .iter()
            .find(|t| &t.conversation.id == request.conversation())
        {
            self.comms.set_thread(&request, thread.messages.clone());
        }
    }

    /// Prepare an addressed intention, preserving the draft. The owning
    /// application MUST recheck current membership, authority and keys before
    /// authoring. A prior lifecycle report cannot grant a later send.
    pub fn prepare_send(&self) -> Result<SendIntent, String> {
        let draft = &self.comms.draft;
        let id = draft
            .conversation
            .as_ref()
            .ok_or("no conversation selected")?;
        if self.comms.selected() != Some(id) {
            return Err("draft target is not selected".into());
        }
        let thread = self
            .snapshot
            .threads
            .iter()
            .find(|t| &t.conversation.id == id)
            .ok_or("draft target is absent from this Moot read")?;
        if draft.is_empty() {
            return Err("message body is empty".into());
        }
        if draft.subject.is_some() {
            return Err("Commons chat carries no subject line".into());
        }
        Ok(SendIntent {
            space: self.snapshot.space,
            channel: thread.channel.clone(),
            submitted: draft.clone(),
        })
    }
}

/// Addressed content to hand back to the existing application owner. It owns
/// no membership, write permission or success claim. Keep it on send failure;
/// after confirmed success use `CommsPane::acknowledge_sent` with `submitted`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SendIntent {
    pub space: [u8; 32],
    pub channel: String,
    pub submitted: Draft,
}

pub fn conversation_id(space: [u8; 32], channel: &str) -> ConversationId {
    ConversationId::new(
        comms::ProtocolKind::CommonsChat,
        format!("{}/{channel}", hex_id(space)),
    )
}

fn hex_id(id: [u8; 32]) -> String {
    id.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Project a real Commons replica under the current authority supplied by its
/// owner. Never uses the unguarded compatibility projection.
#[cfg(feature = "commons-chat")]
pub async fn snapshot<B: muniment::Backend + Clone, A: commons_spine::CommonsAuthority>(
    replica: &commons_spine::chat::ChatReplica<B>,
    authority: &A,
    lifecycle: LifecycleReport,
) -> Result<Snapshot, String> {
    use comms::{Direction, Identity, MessageBody, ProtocolKind};
    use std::collections::BTreeMap;
    if lifecycle.version != crate::coop::VERSION {
        return Err(format!(
            "unsupported coop report version {}",
            lifecycle.version
        ));
    }
    lifecycle.check_invariants()?;
    let projection = replica
        .projection_with_authority(authority)
        .await
        .map_err(|error| error.to_string())?;
    let space = replica.space_id();
    let mut threads = BTreeMap::<String, Thread>::new();
    let make_thread = |channel: String, title: String| Thread {
        conversation: Conversation {
            id: conversation_id(space, &channel),
            title,
            // Message authors are not the membership roster or live presence.
            participants: Vec::new(),
            last_activity_ms: None,
            unread: 0,
        },
        channel,
        messages: Vec::new(),
        context: Vec::new(),
    };
    for channel in projection.channels {
        threads.insert(channel.id.clone(), make_thread(channel.id, channel.title));
    }
    for authored in projection.messages {
        let channel = &authored.message.channel;
        // An authorized message remains visible if a channel-title operation
        // was withheld. Its exact channel address supplies the fallback label.
        let thread = threads
            .entry(channel.clone())
            .or_insert_with(|| make_thread(channel.clone(), channel.clone()));
        let id = MessageId(hex_id(authored.operation));
        thread.context.push(MessageContext {
            id: id.clone(),
            reply_to: authored.message.reply_to.map(|id| MessageId(hex_id(id))),
            latest_edit: authored.latest_edit.map(|id| MessageId(hex_id(id))),
            edited_at_ms: authored.edited_at_ms,
        });
        thread.conversation.last_activity_ms = Some(
            thread
                .conversation
                .last_activity_ms
                .unwrap_or(0)
                .max(authored.message.sent_at_ms),
        );
        thread.messages.push(Message {
            id,
            author: Identity::new(ProtocolKind::CommonsChat, hex_id(authored.author)),
            body: MessageBody::PlainText(authored.message.body),
            subject: None,
            timestamp_ms: Some(authored.message.sent_at_ms),
            direction: if authored.author == replica.stable_author() {
                Direction::Outgoing
            } else {
                Direction::Incoming
            },
        });
    }
    Ok(Snapshot {
        space,
        lifecycle,
        threads: threads.into_values().collect(),
        withheld: Withheld {
            causal: projection.pending.len(),
            authority: projection.pending_authority.len(),
            revoked: projection.revoked.len(),
            retracted: projection.deleted_messages.len(),
        },
    })
}
