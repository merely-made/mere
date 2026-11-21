// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A self-contained HTML invitation pins the temporary WebTransport signaling
//! certificate. The authenticated SDP exchange binds the WebRTC DTLS peers.
//! Only WebRTC carries the SPARQL request and the result.

use std::{net::SocketAddr, path::Path, sync::Arc, time::Duration};

use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::json;
use tokio::{io::AsyncReadExt, task::JoinSet, time::timeout};
use webrtc_carrier::native::{Answerer, AnswererConfig, CarrierConfig, stream_over_frames};
use wtransport::{Endpoint, Identity, ServerConfig, endpoint::IncomingSession};

use crate::protocol::{self, MAX_QUERY, MAX_REPLY, Service, read_frame, write_frame};

const LABEL: &str = "mere/sparql-browser/v1";
const MAX_SDP: usize = 32 * 1024;
const SETUP: Duration = Duration::from_secs(15);

fn invitation(address: SocketAddr, fingerprint: &[u8; 32], service: &Service) -> Result<String> {
    // Encode all variable data outside the HTML/JS grammar, including arbitrary SPARQL literals.
    let config = json!({
        "address": address.to_string(), "fingerprint": fingerprint,
        "token": service.token(), "query": service.allowed_query(), "label": LABEL,
    });
    Ok(include_str!("webrtc/reader.html")
        .replace("MERE_READER_SCRIPT", include_str!("webrtc/reader.js"))
        .replace(
            "MERE_INVITATION_DATA",
            &STANDARD.encode(serde_json::to_vec(&config)?),
        ))
}

pub async fn serve(listen: SocketAddr, file: &Path, service: Service) -> Result<()> {
    ensure!(
        file.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("html")),
        "use an .html filename for the browser invitation"
    );
    ensure!(
        !listen.ip().is_unspecified(),
        "--listen must name an address the reader can reach"
    );
    let identity = Identity::self_signed([listen.ip().to_string()])?;
    let fingerprint = identity.certificate_chain().as_slice()[0].hash();
    let config = ServerConfig::builder()
        .with_bind_address(listen)
        .with_identity(identity)
        .max_idle_timeout(Some(Duration::from_secs(30)))?
        .build();
    let endpoint = Endpoint::server(config).context("bind WebRTC signaling UDP socket")?;
    let address = endpoint.local_addr()?;
    let html = invitation(address, fingerprint.as_ref(), &service)?;
    protocol::write_invitation(file, html.as_bytes())?;
    let service = Arc::new(service);
    let mut sessions = JoinSet::new();
    loop {
        tokio::select! {
            incoming = endpoint.accept() => {
                if sessions.len() >= 8 {
                    incoming.refuse();
                    continue;
                }
                let service = Arc::clone(&service);
                sessions.spawn(async move {
                    if let Err(error) = session(incoming, address, &service).await {
                        eprintln!("Browser session ended: {error}");
                    }
                });
            }
            Some(result) = sessions.join_next(), if !sessions.is_empty() => { result?; }
        }
    }
}

async fn session(incoming: IncomingSession, address: SocketAddr, service: &Service) -> Result<()> {
    let answerer = timeout(SETUP, async {
        let request = incoming.await?;
        if request.path() != "/offer" {
            request.forbidden().await;
            anyhow::bail!("unknown signaling path");
        }
        let connection = request.accept().await?;
        let (mut send, mut recv) = connection.accept_bi().await?;
        let mut token = [0; 32];
        recv.read_exact(&mut token).await?;
        ensure!(service.matches_token(&token), "invalid invitation");
        let offer = read_frame(&mut recv, MAX_SDP).await?;
        let mut answerer = Answerer::bind(AnswererConfig {
            bind: SocketAddr::new(address.ip(), 0),
            carrier: CarrierConfig {
                channel_label: LABEL.to_owned(),
                ..Default::default()
            },
            ..Default::default()
        })
        .await?;
        let answer = answerer.answer(std::str::from_utf8(&offer)?)?;
        ensure!(answer.len() <= MAX_SDP, "answer too large");
        write_frame(&mut send, answer.as_bytes()).await?;
        send.finish().await?;
        // The browser confirms that it read the complete SDP before we close QUIC.
        let mut receipt = [0];
        recv.read_exact(&mut receipt).await?;
        ensure!(receipt == [1], "missing signaling receipt");
        Ok::<_, anyhow::Error>(answerer)
    })
    .await
    .context("browser signaling timed out")??;

    let channel = answerer.accept().await?;
    let (reader, writer, control) = channel.into_parts();
    let (mut stream, pump) = stream_over_frames(reader, writer);
    let result = timeout(Duration::from_secs(90), async {
        let query = read_frame(&mut stream, MAX_QUERY).await?;
        let reply = match service.select_allowed(&query) {
            Ok(result) => json!({"columns": result.variables, "rows": result.rows}),
            Err(error) => json!({"error":error.to_string()}),
        };
        let mut bytes = serde_json::to_vec(&reply)?;
        if bytes.len() > MAX_REPLY {
            bytes = br#"{"error":"result exceeds 1 MiB"}"#.to_vec();
        }
        write_frame(&mut stream, &bytes).await?;
        ensure!(stream.read_u8().await? == 1, "missing result receipt");
        Ok::<_, anyhow::Error>(())
    })
    .await
    .context("browser query timed out");
    drop(stream);
    control.close().await?;
    pump.await?;
    result?
}

#[cfg(test)]
mod tests;
