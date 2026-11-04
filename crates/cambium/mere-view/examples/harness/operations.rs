/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

impl Harness {
    pub(super) fn new() -> Self {
        let mut harness = Self {
            model: catalog(),
            view: MereViewState::default(),
            tile: (1100, 700),
            events: Vec::new(),
            requests: 0,
            decline_next: false,
            next_session: 4,
            placements: BTreeMap::new(),
            keys_seen: BTreeSet::new(),
        };
        harness.relayout();
        harness
    }

    pub(super) fn view_of(&self) -> MereView<'_> {
        MereView {
            model: &self.model,
            state: &self.view,
            leaf_key: GRAPH_LEAF,
            width: self.tile.0,
            height: self.tile.1,
        }
    }

    pub(super) fn relayout(&mut self) {
        self.view.lay_out(&self.model, self.tile.0, self.tile.1);
        let mut keys: Vec<String> = self
            .model
            .graph
            .nodes
            .iter()
            .map(|n| n.key.clone())
            .collect();
        keys.sort();
        let mut hasher = DefaultHasher::new();
        for key in &keys {
            let (x, y) = self.view.position(key).unwrap_or((0.5, 0.5));
            ((x * 1000.0) as i32, (y * 1000.0) as i32).hash(&mut hasher);
        }
        self.placements
            .insert(self.view_of().layout().to_string(), hasher.finish());
        self.keys_seen.insert(keys);
    }

    pub(super) fn name_of(&self, id: SessionId) -> String {
        self.model
            .sessions
            .iter()
            .find(|session| session.id == id)
            .map(|session| session.name.clone())
            .unwrap_or_default()
    }

    /// Carry out what the view asked, or decline it with the reason.
    pub(super) fn carry_out(&mut self, request: MereViewRequest) {
        self.requests += 1;
        let what = match &request {
            MereViewRequest::Activate(key) => format!("open {key}"),
            MereViewRequest::Mint => "mint a session".to_string(),
            MereViewRequest::Archive(request) => format!("archive {request:?}"),
            MereViewRequest::Session(id, step) => format!("{} {}", step.token(), self.name_of(*id)),
            MereViewRequest::Host(key) => format!("run {key}"),
            MereViewRequest::Layout(id) => format!("lay out by {id}"),
            MereViewRequest::Merge(request) => format!("merge {request:?}"),
            other => format!("{other:?}"),
        };
        self.events.push(format!("request {what}"));
        if std::mem::take(&mut self.decline_next) {
            self.decline(format!("The host declined to {what}."));
            return;
        }
        self.model.notice = None;
        match request {
            MereViewRequest::Activate(key) => {
                let Some(node) = self.model.graph.nodes.iter_mut().find(|n| n.key == key) else {
                    return;
                };
                match node.state {
                    NodeState::Unavailable => {
                        let reason = format!("{} is unavailable: its file is missing.", node.label);
                        self.decline(reason);
                        return;
                    },
                    NodeState::Available => node.state = NodeState::Open,
                    NodeState::Open | NodeState::Dirty => {},
                }
                self.events.push(format!("opened {key}"));
            },
            MereViewRequest::Mint => {
                let name = format!("Session {}", self.next_session);
                self.push_session(name);
            },
            MereViewRequest::Session(id, SessionStep::Switch) => {
                for session in &mut self.model.sessions {
                    session.attached = session.id == id;
                }
            },
            MereViewRequest::Session(id, SessionStep::Fork) => {
                let name = format!("Fork of {}", self.name_of(id));
                self.push_session(name);
            },
            MereViewRequest::Session(id, SessionStep::Branch) => {
                let name = format!("Branch of {}", self.name_of(id));
                self.push_session(name);
            },
            MereViewRequest::Session(_, SessionStep::Merge) | MereViewRequest::Merge(_) => {
                self.decline("This harness keeps no sessions to merge.".to_string());
            },
            MereViewRequest::Session(id, step @ (SessionStep::Trash | SessionStep::Restore)) => {
                if let Some(session) = self.model.sessions.iter_mut().find(|s| s.id == id) {
                    session.trashed = step == SessionStep::Trash;
                }
            },
            MereViewRequest::Host(key) => {
                if ["configure", "retry", "cancel"].contains(&key.as_str()) {
                    self.model.status = ViewStatus::Ready;
                }
            },
            MereViewRequest::Layout(id) => self.model.layout = id,
            MereViewRequest::Archive(_) => self.decline("The fixture offers no archive.".into()),
            // A request or step this harness does not know yet.
            _ => self.decline("This harness does not carry that out.".into()),
        }
        for session in &mut self.model.sessions {
            session.steps = steps_for(session.attached, session.trashed);
        }
        self.relayout();
    }

    pub(super) fn decline(&mut self, reason: String) {
        self.events.push(format!("declined: {reason}"));
        self.model.notice = Some(reason);
    }

    pub(super) fn push_session(&mut self, name: String) {
        let id = session_id(self.next_session);
        self.next_session += 1;
        self.model.sessions.push(SessionEntry {
            id,
            name,
            detail: Some("just made".into()),
            attached: false,
            trashed: false,
            steps: Vec::new(),
        });
    }

    pub(super) fn set_status(&mut self, label: &str) -> bool {
        let note = |message: &str, key: &str, action: &str| StatusNote {
            message: message.into(),
            action: Some(HostAction::new(key, action)),
        };
        self.model.status = match label {
            "empty" => ViewStatus::Empty(note(
                "No catalog is configured.",
                "configure",
                "Choose a catalog",
            )),
            "unavailable" => ViewStatus::Unavailable(note(
                "The reservoir is locked by another resident.",
                "retry",
                "Try again",
            )),
            "building" => {
                ViewStatus::Building(note("Still reading links.", "cancel", "Stop reading"))
            },
            "ready" => ViewStatus::Ready,
            _ => return false,
        };
        true
    }
}
