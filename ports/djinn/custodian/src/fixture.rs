// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A custodian served from a thread of the caller over a new synthetic
//! passphrase vault, for test suites and receipt hosts. Only the `fixture`
//! feature builds it; an installed custodian never does. Everything lives
//! under the directory the caller hands over, which must be new.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use castellan::custody::bootstrap::{self, Unlock};
use castellan::custody::{IdentityStorage, Profile};
use personae::{Ed25519Keypair, ProfileId};

use crate::consent::Mechanism;
use crate::core::Core;
use crate::driver::Service;
use crate::policy::Config;

/// A served custodian and where a client finds it.
pub struct Fixture {
    pub vault: PathBuf,
    pub data_root: PathBuf,
    /// The client file a Djinn reads in place of the installed one.
    pub client_file: PathBuf,
    /// Where the SSH agent is served, when asked for.
    pub agent: Option<PathBuf>,
    pub service: Arc<Service>,
}

/// What the fixture serves.
pub struct Spec<'a> {
    pub passphrase: &'a str,
    /// `(id, display name, master seed)`; the first is the one opened.
    pub profiles: &'a [(&'a str, &'a str, [u8; 32])],
    /// The grants of the one principal: this process's own account.
    pub grants: serde_json::Value,
    pub consent: &'a str,
    pub mechanism: Option<Arc<dyn Mechanism>>,
    /// Serve the SSH agent beside the custody socket, as an installed
    /// custodian does (the service bridges its per-use requests to consent).
    pub agent: bool,
}

/// Create the vault under `root` and serve it on a socket there.
#[cfg(unix)]
pub fn start(root: &Path, spec: Spec<'_>) -> Result<Fixture, String> {
    use std::os::unix::fs::PermissionsExt;
    let vault = root.join("vault");
    let data_root = root.join("data");
    std::fs::create_dir_all(&data_root).map_err(|e| e.to_string())?;
    let opened = bootstrap::open_storage(&vault, Unlock::passphrase(spec.passphrase)).map_err(|e| e.to_string())?;
    for (id, name, seed) in spec.profiles {
        let profile = Profile::new(ProfileId((*id).into()), *name, Ed25519Keypair::from_seed(*seed));
        opened.storage.save_profile(&profile).map_err(|e| e.to_string())?;
    }
    drop(opened);
    let uid = custody_wire::peer::effective_uid();
    let first = spec.profiles.first().ok_or("a fixture needs a profile")?.0;
    let config = serde_json::json!({
        "vault": {"directory": vault, "profile": first, "data_root": data_root},
        "owner": {"unix_uid": uid}, "consent": spec.consent, "development": true,
        "principals": [{"name": "djinn", "match": {"unix-uid": uid}, "grants": spec.grants}]
    });
    let config = Config::parse(&serde_json::to_vec(&config).map_err(|e| e.to_string())?).map_err(|e| e.0)?;
    let run = root.join("run");
    std::fs::create_dir(&run).map_err(|e| e.to_string())?;
    std::fs::set_permissions(&run, std::fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    let socket = run.join("custody.sock");
    let listener = custody_wire::transport::UnixSocketListener::bind(&socket).map_err(|e| e.to_string())?;
    let service = Arc::new(Service::new(Core::start(config).map_err(|r| r.to_string())?, spec.mechanism));
    let serving = Arc::clone(&service);
    std::thread::Builder::new()
        .name("fixture-custodian".into())
        .spawn(move || serving.serve(&listener))
        .map_err(|e| e.to_string())?;
    let agent = match spec.agent {
        true => {
            let path = run.join("agent.sock");
            let listener = std::os::unix::net::UnixListener::bind(&path).map_err(|e| e.to_string())?;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
            crate::agent::spawn(Service::clone(&service), listener, Some(uid)).map_err(|e| e.to_string())?;
            Some(path)
        },
        false => None,
    };
    let client_file = root.join("client.json");
    let client = serde_json::json!({"endpoint": socket, "server": {"unix_uids": [uid]}, "agent": agent});
    std::fs::write(&client_file, serde_json::to_vec(&client).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(Fixture {
        vault,
        data_root,
        client_file,
        agent,
        service,
    })
}

impl Fixture {
    /// The vault opened directly, to check what the custodian answered.
    pub fn direct(&self, passphrase: &str, profile: &str) -> Result<castellan::custody::IdentityVault<Box<dyn IdentityStorage>>, String> {
        let opened = bootstrap::open_storage(&self.vault, Unlock::passphrase(passphrase)).map_err(|e| e.to_string())?;
        castellan::custody::IdentityVault::open(opened.storage, &ProfileId(profile.into())).map_err(|e| e.to_string())
    }
}
