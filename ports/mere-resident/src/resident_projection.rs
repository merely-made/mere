// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resident's dialable projection host: a mere's routes for remote peers.
//!
//! The first-party application door serves a mere on `mere/<domain>` and
//! `ambient/mere/<domain>` to applications on this machine. This host serves
//! the same routes to a peer that dials the resident over a transport, on a
//! key of its own (transport agnosticism note, T7): derived from the persona
//! with [`RESIDENT_PROJECTION_SALT`], which neither personal sync's per-graph
//! keys nor places' keys share.
//!
//! What a remote session may open is its admission, set out in the [remote
//! mere route admission note][note]: the requested path names the route and
//! the application acting (`mere_overmap::wire::RemoteMereRequest`), the
//! delegation's leaf must name that application too (T5), and the explicit
//! and ambient routes are separate service paths with separate actions, so a
//! grant for one never confers the other (T4). Once admitted, the session
//! carries the application into the same `AccessEndpoint` the door opens, so
//! the owner's per-application decisions (default grant, denials, the
//! ambient opt-in) apply to a remote application exactly as to a local one.
//!
//! [note]: ../../../../design_docs/mere_docs/technical_architecture/2026-10-15_remote_mere_route_admission.md

use std::sync::Arc;
use std::time::Duration;

use djinn::door::route::ResidentEndpointRoute;
use graphshell_host::admission::{AMBIENT_ACTION, CONNECT_ACTION, GRAPHSHELL_DOMAIN};
use graphshell_host::carrier::ProjectionRefusal;
use graphshell_host::endpoint_catalog::ResidentEndpointCatalog;
use graphshell_host::projection_host::{
    ResidentProjectionError, ResidentProjectionHost, RouteSelector, SelectedRoute,
    ServedProjection, SharedCatalog,
};
use insigne::delegation::SignedDelegationCertificate;
use mere_overmap::wire::{REMOTE_AMBIENT_SERVICE, REMOTE_MERE_SERVICE, RemoteMereRequest};
use notochord::{
    AdmittedPrincipal, LocalNetworkPolicy, NetworkId, ProfileRef, ServiceAccess, ServiceRule,
    TrustedRoot,
};
use transport::Transport;

/// The derivation salt of the dialable host's transport key.
pub const RESIDENT_PROJECTION_SALT: &[u8] = b"mere.djinn/resident-projection/v1";

/// How often a remote mere route's notices are polled, as the door's are.
const REMOTE_NOTICE_POLL: Duration = Duration::from_millis(50);

/// The dialable host's admission policy: closed, offering exactly the
/// explicit and ambient mere services, both member-only, to chains rooted at
/// `roots` (the persona's own root for its devices, and each place's root for
/// its members).
pub fn dial_policy(
    network: NetworkId,
    roots: Vec<TrustedRoot>,
    accepted_profiles: Vec<ProfileRef>,
    max_sessions: Option<u32>,
) -> LocalNetworkPolicy {
    let mut policy = LocalNetworkPolicy::closed(network);
    policy.trusted_roots = roots;
    policy.accepted_profiles = accepted_profiles;
    policy.services.insert(
        REMOTE_MERE_SERVICE.to_string(),
        ServiceRule::new(
            ServiceAccess::MemberOnly,
            GRAPHSHELL_DOMAIN,
            [CONNECT_ACTION],
            false,
            max_sessions,
        ),
    );
    policy.services.insert(
        REMOTE_AMBIENT_SERVICE.to_string(),
        ServiceRule::new(
            ServiceAccess::MemberOnly,
            GRAPHSHELL_DOMAIN,
            [AMBIENT_ACTION],
            false,
            max_sessions,
        ),
    );
    policy
}

/// The route and application a remote session was admitted for, or why this
/// host serves it nothing: its requested action must be a remote mere request
/// in exactly `mere_overmap::wire`'s spelling, and its delegation's leaf must
/// name the request's application.
pub fn select_mere_route(
    principal: &AdmittedPrincipal,
    chain: &[SignedDelegationCertificate],
) -> Result<SelectedRoute, String> {
    let action = &principal.action;
    let request = RemoteMereRequest::parse(&action.domain, &action.path, &action.action)
        .ok_or("this host serves remote mere routes only")?;
    let leaf = chain
        .last()
        .ok_or("a remote mere route needs a delegation")?;
    if !request.named_by(&leaf.certificate.scope.path_prefix) {
        return Err(format!(
            "the delegation does not name {} for {}",
            request.application,
            request.route_id()
        ));
    }
    let route = ResidentEndpointRoute::new(request.route_id(), REMOTE_NOTICE_POLL)
        .map_err(|error| error.to_string())?;
    Ok(SelectedRoute {
        route,
        application: Some(request.application),
    })
}

