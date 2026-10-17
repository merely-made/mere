// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resident-owned placement: attested keyed peers, live offers and real leases.

use super::*;
use esp::infer::decoder::stage::{StageCapacity, assign_ranges};
use esp::infer::library::ResolvedInferenceRow;
use std::sync::atomic::AtomicU64;
use tokio::sync::{mpsc, oneshot};

pub use crate::inference_protocol::AutomaticStageSettings;

impl AutomaticStageSettings {
    pub fn transport(&self, requires: mesh::ResourceRequirements) -> StageSettings {
        StageSettings {
            max_hidden_bytes: self.max_hidden_bytes,
            call_timeout: Duration::from_millis(self.call_timeout_ms),
            poll_every: Duration::from_millis(self.poll_every_ms),
            requires,
        }
    }
}

struct PostStages {
    stages: Vec<([u8; 32], StageRequest, mesh::ResourceRequirements)>,
    terms: mesh::LeaseTerms,
    until: std::time::Instant,
    reply: oneshot::Sender<Result<Vec<JobId>, String>>,
}

struct Reservations {
    transport: Arc<P2pandaTransport>,
    runtime: tokio::runtime::Handle,
    timeout: Duration,
    peers: Vec<(PeerID, StageRequest)>,
    armed: bool,
}
impl Drop for Reservations {
    fn drop(&mut self) {
        if !self.armed || self.peers.is_empty() {
            return;
        }
        let peers = std::mem::take(&mut self.peers);
        let transport = self.transport.clone();
        let timeout = self.timeout;
        self.runtime.spawn(async move {
            for (peer, request) in peers {
                let _ = discovery::query(
                    &transport,
                    peer,
                    request,
                    StageOperation::ReleaseReservation,
                    timeout,
                )
                .await;
            }
        });
    }
}

/// Kept by the authority and inference actor. The snapshot is read-only; only
/// `drive_posts` authors operations, from the authority's serialized tick.
pub struct StagePlanner {
    transport: Arc<P2pandaTransport>,
    mesh: [u8; 32],
    signer: personae::Ed25519Keypair,
    clock: Arc<dyn Clock>,
    policy: mesh::LeasePolicy,
    readers: Arc<StageReaders>,
    board: RwLock<JobBoard>,
    settings: AutomaticStageSettings,
    runtime: tokio::runtime::Handle,
    sender: mpsc::Sender<PostStages>,
    receiver: Mutex<mpsc::Receiver<PostStages>>,
    nonce: AtomicU64,
    last_unavailable: Mutex<Option<String>>,
}

impl RemoteSessionProjection for StagePlanner {
    fn refresh(&self, board: JobBoard) {
        *self.board.write().unwrap() = board;
    }
}

impl StagePlanner {
    pub fn new(
        transport: Arc<P2pandaTransport>,
        mesh: [u8; 32],
        signer: personae::Ed25519Keypair,
        clock: Arc<dyn Clock>,
        policy: mesh::LeasePolicy,
        readers: Arc<StageReaders>,
        settings: AutomaticStageSettings,
    ) -> Result<Arc<Self>, String> {
        settings.validate()?;
        if signer.public_key().to_bytes() != transport.local_peer_id().to_bytes() {
            return Err("stage planner transport differs from mesh author".into());
        }
        let (sender, receiver) = mpsc::channel(1);
        Ok(Arc::new(Self {
            transport,
            mesh,
            signer,
            clock,
            policy,
            readers,
            board: RwLock::new(JobBoard::default()),
            settings,
            runtime: tokio::runtime::Handle::current(),
            sender,
            receiver: Mutex::new(receiver),
            nonce: AtomicU64::new(0),
            last_unavailable: Mutex::new(None),
        }))
    }

    fn permits(&self, peer: PeerID) -> bool {
        let board = self.board.read().unwrap();
        [self.transport.local_peer_id().to_bytes(), peer.to_bytes()]
            .iter()
            .all(|author| {
                board
                    .devices()
                    .master_of(author)
                    .is_some_and(|root| (self.readers)(root))
            })
    }

    /// Owner-only planning diagnostic. Planning receives public model metadata,
    /// never a question, prompt, tensor frame or credential to include here.
    pub fn last_unavailable(&self) -> Option<String> {
        self.last_unavailable.lock().unwrap().clone()
    }

