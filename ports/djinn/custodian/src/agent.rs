// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The SSH agent, served by the custodian over its own vault (§5.7 keeps the
//! small signing adapter with custody). Castellan's `VaultAgent` answers
//! each connection. A connection is admitted only from the vault owner's
//! uid and only once the vault has been opened; before that it is closed.
//! Per-use approvals become the custodian's consent petitions (see
//! [`crate::core::Core::petition_signing`]), never a decision a client sends.

#![cfg(unix)]

use std::fmt;

use ssh_agent_lib::agent::{Agent, ListeningSocket, Session};
use tokio::net::{UnixListener, UnixStream};

use crate::driver::Service;

/// Serve the agent on `listener` until the process exits.
pub fn spawn(service: Service, listener: std::os::unix::net::UnixListener, owner_uid: Option<u32>) -> std::io::Result<()> {
    listener.set_nonblocking(true)?;
    std::thread::Builder::new().name("custody-agent".into()).spawn(move || {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread().enable_io().enable_time().build() else {
            return;
        };
        runtime.block_on(async move {
            let Ok(inner) = UnixListener::from_std(listener) else {
                return;
            };
            let socket = Gated {
                inner,
                service: service.clone(),
                owner_uid,
            };
            let _ = ssh_agent_lib::agent::listen(socket, CustodianAgent { service }).await;
        });
    })?;
    Ok(())
}

struct Gated {
    inner: UnixListener,
    service: Service,
    owner_uid: Option<u32>,
}

impl fmt::Debug for Gated {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("custodian agent socket")
    }
}

#[ssh_agent_lib::async_trait]
impl ListeningSocket for Gated {
    type Stream = UnixStream;

    async fn accept(&mut self) -> std::io::Result<UnixStream> {
        loop {
            let (stream, _) = self.inner.accept().await?;
            let owner = match (self.owner_uid, stream.peer_cred()) {
                (Some(owner), Ok(peer)) => peer.uid() == owner,
                (None, Ok(peer)) => peer.uid() == custody_wire::peer::effective_uid(),
                _ => false,
            };
            if owner && self.service.agent_session().is_some() {
                return Ok(stream);
            }
        }
    }
}

struct CustodianAgent {
    service: Service,
}

impl Agent<Gated> for CustodianAgent {
    fn new_session(&mut self, _socket: &UnixStream) -> impl Session {
        // The gate admits a connection only once the host exists, and the
        // host is never dropped afterwards.
        self.service.agent_session().expect("admitted only after the vault opened")
    }
}
