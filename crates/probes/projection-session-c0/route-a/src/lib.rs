//! Route A (D12): bindings from our own generator, a blocking transport over
//! OS pipes, and adapters between the generated types and the shared
//! contract.

pub mod rt;

#[allow(clippy::all, dead_code)]
pub mod bindings {
    use super::rt::*;
    include!(concat!(env!("OUT_DIR"), "/projection_session.rs"));
}

use std::io::{self, PipeReader, PipeWriter, Read, Write};
use std::thread::JoinHandle;

use bindings::{Client, Handler, Notice as WireNotice, Session};
use c0_contract::{Api, Failure, Notice, Provider, Reply, payload};
use rt::Transport;

/// The largest frame either side reads: one capped payload plus its call.
const MAX_FRAME: usize = payload::MAX_PAYLOAD + 4096;

fn to_wire(f: Failure) -> bindings::Failure {
    match f {
        Failure::Revoked => bindings::Failure::Revoked,
        Failure::Expired => bindings::Failure::Expired,
        Failure::TooLarge(n) => bindings::Failure::TooLarge(n),
        Failure::Refused(s) => bindings::Failure::Refused(s),
        Failure::Malformed(s) => bindings::Failure::Malformed(s),
    }
}

fn from_wire(f: bindings::Failure) -> Failure {
    match f {
        bindings::Failure::Revoked => Failure::Revoked,
        bindings::Failure::Expired => Failure::Expired,
        bindings::Failure::TooLarge(n) => Failure::TooLarge(n),
        bindings::Failure::Refused(s) => Failure::Refused(s),
        bindings::Failure::Malformed(s) => Failure::Malformed(s),
    }
}

fn handle(s: &Session) -> u64 {
    s.0.as_slice()
        .try_into()
        .map(u64::from_le_bytes)
        .unwrap_or(u64::MAX)
}

/// A provider served through the generated `Handler`.
pub struct Serve<P>(pub P);

impl<P: Provider> Handler for Serve<P> {
    fn discover(&mut self) -> Result<Vec<u8>, bindings::Failure> {
        self.0.discover().map_err(to_wire)
    }
    fn open(&mut self, request: Vec<u8>) -> Result<(Session, Vec<u8>), bindings::Failure> {
        let (h, opened) = self.0.open(&request).map_err(to_wire)?;
        Ok((Session(h.to_le_bytes().to_vec()), opened))
    }
    fn session_snapshot(
        &mut self,
        this: &Session,
        ask: Vec<u8>,
    ) -> Result<Vec<u8>, bindings::Failure> {
        self.0.snapshot(handle(this), &ask).map_err(to_wire)
    }
    fn session_fetch(
        &mut self,
        this: &Session,
        address: Vec<u8>,
    ) -> Result<Vec<u8>, bindings::Failure> {
        self.0.fetch(handle(this), &address).map_err(to_wire)
    }
    fn session_chunk(
        &mut self,
        this: &Session,
        address: Vec<u8>,
        offset: u64,
        length: u32,
    ) -> Result<Vec<u8>, bindings::Failure> {
        self.0
            .chunk(handle(this), &address, offset, length)
            .map_err(to_wire)
    }
    fn session_resume(
        &mut self,
        this: &Session,
        epoch: u64,
        revision: u64,
    ) -> Result<Vec<u8>, bindings::Failure> {
        self.0
            .resume(handle(this), epoch, revision)
            .map_err(to_wire)
    }
    fn session_intent(
        &mut self,
        this: &Session,
        call: Vec<u8>,
    ) -> Result<Vec<u8>, bindings::Failure> {
        self.0.intent(handle(this), &call).map_err(to_wire)
    }
    fn session_suspend(&mut self, this: &Session) -> Result<(), bindings::Failure> {
        self.0.suspend(handle(this)).map_err(to_wire)
    }
    fn session_take_notice(&mut self, this: &Session) -> Option<WireNotice> {
        self.0.take_notice(handle(this)).map(|n| WireNotice {
            epoch: n.epoch,
            revision: n.revision,
        })
    }
    fn session_wait_notice(&mut self, this: &Session) -> Result<WireNotice, bindings::Failure> {
        self.0
            .wait_notice(handle(this))
            .map(|n| WireNotice {
                epoch: n.epoch,
                revision: n.revision,
            })
            .map_err(to_wire)
    }
    fn drop_session(&mut self, this: Session) {
        self.0.close(handle(&this))
    }
}

fn write_frame(w: &mut impl Write, frame: &[u8]) -> io::Result<()> {
    w.write_all(&(frame.len() as u32).to_le_bytes())?;
    w.write_all(frame)?;
    w.flush()
}

