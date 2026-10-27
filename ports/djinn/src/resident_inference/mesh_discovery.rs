// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Exchange attested compute endpoints over the existing personal graph link.
//! Endpoint hints follow proof of current graph authority in both directions.

use crate::sync::personal_sync_host::PersonalSyncHost;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::io::AsyncWriteExt;
use transport::{Alpn, P2pandaTransport, PeerID, Transport};

const ALPN: &str = "mere/personal-inference-mesh/v1";
const LIMIT: u32 = 16 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    mesh: [u8; 32],
    source: [u8; 32],
    target: [u8; 32],
    ticket: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    identity: Identity,
    proof: insigne::DerivedKeyAttestation,
    signature: Vec<u8>,
}

impl Packet {
    fn signed(
        identity: Identity,
        proof: &insigne::DerivedKeyAttestation,
        key: &personae::Ed25519Keypair,
    ) -> Result<Self, String> {
        let bytes = Self::signing_bytes(&identity)?;
        Ok(Self {
            identity,
            proof: proof.clone(),
            signature: key.sign(&bytes).to_bytes().to_vec(),
        })
    }
    fn signing_bytes(identity: &Identity) -> Result<Vec<u8>, String> {
        let mut bytes = b"mere.djinn/inference-mesh-discovery/v1\0".to_vec();
        bytes.extend(serde_json::to_vec(identity).map_err(|e| e.to_string())?);
        Ok(bytes)
    }
    fn verify(&self, host: &PersonalSyncHost, peer: PeerID) -> Result<[u8; 32], String> {
        self.verify_facts(
            super::compute_space(host.graph()),
            host.node_id(),
            peer,
            &|root| host.seals().current_reader(root),
        )
    }
    fn verify_facts(
        &self,
        mesh: [u8; 32],
        local: [u8; 32],
        peer: PeerID,
        current_reader: &dyn Fn(&[u8; 32]) -> bool,
    ) -> Result<[u8; 32], String> {
        if self.identity.mesh != mesh
            || self.identity.source != peer.to_bytes()
            || self.identity.target != local
        {
            return Err("compute discovery carrier or graph mismatch".into());
        }
        let author = *self.proof.derived();
        if !mesh::attests(author, &self.proof) || !current_reader(self.proof.master()) {
            return Err("compute discovery root is not a current keyed reader".into());
        }
        let bytes: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| "compute discovery signature size")?;
        let key = personae::Ed25519PublicKey::from_bytes(&author).map_err(|e| e.to_string())?;
        if !key.verify(
            &Self::signing_bytes(&self.identity)?,
            &personae::Ed25519Signature::from_bytes(&bytes),
        ) {
            return Err("compute discovery signature invalid".into());
        }
        Ok(author)
    }
}

struct Bridge {
    host: Arc<PersonalSyncHost>,
    transport: Arc<P2pandaTransport>,
    proof: insigne::DerivedKeyAttestation,
    signer: personae::Ed25519Keypair,
}
impl Bridge {
    async fn packet(&self, target: PeerID, include_endpoint: bool) -> Result<Packet, String> {
        if !self.host.seals().current_reader(self.proof.master()) {
            return Err("compute discovery local graph key unavailable".into());
        }
        let ticket = if include_endpoint {
            Some(self.transport.ticket().await.map_err(|e| e.to_string())?)
        } else {
            None
        };
        Packet::signed(
            Identity {
                mesh: super::compute_space(self.host.graph()),
                source: self.host.node_id(),
                target: target.to_bytes(),
                ticket,
            },
            &self.proof,
            &self.signer,
        )
    }
    async fn import(&self, packet: &Packet, peer: PeerID) -> Result<(), String> {
        let author = packet.verify(&self.host, peer)?;
        let ticket = packet
            .identity
            .ticket
            .as_ref()
            .ok_or("compute endpoint missing")?;
        let remote = self
            .transport
            .add_peer_ticket(ticket)
            .await
            .map_err(|e| e.to_string())?;
        if remote.to_bytes() != author {
            return Err("compute endpoint differs from attested author".into());
        }
        self.transport
            .add_topics(
                remote,
                &[transport::sync_overlay_topic(super::compute_space(
                    self.host.graph(),
                ))],
            )
            .await
            .map_err(|e| e.to_string())
    }
    async fn serve(
        &self,
        accepted: transport::AcceptedSession<transport::P2pandaStream>,
    ) -> Result<(), String> {
        if !accepted.is_transport_authenticated() || accepted.protocol != Alpn::new(ALPN) {
            return Err("compute discovery carrier missing".into());
        }
        let peer = accepted.peer.ok_or("compute discovery peer missing")?;
        let mut stream = accepted.stream;
        let packet = read(&mut stream).await?;
        packet.verify(&self.host, peer)?;
        if packet.identity.ticket.is_some() {
            return Err("compute discovery endpoint precedes mutual authority".into());
        }
        write(&mut stream, &self.packet(peer, true).await?).await?;
        let ack = read(&mut stream).await?;
        if ack.proof != packet.proof {
            return Err("compute discovery author changed".into());
        }
        self.import(&ack, peer).await?;
        stream.shutdown().await.map_err(|e| e.to_string())
    }
    async fn discover(&self, peer: PeerID) -> Result<(), String> {
        let mut stream = self
            .host
            .stage_transport()
            .connect(peer, Alpn::new(ALPN))
            .await
            .map_err(|e| e.to_string())?;
        write(&mut stream, &self.packet(peer, false).await?).await?;
        let response = read(&mut stream).await?;
        self.import(&response, peer).await?;
        write(&mut stream, &self.packet(peer, true).await?).await?;
        stream.shutdown().await.map_err(|e| e.to_string())
    }
}

