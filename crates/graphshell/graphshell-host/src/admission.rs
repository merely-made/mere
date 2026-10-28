// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Graphshell's session admission: `notochord` under the Graphshell
//! action triple.
//!
//! G5c. The carrier's handshake is the **sole** admission step (G5a.1): it
//! binds a Personae subject, its attested signer, its delegation chain, the
//! requested action, and a nonce to *this* connection, and the application
//! stream is withheld until that verifies. This module supplies the two things
//! `notochord` cannot know — what Graphshell's service is called, and how
//! a Graphshell host asks — and returns the conclusion as an
//! [`AdmittedPrincipal`].
//!
//! Nothing here travels in `chirograph`. Admission happens *below*
//! the application protocol, which is why `SessionOpen` negotiates version and
//! capabilities only and carries no principal of its own.
//!
//! ## The triple
//!
//! `mere.graphshell` / `/services/projection` / `connect`. The action
//! vocabulary in `notochord` is deliberately open
//! (`RequestedAction { domain, path, action }`), so adding Graphshell is a new
//! triple rather than a change to that crate — and a delegation minted for
//! Murm's service does not admit a Graphshell session, because the scope's
//! `domain` and `path_prefix` are part of what the chain has to cover.

use insigne::delegation::SignedDelegationCertificate;
use notochord::{
    AdmittedPrincipal, DenyReason, HandshakeError, LocalNetworkPolicy, NetworkId, ProfileRef,
    ProofBinding, RequestedAction, RevocationLedger, SessionDecision, SessionFacts, SessionHello,
    TrafficClass, admit, respond,
};
use personae::IdentityProvider;

/// The application domain owning Graphshell's action vocabulary.
pub const GRAPHSHELL_DOMAIN: &str = "mere.graphshell";

/// The service path a projection session runs under.
pub const PROJECTION_SERVICE: &str = "/services/projection";

/// Opening a projection session.
pub const CONNECT_ACTION: &str = "connect";

/// Opening a read-only ambient crossing of a route, under a path of its own.
///
/// A separate action, not a flavour of [`CONNECT_ACTION`], so a grant for
/// one never confers the other: attenuation can narrow an action set but not
/// swap its members (remote mere route admission note, §3).
pub const AMBIENT_ACTION: &str = "ambient";

/// The ALPN-ish protocol label a Graphshell session is accepted for. It is part
/// of the signed transcript, so a proof minted for another protocol on the same
/// connection does not verify here.
pub const PROJECTION_PROTOCOL: &[u8] = b"mere/graphshell/v1";

/// Graphshell's action triple.
pub fn connect_action() -> RequestedAction {
    RequestedAction {
        domain: GRAPHSHELL_DOMAIN.to_string(),
        path: PROJECTION_SERVICE.to_string(),
        action: CONNECT_ACTION.to_string(),
    }
}

/// Build and sign the client's hello for `binding`.
///
/// Returns the [`SessionHello`] rather than encoded bytes, so it composes with
/// `notochord::initiate_session` (which writes the frame and reads the
/// reply) instead of standing up a second encode path beside it.
///
/// `binding` must describe the connection this hello will actually travel on;
/// the responder rebuilds it from what *it* observed, so a captured hello is
/// useless on a different link.
pub fn open_session<P: IdentityProvider>(
    identity: &P,
    network: NetworkId,
    profile: ProfileRef,
    class: TrafficClass,
    nonce: [u8; 32],
    binding: &ProofBinding,
    delegations: Vec<SignedDelegationCertificate>,
) -> Result<SessionHello, HandshakeError> {
    open_session_for(
        identity,
        network,
        profile,
        connect_action(),
        class,
        nonce,
        binding,
        delegations,
    )
}

/// [`open_session`] for an action other than [`connect_action`]: a child of
/// the projection service a route-selecting host serves, such as a remote
/// mere route naming its application. The host's action check decides
/// whether it serves it.
#[allow(clippy::too_many_arguments)]
pub fn open_session_for<P: IdentityProvider>(
    identity: &P,
    network: NetworkId,
    profile: ProfileRef,
    action: RequestedAction,
    class: TrafficClass,
    nonce: [u8; 32],
    binding: &ProofBinding,
    delegations: Vec<SignedDelegationCertificate>,
) -> Result<SessionHello, HandshakeError> {
    SessionHello::issue(
        identity,
        network,
        profile,
        action,
        class,
        nonce,
        binding,
        delegations,
    )
}

