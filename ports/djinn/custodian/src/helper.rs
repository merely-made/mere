// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The consent channel's client loop, run by the Windows logon helper
//! (`djinn-custodian consent`, started in the owner's interactive session
//! from the installed executable). It holds one session admitted as the
//! policy's `consent_channel` principal, puts each petition to the owner
//! through a [`Verifier`] (Windows Hello, in [`crate::platform::hello`]) and
//! answers with a decision bound to the petition's digest, request and
//! generation. Anything but a verified owner is a refusal. The custodian
//! decides whether the helper is the channel: this loop only declines to
//! run when it was admitted as something else.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use custody_wire::{Client, ClientError, Petition, ServerExpectation};
use serde::Deserialize;

/// The installed client file the helper connects by: the file Djinn reads
/// (`djinn::custodian::InstalledCustodian`), written by the installer where
/// the desktop user cannot change it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientFile {
    pub endpoint: String,
    pub server: ServerExpectation,
    #[serde(default)]
    pub agent: Option<String>,
}

impl ClientFile {
    pub fn read(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// Puts one petition to the owner.
pub trait Verifier {
    /// Block until the owner decides; `true` only for a verified approval.
    /// Unavailability, cancel, timeout and every error are `false`.
    fn verify(&self, petition: &Petition) -> bool;
}

/// How soon a lost custodian is tried again, and how far that backs off.
#[derive(Clone, Copy, Debug)]
pub struct Retry {
    pub first: Duration,
    pub longest: Duration,
}

impl Default for Retry {
    fn default() -> Self {
        Self {
            first: Duration::from_millis(500),
            longest: Duration::from_secs(10),
        }
    }
}

/// What the owner is asked, in the words the operating system's dialog shows.
/// A long namespace is cut: the dialog is small and the digest identifies it.
pub fn message(petition: &Petition) -> String {
    let mut text = format!("Djinn custodian: allow {} to {}", petition.principal, petition.operation);
    for (label, value) in [("profile", &petition.profile), ("namespace", &petition.namespace)] {
        if !value.is_empty() {
            let shown: String = value.chars().take(48).collect();
            let cut = if shown.len() < value.len() { "…" } else { "" };
            text.push_str(&format!(" ({label} {shown}{cut})"));
        }
    }
    if !petition.payload_digest.is_empty() {
        text.push_str(&format!(" [{}]", petition.payload_digest.chars().take(12).collect::<String>()));
    }
    text
}

/// Answer petitions from `client` until the custodian is lost or `stop` is
/// set. Returns whether the session was a consent channel at all.
pub fn serve_session(client: &Client, verifier: &dyn Verifier, stop: &AtomicBool) -> bool {
    if !client.session().consent_channel {
        return false;
    }
    while !stop.load(Ordering::Relaxed) {
        match client.next_petition(Duration::from_millis(500)) {
            Some(petition) => {
                let accept = verifier.verify(&petition);
                // A lost custodian has already forgotten the petition.
                let _ = client.decide(&petition, accept);
            },
            None if !client.observed().alive => break,
            None => {},
        }
    }
    true
}

/// Keep a channel session open: connect, serve, and connect again when the
/// custodian restarts or is not there yet. `report` hears each failure once
/// per streak. Returns when `stop` is set.
pub fn run(connect: &dyn Fn() -> Result<Client, ClientError>, verifier: &dyn Verifier, stop: &AtomicBool, retry: Retry, report: &dyn Fn(&str)) {
    let mut delay = retry.first;
    let mut quiet = false;
    while !stop.load(Ordering::Relaxed) {
        match connect() {
            Ok(client) if serve_session(&client, verifier, stop) => {
                delay = retry.first;
                quiet = false;
            },
            Ok(_) => note(report, &mut quiet, "the custodian did not admit this process as its consent channel"),
            Err(error) => note(report, &mut quiet, &error.to_string()),
        }
        let mut waited = Duration::ZERO;
        while waited < delay && !stop.load(Ordering::Relaxed) {
            let step = Duration::from_millis(50).min(delay - waited);
            std::thread::sleep(step);
            waited += step;
        }
        delay = (delay * 2).min(retry.longest);
    }
}

fn note(report: &dyn Fn(&str), quiet: &mut bool, text: &str) {
    if !std::mem::replace(quiet, true) {
        report(text);
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod client_file {
    use super::*;

    #[test]
    fn the_installed_windows_client_file_names_the_pipe_and_the_service() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("client.json");
        std::fs::write(
            &path,
            br#"{"endpoint": "\\\\.\\pipe\\djinn-custodian-S-1-5-21-1", "server": {"windows_sids": ["S-1-5-80-9"]}, "agent": "\\\\.\\pipe\\djinn-custodian-S-1-5-21-1-agent"}"#,
        )
        .unwrap();
        let file = ClientFile::read(&path).unwrap();
        assert_eq!(file.endpoint, r"\\.\pipe\djinn-custodian-S-1-5-21-1");
        assert_eq!(file.server.windows_sids, vec!["S-1-5-80-9".to_string()]);
        assert!(file.agent.unwrap().ends_with("-agent"));
        std::fs::write(&path, br#"{"endpoint": "x", "server": {}, "extra": 1}"#).unwrap();
        assert!(ClientFile::read(&path).is_err(), "unknown fields are refused");
        assert!(ClientFile::read(&dir.path().join("absent.json")).is_err());
    }
}
