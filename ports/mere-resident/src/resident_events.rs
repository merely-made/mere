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

mod diagnostics;
use diagnostics::LifecycleDiagnostics;
pub use diagnostics::{DiagnosticsConfig, LifecyclePhase};

pub const EVENTS_SCHEMA: &str = "djinn.resident-events/v1";

#[derive(Default)]
struct Inner {
    file: Option<std::fs::File>,
    stopping: Option<String>,
    diagnostics: Option<LifecycleDiagnostics>,
}

/// The event file, or nothing when none was asked for.
#[derive(Clone, Default)]
pub struct EventLog(Arc<Mutex<Inner>>);

impl EventLog {
    pub fn open(
        path: Option<&Path>,
        diagnostics: Option<DiagnosticsConfig>,
    ) -> std::io::Result<Self> {
        let diagnostics = diagnostics.map(LifecycleDiagnostics::new).transpose()?;
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
            diagnostics,
        }))))
    }

    /// Append one event; `fields` (an object) joins the common ones.
    pub fn emit(&self, event: &str, fields: Value) {
        let phase = LifecyclePhase::from_event(event, &fields);
        let mut line = Map::new();
        line.insert("schema".into(), json!(EVENTS_SCHEMA));
        line.insert("at_ms".into(), json!(now_ms()));
        line.insert("pid".into(), json!(std::process::id()));
        line.insert("event".into(), json!(event));
        if let Value::Object(fields) = fields {
            for (name, value) in fields {
                line.entry(name).or_insert(value);
            }
        }
        let mut inner = self.0.lock().expect("event log is never poisoned");
        if let Some(diagnostics) = inner.diagnostics.as_mut() {
            diagnostics.observe(phase);
        }
        if let Some(file) = inner.file.as_mut() {
            // A full disk must not stop the resident; the log says so.
            if let Err(error) =
                writeln!(file, "{}", Value::Object(line)).and_then(|()| file.flush())
            {
                tracing::warn!(%error, "could not write the event file");
            }
        }
    }

    /// Each inspection/receipt reader owns a separate cursor. No backfill when off.
    pub fn diagnostic_cursor(&self) -> Result<Option<apparatus_core::Cursor>, String> {
        let inner = self.0.lock().map_err(|_| "event log lock poisoned")?;
        Ok(inner.diagnostics.as_ref().map(LifecycleDiagnostics::cursor))
    }

    pub fn read_diagnostics(
        &self,
        cursor: &mut apparatus_core::Cursor,
        max_records: usize,
    ) -> Result<Option<registry::observations::CollectedBatch<LifecyclePhase>>, String> {
        let mut inner = self.0.lock().map_err(|_| "event log lock poisoned")?;
        inner
            .diagnostics
            .as_mut()
            .map(|d| d.read(cursor, max_records))
            .transpose()
    }

    /// Bounded current copy for the admitted status route; does not consume any
    /// persistent inspection/receipt cursor and does not export automatically.
    pub fn diagnostic_snapshot(&self) -> Result<Option<Value>, String> {
        let mut inner = self.0.lock().map_err(|_| "event log lock poisoned")?;
        inner
            .diagnostics
            .as_mut()
            .map(LifecycleDiagnostics::snapshot)
            .transpose()
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
        let log = EventLog::open(Some(&path), None).unwrap();
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
        EventLog::open(None, None)
            .unwrap()
            .emit("started", json!({}));
    }
    fn config(records: usize, sample_rate: f32) -> DiagnosticsConfig {
        DiagnosticsConfig {
            ingress_records: 8,
            ingress_bytes: 16384,
            ingress_age_secs: 60,
            retained_records: records,
            retained_bytes: 16384,
            retained_age_secs: 60,
            sample_rate,
        }
    }

    #[test]
    fn lifecycle_diagnostics_preserve_owner_outcomes_and_independent_readers() {
        let log = EventLog::open(None, Some(config(8, 1.0))).unwrap();
        let mut receipt = log.diagnostic_cursor().unwrap().unwrap();
        let mut inspector = log.diagnostic_cursor().unwrap().unwrap();
        log.emit(
            "started",
            json!({"profile":"private", "endpoint":"private"}),
        );
        log.emit(
            "stopped",
            json!({"ok":false,"error":"private","reason":"private"}),
        );
        log.emit("stopped", json!({}));
        log.emit("unknown private name", json!({"credential":"private"}));
        let a = log.read_diagnostics(&mut inspector, 8).unwrap().unwrap();
        let b = log.read_diagnostics(&mut receipt, 8).unwrap().unwrap();
        assert_eq!(a.observations.records, b.observations.records);
        let phases: Vec<_> = a
            .observations
            .records
            .iter()
            .map(|r| r.payload.payload)
            .collect();
        assert_eq!(
            phases,
            [
                LifecyclePhase::Started,
                LifecyclePhase::StoppedFailed,
                LifecyclePhase::StoppedUnknown
            ]
        );
        let snapshot = log.diagnostic_snapshot().unwrap().unwrap();
        assert!(!snapshot.to_string().contains("private"));
        assert_eq!(snapshot["unmapped"], 1);
        assert!(
            a.observations
                .records
                .iter()
                .all(|r| r.envelope.metadata.operation.is_none()
                    && r.envelope.metadata.cause.is_none()
                    && r.envelope.metadata.presented_frame.is_none())
        );
    }

    #[test]
    fn lifecycle_policy_retention_disable_and_run_restart_stay_distinct() {
        let log = EventLog::open(None, Some(config(1, 0.5))).unwrap();
        let mut cursor = log.diagnostic_cursor().unwrap().unwrap();
        for _ in 0..4 {
            log.emit("ready", json!({}));
        }
        let batch = log.read_diagnostics(&mut cursor, 8).unwrap().unwrap();
        assert_eq!(batch.observations.records.len(), 1);
        assert_eq!(batch.observations.stats.loss.evicted, 1);
        assert_eq!(batch.observations.gaps.len(), 1);
        assert_eq!(
            log.diagnostic_snapshot().unwrap().unwrap()["sampled_out"],
            2
        );
        let next = EventLog::open(None, Some(config(0, 1.0))).unwrap();
        next.emit("ready", json!({}));
        let batch = next.read_diagnostics(&mut cursor, 8).unwrap().unwrap();
        assert!(batch.observations.boundary.is_some());
        assert_eq!(batch.observations.stats.loss.rejected_disabled, 1);
        assert!(
            EventLog::open(None, None)
                .unwrap()
                .diagnostic_snapshot()
                .unwrap()
                .is_none()
        );
        assert!(EventLog::open(None, Some(config(8, f32::NAN))).is_err());
    }

    #[test]
    fn event_file_fields_cannot_forge_lifecycle_authority() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");
        let log = EventLog::open(Some(&path), None).unwrap();
        log.emit(
            "started",
            json!({"event":"stopped", "schema":"fake", "pid":0, "at_ms":0, "custom":true}),
        );
        let event: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(event["event"], "started");
        assert_eq!(event["schema"], EVENTS_SCHEMA);
        assert_eq!(event["pid"], std::process::id());
        assert_ne!(event["at_ms"], 0);
        assert_eq!(event["custom"], true);
    }
}