/// Bind the dialable host's own transport: iroh QUIC through p2panda, on the
/// key `identity` derives with [`RESIDENT_PROJECTION_SALT`], accepting only
/// Graphshell's projection ALPN. A dialler addresses the resident by this
/// key's peer id, which no other lane of the resident answers to.
pub async fn bind_dial_transport<P>(identity: &P) -> Result<transport::P2pandaTransport, String>
where
    P: personae::IdentityProvider + ?Sized,
{
    let key = identity
        .derived_keypair(RESIDENT_PROJECTION_SALT)
        .map_err(|error| error.to_string())?;
    transport::P2pandaTransport::builder_for(&key)
        .alpns(vec![graphshell_host::carrier::projection_alpn()])
        .bind()
        .await
        .map_err(|error| error.to_string())
}

/// The resident's dialable projection host.
pub struct ResidentDialHost {
    host: ResidentProjectionHost,
}

impl ResidentDialHost {
    /// A host admitting under `policy` (see [`dial_policy`]) with an empty
    /// catalog; [`crate::resident_mere::MereRoutes::with_remote_catalog`]
    /// fills it with [`Self::catalog`] as each mere is served.
    pub fn new(policy: LocalNetworkPolicy) -> Self {
        let selector: RouteSelector = Arc::new(select_mere_route);
        Self {
            host: ResidentProjectionHost::selecting(
                policy,
                ResidentEndpointCatalog::new(),
                selector,
            ),
        }
    }

    /// The catalog the host opens routes from.
    pub fn catalog(&self) -> SharedCatalog {
        self.host.catalog()
    }

    /// How many remote sessions are being served.
    pub fn live_sessions(&self) -> u32 {
        self.host.live_sessions()
    }

    /// Admit one dialler on `transport` and serve it in the background.
    pub async fn accept_one<T, N>(
        &mut self,
        transport: &T,
        now_ms: N,
    ) -> Result<Result<ServedProjection, ProjectionRefusal>, ResidentProjectionError>
    where
        T: Transport,
        N: Fn() -> u64 + Send + 'static,
    {
        self.host.accept_one(transport, now_ms).await
    }

    /// Accept diallers on `transport` until it fails, serving each in the
    /// background. A refused dialler is logged and the host keeps listening.
    pub async fn serve<T, N>(mut self, transport: Arc<T>, now_ms: N) -> ResidentProjectionError
    where
        T: Transport,
        N: Fn() -> u64 + Clone + Send + 'static,
    {
        loop {
            match self.accept_one(transport.as_ref(), now_ms.clone()).await {
                Ok(Ok(served)) => {
                    tracing::info!(session = %served.session().0, "remote mere session admitted");
                },
                Ok(Err(refusal)) => {
                    tracing::info!(?refusal, "remote mere session refused");
                },
                Err(error) => return error,
            }
        }
    }
}

/// The wire's restatement of Graphshell's action vocabulary, held equal.
#[cfg(test)]
#[test]
fn the_wire_spells_graphshells_actions() {
    use mere_overmap::wire::{REMOTE_AMBIENT_ACTION, REMOTE_DOMAIN, REMOTE_MERE_ACTION};
    assert_eq!(REMOTE_DOMAIN, GRAPHSHELL_DOMAIN);
    assert_eq!(REMOTE_MERE_ACTION, CONNECT_ACTION);
    assert_eq!(REMOTE_AMBIENT_ACTION, AMBIENT_ACTION);
    assert!(REMOTE_MERE_SERVICE.starts_with(graphshell_host::admission::PROJECTION_SERVICE));
    assert!(REMOTE_AMBIENT_SERVICE.starts_with(graphshell_host::admission::PROJECTION_SERVICE));
}
