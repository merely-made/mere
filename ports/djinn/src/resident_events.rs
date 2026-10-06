// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The lifecycle event file (`--events-file`, djinn test harness plan ruling
//! 7): one JSON line per `started`, `listening`, `ready`, `stopping` and
//! `stopped`, so a reader learns where a resident is without its log. A
//! killed resident writes no `stopped`; that absence is the crash's record.

use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde_json::{Map, Value, json};

pub const EVENTS_SCHEMA: &str = "djinn.resident-events/v1";

#[derive(Default)]
struct Inner {
    file: Option<std::fs::File>,
    stopping: Option<String>,
}

/// The event file, or nothing when none was asked for.
#[derive(Clone, Default)]
pub struct EventLog(Arc<Mutex<Inner>>);

impl EventLog {
    pub fn open(path: Option<&Path>) -> std::io::Result<Self> {
        let file = match path {
            None => None,
            Some(path) => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                Some(
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)?,
                )
            },
        };
        Ok(Self(Arc::new(Mutex::new(Inner {
            file,
            stopping: None,
        }))))
    }

    /// Append one event; `fields` (an object) joins the common ones.
    pub fn emit(&self, event: &str, fields: Value) {
        let mut line = Map::new();
        line.insert("schema".into(), json!(EVENTS_SCHEMA));
        line.insert("at_ms".into(), json!(now_ms()));
        line.insert("pid".into(), json!(std::process::id()));
        line.insert("event".into(), json!(event));
        if let Value::Object(fields) = fields {
            line.extend(fields);
        }
        let mut inner = self.0.lock().expect("event log is never poisoned");
        if let Some(file) = inner.file.as_mut() {
            // A full disk must not stop the resident; the log says so.
            if let Err(error) =
                writeln!(file, "{}", Value::Object(line)).and_then(|()| file.flush())
            {
                tracing::warn!(%error, "could not write the event file");
            }
        }
    }

    /// `stopping`, once: the first reason is the one that explains the run.
    pub fn stopping(&self, reason: &str) {
        {
            let mut inner = self.0.lock().expect("event log is never poisoned");
            if inner.stopping.is_some() {
                return;
            }
            inner.stopping = Some(reason.into());
        }
        self.emit("stopping", json!({ "reason": reason }));
    }

    /// The reason `stopping` recorded, if any.
    pub fn stopping_reason(&self) -> Option<String> {
        self.0
            .lock()
            .expect("event log is never poisoned")
            .stopping
            .clone()
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_are_json_lines_and_stopping_keeps_its_first_reason() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");
        let log = EventLog::open(Some(&path)).unwrap();
        log.emit("started", json!({ "installed": false }));
        log.stopping("stop intent");
        log.stopping("interrupt");
        log.emit(
            "stopped",
            json!({ "reason": log.stopping_reason(), "ok": true }),
        );
        let lines: Vec<Value> = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let events: Vec<&str> = lines.iter().map(|l| l["event"].as_str().unwrap()).collect();
        assert_eq!(events, ["started", "stopping", "stopped"]);
        assert!(lines.iter().all(|l| l["schema"] == EVENTS_SCHEMA));
        assert_eq!(lines[2]["reason"], "stop intent");
        EventLog::open(None).unwrap().emit("started", json!({}));
    }
}
