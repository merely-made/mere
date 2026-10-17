//! Public Comms owner routing with the supplied InMemoryAdapter boundary double.
use std::{future::Future, pin::pin, task::{Context, Poll, Waker}};
use comms_audit::{AdapterError, Comms, Conversation, ConversationId, Draft, Identity, MessageId, ProtocolKind};
use comms_audit::in_memory::{InMemoryAdapter, sample_message};

// Every adapter future in this diagnostic is in-memory and immediately ready.
// This driver neither schedules a runtime nor pretends to execute network I/O.
fn ready<T>(future: impl Future<Output = T>) -> T {
    let mut future = pin!(future);
    match future.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("fixture unexpectedly needs an executor"),
    }
}

fn adapter(owner: &str, key: &str) -> InMemoryAdapter {
    let protocol = ProtocolKind::Misfin;
    InMemoryAdapter::new(protocol, Identity::new(protocol, owner)).with_conversation(
        Conversation { id: ConversationId::new(protocol, key), title: owner.into(), participants: vec![], last_activity_ms: Some(1), unread: 0 },
        vec![sample_message("m1", Identity::new(protocol, owner), owner, 1)],
    )
}

fn main() {
    let second = ConversationId::new(ProtocolKind::Misfin, "second-thread");
    let model = Comms::new().with_adapter(Box::new(adapter("first-owner", "first-thread")))
        .with_adapter(Box::new(adapter("second-owner", "second-thread")));
    assert_eq!(ready(model.inbox()).conversations.len(), 2);
    assert_eq!(ready(model.messages(&second)), Err(AdapterError::NotFound));
    assert_eq!(ready(model.send(&Draft::reply_to(second.clone()))), Err(AdapterError::NotFound));
    let reversed = Comms::new().with_adapter(Box::new(adapter("second-owner", "second-thread")))
        .with_adapter(Box::new(adapter("first-owner", "first-thread")));
    assert_eq!(ready(reversed.messages(&second)).unwrap()[0].body.text(), "second-owner");
    println!("PASS inbox lists second same-protocol adapter but read/send return NotFound; reversing registration makes it reachable");

    let model = Comms::new().with_adapter(Box::new(adapter("first-owner", "shared")))
        .with_adapter(Box::new(adapter("second-owner", "shared")));
    let inbox = ready(model.inbox());
    assert_eq!(inbox.conversations.len(), 2);
    let second_row = inbox.conversations.iter().find(|c| c.title == "second-owner").unwrap();
    assert_eq!(ready(model.messages(&second_row.id)).unwrap()[0].body.text(), "first-owner");
    assert_eq!(ready(model.send(&Draft::reply_to(second_row.id.clone()))).unwrap(), MessageId("shared:first-owner".into()));
    println!("PASS selecting second owner's colliding conversation ID reads and fake-sends through the first owner");
}