async fn read(stream: &mut transport::P2pandaStream) -> Result<Packet, String> {
    let bytes = notochord::read_frame(stream, LIMIT)
        .await
        .map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
async fn write(stream: &mut transport::P2pandaStream, packet: &Packet) -> Result<(), String> {
    let bytes = serde_json::to_vec(packet).map_err(|e| e.to_string())?;
    notochord::write_frame(stream, &bytes, LIMIT)
        .await
        .map_err(|e| e.to_string())
}

/// Owned by the works; shutdown unregisters this sibling before releasing its
/// graph/compute carriers. A fallback drop aborts both bounded loops.
pub(crate) struct StageNetwork {
    host: Arc<PersonalSyncHost>,
    tasks: Vec<tokio::task::JoinHandle<()>>,
}
impl StageNetwork {
    pub(crate) async fn open(
        host: Arc<PersonalSyncHost>,
        transport: Arc<P2pandaTransport>,
        proof: insigne::DerivedKeyAttestation,
        signer: personae::Ed25519Keypair,
        settings: &distillery::inference_protocol::AutomaticStageSettings,
    ) -> Result<Self, String> {
        if !mesh::attests(signer.public_key().to_bytes(), &proof) {
            return Err("compute author attestation mismatch".into());
        }
        let timeout = Duration::from_millis(settings.discovery_timeout_ms);
        host.stage_transport()
            .register_bounded_alpn(Alpn::new(ALPN), 4, timeout)
            .await
            .map_err(|e| e.to_string())?;
        let bridge = Arc::new(Bridge {
            host: host.clone(),
            transport,
            proof,
            signer,
        });
        let accepting = bridge.clone();
        let accept = tokio::spawn(async move {
            while let Ok((session, arrived)) = accepting
                .host
                .stage_transport()
                .accept_observed(Alpn::new(ALPN))
                .await
            {
                let _ = tokio::time::timeout_at(arrived + timeout, accepting.serve(session)).await;
            }
        });
        let max_peers = settings.max_peers;
        let cadence = Duration::from_millis(settings.discovery_every_ms);
        let discover = tokio::spawn(async move {
            let mut offset = 0;
            loop {
                if let Ok(peers) = bridge.host.known_peers().await {
                    let count = peers.len();
                    let mut tasks = tokio::task::JoinSet::new();
                    for peer in peers.iter().cycle().skip(offset).take(max_peers.min(count)) {
                        let bridge = bridge.clone();
                        let peer = peer.peer;
                        tasks.spawn(async move {
                            let _ = tokio::time::timeout(timeout, bridge.discover(peer)).await;
                        });
                    }
                    while tasks.join_next().await.is_some() {}
                    if count > 0 {
                        offset = (offset + max_peers) % count;
                    }
                }
                tokio::time::sleep(cadence).await;
            }
        });
        Ok(Self {
            host,
            tasks: vec![accept, discover],
        })
    }
    /// Infallible, so closing the resident Distillery always reaches the
    /// works' own shutdown after this.
    pub(crate) async fn close(mut self) {
        for task in &self.tasks {
            task.abort();
        }
        for task in self.tasks.drain(..) {
            let _ = task.await;
        }
        self.host.stage_transport().stop_accepting(&Alpn::new(ALPN));
    }
}
impl Drop for StageNetwork {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
        self.host.stage_transport().stop_accepting(&Alpn::new(ALPN));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use personae::{Ed25519Keypair, IdentityProvider, InMemoryProvider};

    #[test]
    fn discovery_proof_binds_current_reader_carriers_graph_and_endpoint() {
        let identity = InMemoryProvider::from_seed([91; 32]);
        let signer = identity.derive_keypair(mesh::MESH_AUTHOR_SALT).unwrap();
        let proof = identity.attest_derived_key(mesh::MESH_AUTHOR_SALT).unwrap();
        let source = Ed25519Keypair::from_seed([92; 32]).public_key().to_bytes();
        let target = Ed25519Keypair::from_seed([93; 32]).public_key().to_bytes();
        let peer = PeerID::from_bytes(&source).unwrap();
        let mesh = super::super::compute_space([94; 32]);
        let mut packet = Packet::signed(
            Identity {
                mesh,
                source,
                target,
                ticket: Some("endpoint-a".into()),
            },
            &proof,
            &signer,
        )
        .unwrap();
        let reader = |root: &[u8; 32]| *root == *proof.master();
        assert_eq!(
            packet.verify_facts(mesh, target, peer, &reader).unwrap(),
            signer.public_key().to_bytes()
        );
        assert!(
            packet.verify_facts(mesh, target, peer, &|_| false).is_err(),
            "retirement must take effect on the next exchange"
        );
        assert!(packet.verify_facts([0; 32], target, peer, &reader).is_err());
        assert!(packet.verify_facts(mesh, source, peer, &reader).is_err());
        assert!(
            packet
                .verify_facts(mesh, target, PeerID::from_bytes(&target).unwrap(), &reader)
                .is_err()
        );
        packet.identity.ticket = Some("endpoint-b".into());
        assert!(
            packet.verify_facts(mesh, target, peer, &reader).is_err(),
            "a hint cannot change under the old signature"
        );
    }
}
