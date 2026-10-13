// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::{MESH, Result, require};
use identity::delegation::Issue;
use identity::{IdentityProvider, InMemoryProvider};
use insigne::delegation::{
    CapabilityScope, DelegationCertificate, DelegationParent, SignedDelegationCertificate,
};
use mesh::ingress::{INGRESS_ALPN, INGRESS_DOMAIN, INGRESS_PATH, IngressConfig, requested_action};
use notochord::{
    LocalNetworkPolicy, NetworkId, ProfileRef, RevocationLedger, ServiceAccess, ServiceRule,
    SessionHello, TrafficClass, TrustedRoot,
};
use std::sync::Arc;
use std::time::Duration;
use transport::{
    Alpn, P2pandaStream, P2pandaTransport, PeerAddr, PeerID, Transport, decode_peer_ticket,
    encode_peer_ticket,
};

const ROOT: [u8; 32] = [7; 32];
pub const CALL: Duration = Duration::from_secs(2);

pub fn config() -> IngressConfig {
    let mut policy = LocalNetworkPolicy::closed(NetworkId(MESH));
    policy.accepted_profiles = vec![ProfileRef {
        id: "mere.retained".into(),
        revision: 1,
    }];
    policy.trusted_roots = vec![TrustedRoot {
        authority: ROOT,
        issuer: InMemoryProvider::from_seed([20; 32])
            .master_public_key()
            .to_bytes(),
    }];
    policy.services.insert(
        INGRESS_PATH.into(),
        ServiceRule::new(
            ServiceAccess::MemberOnly,
            INGRESS_DOMAIN,
            ["submit", "inventory"],
            true,
            Some(2),
        ),
    );
    IngressConfig {
        policy,
        revocations: RevocationLedger::new(),
        call_timeout: CALL,
        retained_for: Duration::from_secs(60),
        now_ms: Arc::new(|| 1000),
    }
}

pub struct Client {
    pub transport: Arc<P2pandaTransport>,
    provider: InMemoryProvider,
    server: PeerID,
    nonce: u64,
}

impl Client {
    pub async fn bind(server: &P2pandaTransport) -> Result<Self> {
        let provider = InMemoryProvider::from_seed([21; 32]);
        let transport = Arc::new(
            P2pandaTransport::builder_from_seed_ref(&[21; 32])
                .bind()
                .await?,
        );
        require(
            transport.local_peer_id().to_bytes() == provider.master_public_key().to_bytes(),
            "carrier subject differs from session subject",
        )?;
        let (server_id, routes) = decode_peer_ticket(&server.ticket().await?)?;
        let routes = routes
            .into_iter()
            .filter(|a| matches!(a, PeerAddr::Direct(a) if a.ip().is_loopback()))
            .collect::<Vec<_>>();
        require(!routes.is_empty(), "no loopback dial route")?;
        transport
            .add_peer_ticket(&encode_peer_ticket(server_id, &routes)?)
            .await?;
        Ok(Self {
            transport,
            provider,
            server: server_id,
            nonce: 0,
        })
    }

    pub fn hello(&mut self, action: &str, granted: &[&str]) -> Result<SessionHello> {
        self.nonce += 1;
        let root = InMemoryProvider::from_seed([20; 32]);
        let grant = SignedDelegationCertificate::issue(
            &root,
            DelegationCertificate::new(
                DelegationParent::Root(ROOT),
                root.master_public_key().to_bytes(),
                self.provider.master_public_key().to_bytes(),
                CapabilityScope {
                    domain: INGRESS_DOMAIN.into(),
                    resource: MESH.to_vec(),
                    path_prefix: INGRESS_PATH.into(),
                    actions: granted.iter().map(|s| s.to_string()).collect(),
                },
                5,
                10,
                Some(100_000),
                1,
                [20; 32],
            ),
        )?;
        let mut nonce = [0; 32];
        nonce[..8].copy_from_slice(&self.nonce.to_le_bytes());
        Ok(SessionHello::issue(
            &self.provider,
            NetworkId(MESH),
            ProfileRef {
                id: "mere.retained".into(),
                revision: 1,
            },
            requested_action(action),
            TrafficClass::Background,
            nonce,
            &transport::initiator_binding(&Alpn::new(INGRESS_ALPN), self.transport.local_peer_id()),
            vec![grant],
        )?)
    }

    pub async fn connect(&self) -> Result<P2pandaStream> {
        Ok(tokio::time::timeout(
            CALL,
            self.transport.connect(self.server, Alpn::new(INGRESS_ALPN)),
        )
        .await??)
    }
}