/// [`open_session`] with the session signer bound to `network`, for a door
/// whose kept keys must open nothing else (vault lock ruling 46).
pub fn open_network_bound_session<P: IdentityProvider>(
    identity: &P,
    network: NetworkId,
    profile: ProfileRef,
    class: TrafficClass,
    nonce: [u8; 32],
    binding: &ProofBinding,
    delegations: Vec<SignedDelegationCertificate>,
) -> Result<SessionHello, HandshakeError> {
    SessionHello::issue_network_bound(
        identity,
        network,
        profile,
        connect_action(),
        class,
        nonce,
        binding,
        delegations,
    )
}

/// Whether this service serves the action an admitted principal was admitted
/// for.
///
/// The owner rule names the admission actions it offers. This service check is
/// still kept at the domain boundary so a permissive or stale owner document
/// cannot make Graphshell serve an operation its implementation does not
/// support.
pub fn serves_action(principal: &AdmittedPrincipal) -> bool {
    principal.action == connect_action()
}

/// Whether this host serves what an admitted principal was admitted for,
/// given the delegation chain the conclusion was drawn from. The chain is
/// there for checks that read the grant itself, such as an application its
/// leaf must name.
pub type ActionCheck<'a> =
    &'a (dyn Fn(&AdmittedPrincipal, &[SignedDelegationCertificate]) -> bool + Send + Sync);

/// [`serves_action`] as an [`ActionCheck`].
pub(crate) fn serves_admitted(
    principal: &AdmittedPrincipal,
    _chain: &[SignedDelegationCertificate],
) -> bool {
    serves_action(principal)
}

/// Admit (or refuse) an incoming Graphshell session.
///
/// Returns the reply frame to write in both cases and the principal when the
/// session is admitted. The two always agree: the host's served vocabulary is
/// part of what admission answers, so an action the owner allows and
/// [`serves_action`] does not serve is refused `ActionNotOffered` in the reply
/// as well as in the outcome.
pub fn admit_session(
    policy: &LocalNetworkPolicy,
    ledger: &RevocationLedger,
    hello_bytes: &[u8],
    facts: &SessionFacts,
    now_ms: u64,
    active_sessions: u32,
) -> (Vec<u8>, Result<AdmittedPrincipal, DenyReason>) {
    admit_session_with(
        policy,
        ledger,
        hello_bytes,
        facts,
        now_ms,
        active_sessions,
        &serves_admitted,
    )
}

/// [`admit_session`] with the host's own [`ActionCheck`] in place of
/// [`serves_action`], for a host serving a vocabulary of its own under the
/// projection service.
///
/// Two stages, both decided before the reply: the policy enforces the owner's
/// allow-list, then `serves` enforces what the host implements, so a
/// permissive or stale owner document cannot make the host serve something it
/// does not.
pub fn admit_session_with(
    policy: &LocalNetworkPolicy,
    ledger: &RevocationLedger,
    hello_bytes: &[u8],
    facts: &SessionFacts,
    now_ms: u64,
    active_sessions: u32,
    serves: ActionCheck<'_>,
) -> (Vec<u8>, Result<AdmittedPrincipal, DenyReason>) {
    let (reply, outcome) = admit(policy, ledger, hello_bytes, facts, now_ms, active_sessions);
    let principal = match outcome {
        Ok(principal) => principal,
        Err(reason) => return (reply, Err(reason)),
    };
    // `admit` accepted this frame, so it decodes; a frame that does not is not
    // served.
    let chain = SessionHello::decode(hello_bytes, &policy.limits.clamped())
        .map(|hello| hello.claims().delegations);
    if chain.is_ok_and(|chain| serves(&principal, &chain)) {
        return (reply, Ok(principal));
    }
    unserved_refusal(policy, ledger, hello_bytes, facts, now_ms, active_sessions)
}

