use crate::ocapn_coop::*;
use gemot::moot::constitution::{CapabilityGrant, ConstitutionRules};
use gemot::moot::{
    AvailabilityPolicy, CollectionId, ErasurePolicy, KeepBound, Moot, MootAccessLevel, MootId,
    MootMember, MootMembershipAction, MootOutboundOperation, MootRetentionSettings, PolicyRevision,
};
use insigne::delegation::{
    CapabilityScope, DelegationCertificate, DelegationParent, DelegationRevocation,
    SignedDelegationCertificate, SignedDelegationRevocation,
};
use muniment::MemoryBackend;
use notochord::{
    CarrierKind, LocalNetworkPolicy, NetworkId, ProfileRef, RequestedAction, RevocationLedger,
    SecureChannelFacts, SecureIdentityPolicy, ServiceAccess, ServiceRule, SessionFacts,
    SessionHello, TrafficClass, TrustedRoot,
};
use personae::delegation::Issue;
use personae::{IdentityProvider, InMemoryProvider};
use proofs::Digest;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

const MOOT: [u8; 32] = [71; 32];
const COLLECTION: [u8; 32] = [72; 32];
const NETWORK: NetworkId = NetworkId(MOOT);
const AUTHORITY: [u8; 32] = [73; 32];
const WRITER_GRANT: [u8; 32] = [74; 32];
fn host() -> InMemoryProvider {
    InMemoryProvider::from_seed([11; 32])
}
fn writer() -> InMemoryProvider {
    InMemoryProvider::from_seed([12; 32])
}
fn reader() -> InMemoryProvider {
    InMemoryProvider::from_seed([13; 32])
}
fn retention() -> MootRetentionSettings {
    MootRetentionSettings {
        revision: PolicyRevision(Digest::blake3(b"ocapn-coop-test")),
        availability: AvailabilityPolicy {
            promised_floor: KeepBound::Forever,
        },
        erasure: ErasurePolicy {
            history_ceiling: KeepBound::UntilCheckpoint,
        },
    }
}
struct Queue(Mutex<Vec<MootOutboundOperation>>);
impl PublicationHook for Queue {
    fn offer(
        &self,
        _: &gemot::moot::MootCommandReceipt,
        operation: &MootOutboundOperation,
    ) -> bool {
        let mut queue = self.0.lock().unwrap();
        if queue.len() >= 8 {
            return false;
        }
        queue.push(operation.clone());
        true
    }
}
struct Fixture {
    owner: CollectionOwner<MemoryBackend, InMemoryProvider>,
    peer: Moot<MemoryBackend>,
    clock: Arc<AtomicU64>,
    rules: ConstitutionRules,
    contribution: [u8; 32],
    queue: Arc<Queue>,
}
async fn replicate(
    source: &Moot<MemoryBackend>,
    peer: &Moot<MemoryBackend>,
    receipt: &gemot::moot::MootCommandReceipt,
) {
    match source.outbound(receipt).await.unwrap() {
        MootOutboundOperation::Membership(operation) => {
            peer.membership_store().accept(&operation).await.unwrap();
        },
        MootOutboundOperation::Object(operation) => {
            peer.object_store().accept(MOOT, &operation).await.unwrap();
        },
        _ => panic!("unexpected lane"),
    }
}
impl Fixture {
    async fn new() -> Self {
        Self::with_clock(None).await
    }
    async fn with_clock(clock_override: Option<Arc<dyn Fn() -> u64 + Send + Sync>>) -> Self {
        let a = host().master_public_key().to_bytes();
        let w = writer().master_public_key().to_bytes();
        let r = reader().master_public_key().to_bytes();
        let mut rules = ConstitutionRules::founder_only(a);
        rules.admission = mien::Policy::MembersOnly {
            rate_limit: 100,
            rate_window_ms: 1000,
        };
        let path = servitor::cap_path(&gemot::moot::collection_cap(gemot::moot::CollectionRef {
            moot_id: MOOT,
            collection_id: CollectionId(COLLECTION),
        }));
        for (id, subject, path_prefix) in [
            (
                [75; 32],
                a,
                servitor::cap_path(&servitor::Cap::scope("moot").unwrap()),
            ),
            (WRITER_GRANT, w, path.clone()),
            ([76; 32], r, path),
        ] {
            rules.grant(CapabilityGrant {
                id,
                subject,
                path_prefix,
                not_before_ms: 1,
                expires_at_ms: None,
                delegation_depth: 0,
            });
        }
        let moot = Moot::in_memory(MootId(MOOT), a, retention());
        let peer = Moot::in_memory(MootId(MOOT), a, retention());
        for instance in [&moot, &peer] {
            instance
                .found(
                    host().master_keypair().to_seed(),
                    None,
                    None,
                    rules.clone(),
                    1,
                )
                .await
                .unwrap();
        }
        let membership = moot
            .update_membership_for_identity(
                &host(),
                MootMembershipAction::Create {
                    initial_members: vec![
                        MootMember {
                            member: a,
                            access: MootAccessLevel::Manage,
                        },
                        MootMember {
                            member: w,
                            access: MootAccessLevel::Write,
                        },
                        MootMember {
                            member: r,
                            access: MootAccessLevel::Read,
                        },
                    ],
                },
            )
            .await
            .unwrap();
        replicate(&moot, &peer, &membership).await;
        let declared = moot
            .declare_collection_for_identity(
                &host(),
                CollectionId(COLLECTION),
                "Live notebooks".into(),
                None,
                2,
            )
            .await
            .unwrap();
        replicate(&moot, &peer, &declared).await;
        let shared = moot
            .share_authorized_for_identity(
                &host(),
                [81; 32],
                "test.notebook.v1".into(),
                "Notebook".into(),
                3,
            )
            .await
            .unwrap();
        replicate(&moot, &peer, &shared).await;
        let contribution = shared.operation;
        let clock = Arc::new(AtomicU64::new(10));
        let c = clock.clone();
        let queue = Arc::new(Queue(Mutex::new(Vec::new())));
        let owner = CollectionOwner::new(
            OwnerState::new(moot, host(), RevocationLedger::new()),
            CollectionId(COLLECTION),
            NETWORK,
            clock_override.unwrap_or_else(|| Arc::new(move || c.load(Ordering::SeqCst))),
            queue.clone(),
        );
        Self {
            owner,
            peer,
            clock,
            rules,
            contribution,
            queue,
        }
    }
}
fn grant(subject: [u8; 32]) -> SignedDelegationCertificate {
    SignedDelegationCertificate::issue(
        &host(),
        DelegationCertificate::new(
            DelegationParent::Root(AUTHORITY),
            host().master_public_key().to_bytes(),
            subject,
            CapabilityScope {
                domain: SERVICE_DOMAIN.into(),
                resource: MOOT.to_vec(),
                path_prefix: SERVICE_PATH.into(),
                actions: [SERVICE_ACTION.to_string()].into_iter().collect(),
            },
            0,
            1,
            Some(100),
            1,
            [82; 32],
        ),
    )
    .unwrap()
}
async fn admitted(
    provider: &InMemoryProvider,
    nonce: u8,
) -> notochord::AdmittedSecureSession<tokio::io::DuplexStream> {
    let subject = provider.master_public_key().to_bytes();
    let facts = SessionFacts::authenticated(SERVICE_PROTOCOL, CarrierKind::Memory, subject);
    let secure = SecureChannelFacts::new(
        b"test/verified-channel".to_vec(),
        1,
        vec![nonce; 64],
        subject,
        host().master_public_key().to_bytes(),
        SERVICE_PROTOCOL,
    );
    let mut policy = LocalNetworkPolicy::closed(NETWORK);
    let profile = ProfileRef {
        id: "mere.base".into(),
        revision: 1,
    };
    policy.accepted_profiles.push(profile.clone());
    policy.trusted_roots.push(TrustedRoot {
        authority: AUTHORITY,
        issuer: host().master_public_key().to_bytes(),
    });
    policy.services.insert(
        SERVICE_PATH.into(),
        ServiceRule::new(
            ServiceAccess::MemberOnly,
            SERVICE_DOMAIN,
            [SERVICE_ACTION],
            true,
            None,
        ),
    );
    let hello = SessionHello::issue_secure(
        provider,
        NETWORK,
        profile,
        RequestedAction {
            domain: SERVICE_DOMAIN.into(),
            path: SERVICE_PATH.into(),
            action: SERVICE_ACTION.into(),
        },
        TrafficClass::Interactive,
        [nonce; 32],
        &facts.secure_proof_binding(&secure),
        vec![grant(subject)],
    )
    .unwrap();
    let (mut client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(async move {
        notochord::initiate_session(&mut client, &hello, &Default::default())
            .await
            .unwrap()
    });
    let result = notochord::admit_session_secure(
        server,
        &policy,
        &RevocationLedger::new(),
        &facts,
        &secure,
        SecureIdentityPolicy::MatchCarrierWhenPresent,
        10,
        0,
    )
    .await
    .unwrap()
    .unwrap();
    assert!(task.await.unwrap().is_accept());
    result
}
async fn authority(provider: &InMemoryProvider, nonce: u8) -> MootSessionAuthority {
    MootSessionAuthority::from_admitted(&admitted(provider, nonce).await, NETWORK).unwrap()
}

#[tokio::test]
async fn a_read_member_reads_but_cannot_submit_a_write_even_with_a_typed_grant() {
    let f = Fixture::new().await;
    let auth = authority(&reader(), 1).await;
    let view = f.owner.read(&auth).await.unwrap();
    assert_eq!(
        view.lifecycle.membership.unwrap().access,
        crate::coop::Access::Read
    );
    let error = f
        .owner
        .propose(
            &auth,
            MembershipProposal {
                expected_heads: view.heads,
                contribution: f.contribution,
                included: true,
            },
        )
        .await
        .err()
        .unwrap();
    assert_eq!(error, CoopError::ReadOnly);
    assert!(f.queue.0.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_bounded_proposal_is_a_normal_host_authored_gemot_operation_and_replicates() {
    let f = Fixture::new().await;
    let auth = authority(&writer(), 2).await;
    let view = f.owner.read(&auth).await.unwrap();
    let proposal = MembershipProposal {
        expected_heads: view.heads,
        contribution: f.contribution,
        included: true,
    };
    let accepted = f.owner.propose(&auth, proposal.clone()).await.unwrap();
    assert_eq!(
        accepted.receipt.submitted_by,
        writer().master_public_key().to_bytes()
    );
    assert_eq!(
        accepted.receipt.authored_by,
        host().master_public_key().to_bytes()
    );
    assert_eq!(
        accepted.receipt.operation,
        accepted.domain_receipt.operation
    );
    assert!(accepted.receipt.publication_queued);
    assert_eq!(f.queue.0.lock().unwrap().len(), 1);
    match accepted.outbound {
        MootOutboundOperation::Object(operation) => {
            f.peer
                .object_store()
                .accept(MOOT, &operation)
                .await
                .unwrap();
        },
        _ => panic!("collection command belongs in ordinary object lane"),
    }
    let peer = f
        .peer
        .authorized_collection(CollectionId(COLLECTION), 10)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(peer.heads, accepted.receipt.heads);
    assert_eq!(peer.effective_selected[0].share, f.contribution);
    assert_eq!(
        f.owner
            .propose(&auth, proposal.clone())
            .await
            .err()
            .unwrap(),
        CoopError::StaleHeads
    );
    let new_session = authority(&writer(), 3).await;
    assert_ne!(auth.epoch(), new_session.epoch());
    assert_eq!(
        f.owner.propose(&new_session, proposal).await.err().unwrap(),
        CoopError::StaleHeads
    );
    assert_eq!(f.queue.0.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn service_revocation_and_expiry_apply_to_every_copy_of_the_admitted_authority() {
    let f = Fixture::new().await;
    let auth = authority(&writer(), 4).await;
    let copy = auth.clone();
    assert!(f.owner.read(&copy).await.is_ok());
    let certificate = grant(auth.subject());
    let revocation = SignedDelegationRevocation::issue(
        &host(),
        DelegationRevocation::new(
            certificate.certificate.id(),
            host().master_public_key().to_bytes(),
            certificate.certificate.scope.clone(),
            11,
            [83; 32],
        ),
    )
    .unwrap();
    f.owner
        .host_update()
        .await
        .revocations
        .fold(revocation.check().unwrap());
    assert_eq!(f.owner.read(&auth).await.unwrap_err(), CoopError::Revoked);
    assert_eq!(f.owner.read(&copy).await.unwrap_err(), CoopError::Revoked);
    let f = Fixture::new().await;
    let auth = authority(&writer(), 5).await;
    f.clock.store(101, Ordering::SeqCst);
    assert_eq!(f.owner.read(&auth).await.unwrap_err(), CoopError::Expired);
}

#[tokio::test]
async fn current_signed_collection_grant_and_membership_are_required_on_every_read() {
    let f = Fixture::new().await;
    let auth = authority(&writer(), 6).await;
    assert!(f.owner.read(&auth).await.is_ok());
    let mut rules = f.rules.clone();
    rules.revoke_grant(&WRITER_GRANT);
    f.owner
        .host_update()
        .await
        .moot
        .amend(host().master_keypair().to_seed(), rules, 11)
        .await
        .unwrap();
    assert_eq!(
        f.owner.read(&auth).await.unwrap_err(),
        CoopError::CapabilityDenied
    );
}

#[tokio::test]
async fn a_subject_or_action_changed_after_admission_cannot_construct_context() {
    let mut accepted = admitted(&writer(), 7).await;
    accepted.session.principal.subject = reader().master_public_key().to_bytes();
    assert!(matches!(
        MootSessionAuthority::from_admitted(&accepted, NETWORK),
        Err(CoopError::AdmissionContext)
    ));
    let mut accepted = admitted(&writer(), 8).await;
    accepted.session.principal.action.action = "write-anything".into();
    assert!(matches!(
        MootSessionAuthority::from_admitted(&accepted, NETWORK),
        Err(CoopError::AdmissionContext)
    ));
    assert!(matches!(
        MootSessionAuthority::from_admitted(&admitted(&writer(), 9).await, NetworkId([99; 32])),
        Err(CoopError::AdmissionContext)
    ));
}

struct RuntimePair {
    client: murm_ocapn::SessionHandle,
    server: murm_ocapn::SessionHandle,
    object: murm_ocapn::Capability,
    client_task: tokio::task::JoinHandle<Result<(), murm_ocapn::SessionError>>,
    server_task: tokio::task::JoinHandle<Result<(), murm_ocapn::SessionError>>,
}
impl Drop for RuntimePair {
    fn drop(&mut self) {
        self.client_task.abort();
        self.server_task.abort();
    }
}
async fn runtime(
    f: &Fixture,
    object_auth: MootSessionAuthority,
    context_auth: MootSessionAuthority,
    command_queue: usize,
) -> RuntimePair {
    use murm_ocapn::{Bootstrap, PeerLocator, SessionAuthority, SessionConfig, attach};
    let (client, server) = tokio::io::duplex(64 * 1024);
    let (cr, cw) = tokio::io::split(client);
    let (sr, sw) = tokio::io::split(server);
    let mut bootstrap = Bootstrap::new();
    bootstrap
        .register(
            b"collection".to_vec(),
            f.owner.session_object(object_auth).unwrap(),
        )
        .unwrap();
    let mut server_config = SessionConfig::new(
        PeerLocator::new("fixture", "server"),
        SessionAuthority::new(context_auth),
    );
    server_config.limits.max_command_queue = command_queue;
    server_config.limits.request_timeout = std::time::Duration::from_secs(1);
    let mut client_config = SessionConfig::new(
        PeerLocator::new("fixture", "client"),
        SessionAuthority::new(()),
    );
    client_config.limits.request_timeout = std::time::Duration::from_secs(1);
    let (client, client_driver) = attach(cr, cw, client_config, Bootstrap::new()).unwrap();
    let (server, server_driver) = attach(sr, sw, server_config, bootstrap).unwrap();
    let client_task = tokio::spawn(client_driver.run());
    let server_task = tokio::spawn(server_driver.run());
    client.wait_ready().await.unwrap();
    server.wait_ready().await.unwrap();
    let value = client
        .fetch(b"collection")
        .await
        .unwrap()
        .resolve()
        .await
        .unwrap();
    let object = value.as_capability().unwrap().clone();
    RuntimePair {
        client,
        server,
        object,
        client_task,
        server_task,
    }
}
async fn call(
    pair: &RuntimePair,
    args: Vec<murm_ocapn::Value>,
) -> Result<murm_ocapn::Value, murm_ocapn::CallError> {
    pair.client.call(&pair.object, args).await?.resolve().await
}
fn payload<T: serde::de::DeserializeOwned>(value: murm_ocapn::Value) -> T {
    let murm_ocapn::Value::Record { fields, .. } = value else {
        panic!("expected typed record")
    };
    let [murm_ocapn::Value::Bytes(bytes)] = fields.as_slice() else {
        panic!("expected bounded record body")
    };
    serde_json::from_slice(bytes).unwrap()
}

#[tokio::test]
async fn real_captp_calls_use_the_admitted_context_and_commit_the_domain_proposal() {
    use murm_ocapn::Value;
    let f = Fixture::new().await;
    let auth = authority(&writer(), 20).await;
    let pair = runtime(&f, auth.clone(), auth.clone(), 16).await;
    let view: CollectionSnapshot = payload(call(&pair, vec![Value::symbol("view")]).await.unwrap());
    let proposal = MembershipProposal {
        expected_heads: view.heads,
        contribution: f.contribution,
        included: true,
    };
    let receipt: ProposalReceipt = payload(
        call(
            &pair,
            vec![
                Value::symbol("set-membership"),
                Value::Bytes(serde_json::to_vec(&proposal).unwrap()),
            ],
        )
        .await
        .unwrap(),
    );
    assert_eq!(receipt.submitted_by, auth.subject());
    assert_eq!(receipt.authored_by, host().master_public_key().to_bytes());
    assert!(
        call(
            &pair,
            vec![
                Value::symbol("set-membership"),
                Value::Bytes(serde_json::to_vec(&proposal).unwrap())
            ]
        )
        .await
        .is_err()
    );
    assert!(
        call(
            &pair,
            vec![
                Value::symbol("view"),
                Value::Bytes(reader().master_public_key().to_bytes().to_vec())
            ]
        )
        .await
        .is_err(),
        "wire arguments cannot select a subject"
    );
    let queued = f.queue.0.lock().unwrap().clone();
    assert_eq!(queued.len(), 1);
    match &queued[0] {
        MootOutboundOperation::Object(operation) => {
            f.peer.object_store().accept(MOOT, operation).await.unwrap();
        },
        _ => panic!("ordinary object operation"),
    }
    assert_eq!(
        f.peer
            .authorized_collection(CollectionId(COLLECTION), 10)
            .await
            .unwrap()
            .unwrap()
            .heads,
        receipt.heads
    );
}

#[tokio::test]
async fn an_object_cannot_acquire_another_subject_or_reconnect_epoch_from_host_context() {
    use murm_ocapn::Value;
    let f = Fixture::new().await;
    let old = authority(&writer(), 21).await;
    let wrong_subject = authority(&reader(), 22).await;
    let pair = runtime(&f, old.clone(), wrong_subject, 16).await;
    assert!(call(&pair, vec![Value::symbol("view")]).await.is_err());
    drop(pair);
    let fresh = authority(&writer(), 23).await;
    let pair = runtime(&f, old, fresh, 16).await;
    assert!(call(&pair, vec![Value::symbol("view")]).await.is_err());
}

struct Callback {
    updates: Arc<Mutex<usize>>,
    slow: bool,
}
impl murm_ocapn::HostObject for Callback {
    fn call<'a>(
        &'a self,
        _: murm_ocapn::CallContext,
        args: Vec<murm_ocapn::Value>,
    ) -> murm_ocapn::BoxFuture<'a, Result<murm_ocapn::Value, murm_ocapn::CallError>> {
        Box::pin(async move {
            if self.slow {
                std::future::pending::<()>().await;
            }
            assert!(
                matches!(args.first(),Some(murm_ocapn::Value::Symbol(method)) if method=="changed")
            );
            *self.updates.lock().unwrap() += 1;
            Ok(murm_ocapn::Value::Bool(true))
        })
    }
}
async fn subscribe(pair: &RuntimePair, slow: bool) -> Arc<Mutex<usize>> {
    use murm_ocapn::Value;
    let updates = Arc::new(Mutex::new(0));
    let cap = pair
        .client
        .export(Arc::new(Callback {
            updates: updates.clone(),
            slow,
        }))
        .await
        .unwrap();
    assert!(matches!(
        call(pair, vec![Value::symbol("subscribe"), Value::Cap(cap)])
            .await
            .unwrap(),
        Value::Integer(_)
    ));
    updates
}

#[tokio::test]
async fn subscriptions_recheck_revocation_and_disconnect_releases_callback_caps() {
    use murm_ocapn::Value;
    let f = Fixture::new().await;
    let auth = authority(&writer(), 24).await;
    let pair = runtime(&f, auth.clone(), auth.clone(), 16).await;
    let updates = subscribe(&pair, false).await;
    assert_eq!(f.owner.subscription_count().await, 1);
    assert_eq!(f.owner.emit_current().await.enqueued, 1);
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while *updates.lock().unwrap() == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let copy = pair.object.clone();
    let certificate = grant(auth.subject());
    let revocation = SignedDelegationRevocation::issue(
        &host(),
        DelegationRevocation::new(
            certificate.certificate.id(),
            host().master_public_key().to_bytes(),
            certificate.certificate.scope.clone(),
            11,
            [84; 32],
        ),
    )
    .unwrap();
    f.owner
        .host_update()
        .await
        .revocations
        .fold(revocation.check().unwrap());
    assert_eq!(f.owner.emit_current().await.removed, 1);
    assert_eq!(f.owner.subscription_count().await, 0);
    assert!(call(&pair, vec![Value::symbol("view")]).await.is_err());
    assert!(
        pair.client
            .call(&copy, vec![Value::symbol("view")])
            .await
            .unwrap()
            .resolve()
            .await
            .is_err()
    );
    assert_eq!(*updates.lock().unwrap(), 1);
    drop(pair);

    let f = Fixture::new().await;
    let auth = authority(&writer(), 25).await;
    let pair = runtime(&f, auth.clone(), auth, 16).await;
    subscribe(&pair, false).await;
    pair.client.close("disconnect").await.unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while f.owner.subscription_count().await != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(pair.server.is_closed());
}

#[tokio::test]
async fn a_slow_callback_cannot_grow_the_owner_queue_or_block_authority_updates() {
    let f = Fixture::new().await;
    let auth = authority(&writer(), 26).await;
    let pair = runtime(&f, auth.clone(), auth, 1).await;
    subscribe(&pair, true).await;
    // Current-thread execution and local ready stores let the bounded queue
    // fill before the driver is polled again; no callback response is awaited.
    let first = f.owner.emit_current().await;
    let second = f.owner.emit_current().await;
    assert_eq!(first.enqueued, 1);
    assert_eq!(second.removed, 1);
    assert_eq!(f.owner.subscription_count().await, 0);
    let update = tokio::time::timeout(std::time::Duration::from_millis(100), f.owner.host_update())
        .await
        .unwrap();
    drop(update);
    pair.client.close("cancel slow callback").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn existing_native_endpoints_admit_root_authority_then_run_the_real_gemot_adapter() {
    use murm_ocapn::{Bootstrap, PeerLocator, SessionAuthority, SessionConfig, Value, attach};
    use murm_ocapn_transport::{Credentials, NativeService};
    use transport::security::SetupLimits;
    use transport::{P2pandaTransport, Transport};
    tokio::time::timeout(std::time::Duration::from_secs(20), async {
        let f = Fixture::new().await;
        let server_identity = host();
        let client_identity = writer();
        let server_key = server_identity
            .derived_keypair(b"test/gemot-existing-endpoint")
            .unwrap();
        let client_key = client_identity
            .derived_keypair(b"test/gemot-existing-endpoint")
            .unwrap();
        let service = NativeService::gemot(2, SetupLimits::default());
        let server = P2pandaTransport::builder_for(&server_key)
            .alpns(vec![])
            .bind()
            .await
            .unwrap();
        let client = P2pandaTransport::builder_for(&client_key)
            .alpns(vec![])
            .bind()
            .await
            .unwrap();
        let listener = service
            .register(&server, 2, std::time::Duration::from_secs(1))
            .await
            .unwrap();
        assert_ne!(
            server.local_peer_id().to_bytes(),
            server_identity.master_public_key().to_bytes()
        );
        assert_ne!(
            client.local_peer_id().to_bytes(),
            client_identity.master_public_key().to_bytes()
        );
        client
            .add_peer(server.endpoint_addr().await.unwrap())
            .await
            .unwrap();
        server
            .add_peer(client.endpoint_addr().await.unwrap())
            .await
            .unwrap();
        let profile = ProfileRef {
            id: "mere.base".into(),
            revision: 1,
        };
        let mut policy = LocalNetworkPolicy::closed(NETWORK);
        policy.accepted_profiles.push(profile.clone());
        policy.trusted_roots.push(TrustedRoot {
            authority: AUTHORITY,
            issuer: server_identity.master_public_key().to_bytes(),
        });
        policy.services.insert(
            SERVICE_PATH.into(),
            ServiceRule::new(
                ServiceAccess::MemberOnly,
                SERVICE_DOMAIN,
                [SERVICE_ACTION],
                false,
                Some(2),
            ),
        );
        let ledger = RevocationLedger::new();
        let (admitted, connected) = tokio::join!(
            listener.accept(&policy, &ledger, || 10, 0),
            service.connect(
                &client,
                server.local_peer_id(),
                Credentials {
                    identity: &client_identity,
                    network: NETWORK,
                    profile,
                    nonce: [40; 32],
                    delegations: vec![grant(client_identity.master_public_key().to_bytes())]
                }
            ),
        );
        let admitted = admitted.unwrap();
        let connected = connected.unwrap();
        assert_eq!(
            admitted.session.principal.subject,
            client_identity.master_public_key().to_bytes()
        );
        assert_eq!(
            admitted.session.facts.authenticated_initiator,
            Some(client.local_peer_id().to_bytes())
        );
        assert_eq!(admitted.secure_channel, connected.secure_channel);
        let authority = MootSessionAuthority::from_admitted(&admitted, NETWORK).unwrap();
        assert_ne!(authority.subject(), admitted.secure_channel.initiator);
        let object = f.owner.session_object(authority.clone()).unwrap();
        let mut bootstrap = Bootstrap::new();
        bootstrap.register(b"collection".to_vec(), object).unwrap();
        let server_locator = PeerLocator::new("murm-iroh", "gemot-server");
        let client_locator = PeerLocator::new("murm-iroh", "gemot-client");
        let mut server_config = SessionConfig::new(
            server_locator.clone(),
            SessionAuthority::new(authority.clone()),
        );
        server_config.expected_remote = Some(client_locator.clone());
        let mut client_config = SessionConfig::new(client_locator, SessionAuthority::new(()));
        client_config.expected_remote = Some(server_locator);
        let (sr, sw) = tokio::io::split(admitted.into_stream());
        let (cr, cw) = tokio::io::split(connected.stream);
        let (server_handle, server_driver) = attach(sr, sw, server_config, bootstrap).unwrap();
        let (client_handle, client_driver) =
            attach(cr, cw, client_config, Bootstrap::new()).unwrap();
        let server_task = tokio::spawn(server_driver.run());
        let client_task = tokio::spawn(client_driver.run());
        client_handle.wait_ready().await.unwrap();
        let cap = client_handle
            .fetch(b"collection")
            .await
            .unwrap()
            .resolve()
            .await
            .unwrap()
            .as_capability()
            .unwrap()
            .clone();
        let view: CollectionSnapshot = payload(
            client_handle
                .call(&cap, vec![Value::symbol("view")])
                .await
                .unwrap()
                .resolve()
                .await
                .unwrap(),
        );
        let proposal = MembershipProposal {
            expected_heads: view.heads,
            contribution: f.contribution,
            included: true,
        };
        let receipt: ProposalReceipt = payload(
            client_handle
                .call(
                    &cap,
                    vec![
                        Value::symbol("set-membership"),
                        Value::Bytes(serde_json::to_vec(&proposal).unwrap()),
                    ],
                )
                .await
                .unwrap()
                .resolve()
                .await
                .unwrap(),
        );
        assert_eq!(receipt.submitted_by, authority.subject());
        assert_eq!(
            receipt.authored_by,
            server_identity.master_public_key().to_bytes()
        );
        assert!(receipt.publication_queued);
        let queued = f.queue.0.lock().unwrap().clone();
        assert_eq!(queued.len(), 1);
        match &queued[0] {
            MootOutboundOperation::Object(operation) => {
                f.peer.object_store().accept(MOOT, operation).await.unwrap();
            },
            _ => panic!("ordinary Gemot object command"),
        }
        assert_eq!(
            f.peer
                .authorized_collection(CollectionId(COLLECTION), 10)
                .await
                .unwrap()
                .unwrap()
                .heads,
            receipt.heads
        );
        client_handle.close("fixture complete").await.unwrap();
        server_handle.close("fixture complete").await.unwrap();
        client_task.abort();
        server_task.abort();
    })
    .await
    .expect("native Gemot composition deadline");
}

#[tokio::test]
async fn a_current_membership_removal_denies_reading_even_when_the_signed_grant_remains() {
    let f = Fixture::new().await;
    let auth = authority(&writer(), 41).await;
    let copy = auth.clone();
    assert!(f.owner.read(&auth).await.is_ok());
    f.owner
        .host_update()
        .await
        .moot
        .update_membership_for_identity(
            &host(),
            MootMembershipAction::Remove {
                member: auth.subject(),
            },
        )
        .await
        .unwrap();
    assert_eq!(f.owner.read(&auth).await.unwrap_err(), CoopError::NotMember);
    assert_eq!(f.owner.read(&copy).await.unwrap_err(), CoopError::NotMember);
}

#[tokio::test]
async fn callback_reference_copies_do_not_bypass_the_per_session_subscription_ceiling() {
    use murm_ocapn::Value;
    let f = Fixture::new().await;
    let auth = authority(&writer(), 42).await;
    let pair = runtime(&f, auth.clone(), auth, 16).await;
    let cap = pair
        .client
        .export(Arc::new(Callback {
            updates: Arc::new(Mutex::new(0)),
            slow: false,
        }))
        .await
        .unwrap();
    for _ in 0..4 {
        assert!(matches!(
            call(
                &pair,
                vec![Value::symbol("subscribe"), Value::Cap(cap.clone())]
            )
            .await
            .unwrap(),
            Value::Integer(_)
        ));
    }
    assert!(
        call(&pair, vec![Value::symbol("subscribe"), Value::Cap(cap)])
            .await
            .is_err()
    );
    assert_eq!(f.owner.subscription_count().await, 4);
    pair.client.close("quota fixture complete").await.unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while f.owner.subscription_count().await != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn a_collection_grant_expiring_during_snapshot_preparation_cannot_enqueue_a_callback() {
    let phase = Arc::new(AtomicU64::new(0));
    let c = phase.clone();
    let clock = Arc::new(move || {
        if c.load(Ordering::SeqCst) == 0 || c.fetch_add(1, Ordering::SeqCst) == 1 {
            10
        } else {
            101
        }
    });
    let f = Fixture::with_clock(Some(clock)).await;
    let auth = authority(&writer(), 43).await;
    let pair = runtime(&f, auth.clone(), auth, 16).await;
    let updates = subscribe(&pair, false).await;
    let mut rules = f.rules.clone();
    rules
        .capability_grants
        .get_mut(&WRITER_GRANT)
        .unwrap()
        .expires_at_ms = Some(20);
    f.owner
        .host_update()
        .await
        .moot
        .amend(host().master_keypair().to_seed(), rules, 11)
        .await
        .unwrap();
    phase.store(1, Ordering::SeqCst);
    let report = f.owner.emit_current().await;
    assert_eq!(report.enqueued, 0);
    assert_eq!(report.removed, 1);
    assert_eq!(f.owner.subscription_count().await, 0);
    assert_eq!(*updates.lock().unwrap(), 0);
}