/// `None` at a clean end of stream.
fn read_frame(r: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut len = [0; 4];
    match r.read_exact(&mut len) {
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        other => other?,
    }
    let len = u32::from_le_bytes(len) as usize;
    if len > MAX_FRAME {
        return Err(rt::bad(format!("frame of {len} bytes")));
    }
    let mut frame = vec![0; len];
    r.read_exact(&mut frame)?;
    Ok(Some(frame))
}

/// The consumer's end: two OS pipes to a serving thread.
pub struct Pipe {
    tx: PipeWriter,
    rx: PipeReader,
}

impl Transport for Pipe {
    fn call(&mut self, frame: Vec<u8>) -> io::Result<Vec<u8>> {
        write_frame(&mut self.tx, &frame)?;
        read_frame(&mut self.rx)?.ok_or_else(|| rt::bad("provider hung up".into()))
    }
}

/// Serve `handler` on a thread; it returns the handler when the consumer hangs up.
pub fn spawn<H: Handler + Send + 'static>(
    mut handler: H,
) -> io::Result<(Pipe, JoinHandle<io::Result<H>>)> {
    let (mut server_rx, client_tx) = io::pipe()?;
    let (client_rx, mut server_tx) = io::pipe()?;
    let thread = std::thread::spawn(move || {
        while let Some(frame) = read_frame(&mut server_rx)? {
            let reply = bindings::dispatch(&mut handler, &frame)?;
            write_frame(&mut server_tx, &reply)?;
        }
        Ok(handler)
    });
    Ok((
        Pipe {
            tx: client_tx,
            rx: client_rx,
        },
        thread,
    ))
}

fn carrier(e: io::Error) -> Failure {
    Failure::Refused(format!("carrier: {e}"))
}

/// The consumer's view, as the shared script drives it.
pub struct Consumer<T: Transport>(pub Client<T>);

impl<T: Transport> Api for Consumer<T> {
    type Session = Session;
    fn discover(&mut self) -> Reply {
        self.0.discover().map_err(carrier)?.map_err(from_wire)
    }
    fn open(&mut self, request: Vec<u8>) -> Result<(Session, Vec<u8>), Failure> {
        self.0.open(request).map_err(carrier)?.map_err(from_wire)
    }
    fn snapshot(&mut self, s: &Session, ask: Vec<u8>) -> Reply {
        self.0
            .session_snapshot(s, ask)
            .map_err(carrier)?
            .map_err(from_wire)
    }
    fn fetch(&mut self, s: &Session, address: Vec<u8>) -> Reply {
        self.0
            .session_fetch(s, address)
            .map_err(carrier)?
            .map_err(from_wire)
    }
    fn chunk(&mut self, s: &Session, address: Vec<u8>, offset: u64, length: u32) -> Reply {
        self.0
            .session_chunk(s, address, offset, length)
            .map_err(carrier)?
            .map_err(from_wire)
    }
    fn resume(&mut self, s: &Session, epoch: u64, revision: u64) -> Reply {
        self.0
            .session_resume(s, epoch, revision)
            .map_err(carrier)?
            .map_err(from_wire)
    }
    fn intent(&mut self, s: &Session, call: Vec<u8>) -> Reply {
        self.0
            .session_intent(s, call)
            .map_err(carrier)?
            .map_err(from_wire)
    }
    fn suspend(&mut self, s: &Session) -> Result<(), Failure> {
        self.0
            .session_suspend(s)
            .map_err(carrier)?
            .map_err(from_wire)
    }
    fn take_notice(&mut self, s: &Session) -> Option<Notice> {
        let n = self.0.session_take_notice(s).ok()??;
        Some(Notice {
            epoch: n.epoch,
            revision: n.revision,
        })
    }
    fn wait_notice(&mut self, s: &Session) -> Result<Notice, Failure> {
        let n = self
            .0
            .session_wait_notice(s)
            .map_err(carrier)?
            .map_err(from_wire)?;
        Ok(Notice {
            epoch: n.epoch,
            revision: n.revision,
        })
    }
    fn close(&mut self, s: Session) {
        let _ = self.0.drop_session(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use c0_contract::provider::TestProvider;
    use c0_contract::script::{CALLS_BEFORE_REVOCATION, Direct, run};

    #[test]
    fn route_a_matches_the_reference_transcript() {
        let reference = run(&mut Direct(TestProvider::new(CALLS_BEFORE_REVOCATION)));
        let (pipe, server) = spawn(Serve(TestProvider::new(CALLS_BEFORE_REVOCATION))).unwrap();
        let mut consumer = Consumer(Client(pipe));
        let transcript = run(&mut consumer);
        drop(consumer);
        let served = server.join().unwrap().unwrap();
        assert_eq!(transcript, reference);
        // Close is the resource's drop: the provider saw it.
        assert_eq!(served.0.closed, vec![1]);
    }
}