/// The refusal for an action the owner allows and this host does not serve.
///
/// Notochord writes every reply and exports no encoder for a bare refusal, so
/// the host's vocabulary is applied as one more narrowing of the owner's rule:
/// the same hello, judged against a policy that offers no action, is refused
/// `ActionNotOffered` by Notochord itself, and reply and outcome come from the
/// one decision.
fn unserved_refusal(
    policy: &LocalNetworkPolicy,
    ledger: &RevocationLedger,
    hello_bytes: &[u8],
    facts: &SessionFacts,
    now_ms: u64,
    active_sessions: u32,
) -> (Vec<u8>, Result<AdmittedPrincipal, DenyReason>) {
    let mut offers_nothing = policy.clone();
    for rule in offers_nothing.services.values_mut() {
        rule.actions.clear();
    }
    let (reply, decision) = respond(
        &offers_nothing,
        ledger,
        hello_bytes,
        facts,
        now_ms,
        active_sessions,
    );
    match decision {
        SessionDecision::Deny { reason } => (reply, Err(reason)),
        SessionDecision::Accept { .. } => {
            unreachable!("a policy that offers no action admits none")
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insigne::delegation::{
        CapabilityScope, DelegationCertificate, DelegationParent, SignedDelegationCertificate,
    };
    use notochord::{CarrierKind, ServiceAccess, ServiceRule, SessionReply, TrustedRoot};
    use personae::InMemoryProvider;
    use personae::delegation::Issue;
    use std::collections::BTreeMap;

    const NETWORK: NetworkId = NetworkId([3; 32]);
    const ROOT_AUTHORITY: [u8; 32] = [7; 32];
    const NOW_MS: u64 = 50;

    fn owner() -> InMemoryProvider {
        InMemoryProvider::from_seed([1; 32])
    }

    fn viewer() -> InMemoryProvider {
        InMemoryProvider::from_seed([4; 32])
    }

    fn stranger() -> InMemoryProvider {
        InMemoryProvider::from_seed([11; 32])
    }

    fn profile_ref() -> ProfileRef {
        ProfileRef {
            id: "mere.base".into(),
            revision: 1,
        }
    }

    /// A grant from the owner letting `subject` open projection sessions.
    fn projection_grant(
        subject: [u8; 32],
        path: &str,
        domain: &str,
    ) -> SignedDelegationCertificate {
        grant_for(subject, path, domain, CONNECT_ACTION)
    }

    fn grant_for(
        subject: [u8; 32],
        path: &str,
        domain: &str,
        action: &str,
    ) -> SignedDelegationCertificate {
        SignedDelegationCertificate::issue(
            &owner(),
            DelegationCertificate::new(
                DelegationParent::Root(ROOT_AUTHORITY),
                owner().master_public_key().to_bytes(),
                subject,
                CapabilityScope {
                    domain: domain.into(),
                    resource: NETWORK.0.to_vec(),
                    path_prefix: path.into(),
                    actions: [action.to_string()].into_iter().collect(),
                },
                5,
                10,
                Some(100),
                1,
                [1; 32],
            ),
        )
        .expect("issue certificate")
    }

    fn policy() -> LocalNetworkPolicy {
        let mut policy = LocalNetworkPolicy::closed(NETWORK);
        policy.accepted_profiles = vec![profile_ref()];
        policy.trusted_roots = vec![TrustedRoot {
            authority: ROOT_AUTHORITY,
            issuer: owner().master_public_key().to_bytes(),
        }];
        policy.services = BTreeMap::from([(
            PROJECTION_SERVICE.to_string(),
            ServiceRule::new(
                ServiceAccess::MemberOnly,
                GRAPHSHELL_DOMAIN,
                [CONNECT_ACTION],
                false,
                None,
            ),
        )]);
        policy
    }

    /// What the initiator signs against: an unauthenticated carrier here, so
    /// only the protocol is bound.
    fn binding() -> ProofBinding {
        ProofBinding::initiator(PROJECTION_PROTOCOL, None, None)
    }

    /// What the responder observed for that same connection.
    fn facts() -> SessionFacts {
        SessionFacts::new(PROJECTION_PROTOCOL, CarrierKind::P2panda)
    }

    fn hello_from<P: IdentityProvider>(
        identity: &P,
        binding: &ProofBinding,
        delegations: Vec<SignedDelegationCertificate>,
    ) -> Vec<u8> {
        open_session(
            identity,
            NETWORK,
            profile_ref(),
            TrafficClass::Interactive,
            [5; 32],
            binding,
            delegations,
        )
        .expect("issue hello")
        .encode(&LocalNetworkPolicy::closed(NETWORK).limits.clamped())
        .expect("encode hello")
    }

    #[test]
    fn a_granted_viewer_is_admitted_and_named() {
        let viewer = viewer();
        let subject = viewer.master_public_key().to_bytes();
        let hello = hello_from(
            &viewer,
            &binding(),
            vec![projection_grant(
                subject,
                PROJECTION_SERVICE,
                GRAPHSHELL_DOMAIN,
            )],
        );

        let (reply, outcome) = admit_session(
            &policy(),
            &RevocationLedger::default(),
            &hello,
            &facts(),
            NOW_MS,
            0,
        );
        assert!(
            SessionReply::decode(&reply, &policy().limits.clamped())
                .expect("a well-formed reply")
                .is_accept()
        );
        let principal = outcome.expect("a granted viewer opens a projection session");
        assert_eq!(
            principal.subject, subject,
            "the admitted principal is the peer, established by the handshake"
        );
        assert_eq!(principal.action, connect_action());
    }

    #[test]
    fn an_ungranted_stranger_is_refused() {
        let hello = hello_from(&stranger(), &binding(), Vec::new());
        let (_, outcome) = admit_session(
            &policy(),
            &RevocationLedger::default(),
            &hello,
            &facts(),
            NOW_MS,
            0,
        );
        assert!(
            outcome.is_err(),
            "no chain, no session: MemberOnly means a delegation is required"
        );
    }

    #[test]
    fn a_grant_for_another_service_does_not_open_projections() {
        // The whole point of a triple: authority is per-service, so a peer
        // admitted to Murm cannot reach Graphshell's projections with it.
        let viewer = viewer();
        let subject = viewer.master_public_key().to_bytes();
        let hello = hello_from(
            &viewer,
            &binding(),
            vec![projection_grant(subject, "/services/murm", "mere.network")],
        );
        let (_, outcome) = admit_session(
            &policy(),
            &RevocationLedger::default(),
            &hello,
            &facts(),
            NOW_MS,
            0,
        );
        assert!(
            outcome.is_err(),
            "a Murm grant is not a Graphshell grant, even for the same subject"
        );
    }

    #[test]
    fn an_action_the_owner_allows_but_this_host_does_not_serve_is_refused_in_the_reply() {
        // The owner's rule offers `administer` and the grant covers it, so
        // Notochord alone would admit. The host's vocabulary is part of what
        // admission answers: the peer is told, not left to see a close after
        // an accept.
        const ADMINISTER: &str = "administer";
        let mut permissive = policy();
        permissive
            .services
            .get_mut(PROJECTION_SERVICE)
            .expect("the projection rule")
            .actions
            .insert(ADMINISTER.to_string());
        let viewer = viewer();
        let subject = viewer.master_public_key().to_bytes();
        let mut requested = connect_action();
        requested.action = ADMINISTER.to_string();
        let hello = open_session_for(
            &viewer,
            NETWORK,
            profile_ref(),
            requested,
            TrafficClass::Interactive,
            [5; 32],
            &binding(),
            vec![grant_for(
                subject,
                PROJECTION_SERVICE,
                GRAPHSHELL_DOMAIN,
                ADMINISTER,
            )],
        )
        .expect("issue hello")
        .encode(&permissive.limits.clamped())
        .expect("encode hello");

        let (reply, outcome) = admit_session(
            &permissive,
            &RevocationLedger::default(),
            &hello,
            &facts(),
            NOW_MS,
            0,
        );
        assert_eq!(
            SessionReply::decode(&reply, &permissive.limits.clamped())
                .expect("a well-formed reply"),
            SessionReply::Reject {
                reason: DenyReason::ActionNotOffered
            },
            "the reply the peer reads is the refusal"
        );
        assert_eq!(
            outcome.unwrap_err(),
            DenyReason::ActionNotOffered,
            "and it agrees with the outcome the host acts on"
        );
    }

    #[test]
    fn a_captured_hello_does_not_open_a_different_connection() {
        // The transcript binds the connection, so replaying a valid hello onto
        // another one fails: this is what makes admission per-connection rather
        // than per-credential.
        let viewer = viewer();
        let subject = viewer.master_public_key().to_bytes();
        let hello = hello_from(
            &viewer,
            &binding(),
            vec![projection_grant(
                subject,
                PROJECTION_SERVICE,
                GRAPHSHELL_DOMAIN,
            )],
        );

        let elsewhere =
            SessionFacts::authenticated(PROJECTION_PROTOCOL, CarrierKind::P2panda, [42; 32]);
        let (_, outcome) = admit_session(
            &policy(),
            &RevocationLedger::default(),
            &hello,
            &elsewhere,
            NOW_MS,
            0,
        );
        assert!(
            outcome.is_err(),
            "a proof minted for one connection is worthless on another"
        );
    }
}
