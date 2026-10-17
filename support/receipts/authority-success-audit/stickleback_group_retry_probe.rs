//! Public group lane/session persistence retry diagnostic. Secret snapshots stay in memory.
use std::{future::Future, pin::pin, task::{Context, Poll, Waker}};
use identity::InMemoryProvider;
use muniment::MemoryBackend;
use stickleback_audit::{GroupSession, GroupSessionId, GroupKeyLane};
fn ready<T>(future: impl Future<Output=T>) -> T {
    let mut future = pin!(future);
    match future.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("in-memory fixture unexpectedly needs an executor"),
    }
}
fn case(mode: u8) {
    let group = GroupSessionId([0x74; 32]);
    let ai = InMemoryProvider::from_seed([0xa1; 32]);
    let bi = InMemoryProvider::from_seed([0xb2; 32]);
    let (mut a, ap) = GroupSession::new(group, &ai).unwrap();
    let (mut b, bp) = GroupSession::new(group, &bi).unwrap();
    a.register_prekey(&bp).unwrap(); b.register_prekey(&ap).unwrap();
    a.create(&[]).unwrap();
    let welcome = a.add(b.member()).unwrap();
    b.process(a.personae_root(), &welcome.control, welcome.direct_for(b.member())).unwrap();
    let la = GroupKeyLane::for_identity(MemoryBackend::new(), group, &ai).unwrap();
    let lb = GroupKeyLane::for_identity(MemoryBackend::new(), group, &bi).unwrap();
    let add = ready(la.author_dispatch(&welcome)).unwrap();
    ready(lb.accept(&add)).unwrap();
    let welcome_drain = ready(lb.drain_ready(&mut b, |_| Ok::<(), String>(()))).unwrap();
    assert_eq!(welcome_drain.superseded.len(), 1);
    let mut persisted = b.to_bytes().unwrap();
    let old_epoch = b.current_epoch();
    let update = a.update().unwrap();
    let operation = ready(la.author_dispatch(&update)).unwrap();
    ready(lb.accept(&operation)).unwrap();
    let ciphertext = a.seal_random(b"new epoch survives only if saved").unwrap();
    let mut calls = 0;
    if mode != 0 {
        let failed = ready(lb.drain_ready(&mut b, |_| Err::<(), _>("injected save refusal")));
        assert!(failed.is_err());
        assert_eq!(b.current_epoch(), a.current_epoch());
        assert_eq!(b.open(&ciphertext).unwrap(), b"new epoch survives only if saved");
        if mode == 2 { b = GroupSession::from_bytes(&persisted).unwrap(); }
    }
    let drain = ready(lb.drain_ready(&mut b, |session| {
        calls += 1; persisted = session.to_bytes().unwrap(); Ok::<(), String>(())
    })).unwrap();
    let mut reopened = GroupSession::from_bytes(&persisted).unwrap();
    let replay = ready(lb.drain_ready(&mut reopened, |_| Ok::<(), String>(()))).unwrap();
    assert!(replay.applied.is_empty() && replay.pending.is_empty() && replay.superseded.is_empty() && replay.refused.is_empty());
    if mode == 1 {
        assert_eq!(calls, 0); assert!(drain.applied.is_empty()); assert_eq!(drain.superseded.len(), 1);
        assert_eq!(reopened.current_epoch(), old_epoch); assert_ne!(reopened.current_epoch(), a.current_epoch());
        assert!(reopened.open(&ciphertext).is_err());
        println!("PASS failed save then same-session retry settles the update without invoking save; reopen cannot recover or decrypt the new epoch");
    } else {
        assert_eq!(calls, 1); assert_eq!(drain.applied.len(), 1);
        assert_eq!(reopened.current_epoch(), a.current_epoch());
        assert_eq!(reopened.open(&ciphertext).unwrap(), b"new epoch survives only if saved");
        println!("PASS control mode {mode} persists and reopens the updated epoch with successful decryption");
    }
}
fn main() { case(0); case(1); case(2); }
