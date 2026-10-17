//! Native public notifying loop over a duplex; constructed admitted session,
//! real signed grant/revocation, no network handshake or installed resident.
use std::sync::{Arc, Mutex, RwLock, atomic::{AtomicU64, Ordering}};
use std::time::Duration;
use chirograph::*;
use graphshell_endpoint::{ProjectionCatalog, ProjectionSource, PresentationSource, IntentSink, ProjectionNoticeSource};
use graphshell_host::{admission::*, lifecycle::SessionAuthority, session_loop::SessionEnd, session_notices::serve_admitted_session_notifying};
use insigne::delegation::{CapabilityScope, DelegationCertificate, DelegationParent, SignedDelegationCertificate, DelegationRevocation, SignedDelegationRevocation};
use notochord::{AdmittedPrincipal, AdmittedSession, CarrierKind, NetworkId, ProfileRef, RevocationLedger, SessionClaims, SessionFacts, TrafficClass};
use personae::{IdentityProvider, InMemoryProvider, delegation::Issue};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

fn owner() -> InMemoryProvider { InMemoryProvider::from_seed([1; 32]) }
fn viewer() -> InMemoryProvider { InMemoryProvider::from_seed([4; 32]) }
fn grant() -> SignedDelegationCertificate {
    SignedDelegationCertificate::issue(&owner(), DelegationCertificate::new(
        DelegationParent::Root([7; 32]), owner().master_public_key().to_bytes(), viewer().master_public_key().to_bytes(),
        CapabilityScope { domain: GRAPHSHELL_DOMAIN.into(), resource: vec![3; 32], path_prefix: PROJECTION_SERVICE.into(), actions: [CONNECT_ACTION.into()].into_iter().collect() },
        5, 10, Some(100), 1, [1; 32])).unwrap()
}
fn admitted(stream: tokio::io::DuplexStream) -> AdmittedSession<tokio::io::DuplexStream> {
    let principal = AdmittedPrincipal { subject: viewer().master_public_key().to_bytes(), class: TrafficClass::Interactive, session_id: [21; 32], action: connect_action() };
    AdmittedSession { stream, claims: SessionClaims { wire_version: 1, network: NetworkId([3; 32]), profile: ProfileRef { id: "mere.base".into(), revision: 1 }, action: principal.action.clone(), class: principal.class, subject: principal.subject, delegations: vec![grant()] },
        facts: SessionFacts::authenticated(PROJECTION_PROTOCOL, CarrierKind::Memory, principal.subject), principal, limits: Default::default() }
}
struct Bell { pending: Arc<Mutex<Vec<CarrierNotice>>> }
impl ProjectionCatalog for Bell {
    fn describe(&self) -> EndpointDescriptor { EndpointDescriptor { label: "authority probe".into(), projections: vec![] } }
}
impl ProjectionSource for Bell {
    type Error = String;
    fn snapshot(&mut self, _: ProjectionRequest) -> Result<ProjectionSnapshot, String> { Err("not used".into()) }
}
impl PresentationSource for Bell {
    type Error = String;
    fn resource(&mut self, _: ResourceRequest) -> Result<ResourceResponse, String> { Err("not used".into()) }
}
impl IntentSink for Bell {
    type Error = String;
    fn invoke(&mut self, _: IntentInvocation) -> Result<IntentResult, String> { Err("not used".into()) }
}
impl ProjectionNoticeSource for Bell {
    type Error = String;
    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, String> { Ok(self.pending.lock().unwrap().pop()) }
}
fn notice(revision: u64) -> CarrierNotice { CarrierNotice { session: ProjectionSession("probe:scene".into()), epoch: SceneEpoch(3), revision: Revision(revision) } }

fn observe(mode: &str) {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        tokio::time::timeout(Duration::from_secs(5), async {
            let (peer, stream) = tokio::io::duplex(65536);
            let mut session = admitted(stream);
            let authority = SessionAuthority::retain_admitted(&session);
            let retained = authority.clone();
            let clock = Arc::new(AtomicU64::new(50));
            let serve_clock = Arc::clone(&clock);
            let ledger = Arc::new(RwLock::new(RevocationLedger::new()));
            let serve_ledger = Arc::clone(&ledger);
            let pending = Arc::new(Mutex::new(vec![notice(1)]));
            let mut endpoint = Bell { pending: Arc::clone(&pending) };
            let served = tokio::spawn(async move {
                let mut resume = |_: &mut Bell, _: ResumeRequest| -> Result<ResumeReply, String> { Err("not used".into()) };
                serve_admitted_session_notifying(&mut session, &authority, &serve_ledger, &mut endpoint, &mut resume,
                    || serve_clock.load(Ordering::SeqCst), Duration::from_millis(10)).await.unwrap()
            });
            let (read, mut write) = tokio::io::split(peer);
            let mut lines = BufReader::new(read).lines();
            let first: CarrierOutput = serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
            assert!(matches!(first, CarrierOutput::Notice(n) if n == notice(1)));
            assert!(retained.lapse(&ledger.read().unwrap(), 50).is_none());
            if mode == "expired" { clock.store(101, Ordering::SeqCst); }
            if mode == "revoked" {
                let certificate = grant();
                let statement = SignedDelegationRevocation::issue(&owner(), DelegationRevocation::new(
                    certificate.certificate.id(), owner().master_public_key().to_bytes(), certificate.certificate.scope.clone(), 50, [2; 32])).unwrap();
                ledger.write().unwrap().fold(statement.check().unwrap());
            }
            let lapse = retained.lapse(&ledger.read().unwrap(), clock.load(Ordering::SeqCst));
            assert_eq!(lapse.is_some(), mode != "live");
            pending.lock().unwrap().push(notice(2));
            let second: CarrierOutput = serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
            assert!(matches!(second, CarrierOutput::Notice(n) if n == notice(2)), "the idle peer receives the newly queued notice");
            let request = CarrierRequest { id: 1, body: CarrierRequestBody::Close };
            write.write_all(format!("{}\n", serde_json::to_string(&request).unwrap()).as_bytes()).await.unwrap();
            let answer: CarrierOutput = serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
            let summary = served.await.unwrap();
            match mode {
                "live" => {
                    assert!(matches!(answer, CarrierOutput::Response(r) if matches!(r.body, Ok(CarrierResponseBody::Closed))));
                    assert_eq!(summary.end, SessionEnd::Closed);
                },
                _ => {
                    assert!(matches!(answer, CarrierOutput::Response(r) if r.body.is_err()));
                    assert!(matches!(summary.end, SessionEnd::Lapsed(_)));
                }
            }
            println!("OBSERVED mode={mode} idle-notice-received=true request-refused={} end={:?}", mode != "live", summary.end);
        }).await.expect("bounded diagnostic completes");
    });
}
#[test] fn live_notice_and_request_control() { observe("live"); }
#[test] fn expired_idle_peer_receives_notice_before_request_refusal() { observe("expired"); }
#[test] fn revoked_idle_peer_receives_notice_before_request_refusal() { observe("revoked"); }
