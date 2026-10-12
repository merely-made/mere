// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Finite local QUIC handoff; the mesh callback remains the admission owner.

use crate::{Result, campaign, hex, peer, require};
use mesh::MeshStore;
use muniment::RedbBackend;
use serde_json::{Value, json};
use std::time::Duration;
use stickleback::{ObjectStreamError, ObjectStreamLimits, read_object, write_object};
use tokio::io::AsyncWriteExt;
use tokio::time::{Instant, timeout, timeout_at};
use transport::{
    Alpn, P2pandaStream, P2pandaTransport, PeerAddr, Transport, TransportKind, decode_peer_ticket,
    encode_peer_ticket,
};

const ALPN: &str = "mere/retained-receive/v1";
const IO_SECONDS: u64 = 10;
const EXCHANGE_SECONDS: u64 = 30;
const CAMPAIGN_SECONDS: u64 = 90;
const PATH_OBSERVATION_MILLIS: u64 = 2000;
const PATH_POLL_MILLIS: u64 = 50;
const MAX_ACK_BYTES: u32 = 2048;
const MAX_ATTEMPTS: usize = 12;
const MAX_ATTEMPTED_FRAME_BYTES: usize = 64 * 1024;
const MAX_RESERVED_BYTES: usize = 128 * 1024;

#[derive(Clone, Copy)]
pub enum Fault {
    Oversized,
    Truncated,
    Stalled,
}

impl Fault {
    fn name(self) -> &'static str {
        match self {
            Self::Oversized => "oversized-declared-length",
            Self::Truncated => "truncated-payload-with-fin",
            Self::Stalled => "stalled-payload-without-fin",
        }
    }
}

pub struct Carrier {
    sender: P2pandaTransport,
    receiver: P2pandaTransport,
    routes: Value,
    deadline: Instant,
    attempts: usize,
    attempted_frame_octets: usize,
    reserved_octets: usize,
    ack_octets: usize,
}

struct Received {
    stream: P2pandaStream,
    owner: Option<Value>,
    object: Option<Vec<u8>>,
    ack: Option<Value>,
    framing_refusal: Option<ObjectStreamError>,
    callback_calls: usize,
}

impl Carrier {
    pub async fn bind(author: &[u8; 32]) -> Result<Self> {
        timeout(Duration::from_secs(20), async {
            let alpn = Alpn::new(ALPN);
            let sender = P2pandaTransport::builder_from_seed_ref(&[0x41; 32])
                .alpns(vec![alpn.clone()])
                .bind()
                .await?;
            let receiver = P2pandaTransport::builder_from_seed_ref(&[0x42; 32])
                .alpns(vec![alpn])
                .bind()
                .await?;
            require(
                sender.local_peer_id() != receiver.local_peer_id(),
                "transport fixture identities equal",
            )?;
            require(
                sender.local_peer_id().to_bytes() != *author
                    && receiver.local_peer_id().to_bytes() != *author,
                "transport identity must differ from operation author",
            )?;
            let (sender_ticket, sender_routes) = loopback_ticket(&sender).await?;
            let (receiver_ticket, receiver_routes) = loopback_ticket(&receiver).await?;
            require(
                sender.add_peer_ticket(&receiver_ticket).await? == receiver.local_peer_id(),
                "receiver ticket peer",
            )?;
            require(
                receiver.add_peer_ticket(&sender_ticket).await? == sender.local_peer_id(),
                "sender ticket peer",
            )?;
            Ok(Self {
                sender,
                receiver,
                routes: json!({"sender":sender_routes,"receiver":receiver_routes}),
                deadline: Instant::now() + Duration::from_secs(CAMPAIGN_SECONDS),
                attempts: 0,
                attempted_frame_octets: 0,
                reserved_octets: 0,
                ack_octets: 0,
            })
        })
        .await
        .map_err(|_| "transport setup deadline expired")?
    }

    // &mut self serializes this finite host's attempts; production queues remain unbounded.
    fn reserve(&mut self, frame_octets: usize) -> Result<usize> {
        require(
            Instant::now() < self.deadline,
            "carrier campaign deadline expired",
        )?;
        require(self.attempts < MAX_ATTEMPTS, "attempted session budget")?;
        let frames = self
            .attempted_frame_octets
            .checked_add(frame_octets)
            .ok_or("frame budget overflow")?;
        let reserved = self
            .reserved_octets
            .checked_add(frame_octets + MAX_ACK_BYTES as usize + 4)
            .ok_or("reserved byte budget overflow")?;
        require(
            frames <= MAX_ATTEMPTED_FRAME_BYTES && reserved <= MAX_RESERVED_BYTES,
            "total carrier byte budget",
        )?;
        self.attempts += 1;
        self.attempted_frame_octets = frames;
        self.reserved_octets = reserved;
        Ok(self.attempts)
    }

