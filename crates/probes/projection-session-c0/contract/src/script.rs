//! The call script every binding runs, and its transcript.

use chirograph::{
    CapabilityProfile, EndpointDescriptor, PresentationCapability, ProjectionSnapshot,
    ProtocolVersion, Revision, SceneEpoch, SessionOpen, SessionOpened,
};
use sceno::{Arrangement, InstanceId, Score, Spiral};

use crate::{Api, Failure, IntentCall, Notice, ProjectionAsk, Reply, payload, provider};

/// Session calls the provider allows before revoking; the script's last calls
/// land after it.
pub const CALLS_BEFORE_REVOCATION: usize = 11;

pub type Transcript = Vec<(String, Vec<u8>)>;

fn reply(name: &str, r: &Reply) -> (String, Vec<u8>) {
    let bytes = match r {
        Ok(bytes) => [b"ok:".as_slice(), bytes].concat(),
        Err(f) => format!("err:{f:?}").into_bytes(),
    };
    (name.into(), bytes)
}

fn unit(name: &str, r: &Result<(), Failure>) -> (String, Vec<u8>) {
    reply(name, &r.clone().map(|()| Vec::new()))
}

fn notice(name: &str, r: &Result<Option<Notice>, Failure>) -> (String, Vec<u8>) {
    match r {
        Ok(n) => (name.into(), format!("{n:?}").into_bytes()),
        Err(f) => reply(name, &Err(f.clone())),
    }
}

fn intent(revision: u64, payload_len: usize) -> Reply {
    payload::encode(&IntentCall {
        target: InstanceId(0),
        observed_epoch: SceneEpoch(1),
        observed_revision: Revision(revision),
        intent: "select".into(),
        payload: vec![7; payload_len],
    })
}

/// Drive `api` through every call and both caps; the transcript is what the
/// routes must agree on.
pub fn run<A: Api>(api: &mut A) -> Transcript {
    let mut t = Transcript::new();
    let found = api.discover();
    t.push(reply("discover", &found));
    if let Ok(bytes) = &found {
        let d: Result<EndpointDescriptor, _> = payload::decode(bytes);
        t.push((
            "discover.decoded".into(),
            format!("{}", d.is_ok()).into_bytes(),
        ));
    }

    let refused = payload::encode(&SessionOpen {
        version: ProtocolVersion::V1,
        capabilities: CapabilityProfile::new([]),
    })
    .unwrap();
    match api.open(refused) {
        Ok((s, _)) => {
            t.push(("open.empty".into(), b"opened".to_vec()));
            api.close(s);
        },
        Err(f) => t.push(reply("open.empty", &Err(f))),
    }

    let request = payload::encode(&SessionOpen {
        version: ProtocolVersion::V1,
        capabilities: CapabilityProfile::new([PresentationCapability::PortableCard]),
    })
    .unwrap();
    let (session, opened) = match api.open(request) {
        Ok(pair) => pair,
        Err(f) => {
            t.push(reply("open", &Err(f)));
            return t;
        },
    };
    t.push(reply("open", &Ok(opened.clone())));
    let opened: SessionOpened = payload::decode(&opened).unwrap();
    let id = opened.descriptor.projections[0].request.session.clone();

    let ask = payload::encode(&ProjectionAsk {
        version: ProtocolVersion::V1,
        score: Score::new(Arrangement::Spiral(Spiral::default())),
    })
    .unwrap();
    let snap = api.snapshot(&session, ask.clone());
    t.push(reply("snapshot", &snap));
    // The binding refuses a reply naming another session.
    if let Ok(bytes) = &snap {
        let s: ProjectionSnapshot = payload::decode(bytes).unwrap();
        t.push((
            "snapshot.session".into(),
            format!("{}", s.session == id).into_bytes(),
        ));
    }

    let card = payload::encode(&provider::card_address()).unwrap();
    t.push(reply("fetch", &api.fetch(&session, card.clone())));
    let huge = payload::encode(&provider::huge_address()).unwrap();
    t.push(reply("fetch.huge", &api.fetch(&session, huge)));
    t.push(reply("chunk.0", &api.chunk(&session, card.clone(), 0, 4)));
    t.push(reply("chunk.4", &api.chunk(&session, card.clone(), 4, 0)));

    t.push(reply(
        "intent",
        &api.intent(&session, intent(1, 8).unwrap()),
    ));
    t.push(notice("wait-notice", &api.wait_notice(&session).map(Some)));
    t.push(notice("take-notice", &Ok(api.take_notice(&session))));
    t.push(reply(
        "intent.stale",
        &api.intent(&session, intent(7, 8).unwrap()),
    ));
    t.push(reply("resume", &api.resume(&session, 1, 2)));
    // Over the cap on the consumer side: refused before it is sent.
    t.push(reply("intent.huge", &intent(1, payload::MAX_PAYLOAD)));
    t.push(unit("suspend", &api.suspend(&session)));

    // The authority has lapsed: every method rechecks and discloses nothing.
    t.push(reply("snapshot.revoked", &api.snapshot(&session, ask)));
    t.push(notice(
        "take-notice.revoked",
        &Ok(api.take_notice(&session)),
    ));
    t.push(notice(
        "wait-notice.revoked",
        &api.wait_notice(&session).map(Some),
    ));
    api.close(session);
    t
}

