//! Current public SessionDriver and SessionCore; no I/O or fake protocol owner.
use chirograph::{CapabilityProfile, CarrierOutput, CarrierRequest, CarrierResponse, CarrierResponseBody, EndpointDescriptor};
use graphshell_client::{Advance, Outcome, SessionDriver};

fn closing() -> (SessionDriver, u64) {
    let mut driver = SessionDriver::from_descriptor(EndpointDescriptor { label: "pending probe".into(), projections: vec![] }, CapabilityProfile::default());
    let progress = driver.core_mut().unwrap().close();
    let Advance::Send(line) = driver.begin(progress).unwrap() else { panic!("close sends") };
    let request: CarrierRequest = serde_json::from_str(&line).unwrap();
    (driver, request.id)
}
fn close_reply(id: u64) -> String {
    serde_json::to_string(&CarrierOutput::Response(CarrierResponse { id, body: Ok(CarrierResponseBody::Closed) })).unwrap()
}
#[test]
fn close_control_completes() {
    let (mut driver, id) = closing();
    assert!(matches!(driver.on_line(&close_reply(id)).unwrap(), Advance::Done(Outcome::Closed)));
    println!("CONTROL original-close-reply accepted=true");
}
#[test]
fn refused_rediscovery_changes_the_original_pending_operation() {
    let (mut driver, id) = closing();
    let refusal = driver.discover().expect_err("second request must be refused");
    assert!(refusal.contains("still waiting on request"));
    assert!(driver.is_awaiting());
    let error = driver.on_line(&close_reply(id)).expect_err("the core now wrongly expects a descriptor");
    assert!(error.contains("expected descriptor"), "{error}");
    assert!(!driver.is_awaiting());
    println!("OBSERVED second-send-refused=true original-close-reply-rejected=true error={error}");
}
