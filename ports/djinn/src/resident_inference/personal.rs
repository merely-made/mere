// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Inference's private log bound to the personal graph's existing group.

use std::{path::Path, sync::Arc};

use distillery::inference::{
    RunJournal,
    durable::{PersonaRunJournal, RunExt, RunGroupProtection},
};
use graphshell::native::{
    personal_seals::PersonalGraphSeals, personal_sync_host::PersonalSyncHost,
};
use muniment::RedbBackend;
use personae::IdentityProvider;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

/// Private inference topic, distinct from the graph and carriage topics.
pub fn inference_space(graph: [u8; 32]) -> [u8; 32] {
    let mut hash = blake3::Hasher::new();
    hash.update(b"mere.djinn/personal-inference/v1");
    hash.update(&graph);
    *hash.finalize().as_bytes()
}

fn writer_salt(graph: [u8; 32]) -> Vec<u8> {
    let mut salt = b"mere.djinn/personal-inference/writer/v1/".to_vec();
    salt.extend_from_slice(&graph);
    salt
}

/// Compact version-one public attestation: tag, master, derived key, signature.
/// Keeping it fixed at 129 bytes also fits p2panda's header collection bounds.
/// Verification remains Insigne's domain-bound attestation check.
pub fn writer_authority(proof: &insigne::DerivedKeyAttestation) -> Result<Vec<u8>, String> {
    if proof.signature().len() != 64 {
        return Err("inference writer requires an Ed25519 attestation".into());
    }
    let mut bytes = Vec::with_capacity(129);
    bytes.push(1);
    bytes.extend_from_slice(proof.master());
    bytes.extend_from_slice(proof.derived());
    bytes.extend_from_slice(proof.signature());
    Ok(bytes)
}

