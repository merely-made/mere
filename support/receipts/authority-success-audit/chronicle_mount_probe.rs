// Audit-only executable: compile the current original module by path.
#[path = "../../../..//ports/graphshell/src/distillery_w1.rs"]
mod current;

use chirograph::{ContentHash, EndpointDescriptor, PortableCardV1, ProjectionRequest,
    ProjectionSnapshot, ResourceRequest, ResourceResponse, ResumeReply, ResumeRequest};
use graphshell_client::ResumeApplication;
use graphshell_endpoint::{ProjectionCatalog, ProjectionSource, PresentationSource, ResumableProjectionSource};
use graphshell_host::live_endpoint::LiveEndpoint;

#[derive(Clone, Copy)]
enum Fault { None, WrongResumeSession, Gap, BadSnapshot, OtherResource }
struct Endpoint { live: LiveEndpoint, fault: Fault }
impl Endpoint { fn new() -> Self { Self { live: LiveEndpoint::new(), fault: Fault::None } } }
impl ProjectionCatalog for Endpoint {
    fn describe(&self) -> EndpointDescriptor { self.live.describe() }
}
impl ProjectionSource for Endpoint {
    type Error = String;
    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        let mut reply = self.live.snapshot(request).map_err(|e| e.to_string())?;
        if matches!(self.fault, Fault::BadSnapshot) {
            reply.scene.revision.0 = 999;
            reply.scene.tables.items.iter_mut().flatten().next().unwrap().source.0 = u32::MAX;
        }
        Ok(reply)
    }
}
impl PresentationSource for Endpoint {
    type Error = String;
    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, String> {
        if matches!(self.fault, Fault::OtherResource) {
            let card = PortableCardV1 { title: "Unrequested card".into(),
                values: vec![chirograph::CardValueV1 { label: "Unrequested".into(), value: "substituted resource".into() }],
                badges: vec![], media: vec![] };
            let bytes = serde_json::to_vec(&card).unwrap();
            let resource = ContentHash::of(&bytes);
            assert_ne!(resource, request.resource);
            let reply = ResourceResponse { session: request.session, resource, bytes };
            assert!(reply.has_valid_address());
            return Ok(reply);
        }
        self.live.resource(request).map_err(|e| e.to_string())
    }
}
impl ResumableProjectionSource for Endpoint {
    type Error = String;
    fn resume(&mut self, request: ResumeRequest) -> Result<ResumeReply, String> {
        let mut reply = self.live.resume(request).map_err(|e| e.to_string())?;
        if let ResumeReply::Diffs(diffs) = &mut reply {
            assert!(!diffs.is_empty());
            match self.fault {
                Fault::WrongResumeSession => diffs[0].session.0 = "another-session".into(),
                Fault::Gap => diffs[0].scene.base.0 = 999,
                _ => {},
            }
        }
        Ok(reply)
    }
}
fn mounted() -> current::ChronicleMount<Endpoint> {
    let mut mount = current::ChronicleMount::discover(Endpoint::new()).unwrap();
    mount.mount().unwrap();
    mount
}
fn main() {
    let mut normal = mounted();
    normal.endpoint_mut().live.append();
    assert!(matches!(normal.resume().unwrap(), ResumeApplication::Applied(_)));
    let receipt = normal.receipt("control").unwrap();
    assert!(receipt.resumed_by_diff);
    assert_eq!(receipt.resumed_revision, 2);
    println!("PASS normal applied diff advances acknowledged revision");

    let mut wrong = mounted();
    wrong.endpoint_mut().live.append();
    wrong.endpoint_mut().fault = Fault::WrongResumeSession;
    assert!(wrong.resume().is_err());
    let receipt = wrong.receipt("failed resume").unwrap();
    assert_eq!(receipt.resumed_revision, 1);
    assert!(receipt.resumed_by_diff);
    println!("PASS failed resume still advertises resumed_by_diff=true at old revision");

    let mut gap = mounted();
    gap.endpoint_mut().live.append();
    gap.endpoint_mut().fault = Fault::Gap;
    assert!(matches!(gap.resume().unwrap(), ResumeApplication::Resynchronize(_)));
    let receipt = gap.receipt("gap").unwrap();
    assert_eq!(receipt.resumed_revision, 1);
    assert!(receipt.resumed_by_diff);
    println!("PASS unapplied gap requests resynchronization but advertises resumed_by_diff=true");

    let mut remount = mounted();
    remount.endpoint_mut().fault = Fault::BadSnapshot;
    assert!(remount.mount().is_err());
    let receipt = remount.receipt("failed remount").unwrap();
    assert_eq!(receipt.snapshot_revision, 999);
    assert_eq!(receipt.resumed_revision, 1);
    println!("PASS refused remount stamps snapshot revision 999 over acknowledged revision 1");

    let mut substituted = current::ChronicleMount::discover(Endpoint {
        live: LiveEndpoint::new(), fault: Fault::OtherResource }).unwrap();
    substituted.mount().unwrap();
    let html = substituted.frozen_html("substitution").unwrap();
    assert!(html.contains("substituted resource"));
    println!("PASS self-validating unrequested resource enters frozen card details");
    println!("5 checks passed; 1 positive control and 4 counterexamples");
}
