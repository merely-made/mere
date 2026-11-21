// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use serde::Deserialize;
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use webrtc_carrier::{
    FingerprintRole,
    native::{self, LoopbackOfferer},
};
use wtransport::{ClientConfig, tls::Sha256Digest};

#[derive(Clone, Deserialize)]
struct BrowserInvite {
    address: SocketAddr,
    token: [u8; 32],
    fingerprint: [u8; 32],
    query: String,
}

fn config(html: &str) -> Result<BrowserInvite> {
    let (_, rest) = html
        .split_once("application/octet-stream\">")
        .context("invitation missing")?;
    Ok(serde_json::from_slice(
        &STANDARD.decode(rest.split('<').next().unwrap())?,
    )?)
}

async fn request(invite: &BrowserInvite, query: &str) -> Result<Value> {
    let endpoint = Endpoint::client(
        ClientConfig::builder()
            .with_bind_default()
            .with_server_certificate_hashes([Sha256Digest::new(invite.fingerprint)])
            .build(),
    )?;
    let signal = endpoint
        .connect(format!("https://{}/offer", invite.address))
        .await?;
    let mut peer = LoopbackOfferer::create(LABEL).await;
    let (mut tx, mut rx) = signal.open_bi().await?.await?;
    tx.write_all(&invite.token).await?;
    write_frame(&mut tx, peer.offer.as_bytes()).await?;
    let answer = read_frame(&mut rx, MAX_SDP).await?;
    peer.accept_answer(std::str::from_utf8(&answer)?);
    tx.write_all(&[1]).await?;
    let carrier = native::serve(
        peer.rtc,
        peer.socket,
        CarrierConfig {
            channel_label: LABEL.to_owned(),
            local_dtls_role: FingerprintRole::Client,
            ..Default::default()
        },
    )
    .await?;
    let (reader, writer, control) = carrier.into_parts();
    let (mut stream, _pump) = stream_over_frames(reader, writer);
    write_frame(&mut stream, query.as_bytes()).await?;
    let reply = serde_json::from_slice(&read_frame(&mut stream, MAX_REPLY).await?)?;
    stream.write_u8(1).await?;
    control.join().await?;
    Ok(reply)
}

async fn start_owner(query: &str) -> Result<(BrowserInvite, tokio::task::JoinHandle<Result<()>>)> {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("query.html");
    let graph = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../graph/engine/examples/fixtures/paper-authors.trig");
    let service = Service::new(crate::dataset(&graph)?, query.to_owned())?;
    let file = path.clone();
    let owner = tokio::spawn(async move { serve("127.0.0.1:0".parse()?, &file, service).await });
    while !path.exists() {
        ensure!(!owner.is_finished(), "owner exited");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let html = std::fs::read_to_string(path)?;
    assert!(!html.contains("</script><test>"));
    Ok((config(&html)?, owner))
}

#[tokio::test(flavor = "multi_thread")]
async fn browser_invitation_authenticates_owner_and_limits_queries() -> Result<()> {
    let paper_query = include_str!("../../../graph/engine/examples/fixtures/paper-authors.rq");
    let query = format!("# </script><test>\n{paper_query}");
    let (invite, owner) = timeout(SETUP, start_owner(&query)).await??;
    assert_eq!(invite.query, query);
    let mut wrong = invite.clone();
    wrong.fingerprint[0] ^= 1;
    assert!(timeout(SETUP, request(&wrong, &query)).await?.is_err());
    let mut wrong = invite.clone();
    wrong.token[0] ^= 1;
    assert!(timeout(SETUP, request(&wrong, &query)).await?.is_err());
    let denied = timeout(SETUP, request(&invite, "ASK {}")).await??;
    assert_eq!(
        denied,
        json!({"error":"query is not enabled by this invitation"})
    );
    let reply = timeout(SETUP, request(&invite, &query)).await??;
    assert_eq!(
        reply,
        json!({"columns":["author","namedBy"],"rows":[["Ada","Library catalog"],["Ada","Publisher"],["Grace","Publisher"]]})
    );
    owner.abort();
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn browser_reply_spans_multiple_carrier_frames() -> Result<()> {
    let value = "x".repeat(4096);
    let indices = (0..24).map(|n| n.to_string()).collect::<Vec<_>>().join(" ");
    let query = format!(
        r#"SELECT ?text ?missing WHERE {{ VALUES ?i {{ {indices} }} BIND("{value}" AS ?text) }}"#
    );
    let (invite, owner) = timeout(SETUP, start_owner(&query)).await??;
    let reply = timeout(Duration::from_secs(30), request(&invite, &query)).await??;
    assert_eq!(reply["columns"], json!(["text", "missing"]));
    assert_eq!(reply["rows"], json!(vec![json!([value, null]); 24]));
    owner.abort();
    Ok(())
}