fn writer_proof(bytes: &[u8]) -> Result<insigne::DerivedKeyAttestation, String> {
    if bytes.len() != 129 || bytes[0] != 1 {
        return Err("inference writer has no valid root attestation".into());
    }
    Ok(insigne::DerivedKeyAttestation::from_parts(
        bytes[1..33].try_into().unwrap(),
        bytes[33..65].try_into().unwrap(),
        bytes[65..].to_vec(),
    ))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Protected {
    domain: String,
    address: RunExt,
    payload: Vec<u8>,
}

impl Drop for Protected {
    fn drop(&mut self) {
        self.payload.zeroize();
    }
}

struct Protection {
    graph: [u8; 32],
    seals: PersonalGraphSeals,
}

impl RunGroupProtection for Protection {
    fn can_write(&self) -> bool {
        self.seals.can_write()
    }
    fn revision(&self) -> u64 {
        self.seals.revision()
    }

    fn seal(&self, address: &RunExt, clear: &[u8]) -> Result<Vec<u8>, String> {
        let protected = Protected {
            domain: "mere.djinn/personal-inference/v1".into(),
            address: address.clone(),
            payload: clear.to_vec(),
        };
        let encoded =
            Zeroizing::new(serde_json::to_vec(&protected).map_err(|error| error.to_string())?);
        let sealed = self.seals.seal(&encoded)?;
        serde_json::to_vec(&sealed).map_err(|error| error.to_string())
    }

    fn admit(&self, writer: [u8; 32], address: &RunExt, sealed: &[u8]) -> Result<Vec<u8>, String> {
        let proof = writer_proof(&address.authority)?;
        let root =
            stickleback::stable_writer_subject(writer, Some(&proof), &writer_salt(self.graph))
                .map_err(|error| error.to_string())?;
        let envelope = serde_json::from_slice(sealed).map_err(|error| error.to_string())?;
        if !self.seals.admits(&root, &envelope) {
            return Err(
                "inference writer is outside the personal graph key group for this epoch".into(),
            );
        }
        self.open(writer, address, sealed)
    }

    fn open(&self, writer: [u8; 32], address: &RunExt, sealed: &[u8]) -> Result<Vec<u8>, String> {
        if address.space != inference_space(self.graph) {
            return Err("inference checkpoint belongs to another personal graph".into());
        }
        let proof = writer_proof(&address.authority)?;
        stickleback::stable_writer_subject(writer, Some(&proof), &writer_salt(self.graph))
            .map_err(|error| error.to_string())?;
        let sealed = serde_json::from_slice(sealed).map_err(|error| error.to_string())?;
        let clear = Zeroizing::new(self.seals.open(&sealed)?);
        let mut protected: Protected =
            serde_json::from_slice(&clear).map_err(|error| error.to_string())?;
        if protected.domain != "mere.djinn/personal-inference/v1" || &protected.address != address {
            return Err(
                "inference checkpoint authentication addresses another domain or log".into(),
            );
        }
        Ok(std::mem::take(&mut protected.payload))
    }
}

/// Prepare a journal before starting its actor. The graph host retains every
/// group key; the journal gets a live, non-exporting protection handle.
pub fn open<P: IdentityProvider + ?Sized>(
    identity: &P,
    root: &Path,
    host: &PersonalSyncHost,
    local: Arc<PersonaRunJournal<RedbBackend>>,
    local_key: [u8; 32],
) -> Result<Arc<PersonaRunJournal<RedbBackend>>, String> {
    let graph = host.graph();
    let author = identity
        .derive_keypair(&writer_salt(graph))
        .map_err(|error| error.to_string())?;
    let proof = identity
        .attest_derived_key(&writer_salt(graph))
        .map_err(|error| error.to_string())?;
    let authority = writer_authority(&proof)?;
    let backend = RedbBackend::open(root.join(format!(
        "inference-personal-{}.redb",
        crate::settings::hex32(&graph)
    )))
    .map_err(|error| error.to_string())?;
    Ok(Arc::new(PersonaRunJournal::for_group(
        backend,
        inference_space(graph),
        &author,
        local.device().to_string(),
        local_key,
        authority,
        Arc::new(Protection {
            graph,
            seals: host.seals(),
        }),
        Some(local),
    )))
}

/// Add this private sibling lane without replacing the graph or carriage tags.
pub async fn join(
    host: &PersonalSyncHost,
    journal: &Arc<PersonaRunJournal<RedbBackend>>,
) -> Result<(), String> {
    let (endpoint, gossip) = host
        .sync_parts()
        .ok_or("personal graph gossip is unavailable")?;
    journal.join(endpoint, gossip).await?;
    if let Err(error) = host
        .register_sibling_topic(transport::sync_overlay_topic(inference_space(host.graph())))
        .await
    {
        journal.shutdown().await?;
        return Err(error.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_attestation_round_trips_through_the_real_header_codec() {
        let identity = personae::InMemoryProvider::from_seed([0x72; 32]);
        let salt = writer_salt([0x73; 32]);
        let proof = identity.attest_derived_key(&salt).unwrap();
        let bytes = writer_authority(&proof).unwrap();
        assert_eq!(bytes.len(), 129);
        assert_eq!(
            writer_proof(&bytes).unwrap().check(&salt).unwrap().master(),
            proof.master()
        );
        let key = p2panda_core::SigningKey::from_bytes(
            &identity.derive_keypair(&salt).unwrap().to_seed(),
        );
        let header = p2panda_core::Header::builder()
            .body(b"sealed fixture")
            .build(
                &key,
                RunExt {
                    space: [0x74; 32],
                    device: [0x75; 32],
                    run: [0x76; 32],
                    authority: bytes.clone(),
                },
            );
        let decoded = p2panda_core::Header::<RunExt>::decode(&header.encode()).unwrap();
        assert_eq!(decoded.hash(), header.hash());
        assert_eq!(decoded.extensions.authority, bytes);
        assert!(writer_proof(&bytes[..128]).is_err());
        let mut future = bytes;
        future[0] = 2;
        assert!(writer_proof(&future).is_err());
    }
}