    /// Called before a supervisor tick, never from the actor. Queue cancellation
    /// and deadlines prevent abandoned actor requests from posting new work.
    pub(crate) async fn drive_posts<B: muniment::Backend + Clone + Send + Sync + 'static>(
        &self,
        host: &crate::mesh_host::MeshHost<B>,
    ) {
        let command = self.receiver.lock().unwrap().try_recv().ok();
        let Some(command) = command else { return };
        if command.reply.is_closed() || command.until <= std::time::Instant::now() {
            return;
        }
        let mut jobs = Vec::new();
        let mut result = Ok(());
        for (peer, request, requires) in command.stages {
            if command.reply.is_closed()
                || command.until <= std::time::Instant::now()
                || !PeerID::from_bytes(&peer).is_ok_and(|p| self.permits(p))
            {
                result = Err("stage plan expired or graph authority changed".into());
                break;
            }
            match host
                .post_inference_stage(
                    peer,
                    &request,
                    requires,
                    command.terms,
                    self.nonce.fetch_add(1, Ordering::SeqCst),
                )
                .await
            {
                Ok(job) => jobs.push(job),
                Err(e) => {
                    result = Err(e);
                    break;
                },
            }
        }
        let _ = command.reply.send(result.map(|_| jobs));
    }

    /// Factory invoked by an ordinary actor generation/resume. No question or
    /// prompt enters discovery, reservation, the mesh job or its receipt.
    pub fn factory(
        self: &Arc<Self>,
        row: eidetic::ManifestId,
        loader: String,
    ) -> Arc<StageChainFactory> {
        let planner = self.clone();
        Arc::new(move |resolved| {
            *planner.last_unavailable.lock().unwrap() = None;
            let result = planner
                .runtime
                .block_on(planner.plan(row, &loader, resolved));
            let mut diagnostic = planner.last_unavailable.lock().unwrap();
            *diagnostic = match result.as_ref().err() {
                Some(error) => Some(match diagnostic.take() {
                    Some(detail) => format!("{error}; {detail}"),
                    None => error.clone(),
                }),
                None => None,
            };
            result.map_err(InferError::StageUnavailable)
        })
    }

    async fn plan(
        self: &Arc<Self>,
        row: eidetic::ManifestId,
        loader: &str,
        resolved: &ResolvedInferenceRow,
    ) -> Result<Arc<StageChain>, String> {
        tokio::time::timeout(
            Duration::from_millis(self.settings.lease_wait_ms),
            self.plan_inner(row, loader, resolved),
        )
        .await
        .map_err(|_| "stage planning deadline expired".to_string())?
    }

    async fn plan_inner(
        self: &Arc<Self>,
        row: eidetic::ManifestId,
        loader: &str,
        resolved: &ResolvedInferenceRow,
    ) -> Result<Arc<StageChain>, String> {
        let config = resolved.decoder_config().map_err(|e| e.to_string())?;
        let request = StageRequest {
            row,
            session: resolved
                .session(loader)
                .id()
                .map_err(|e| e.to_string())?
                .to_string(),
            loader: loader.into(),
            range: LayerRange {
                first: 0,
                end: config.num_hidden_layers,
            },
        };
        let costs = resolved.stage_layer_costs().map_err(|e| e.to_string())?;
        let total = costs
            .iter()
            .try_fold(0u64, |a, b| a.checked_add(*b))
            .ok_or("layer budget overflow")?;
        let ceiling = total.saturating_sub(*costs.last().ok_or("empty model")?);
        let mut offers = BTreeMap::new();
        let assignments = loop {
            let candidates: Vec<_> = self
                .board
                .read()
                .unwrap()
                .devices()
                .entries()
                .filter_map(|(author, root)| {
                    (*author != self.transport.local_peer_id().to_bytes() && (self.readers)(*root))
                        .then_some(*author)
                })
                .take(self.settings.max_peers)
                .collect();
            let mut tasks = tokio::task::JoinSet::new();
            for author in candidates {
                let Ok(peer) = PeerID::from_bytes(&author) else {
                    continue;
                };
                if !self.permits(peer) || offers.contains_key(&author) {
                    continue;
                }
                let transport = self.transport.clone();
                let req = request.clone();
                let timeout = Duration::from_millis(self.settings.discovery_timeout_ms);
                tasks.spawn(async move {
                    let bytes = discovery::query(
                        &transport,
                        peer,
                        req.clone(),
                        StageOperation::Discover,
                        timeout,
                    )
                    .await?;
                    let offer: StageOffer =
                        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    if offer.request != req || offer.layer_bytes == 0 {
                        return Err("stage offer differs from requested session".to_string());
                    }
                    Ok::<_, String>((author, offer))
                });
            }
            let mut errors = Vec::new();
            while let Some(result) = tasks.join_next().await {
                match result {
                    Ok(Ok((author, offer))) => {
                        offers.insert(author, offer);
                    },
                    Ok(Err(error)) => errors.push(error),
                    Err(error) => errors.push(error.to_string()),
                }
            }
            offers.retain(|author, _| PeerID::from_bytes(author).is_ok_and(|p| self.permits(p)));
            let capacities: Vec<_> = offers
                .iter()
                .map(|(device, offer)| StageCapacity {
                    device: *device,
                    layer_bytes: offer.layer_bytes.min(ceiling),
                })
                .collect();
            if let Ok(assignments) = assign_ranges(&costs, &capacities) {
                if assignments.len() >= 2 {
                    break assignments;
                }
            }
            *self.last_unavailable.lock().unwrap() = Some(format!(
                "no complete stage cover: {} offers; discovery errors: {}",
                offers.len(),
                errors.join("; ")
            ));
            tokio::time::sleep(Duration::from_millis(self.settings.discovery_every_ms)).await;
        };
        let mut stages = Vec::new();
        let mut reservations = Reservations {
            transport: self.transport.clone(),
            runtime: self.runtime.clone(),
            timeout: Duration::from_millis(self.settings.discovery_timeout_ms),
            peers: Vec::new(),
            armed: true,
        };
        let mut hidden_limit = self.settings.max_hidden_bytes;
        for assignment in assignments {
            let peer = PeerID::from_bytes(&assignment.device).map_err(|e| e.to_string())?;
            if !self.permits(peer) {
                return Err("stage peer retired".into());
            }
            let offer = &offers[&assignment.device];
            if (offer.requires.compute == mesh::ComputeClass::Gpu) != (loader == "burn-wgpu") {
                return Err("stage compute class disagrees with backend".into());
            }
            hidden_limit = hidden_limit.min(offer.max_hidden_bytes);
            let mut req = request.clone();
            req.range = assignment.range;
            let reserved = discovery::query(
                &self.transport,
                peer,
                req.clone(),
                StageOperation::Reserve,
                Duration::from_millis(self.settings.discovery_timeout_ms),
            )
            .await?;
            if reserved != b"reserved" {
                return Err("stage reservation refused".into());
            }
            reservations.peers.push((peer, req.clone()));
            stages.push((assignment.device, req, offer.requires));
        }
        let (reply, receive) = oneshot::channel();
        let until = std::time::Instant::now() + Duration::from_millis(self.settings.lease_wait_ms);
        let jobs =
            tokio::time::timeout(Duration::from_millis(self.settings.lease_wait_ms), async {
                self.sender
                    .send(PostStages {
                        stages: stages.clone(),
                        terms: self.settings.terms(),
                        until,
                        reply,
                    })
                    .await
                    .map_err(|_| "stage host closed".to_string())?;
                receive
                    .await
                    .map_err(|_| "stage host stopped".to_string())?
            })
            .await
            .map_err(|_| "stage post deadline expired".to_string())??;
        let mut peers = Vec::new();
        loop {
            if std::time::Instant::now() >= until {
                return Err("stage lease deadline expired".into());
            }
            peers.clear();
            {
                let board = self.board.read().unwrap();
                for ((author, request, _), job) in stages.iter().zip(&jobs) {
                    if let Some(mesh::LeasePhase::Held {
                        holder,
                        lease,
                        epoch,
                        ..
                    }) = board
                        .job(*job)
                        .map(|j| j.lease_at(self.clock.now_ms(), &self.policy))
                    {
                        if holder != *author {
                            return Err("stage lease assigned elsewhere".into());
                        }
                        peers.push(StagePeer {
                            peer: PeerID::from_bytes(author).map_err(|e| e.to_string())?,
                            credential: mesh::RemoteSessionClaim::signed(
                                &self.signer,
                                self.mesh,
                                *job,
                                lease,
                                epoch,
                                *author,
                                0,
                            )
                            .encode()
                            .map_err(|e| e.to_string())?,
                            request: request.clone(),
                        });
                    }
                }
            }
            if peers.len() == stages.len() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(self.settings.poll_every_ms)).await;
        }
        let requires = mesh::ResourceRequirements::cpu();
        let mut settings = self.settings.transport(requires);
        settings.max_hidden_bytes = hidden_limit;
        let weak = Arc::downgrade(self);
        let chain = StageChain::new(
            self.transport.clone(),
            self.runtime.clone(),
            peers,
            settings,
            resolved.weight_format().unwrap_or_default(),
            StageShape {
                hidden: config.hidden_size,
                vocab: config.vocab_size,
                context: config.max_position_embeddings,
                layers: config.num_hidden_layers,
            },
            Arc::new(move |peer| weak.upgrade().is_some_and(|planner| planner.permits(peer))),
        )
        .and_then(|chain| {
            chain.with_cleanup_timeout(Duration::from_millis(self.settings.discovery_timeout_ms))
        })
        .map(Arc::new)?;
        reservations.armed = false;
        Ok(chain)
    }
}