    pub async fn transfer(
        &mut self,
        store: &MeshStore<RedbBackend>,
        bytes: &[u8],
    ) -> Result<(Value, Value)> {
        require(
            bytes.len() <= peer::MAX_OBJECT_BYTES,
            "outgoing record byte budget",
        )?;
        let attempt = self.reserve(bytes.len() + 4)?;
        let deadline = self
            .deadline
            .min(Instant::now() + Duration::from_secs(EXCHANGE_SECONDS));
        let (sent, received) = timeout_at(deadline, async {
            tokio::join!(self.send(bytes, None), self.receive(store))
        })
        .await
        .map_err(|_| "carrier exchange deadline expired")?;
        let (sender_stream, sender_ack) = sent?;
        let received = received?;
        require(
            received.object.as_deref() == Some(bytes),
            "QUIC handoff changed exact record bytes",
        )?;
        require(
            received.callback_calls == 1,
            "expected one receiver owner callback",
        )?;
        let owner = received
            .owner
            .as_ref()
            .ok_or("normal object framing refused")?;
        let receiver_ack = received
            .ack
            .as_ref()
            .ok_or("owner acknowledgement absent")?;
        let sender_ack = sender_ack.ok_or("sender acknowledgement absent")?;
        require(
            sender_ack == *receiver_ack,
            "sender acknowledgement differs from receiver result",
        )?;
        require(
            sender_ack["record_blake3"] == blake3::hash(bytes).to_hex().as_str(),
            "ack record digest differs",
        )?;
        let ack_octets = serde_json::to_vec(&sender_ack)?.len();
        self.ack_octets += ack_octets;
        let paths = self.active_paths(deadline).await?;
        let evidence = json!({"attempt":attempt,"kind":"exact_object_handoff",
            "attempted_frame_octets":bytes.len()+4,"received_object_octets":bytes.len(),
            "record_blake3":blake3::hash(bytes).to_hex().as_str(),"exact_record_bytes":true,
            "transport_authenticated_sender":hex(&self.sender.local_peer_id().to_bytes()),
            "expected_receiver":hex(&self.receiver.local_peer_id().to_bytes()),"accepted_alpn":ALPN,
            "peer_and_alpn_checked_before_object_read":true,"receiver_owner_callback_calls":1,
            "receiver_owner_result":owner,"sender_received_ack":sender_ack,"ack_octets":ack_octets,
            "ack_written_after_owner_result":true,"active_paths_after_io":paths});
        // Hold both connection-backed streams until active-path observation is complete.
        drop((sender_stream, received.stream));
        Ok((owner.clone(), evidence))
    }

    pub async fn boundary(
        &mut self,
        store: &MeshStore<RedbBackend>,
        bytes: &[u8],
        fault: Fault,
    ) -> Result<Value> {
        let frame_octets = match fault {
            Fault::Oversized => 4,
            _ => 5,
        };
        let attempt = self.reserve(frame_octets)?;
        let deadline = self
            .deadline
            .min(Instant::now() + Duration::from_secs(EXCHANGE_SECONDS));
        let (sent, received) = timeout_at(deadline, async {
            tokio::join!(self.send(bytes, Some(fault)), self.receive(store))
        })
        .await
        .map_err(|_| "carrier boundary exchange deadline expired")?;
        let (sender_stream, sender_ack) = sent?;
        let received = received?;
        require(
            received.callback_calls == 0
                && received.owner.is_none()
                && sender_ack.is_none()
                && received.ack.is_none(),
            "framing refusal reached owner or acknowledged admission",
        )?;
        let error = received
            .framing_refusal
            .as_ref()
            .ok_or("invalid frame accepted")?;
        let matches = match (fault, error) {
            (Fault::Oversized, ObjectStreamError::TooLarge { declared, max }) => {
                *declared == peer::MAX_OBJECT_BYTES as u64 + 1
                    && *max == peer::MAX_OBJECT_BYTES as u32
            },
            (Fault::Truncated, ObjectStreamError::Io(error)) => {
                error.kind() == std::io::ErrorKind::UnexpectedEof
            },
            (Fault::Stalled, ObjectStreamError::Timeout) => true,
            _ => false,
        };
        require(
            matches,
            "unexpected framing refusal; infrastructure failure is not evidence",
        )?;
        let paths = self.active_paths(deadline).await?;
        let evidence = json!({"attempt":attempt,"id":fault.name(),"attempted_frame_octets":frame_octets,
            "declared_object_octets":if matches!(fault, Fault::Oversized) {peer::MAX_OBJECT_BYTES+1} else {bytes.len()},
            "framing_refusal":error.to_string(),"receiver_owner_callback_calls":0,
            "ack_sent":false,"peer_and_alpn_checked_before_object_read":true,
            "active_paths_after_io":paths});
        drop((sender_stream, received.stream));
        Ok(evidence)
    }

