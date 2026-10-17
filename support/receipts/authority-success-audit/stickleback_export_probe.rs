//! Fresh complete owner, real MemoryBackend/processor, audit-owned file fixtures.
use std::{future::Future, pin::pin, task::{Context, Poll, Waker}, path::Path};
use muniment::MemoryBackend;
use p2panda_core::{Body, Header, Operation, SigningKey, Topic};
use serde::{Serialize, Deserialize};
use stickleback_audit::{Admission, OperationPolicy, OperationProcessor, MunimentStore, Reject, StoreTarget};
use stickleback_audit::drop::{DropLimits, read_plain_drop};
use stickleback_audit::drop_io::{DropExportProfile, DropExportBudget, export_plain_topic_file, export_selected_plain_topic_file};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Ext { space: u8 }
struct Policy;
impl OperationPolicy<Ext> for Policy {
    type LogId = u64;
    fn admit(&self, _: &Operation<Ext>) -> Result<Admission<u64>, Reject> {
        Ok(Admission::keep(StoreTarget::new(Topic::from([7; 32]), 0)))
    }
}
fn ready<T>(future: impl Future<Output=T>) -> T {
    let mut future = pin!(future);
    match future.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("in-memory fixture unexpectedly needs an executor"),
    }
}
fn main() {
    let directory = std::env::args().nth(1).unwrap();
    let base = Path::new(&directory);
    let processor = OperationProcessor::new(MunimentStore::new(MemoryBackend::new()), Policy);
    let body = Body::from_bytes(b"retained signed export body");
    let header = Header::builder().body(body.as_bytes()).seq_num(0).backlink(None)
        .build(&SigningKey::generate(), Ext { space: 7 });
    let operation = Operation { hash: header.hash(), header, body: Some(body) };
    ready(processor.process(&operation)).unwrap();
    let topic = Topic::from([7; 32]);
    let limits = DropLimits { max_protected_bytes: 0, ..DropLimits::default() };
    for (name, selected) in [("drop-export-refused-plain.meredrop", false), ("drop-export-refused-selected.meredrop", true)] {
        let path = base.join(name); assert!(!path.exists());
        std::fs::write(&path, b"PREVIOUS AUDIT-OWNED EXPORT SENTINEL").unwrap();
        let refused = if selected {
            ready(export_selected_plain_topic_file::<_, _, u64, _>(&path, processor.store(), &topic,
                &DropExportProfile::default(), DropExportBudget::default(), limits)).is_err()
        } else {
            ready(export_plain_topic_file::<_, _, u64>(&path, processor.store(), &topic, DropExportProfile::default(), limits)).is_err()
        };
        assert!(refused);
        assert!(std::fs::read(&path).unwrap().is_empty());
    }
    println!("PASS both public plaintext export variants truncate an existing destination before returning the configured-limit refusal");
    let path = base.join("drop-export-success-control.meredrop"); assert!(!path.exists());
    std::fs::write(&path, b"PREVIOUS AUDIT-OWNED CONTROL SENTINEL").unwrap();
    let written = ready(export_plain_topic_file::<_, _, u64>(&path, processor.store(), &topic,
        DropExportProfile::default(), DropLimits::default())).unwrap();
    let (report, records) = read_plain_drop(std::fs::File::open(path).unwrap(), DropLimits::default()).unwrap();
    assert_eq!(written.id, report.id); assert_eq!(records.len(), 1);
    assert_eq!(ready(processor.store().operation_count()).unwrap(), 1);
    println!("PASS successful export control round-trips a real signed stored operation without changing the source store");
}
