// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The SSH agent on Windows, served on its own named pipe as the Unix agent
//! is on its own socket ([`crate::agent`]). The pipe carries the same
//! protected DACL as the custody pipe, and a connection is admitted only
//! when the client process's token is the vault owner's and the vault has
//! been opened; before that the connection is closed. Win32-OpenSSH's
//! `ssh.exe` treats `SSH_AUTH_SOCK` as a pipe name, so the owner points it
//! at [`default_endpoint`]. Per-use approvals become the custodian's consent
//! petitions, never a decision a client sends.

#![cfg(windows)]

use std::fmt;
use std::os::windows::io::AsRawHandle;

use custody_wire::Peer;
use custody_wire::peer::{PipeSecurity, owner_pipe_sddl, pipe_client_peer};
use ssh_agent_lib::agent::{Agent, ListeningSocket, Session};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

use crate::driver::Service;

/// The agent's pipe for `owner_sid`, unless `vault.agent_endpoint` names another.
pub fn default_endpoint(owner_sid: &str) -> String {
    format!(r"\\.\pipe\djinn-custodian-{owner_sid}-agent")
}

/// Create the pipe `name` and serve the agent on it until the process
/// exits. The first instance is created here, so a name another process
/// already holds is an error to the caller.
pub fn spawn(service: Service, name: String, owner_sid: String) -> std::io::Result<()> {
    let security = PipeSecurity::from_sddl(&owner_pipe_sddl(&owner_sid))?;
    let runtime = tokio::runtime::Builder::new_current_thread().enable_io().enable_time().build()?;
    let first = {
        let _entered = runtime.enter();
        create(&name, &security, true)?
    };
    std::thread::Builder::new().name("custody-agent".into()).spawn(move || {
        runtime.block_on(async move {
            let socket = Gated {
                next: first,
                name,
                security,
                owner_sid,
                service: service.clone(),
            };
            let _ = ssh_agent_lib::agent::listen(socket, CustodianAgent { service }).await;
        });
    })?;
    Ok(())
}

/// One pipe instance, protected, local clients only.
fn create(name: &str, security: &PipeSecurity, first: bool) -> std::io::Result<NamedPipeServer> {
    let mut attributes = security.attributes();
    let mut options = ServerOptions::new();
    options.first_pipe_instance(first).reject_remote_clients(true);
    // SAFETY: `attributes` points at a descriptor `security` keeps alive for the call.
    unsafe { options.create_with_security_attributes_raw(name, std::ptr::from_mut(&mut attributes).cast::<core::ffi::c_void>()) }
}

/// Failed connects tolerated in a row before the agent gives up.
const MAX_FAILURES: u32 = 16;

struct Gated {
    next: NamedPipeServer,
    name: String,
    security: PipeSecurity,
    owner_sid: String,
    service: Service,
}

impl fmt::Debug for Gated {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("custodian agent pipe")
    }
}

impl Gated {
    fn admits(&self, connected: &NamedPipeServer) -> bool {
        let owner = matches!(pipe_client_peer(connected.as_raw_handle()), Ok(Peer::Windows { sid, .. }) if sid == self.owner_sid);
        owner && self.service.agent_session().is_some()
    }
}

#[ssh_agent_lib::async_trait]
impl ListeningSocket for Gated {
    type Stream = NamedPipeServer;

    async fn accept(&mut self) -> std::io::Result<NamedPipeServer> {
        let mut failures = 0;
        loop {
            let connected = match self.next.connect().await {
                Ok(()) => {
                    failures = 0;
                    true
                },
                Err(error) if failures + 1 >= MAX_FAILURES => return Err(error),
                Err(_) => {
                    failures += 1;
                    false
                },
            };
            // The next instance exists before this one is handed on, so the
            // name is never without a listening instance.
            let next = create(&self.name, &self.security, false)?;
            let pipe = std::mem::replace(&mut self.next, next);
            if connected && self.admits(&pipe) {
                return Ok(pipe);
            }
        }
    }
}

struct CustodianAgent {
    service: Service,
}

impl Agent<Gated> for CustodianAgent {
    fn new_session(&mut self, _socket: &NamedPipeServer) -> impl Session {
        // The gate admits a connection only once the host exists, and the
        // host is never dropped afterwards.
        self.service.agent_session().expect("admitted only after the vault opened")
    }
}
