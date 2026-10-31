// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The D-Bus adapter's transfer sessions, kept off the Linux-only module so
//! their limit is tested on every target (ruling 49). A `plain` session
//! carries no key material, so the table holds no secret: only each session's
//! object path and the bus connection that owns it.

use std::collections::HashMap;
use std::sync::Mutex;

/// Why a session operation was refused.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum SessionRefusal {
    /// The table already holds its limit of sessions.
    Full(usize),
    /// No session at that path belongs to the caller.
    Absent,
}

/// Open sessions by object path, each bound to the connection that opened it.
pub(super) struct SessionTable {
    max: usize,
    sessions: Mutex<HashMap<String, String>>,
}

impl SessionTable {
    pub(super) fn new(max: usize) -> Self {
        Self {
            max,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Open a session for `owner` and return its object path.
    pub(super) fn open(&self, owner: &str) -> Result<String, SessionRefusal> {
        let mut sessions = self.sessions.lock().unwrap();
        if sessions.len() >= self.max {
            return Err(SessionRefusal::Full(self.max));
        }
        let path = format!(
            "/org/freedesktop/secrets/session/{}",
            uuid::Uuid::new_v4().simple()
        );
        sessions.insert(path.clone(), owner.to_string());
        Ok(path)
    }

    /// Succeed only when the session at `path` belongs to `owner`.
    pub(super) fn require(&self, path: &str, owner: &str) -> Result<(), SessionRefusal> {
        match self.sessions.lock().unwrap().get(path) {
            Some(session_owner) if session_owner == owner => Ok(()),
            _ => Err(SessionRefusal::Absent),
        }
    }

    /// Close the session at `path`, which must belong to `owner`.
    pub(super) fn close(&self, path: &str, owner: &str) -> Result<(), SessionRefusal> {
        let mut sessions = self.sessions.lock().unwrap();
        match sessions.get(path) {
            Some(session_owner) if session_owner == owner => {
                sessions.remove(path);
                Ok(())
            },
            _ => Err(SessionRefusal::Absent),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_session_limit_refuses_one_past_it() {
        let table = SessionTable::new(2);
        let first = table.open(":1.1").unwrap();
        // Control: the second session is at the limit.
        let second = table.open(":1.2").unwrap();
        assert_ne!(first, second);
        assert_eq!(table.open(":1.3"), Err(SessionRefusal::Full(2)));

        // Closing one frees its place, for any caller.
        table.close(&first, ":1.1").unwrap();
        let third = table.open(":1.3").unwrap();
        assert_eq!(table.require(&third, ":1.3"), Ok(()));
        assert_eq!(table.open(":1.4"), Err(SessionRefusal::Full(2)));
    }

    #[test]
    fn a_session_belongs_to_the_connection_that_opened_it() {
        let table = SessionTable::new(4);
        let path = table.open(":1.1").unwrap();
        assert!(path.starts_with("/org/freedesktop/secrets/session/"));
        assert_eq!(table.require(&path, ":1.1"), Ok(()));
        assert_eq!(table.require(&path, ":1.2"), Err(SessionRefusal::Absent));
        assert_eq!(table.close(&path, ":1.2"), Err(SessionRefusal::Absent));
        assert_eq!(table.require(&path, ":1.1"), Ok(()));
        table.close(&path, ":1.1").unwrap();
        assert_eq!(table.require(&path, ":1.1"), Err(SessionRefusal::Absent));
        assert_eq!(table.close(&path, ":1.1"), Err(SessionRefusal::Absent));
    }
}