    async fn send(
        &self,
        bytes: &[u8],
        fault: Option<Fault>,
    ) -> Result<(P2pandaStream, Option<Value>)> {
        let mut stream = timeout(
            Duration::from_secs(IO_SECONDS),
            self.sender
                .connect(self.receiver.local_peer_id(), Alpn::new(ALPN)),
        )
        .await
        .map_err(|_| "connect deadline expired")??;
        let object_limits = ObjectStreamLimits::new(
            peer::MAX_OBJECT_BYTES as u32,
            Duration::from_secs(IO_SECONDS),
        )?;
        if let Some(fault) = fault {
            timeout(Duration::from_secs(IO_SECONDS), async {
                let declared = if matches!(fault, Fault::Oversized) {
                    peer::MAX_OBJECT_BYTES + 1
                } else {
                    bytes.len()
                };
                stream.write_all(&(declared as u32).to_be_bytes()).await?;
                if !matches!(fault, Fault::Oversized) {
                    stream.write_all(&bytes[..1]).await?;
                }
                stream.flush().await?;
                if matches!(fault, Fault::Truncated) {
                    stream.shutdown().await?;
                }
                Ok::<_, crate::Error>(())
            })
            .await
            .map_err(|_| "raw frame write deadline expired")??;
            // Oversize and stalled send halves stay open while the receiver applies its gate.
            return Ok((stream, None));
        }
        write_object(&mut stream, bytes, object_limits).await?;
        let ack_limits = ObjectStreamLimits::new(MAX_ACK_BYTES, Duration::from_secs(IO_SECONDS))?;
        let ack = read_object(&mut stream, ack_limits).await?;
        let ack: Value = serde_json::from_slice(&ack)?;
        require(
            ack["schema"] == "mere.retained-receive-ack/v1",
            "ack schema",
        )?;
        Ok((stream, Some(ack)))
    }

    async fn receive(&self, store: &MeshStore<RedbBackend>) -> Result<Received> {
        let session = timeout(
            Duration::from_secs(IO_SECONDS),
            self.receiver.accept(Alpn::new(ALPN)),
        )
        .await
        .map_err(|_| "accept deadline expired")??;
        require(
            session.protocol == Alpn::new(ALPN)
                && session.peer == Some(self.sender.local_peer_id())
                && session.ingress.transport == TransportKind::P2panda,
            "unexpected accepted ALPN or authenticated peer",
        )?;
        let mut stream = session.stream;
        let limits = ObjectStreamLimits::new(
            peer::MAX_OBJECT_BYTES as u32,
            Duration::from_secs(IO_SECONDS),
        )?;
        let bytes = match read_object(&mut stream, limits).await {
            Ok(bytes) => bytes,
            Err(error) => {
                return Ok(Received {
                    stream,
                    owner: None,
                    object: None,
                    ack: None,
                    framing_refusal: Some(error),
                    callback_calls: 0,
                });
            },
        };
        // All normal and malformed traffic uses this same framing gate before the owner.
        let owner = campaign::admit_received(store, &bytes).await?;
        let ack = json!({"schema":"mere.retained-receive-ack/v1",
            "record_blake3":blake3::hash(&bytes).to_hex().as_str(),
            "operation_id":owner["operation_id"],"outcome":owner["outcome"],
            "rejection":owner["rejection"].as_object().map(|r| json!({"stage":r["stage"],"code":r["code"]}))});
        let ack_bytes = serde_json::to_vec(&ack)?;
        require(
            ack_bytes.len() <= MAX_ACK_BYTES as usize,
            "owner ACK byte budget",
        )?;
        write_object(
            &mut stream,
            &ack_bytes,
            ObjectStreamLimits::new(MAX_ACK_BYTES, Duration::from_secs(IO_SECONDS))?,
        )
        .await?;
        Ok(Received {
            stream,
            owner: Some(owner),
            object: Some(bytes),
            ack: Some(ack),
            framing_refusal: None,
            callback_calls: 1,
        })
    }

