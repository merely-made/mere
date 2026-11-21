// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::{
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    time::Duration,
};

use anyhow::{Context, Result, ensure};
use identity::UnassociatedKeypair;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::{sleep, timeout};
use transport::i2p::{Bridge, I2pAddress, I2pSession, Key, TunnelProfile};
use transport::security::{
    ServiceSecurityPolicy, SessionSecurityFacts, SetupGate, SetupGuard, SetupLimits,
};
use transport::{
    Alpn, P2pandaTransport, PeerID, ReticulumInterface, ReticulumTransport, Transport,
};

use crate::Carrier;
use crate::protocol::{Invitation, PROTOCOL, Route, Rows, Service, request};

const CONNECT: Duration = Duration::from_secs(90);

fn policy(native: bool, owner: Option<PeerID>) -> ServiceSecurityPolicy {
    let limits = SetupLimits {
        timeout: CONNECT,
        ..Default::default()
    };
    let alpn = Alpn::new(PROTOCOL);
    if native {
        ServiceSecurityPolicy::native(alpn, owner, limits)
    } else {
        ServiceSecurityPolicy::noise(alpn, owner, limits)
    }
}

async fn iroh(key: &UnassociatedKeypair) -> Result<P2pandaTransport> {
    Ok(P2pandaTransport::builder_for(key).bind().await?)
}

async fn reticulum(interface: ReticulumInterface) -> Result<ReticulumTransport> {
    Ok(
        ReticulumTransport::builder_for(&UnassociatedKeypair::generate())
            .interfaces(vec![interface])
            .alpns(vec![Alpn::new(PROTOCOL)])
            .announce_interval(Duration::from_millis(750))
            .connect_timeout(CONNECT)
            .bind()
            .await?,
    )
}

async fn i2p(sam: Option<SocketAddr>, publish: bool) -> Result<I2pSession> {
    let bridge = sam.unwrap_or(SocketAddr::from((Ipv4Addr::LOCALHOST, 7656)));
    ensure!(
        bridge.ip().is_loopback() && bridge.port() > 0,
        "--sam needs a loopback address and port"
    );
    I2pSession::create(
        Bridge::new(bridge),
        Key::Transient,
        TunnelProfile::new(2, 2, 2, publish)?,
        Duration::from_secs(285),
    )
    .await
    .context("create I2P session through the local SAM router")
}

async fn answer<S: AsyncRead + AsyncWrite + Unpin>(
    service: &Service,
    secured: std::result::Result<
        (S, SessionSecurityFacts, SetupGuard),
        transport::security::SetupError,
    >,
) {
    let result = async {
        let (stream, _, guard) = secured?;
        service.answer(stream, guard).await
    }
    .await;
    if let Err(error) = result {
        eprintln!("Query session ended: {error}");
    }
}

pub async fn serve(
    via: Carrier,
    listen: Option<SocketAddr>,
    sam: Option<SocketAddr>,
    file: &Path,
    service: Service,
) -> Result<()> {
    ensure!(via == Carrier::I2p || sam.is_none(), "--sam is for I2P");
    ensure!(
        matches!(via, Carrier::Reticulum | Carrier::Webrtc) || listen.is_none(),
        "--listen is for Reticulum or WebRTC"
    );
    let key = UnassociatedKeypair::generate();
    let owner = PeerID::from_public_key(key.public_key());
    let alpn = Alpn::new(PROTOCOL);
    let gate = SetupGate::new(1);
    let policy = policy(via == Carrier::Iroh, None);
    match via {
        Carrier::Webrtc => {
            crate::webrtc::serve(
                listen.context("WebRTC needs --listen <reachable-local-IP:port>")?,
                file,
                service,
            )
            .await
        },
        Carrier::Iroh => {
            let t = iroh(&key).await?;
            t.register_bounded_alpn(alpn.clone(), 8, Duration::from_secs(15))
                .await?;
            service.invite(
                file,
                owner,
                Route::Iroh {
                    ticket: t.ticket().await?,
                },
            )?;
            loop {
                let stream = t.accept(alpn.clone()).await?.into_stream();
                answer(&service, gate.reserve(&policy)?.native(stream)).await;
            }
        },
        Carrier::Reticulum => {
            let address = listen.context("Reticulum needs --listen <reachable-local-IP:port>")?;
            ensure!(
                !address.ip().is_unspecified() && address.port() > 0,
                "--listen needs a reachable IP and nonzero port"
            );
            let t = reticulum(ReticulumInterface::TcpServer { bind: address }).await?;
            service.invite(
                file,
                owner,
                Route::Reticulum {
                    address,
                    destination: t.local_peer_id().to_bytes(),
                },
            )?;
            loop {
                let stream = t.accept(alpn.clone()).await?.into_stream();
                answer(&service, gate.noise_responder(&key, stream, &policy).await).await;
            }
        },
        Carrier::I2p => {
            let t = i2p(sam, true).await?;
            service.invite(
                file,
                owner,
                Route::I2p {
                    destination: t.address()?.as_str().into(),
                },
            )?;
            loop {
                let stream = t.accept().await?;
                answer(&service, gate.noise_responder(&key, stream, &policy).await).await;
            }
        },
    }
}

async fn read_rows<S: AsyncRead + AsyncWrite + Unpin>(
    secured: (S, SessionSecurityFacts, SetupGuard),
    invite: &Invitation,
    query: &str,
) -> Result<Rows> {
    let (stream, _, guard) = secured;
    // The setup policy verified the owner's identity and protocol on this stream.
    guard.admitted()?;
    request(stream, invite, query).await
}

pub async fn query(invite: &Invitation, query: &str, sam: Option<SocketAddr>) -> Result<Rows> {
    ensure!(
        matches!(invite.route, Route::I2p { .. }) || sam.is_none(),
        "--sam is for I2P"
    );
    let owner = PeerID::from_bytes(&invite.owner)?;
    let key = UnassociatedKeypair::generate();
    let alpn = Alpn::new(PROTOCOL);
    let gate = SetupGate::new(1);
    let policy = policy(matches!(invite.route, Route::Iroh { .. }), Some(owner));
    match &invite.route {
        Route::Iroh { ticket } => {
            let t = iroh(&key).await?;
            let peer = t.add_peer_ticket(ticket).await?;
            ensure!(peer == owner, "invitation ticket and owner disagree");
            let session = timeout(CONNECT, t.connect_session(peer, alpn)).await??;
            let rows = read_rows(
                gate.reserve(&policy)?.native(session.stream)?,
                invite,
                query,
            )
            .await;
            t.close().await?;
            rows
        },
        Route::Reticulum {
            address,
            destination,
        } => {
            let t = reticulum(ReticulumInterface::TcpClient { addr: *address }).await?;
            let stream = t.connect(PeerID::from_bytes(destination)?, alpn).await?;
            read_rows(
                gate.noise_initiator(&key, stream, &policy).await?,
                invite,
                query,
            )
            .await
        },
        Route::I2p { destination } => {
            let addr = I2pAddress::parse(destination)?;
            let t = i2p(sam, false).await?;
            // Allow a newly published service time to reach the reader's router.
            let stream = timeout(Duration::from_secs(240), async {
                loop {
                    match t.connect(&addr).await {
                        Ok(stream) => return Ok(stream),
                        Err(error) if !t.is_alive() => return Err(error),
                        Err(_) => sleep(Duration::from_secs(2)).await,
                    }
                }
            })
            .await
            .context("timed out connecting to I2P destination")??;
            read_rows(
                gate.noise_initiator(&key, stream, &policy).await?,
                invite,
                query,
            )
            .await
        },
    }
}
