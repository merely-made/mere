//! The C0 test guest: runs the shared call script against the imported
//! projection session and echoes values for the encoding test.

wit_bindgen::generate!({
    path: "../wit",
    world: "projection-consumer",
});

use c0_contract::{Api, Failure as NativeFailure, Notice as NativeNotice, Reply, script};
use mere::script::projection_session as ps;

fn from_wire(f: ps::Failure) -> NativeFailure {
    match f {
        ps::Failure::Revoked => NativeFailure::Revoked,
        ps::Failure::Expired => NativeFailure::Expired,
        ps::Failure::TooLarge(n) => NativeFailure::TooLarge(n),
        ps::Failure::Refused(s) => NativeFailure::Refused(s),
        ps::Failure::Malformed(s) => NativeFailure::Malformed(s),
    }
}

struct Imports;

impl Api for Imports {
    type Session = ps::Session;
    fn discover(&mut self) -> Reply {
        ps::discover().map_err(from_wire)
    }
    fn open(&mut self, request: Vec<u8>) -> Result<(ps::Session, Vec<u8>), NativeFailure> {
        ps::open(&request).map_err(from_wire)
    }
    fn snapshot(&mut self, s: &ps::Session, ask: Vec<u8>) -> Reply {
        s.snapshot(&ask).map_err(from_wire)
    }
    fn fetch(&mut self, s: &ps::Session, address: Vec<u8>) -> Reply {
        s.fetch(&address).map_err(from_wire)
    }
    fn chunk(&mut self, s: &ps::Session, address: Vec<u8>, offset: u64, length: u32) -> Reply {
        s.chunk(&address, offset, length).map_err(from_wire)
    }
    fn resume(&mut self, s: &ps::Session, epoch: u64, revision: u64) -> Reply {
        s.resume(epoch, revision).map_err(from_wire)
    }
    fn intent(&mut self, s: &ps::Session, call: Vec<u8>) -> Reply {
        s.intent(&call).map_err(from_wire)
    }
    fn suspend(&mut self, s: &ps::Session) -> Result<(), NativeFailure> {
        s.suspend().map_err(from_wire)
    }
    fn take_notice(&mut self, s: &ps::Session) -> Option<NativeNotice> {
        s.take_notice().map(|n| NativeNotice {
            epoch: n.epoch,
            revision: n.revision,
        })
    }
    fn wait_notice(&mut self, s: &ps::Session) -> Result<NativeNotice, NativeFailure> {
        s.wait_notice()
            .map(|n| NativeNotice {
                epoch: n.epoch,
                revision: n.revision,
            })
            .map_err(from_wire)
    }
    fn close(&mut self, s: ps::Session) {
        drop(s)
    }
}

struct Component;

impl Guest for Component {
    fn run() -> Vec<(String, Vec<u8>)> {
        script::run(&mut Imports)
    }
    fn echo_failure(value: ps::Failure) -> ps::Failure {
        value
    }
    fn echo_notice(value: Option<ps::Notice>) -> Option<ps::Notice> {
        value
    }
}

export!(Component);