    async fn active_paths(&self, exchange_deadline: Instant) -> Result<Value> {
        let started = Instant::now();
        let deadline = self
            .deadline
            .min(exchange_deadline)
            .min(started + Duration::from_millis(PATH_OBSERVATION_MILLIS));
        let mut reads = 0usize;
        let mut initial = None;
        let mut last = Value::Null;
        let observed = timeout_at(deadline, async {
            loop {
                // remote_info is an actor snapshot, not a connection watcher.
                let (sender, receiver) = tokio::join!(
                    self.sender.peer_paths(self.receiver.local_peer_id()),
                    self.receiver.peer_paths(self.sender.local_peer_id()));
                let sender = sender?;
                let receiver = receiver?;
                let describe = |paths: &[transport::PeerPath]| json!(paths.iter()
                    .map(|path| json!({"addr":path.addr.to_string(),"active":path.active}))
                    .collect::<Vec<_>>());
                last = json!({"sender":describe(&sender),"receiver":describe(&receiver)});
                reads += 1;
                initial.get_or_insert_with(|| last.clone());
                let active_valid = |paths: &[transport::PeerPath]| paths.iter()
                    .filter(|path| path.active).all(|path|
                        matches!(&path.addr, PeerAddr::Direct(addr) if addr.ip().is_loopback()));
                if !active_valid(&sender) || !active_valid(&receiver) {
                    return Err(format!("unexpected active non-loopback route on attempt {}; both-side snapshot: {}",
                        self.attempts, last).into());
                }
                if sender.iter().any(|path| path.active) && receiver.iter().any(|path| path.active) {
                    return Ok::<_, crate::Error>(());
                }
                // Only absent active state can be retried; never hide an out-of-scope route.
                tokio::time::sleep(Duration::from_millis(PATH_POLL_MILLIS)).await;
            }
        }).await;
        match observed {
            Ok(result) => result?,
            Err(_) => {
                return Err(format!(
                    "active loopback path observation deadline expired on attempt {}; \
                snapshots_read={reads}; initial={}; last={last}",
                    self.attempts,
                    initial.as_ref().unwrap_or(&Value::Null)
                )
                .into());
            },
        }
        last["observation"] = json!({"snapshot_reads":reads,
            "elapsed_millis":started.elapsed().as_millis(),"initial_snapshot":initial,
            "retry_condition":"one or both sides have no active address",
            "all_observed_active_routes_direct_loopback":true});
        Ok(last)
    }

    pub async fn finish(&self) -> Result<Value> {
        timeout(Duration::from_secs(IO_SECONDS), async {
            self.sender.close().await?;
            self.receiver.close().await?;
            Ok::<_, crate::Error>(())
        })
        .await
        .map_err(|_| "endpoint close deadline expired")??;
        Ok(
            json!({"kind":"bounded_local_p2panda_quic_handoff","alpn":ALPN,
            "public_throwaway_transport_identities":true,"transport_keys_distinct_from_operation_author":true,
            "sender_peer":hex(&self.sender.local_peer_id().to_bytes()),
            "receiver_peer":hex(&self.receiver.local_peer_id().to_bytes()),"offered_loopback_routes":self.routes,
            "configured_relays":0,"mdns_enabled":false,"discovery_enabled":false,"gossip_enabled":false,
            "endpoint_bind":"production_defaults; only offered and observed dial paths are loopback",
            "stream_selection":"first bidi stream on each newly connected QUIC connection",
            "p2panda_net_revision":"d8acabc96ecc59016e40edf4a47ed5eaf5360285",
            "attempts":self.attempts,"attempted_frame_octets":self.attempted_frame_octets,
            "received_ack_octets":self.ack_octets,"reserved_frame_and_ack_octets":self.reserved_octets,
            "limits":{"in_flight_objects":1,"attempted_sessions":MAX_ATTEMPTS,
                "object_octets":peer::MAX_OBJECT_BYTES,"ack_octets":MAX_ACK_BYTES,
                "attempted_frame_octets":MAX_ATTEMPTED_FRAME_BYTES,"reserved_frame_and_ack_octets":MAX_RESERVED_BYTES,
                "connect_accept_seconds":IO_SECONDS,"object_transfer_seconds":IO_SECONDS,"ack_transfer_seconds":IO_SECONDS,
                "exchange_seconds":EXCHANGE_SECONDS,"carrier_campaign_seconds":CAMPAIGN_SECONDS,
                "endpoint_setup_seconds":20,"endpoint_close_seconds":IO_SECONDS,
                "active_path_observation_millis":PATH_OBSERVATION_MILLIS,
                "active_path_poll_millis":PATH_POLL_MILLIS},
            "global_production_accept_queue_bounded":false,"endpoints_closed":true}),
        )
    }
}

async fn loopback_ticket(local: &P2pandaTransport) -> Result<(String, Value)> {
    let (peer, routes) = decode_peer_ticket(&local.ticket().await?)?;
    require(peer == local.local_peer_id(), "local ticket identity")?;
    let routes: Vec<_> = routes
        .into_iter()
        .filter(|route| matches!(route, PeerAddr::Direct(addr) if addr.ip().is_loopback()))
        .collect();
    require(!routes.is_empty(), "no offered loopback address")?;
    let ticket = encode_peer_ticket(peer, &routes)?;
    let (decoded_peer, decoded_routes) = decode_peer_ticket(&ticket)?;
    require(
        decoded_peer == peer && decoded_routes == routes,
        "filtered ticket changed routes",
    )?;
    Ok((
        ticket,
        json!(routes.iter().map(ToString::to_string).collect::<Vec<_>>()),
    ))
}
