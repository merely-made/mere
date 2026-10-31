// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Host-owned protection for retained workshop records. This seam holds no
//! keys. Production sealing is delegated to the current custody PayloadSealer.

use crate::workshop::Error;

/// Deliberately below the custody route's 512 KiB clear-payload ceiling, leaving
/// room for authenticated context. Public workshop retention is independent.
pub const MAX_SEALED_CLEAR: usize = 384 * 1024;

/// Context must be authenticated together with the clear bytes. An owner checks
/// its live custody generation around async work and serializes effect admission;
/// a positive `is_open` sample alone is not a graph transaction lease.
pub trait Protection: Send + Sync {
    fn epoch(&self) -> Option<u64>;
    fn is_open(&self) -> bool {
        self.epoch().is_some()
    }
    fn seal(&self, context: &[u8], clear: &[u8]) -> Result<Vec<u8>, Error>;
    fn open(&self, context: &[u8], sealed: &[u8]) -> Result<Vec<u8>, Error>;
}

#[cfg(feature = "workshop-sealed")]
mod eidetic_adapter {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    const PREFIX: &[u8] = b"alembic-sealed-v1\0";
    const MAX_ENVELOPE: usize = 1024 * 1024;

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Envelope {
        schema: u32,
        hash: String,
        marker: eidetic::SealedBlobRef,
        ciphertext: String,
    }

    /// The epoch callback returns None while custody is locked/unavailable and
    /// changes whenever its profile or custody binding changes. It is not a key.
    pub struct EideticProtection<S: ?Sized, G> {
        sealer: Arc<S>,
        epoch: G,
    }

    impl<S: ?Sized, G> EideticProtection<S, G> {
        pub fn new(sealer: Arc<S>, epoch: G) -> Self {
            Self { sealer, epoch }
        }
    }

    fn unavailable() -> Error {
        Error::Invalid("sealed workshop custody unavailable or changed")
    }

    fn encode_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn decode_hex(text: &str) -> Result<Vec<u8>, Error> {
        if text.len() % 2 != 0 || text.len() > MAX_ENVELOPE {
            return Err(Error::Invalid("sealed workshop ciphertext size"));
        }
        text.as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let nibble = |byte| match byte {
                    b'0'..=b'9' => Ok(byte - b'0'),
                    b'a'..=b'f' => Ok(byte - b'a' + 10),
                    _ => Err(Error::Invalid("sealed workshop ciphertext encoding")),
                };
                Ok((nibble(pair[0])? << 4) | nibble(pair[1])?)
            })
            .collect()
    }

    impl<S, G> Protection for EideticProtection<S, G>
    where
        S: eidetic::PayloadSealer + Send + Sync + ?Sized,
        G: Fn() -> Option<u64> + Send + Sync,
    {
        fn epoch(&self) -> Option<u64> {
            (self.epoch)()
        }

        fn seal(&self, context: &[u8], clear: &[u8]) -> Result<Vec<u8>, Error> {
            let epoch = (self.epoch)().ok_or_else(unavailable)?;
            if clear.len() > MAX_SEALED_CLEAR || context.len() > 4096 {
                return Err(Error::Invalid("sealed workshop cleartext bound"));
            }
            let mut bound = Vec::with_capacity(PREFIX.len() + 32 + clear.len());
            bound.extend_from_slice(PREFIX);
            bound.extend_from_slice(blake3::hash(context).as_bytes());
            bound.extend_from_slice(clear);
            let hash = eidetic::Hash::of(&bound);
            let (ciphertext, marker) =
                self.sealer.seal(&hash, &bound).map_err(|_| unavailable())?;
            if (self.epoch)() != Some(epoch) {
                return Err(unavailable());
            }
            let value = serde_json::to_vec(&Envelope {
                schema: 1,
                hash: hash.to_string(),
                marker,
                ciphertext: encode_hex(&ciphertext),
            })
            .map_err(|_| Error::Invalid("sealed workshop envelope encoding"))?;
            if value.len() > MAX_ENVELOPE {
                return Err(Error::Invalid("sealed workshop envelope bound"));
            }
            Ok(value)
        }

        fn open(&self, context: &[u8], sealed: &[u8]) -> Result<Vec<u8>, Error> {
            let epoch = (self.epoch)().ok_or_else(unavailable)?;
            if sealed.len() > MAX_ENVELOPE || context.len() > 4096 {
                return Err(Error::Invalid("sealed workshop envelope bound"));
            }
            let envelope: Envelope = serde_json::from_slice(sealed)
                .map_err(|_| Error::Invalid("sealed workshop envelope schema"))?;
            if envelope.schema != 1 {
                return Err(Error::Invalid("sealed workshop envelope version"));
            }
            let hash = eidetic::Hash::parse(&envelope.hash)
                .map_err(|_| Error::Invalid("sealed workshop content hash"))?;
            let ciphertext = decode_hex(&envelope.ciphertext)?;
            let bound = self
                .sealer
                .unseal(&hash, &envelope.marker, &ciphertext)
                .map_err(|_| unavailable())?;
            if (self.epoch)() != Some(epoch) {
                return Err(unavailable());
            }
            let header = PREFIX.len() + 32;
            if bound.len() < header
                || bound.len() - header > MAX_SEALED_CLEAR
                || eidetic::Hash::of(&bound) != hash
                || !bound.starts_with(PREFIX)
                || &bound[PREFIX.len()..header] != blake3::hash(context).as_bytes()
            {
                return Err(Error::Invalid("sealed workshop context or integrity"));
            }
            Ok(bound[header..].to_vec())
        }
    }
}

