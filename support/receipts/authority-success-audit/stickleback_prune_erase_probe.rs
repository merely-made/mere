//! Compare single-operation and carried-batch effects over the same signed corpus.
use std::{future::Future, pin::pin, task::{Context, Poll, Waker}};
use muniment::MemoryBackend;
use p2panda_core::{Body, Hash, Header, Operation, SigningKey, Topic};
use serde::{Deserialize, Serialize};
use stickleback_audit::{Admission, DropExportProfile, DropLimits, MunimentStore, OperationPolicy,
    OperationProcessor, ProcessOutcome, Reject, StoreTarget, export_topic_operations, import_plain_drop, write_plain_drop};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Ext { prune: bool, erase: Vec<Hash> }
#[derive(Clone)] struct Policy { effects: bool }
impl OperationPolicy<Ext> for Policy {
    type LogId = u64;
    fn admit(&self, operation: &Operation<Ext>) -> Result<Admission<u64>, Reject> {
        let target = StoreTarget::new(Topic::from([7; 32]), 0);
        if self.effects && operation.header.extensions.prune {
            Ok(Admission::prune_before_current(target).erasing_payloads(operation.header.extensions.erase.clone()))
        } else { Ok(Admission::keep(target)) }
    }
}
fn ready<T>(future: impl Future<Output=T>) -> T {
    let mut future = pin!(future);
    match future.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value, Poll::Pending => panic!("MemoryBackend unexpectedly pending"),
    }
}
fn processor(effects: bool) -> OperationProcessor<MemoryBackend, Ext, Policy> {
    OperationProcessor::new(MunimentStore::new(MemoryBackend::new()), Policy { effects })
}
fn operation(key: &SigningKey, seq: u32, back: Option<Hash>, erase: Vec<Hash>) -> Operation<Ext> {
    let body = Body::from_bytes(format!("operation {seq}").as_bytes());
    let header = Header::builder().body(body.as_bytes()).seq_num(seq).backlink(back)
        .build(key, Ext { prune: seq == 2, erase });
    Operation { hash: header.hash(), header, body: Some(body) }
}
fn main() {
    let key = SigningKey::generate();
    let first = operation(&key, 0, None, vec![]);
    let second = operation(&key, 1, Some(first.hash), vec![]);
    let checkpoint = operation(&key, 2, Some(second.hash), vec![first.hash]);
    let direct = processor(true);
    ready(direct.process(&first)).unwrap(); ready(direct.process(&second)).unwrap();
    let outcome = ready(direct.process(&checkpoint)).unwrap();
    assert_eq!(outcome, ProcessOutcome::Inserted { pruned_entries: 2, erased_payloads: 1 });
    assert_eq!(ready(direct.store().operation_count()).unwrap(), 1);
    assert!(ready(direct.store().get_operation(&first.hash)).unwrap().is_none());
    let history = ready(direct.store().get_log_entries(&key.verifying_key(), &0u64, None, None)).unwrap().unwrap();
    assert_eq!(history.len(), 2); assert_eq!(history[0].0.hash, first.hash);
    assert!(history[0].0.body.is_none()); assert_eq!(history[1].0.hash, checkpoint.hash);
    println!("PASS single-operation prune reports two removed entries but payload erasure recreates a pruned header in the log without its operation pointer");
    let source = processor(false);
    for op in [&first, &second, &checkpoint] { ready(source.process(op)).unwrap(); }
    let records = ready(export_topic_operations::<_,_,u64>(source.store(), &Topic::from([7;32]), DropExportProfile::default())).unwrap();
    let mut bytes = Vec::new(); write_plain_drop(&mut bytes, &records, DropLimits::default()).unwrap();
    let batch = processor(true);
    let report = ready(import_plain_drop(bytes.as_slice(), DropLimits::default(), &batch)).unwrap();
    assert_eq!(report.accepted, 3); assert_eq!(ready(batch.store().operation_count()).unwrap(), 1);
    let history = ready(batch.store().get_log_entries(&key.verifying_key(), &0u64, None, None)).unwrap().unwrap();
    assert_eq!(history.len(), 1); assert_eq!(history[0].0.hash, checkpoint.hash);
    println!("PASS native-drop batch control applies the identical signed corpus and policy with only the surviving checkpoint in the log");
}
