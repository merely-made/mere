//! Current public sans-I/O admission, synthetic signed identities and facts.
use std::collections::BTreeMap;
use graphshell_host::admission::{self, CONNECT_ACTION, GRAPHSHELL_DOMAIN, PROJECTION_PROTOCOL, PROJECTION_SERVICE};
use insigne::delegation::{CapabilityScope, DelegationCertificate, DelegationParent, SignedDelegationCertificate};
use notochord::{CarrierKind, DenyReason, LocalNetworkPolicy, NetworkId, ProfileRef, ProofBinding, RevocationLedger, ServiceAccess, ServiceRule, SessionFacts, SessionHello, SessionReply, TrafficClass, TrustedRoot};
use personae::{IdentityProvider, InMemoryProvider};
use personae::delegation::Issue;

// Extracted byte-for-byte from policy_projection.rs by the runner.
include!("n4_policy_rule_extracted.rs");

const NETWORK: NetworkId = NetworkId([3; 32]);
const AUTHORITY: [u8; 32] = [7; 32];
fn owner() -> InMemoryProvider { InMemoryProvider::from_seed([1; 32]) }
fn viewer() -> InMemoryProvider { InMemoryProvider::from_seed([4; 32]) }
fn profile() -> ProfileRef { ProfileRef { id: "mere.base".into(), revision: 1 } }
fn policy(actions: &[&str]) -> LocalNetworkPolicy {
    let mut policy = LocalNetworkPolicy::closed(NETWORK);
    policy.accepted_profiles = vec![profile()];
    policy.trusted_roots = vec![TrustedRoot { authority: AUTHORITY, issuer: owner().master_public_key().to_bytes() }];
    policy.services = BTreeMap::from([(PROJECTION_SERVICE.into(), ServiceRule::new(ServiceAccess::MemberOnly, GRAPHSHELL_DOMAIN, actions.iter().copied(), false, None))]);
    policy
}
fn hello(action: &str, nonce: [u8; 32]) -> SessionHello {
    let viewer = viewer();
    let subject = viewer.master_public_key().to_bytes();
    let grant = SignedDelegationCertificate::issue(&owner(), DelegationCertificate::new(
        DelegationParent::Root(AUTHORITY), owner().master_public_key().to_bytes(), subject,
        CapabilityScope { domain: GRAPHSHELL_DOMAIN.into(), resource: NETWORK.0.to_vec(), path_prefix: PROJECTION_SERVICE.into(), actions: [action.to_string()].into_iter().collect() },
        5, 10, Some(100), 1, [1; 32])).unwrap();
    let mut requested = admission::connect_action();
    requested.action = action.into();
    admission::open_session_for(&viewer, NETWORK, profile(), requested, TrafficClass::Interactive, nonce,
        &ProofBinding::initiator(PROJECTION_PROTOCOL.to_vec(), Some(subject), None), vec![grant]).unwrap()
}
fn facts() -> SessionFacts {
    SessionFacts::authenticated(PROJECTION_PROTOCOL, CarrierKind::Memory, viewer().master_public_key().to_bytes())
}
fn decide(policy: &LocalNetworkPolicy, hello: &SessionHello) -> (SessionReply, Result<notochord::AdmittedPrincipal, DenyReason>) {
    let (bytes, outcome) = admission::admit_session(policy, &RevocationLedger::default(), &hello.encode(&policy.limits.clamped()).unwrap(), &facts(), 50, 0);
    (SessionReply::decode(&bytes, &policy.limits.clamped()).unwrap(), outcome)
}

#[test]
fn supported_action_control_agrees_on_success() {
    let (reply, outcome) = decide(&policy(&[CONNECT_ACTION]), &hello(CONNECT_ACTION, [5; 32]));
    assert!(reply.is_accept() && outcome.is_ok());
    println!("CONTROL connect wire=accept local=accepted");
}
#[test]
fn owner_policy_refusal_control_agrees_on_failure() {
    let (reply, outcome) = decide(&policy(&[CONNECT_ACTION]), &hello("administer", [6; 32]));
    assert!(!reply.is_accept() && outcome.is_err());
    println!("CONTROL policy-not-offered wire=reject local=refused");
}
#[test]
fn service_vocabulary_refusal_retains_the_accept_reply() {
    let (reply, outcome) = decide(&policy(&[CONNECT_ACTION, "administer"]), &hello("administer", [7; 32]));
    assert!(reply.is_accept());
    assert!(matches!(outcome, Err(DenyReason::ActionNotOffered)));
    println!("OBSERVED implementation-not-offered wire=accept local=ActionNotOffered");
}
#[test]
fn repeated_hello_reuses_the_session_id_but_fresh_nonce_changes_it() {
    let policy = policy(&[CONNECT_ACTION]);
    let hello_a = hello(CONNECT_ACTION, [8; 32]);
    let a = decide(&policy, &hello_a).1.unwrap();
    let b = decide(&policy, &hello_a).1.unwrap();
    let c = decide(&policy, &hello(CONNECT_ACTION, [9; 32])).1.unwrap();
    assert_eq!(a.session_id, b.session_id);
    assert_ne!(a.session_id, c.session_id);
    println!("OBSERVED repeated-hello both-admitted=true same-session-id=true fresh-nonce-id-differs=true");
}
#[test]
fn n4_projection_public_rule_offers_the_foreign_domain() {
    let mut mismatched = policy(&[CONNECT_ACTION]);
    let offered = rule(ServiceAccess::Public, Some(2));
    assert_eq!(offered.domain, "mere.network");
    assert_ne!(offered.domain, GRAPHSHELL_DOMAIN);
    mismatched.services.insert(PROJECTION_SERVICE.into(), offered);
    let (reply, outcome) = decide(&mismatched, &hello(CONNECT_ACTION, [10; 32]));
    assert!(!reply.is_accept() && outcome.is_err());
    println!("OBSERVED N4-template access=Public projection-domain=mere.network graphshell-hello-refused=true");
}