/// Calls straight into a provider, with no binding: the reference transcript.
pub struct Direct<P>(pub P);

impl<P: crate::Provider> Api for Direct<P> {
    type Session = u64;
    fn discover(&mut self) -> Reply {
        self.0.discover()
    }
    fn open(&mut self, request: Vec<u8>) -> Result<(u64, Vec<u8>), Failure> {
        self.0.open(&request)
    }
    fn snapshot(&mut self, s: &u64, ask: Vec<u8>) -> Reply {
        self.0.snapshot(*s, &ask)
    }
    fn fetch(&mut self, s: &u64, address: Vec<u8>) -> Reply {
        self.0.fetch(*s, &address)
    }
    fn chunk(&mut self, s: &u64, address: Vec<u8>, offset: u64, length: u32) -> Reply {
        self.0.chunk(*s, &address, offset, length)
    }
    fn resume(&mut self, s: &u64, epoch: u64, revision: u64) -> Reply {
        self.0.resume(*s, epoch, revision)
    }
    fn intent(&mut self, s: &u64, call: Vec<u8>) -> Reply {
        self.0.intent(*s, &call)
    }
    fn suspend(&mut self, s: &u64) -> Result<(), Failure> {
        self.0.suspend(*s)
    }
    fn take_notice(&mut self, s: &u64) -> Option<Notice> {
        self.0.take_notice(*s)
    }
    fn wait_notice(&mut self, s: &u64) -> Result<Notice, Failure> {
        self.0.wait_notice(*s)
    }
    fn close(&mut self, s: u64) {
        self.0.close(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::TestProvider;

    #[test]
    fn the_reference_transcript_covers_every_outcome() {
        let t = run(&mut Direct(TestProvider::new(CALLS_BEFORE_REVOCATION)));
        let get = |name: &str| {
            let (_, v) = t.iter().find(|(n, _)| n == name).unwrap();
            String::from_utf8_lossy(v).into_owned()
        };
        assert!(get("open.empty").starts_with("err:Refused"));
        assert_eq!(get("snapshot.session"), "true");
        assert!(get("fetch").starts_with("ok:"));
        assert!(get("fetch.huge").starts_with("err:TooLarge"));
        assert_eq!(get("wait-notice"), "Some(Notice { epoch: 1, revision: 2 })");
        assert_eq!(get("take-notice"), "None");
        assert!(get("intent.huge").starts_with("err:TooLarge"));
        assert_eq!(get("suspend"), "ok:");
        assert_eq!(get("snapshot.revoked"), "err:Revoked");
        assert_eq!(get("take-notice.revoked"), "None");
        assert_eq!(get("wait-notice.revoked"), "err:Revoked");
    }
}