#[cfg(feature = "workshop-sealed")]
pub use eidetic_adapter::EideticProtection;

#[cfg(all(test, feature = "workshop-sealed"))]
mod tests {
    use super::*;
    use alembic::workshop::{Owner, Source};
    use chacha20poly1305::{
        ChaCha20Poly1305, KeyInit, Nonce,
        aead::{Aead, Payload},
    };
    use eidetic::{PayloadSealer, SealEpochId, SealedBlobRef};
    use muniment::{Backend, MemoryBackend};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    };

    // An actual authenticated cipher with synthetic keys. This tests the
    // adapter contract; it does not qualify a production custody process.
    struct TestSealer {
        epoch: Arc<AtomicU64>,
        nonce: AtomicU64,
        lock_after_seal: AtomicBool,
    }
    impl PayloadSealer for TestSealer {
        fn seal(
            &self,
            hash: &eidetic::Hash,
            clear: &[u8],
        ) -> eidetic::Result<(Vec<u8>, SealedBlobRef)> {
            let mut nonce = [0; 12];
            nonce[4..].copy_from_slice(&self.nonce.fetch_add(1, Ordering::SeqCst).to_be_bytes());
            let cipher = ChaCha20Poly1305::new_from_slice(&[0x71; 32]).unwrap();
            let aad = hash.to_string();
            let ciphertext = cipher
                .encrypt(
                    Nonce::from_slice(&nonce),
                    Payload {
                        msg: clear,
                        aad: aad.as_bytes(),
                    },
                )
                .map_err(|_| eidetic::Error::new("test sealing failed"))?;
            let mut bytes = nonce.to_vec();
            bytes.extend(ciphertext);
            if self.lock_after_seal.swap(false, Ordering::SeqCst) {
                self.epoch.store(0, Ordering::SeqCst);
            }
            Ok((
                bytes,
                SealedBlobRef {
                    epoch: SealEpochId([7; 16]),
                    format: "test.chacha20poly1305.v1".into(),
                },
            ))
        }
        fn unseal(
            &self,
            hash: &eidetic::Hash,
            marker: &SealedBlobRef,
            sealed: &[u8],
        ) -> eidetic::Result<Vec<u8>> {
            if marker.epoch != SealEpochId([7; 16])
                || marker.format != "test.chacha20poly1305.v1"
                || sealed.len() < 12
            {
                return Err(eidetic::Error::new("test envelope refused"));
            }
            let cipher = ChaCha20Poly1305::new_from_slice(&[0x71; 32]).unwrap();
            let aad = hash.to_string();
            cipher
                .decrypt(
                    Nonce::from_slice(&sealed[..12]),
                    Payload {
                        msg: &sealed[12..],
                        aad: aad.as_bytes(),
                    },
                )
                .map_err(|_| eidetic::Error::new("test authentication failed"))
        }
    }
    fn fixture() -> (Arc<TestSealer>, Arc<dyn Protection>) {
        let epoch = Arc::new(AtomicU64::new(1));
        let sealer = Arc::new(TestSealer {
            epoch: epoch.clone(),
            nonce: AtomicU64::new(1),
            lock_after_seal: AtomicBool::new(false),
        });
        let protection = Arc::new(EideticProtection::new(sealer.clone(), move || {
            let value = epoch.load(Ordering::SeqCst);
            (value != 0).then_some(value)
        }));
        (sealer, protection)
    }

    #[test]
    fn sealed_adapter_authenticates_context_tamper_bounds_and_custody_epoch() {
        let (sealer, protection) = fixture();
        let canary = b"private workshop canary / caf\xc3\xa9 / 28";
        let first = protection
            .seal(b"owner-A/install-A/journal", canary)
            .unwrap();
        let second = protection
            .seal(b"owner-A/install-A/journal", canary)
            .unwrap();
        assert_ne!(
            first, second,
            "fresh ciphertext must not become CAS identity"
        );
        assert!(!first.windows(canary.len()).any(|window| window == canary));
        assert_eq!(
            protection
                .open(b"owner-A/install-A/journal", &first)
                .unwrap(),
            canary
        );
        assert!(
            protection
                .open(b"owner-B/install-A/journal", &first)
                .is_err()
        );
        assert!(protection.open(b"owner-A/install-A/note", &first).is_err());
        let mut tampered: serde_json::Value = serde_json::from_slice(&first).unwrap();
        let text = tampered["ciphertext"].as_str().unwrap();
        tampered["ciphertext"] = format!(
            "{}{}",
            if &text[..1] == "0" { "1" } else { "0" },
            &text[1..]
        )
        .into();
        assert!(
            protection
                .open(
                    b"owner-A/install-A/journal",
                    &serde_json::to_vec(&tampered).unwrap()
                )
                .is_err()
        );
        assert!(
            protection
                .seal(b"context", &vec![0; MAX_SEALED_CLEAR + 1])
                .is_err()
        );
        sealer.lock_after_seal.store(true, Ordering::SeqCst);
        assert!(protection.seal(b"context", canary).is_err());
        assert!(!protection.is_open());
        assert!(
            protection
                .open(b"owner-A/install-A/journal", &first)
                .is_err()
        );
        sealer.epoch.store(2, Ordering::SeqCst);
        assert_eq!(
            protection
                .open(b"owner-A/install-A/journal", &first)
                .unwrap(),
            canary
        );
    }

    struct Granted;
    impl servitor::AuthorityProvider for Granted {
        fn covers(&self, _: servitor::Subject, _: &servitor::Cap, _: servitor::Mode) -> bool {
            true
        }
    }

    #[test]
    fn private_workshop_seals_entire_journal_preserves_ciphertext_cas_and_reopens() {
        pollster::block_on(async {
            use crate::workshop::{
                Workshop, capture, capture_private, evaluate, validate_private_proposal,
            };
            let (sealer, protection) = fixture();
            let store = MemoryBackend::new();
            let owner = Owner {
                host: "private-test".into(),
                workspace: "graph-A".into(),
                session: "session-A".into(),
            };
            let canary = "private retained caf\u{e9} canary 20261028";
            let source = |name: &str, text: &str| Source {
                owner: owner.clone(),
                resource: name.into(),
                version: blake3::hash(text.as_bytes()).to_hex().to_string(),
                content_digest: [0; 32],
                start: 0,
                end: text.len() as u64,
                title: name.into(),
            };
            let inputs = vec![
                capture_private(source("private-title", canary), canary, protection.as_ref())
                    .unwrap(),
                capture(source("public-title", "public note"), "public note", true).unwrap(),
            ];
            assert!(capture(source("refused", canary), canary, false).is_err());
            let workshop =
                Workshop::new_sealed(store.clone(), owner.clone(), [5; 16], protection.clone());
            workshop.install([7; 32]).await.unwrap();
            workshop
                .begin(1, inputs.clone(), &Granted, 1)
                .await
                .unwrap();
            for step in 0..3 {
                let dispatch = workshop.dispatch(1, &Granted, 2 + step * 2).await.unwrap();
                workshop
                    .observe(evaluate(dispatch).unwrap(), &Granted, 3 + step * 2)
                    .await
                    .unwrap();
            }
            let expected = workshop.view(Some(1)).await.unwrap().proposal.unwrap();
            validate_private_proposal(&expected).unwrap();
            for key in store.list("").await.unwrap() {
                let bytes = store.get(&key).await.unwrap().unwrap();
                for secret in [canary.as_bytes(), b"private-title".as_slice()] {
                    assert!(
                        !bytes.windows(secret.len()).any(|window| window == secret),
                        "journal retained a private field in cleartext"
                    );
                }
            }
            let reopened =
                Workshop::new_sealed(store.clone(), owner.clone(), [5; 16], protection.clone());
            assert_eq!(
                reopened.view(Some(1)).await.unwrap().proposal,
                Some(expected.clone())
            );
            assert!(
                Workshop::new(store.clone(), owner.clone(), [5; 16])
                    .view(None)
                    .await
                    .is_err()
            );
            let public = Workshop::new(store.clone(), owner.clone(), [6; 16]);
            public.install([7; 32]).await.unwrap();
            assert!(public.begin(1, inputs, &Granted, 1).await.is_err());
            sealer.epoch.store(0, Ordering::SeqCst);
            assert!(reopened.view(Some(1)).await.is_err());
            sealer.epoch.store(2, Ordering::SeqCst);
            assert_eq!(
                reopened.view(Some(1)).await.unwrap().proposal,
                Some(expected)
            );
        });
    }
    #[cfg(feature = "workshop-destination")]
    #[test]
    fn accepted_private_note_outlives_dissolved_and_deleted_helper_journal() {
        pollster::block_on(async {
            use crate::workshop::{Workshop, capture_private, evaluate};
            use crate::workshop_destination::graph_revision;
            use crate::workshop_sealed_destination::{self as destination, Preparation};
            use alembic::workshop::{Control, Destination};
            use kernel::graph::{
                Graph,
                apply::{GraphDeltaResult, apply_graph_delta},
            };

            let (_, protection) = fixture();
            let store = MemoryBackend::new();
            let owner = Owner {
                host: "retained-note-test".into(),
                workspace: "graph".into(),
                session: "session".into(),
            };
            let canary = "private accepted café canary / independent reader";
            let inputs = (0..2)
                .map(|i| {
                    capture_private(
                        Source {
                            owner: owner.clone(),
                            resource: format!("retained-source-{i}"),
                            version: "captured-v1".into(),
                            content_digest: [0; 32],
                            start: 0,
                            end: canary.len() as u64,
                            title: "Private retained title".into(),
                        },
                        canary,
                        protection.as_ref(),
                    )
                    .unwrap()
                })
                .collect();
            let workshop =
                Workshop::new_sealed(store.clone(), owner.clone(), [6; 16], protection.clone());
            workshop.install([7; 32]).await.unwrap();
            workshop.begin(1, inputs, &Granted, 1).await.unwrap();
            for step in 0..3 {
                let dispatch = workshop.dispatch(1, &Granted, 2 + step * 2).await.unwrap();
                workshop
                    .observe(evaluate(dispatch).unwrap(), &Granted, 3 + step * 2)
                    .await
                    .unwrap();
            }
            let proposal = workshop.view(Some(1)).await.unwrap().proposal.unwrap();
            let mut graph = Graph::default();
            let acceptance = workshop
                .accept(
                    1,
                    Destination {
                        owner: owner.clone(),
                        graph: "graph".into(),
                        revision: graph_revision(&graph).unwrap(),
                    },
                    [8; 32],
                    &Granted,
                    10,
                )
                .await
                .unwrap();
            let Preparation::New {
                deltas, receipt, ..
            } = destination::prepare_graph(
                &graph,
                &owner,
                "graph",
                &acceptance,
                &proposal,
                [8; 32],
                &Granted,
                11,
                protection.as_ref(),
            )
            .unwrap()
            else {
                panic!("new effect expected")
            };
            for delta in deltas {
                assert!(!matches!(
                    apply_graph_delta(&mut graph, delta),
                    GraphDeltaResult::Refused(_)
                ));
            }
            destination::verify_graph_effect(
                &graph,
                &owner,
                "graph",
                &acceptance,
                &proposal,
                protection.as_ref(),
            )
            .unwrap();
            // This is a graph snapshot round trip, not a product filesystem receipt.
            let persisted = serde_json::to_vec(&(graph.to_snapshot_at(0), graph.facets())).unwrap();
            assert!(
                !persisted
                    .windows(canary.len())
                    .any(|bytes| bytes == canary.as_bytes())
            );
            workshop.acknowledge(1, receipt.clone()).await.unwrap();
            workshop
                .lifecycle(Control::Dissolve, &Granted, 12)
                .await
                .unwrap();
            drop(workshop);
            for key in store.list("").await.unwrap() {
                let bytes = store.get(&key).await.unwrap().unwrap();
                assert!(
                    !bytes
                        .windows(canary.len())
                        .any(|bytes| bytes == canary.as_bytes())
                );
                store.delete(&key).await.unwrap();
            }
            assert!(store.list("").await.unwrap().is_empty());
            let (snapshot, facets): (kernel::persistence::GraphSnapshot, _) =
                serde_json::from_slice(&persisted).unwrap();
            let mut reopened = Graph::try_from_recorded_snapshot(&snapshot).unwrap();
            reopened.install_facets(facets).unwrap();
            assert_eq!(
                destination::open_result_resource(
                    &reopened,
                    &owner,
                    "graph",
                    &receipt.resource,
                    protection.as_ref()
                )
                .unwrap(),
                (receipt, proposal.djot)
            );
        });
    }
}
