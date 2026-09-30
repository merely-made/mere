// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable diagnostic producers and a bounded, nonblocking staging queue.
//!
//! Hosts explicitly install a configurable bounded sender. Existing producers
//! keep `emit_event`; callers needing admission results use `try_emit_event`.
//! No sink, disabled collection, contention, disconnection and rejection have
//! observable counters. Producers must redact and bound payload construction
//! before emission; already allocated strings are not retroactively bounded.
//! A receiver feeds product-owned collectors. It is not an independent-reader
//! store: Apparatus supplies independent cursors after product projection.

use std::sync::OnceLock;
use std::{marker::PhantomData, rc::Rc};

mod queue;
use crate::diagnostics::DiagnosticCorrelation;
pub use queue::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanPhase {
    Enter,
    Exit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructuredPayloadField {
    pub name: &'static str,
    pub value: String,
}

/// Portable subset of the diagnostic event taxonomy. Rich shell-side
/// variants (CompositorFrame, IntentBatch) live in the shell-side
/// runtime's own DiagnosticEvent enum and don't appear here.
#[derive(Clone, Debug)]
pub enum DiagnosticEvent {
    Span {
        name: &'static str,
        phase: SpanPhase,
        duration_us: Option<u64>,
    },
    MessageSent {
        channel_id: &'static str,
        byte_len: usize,
    },
    MessageSentStructured {
        channel_id: &'static str,
        byte_len: usize,
        fields: Vec<StructuredPayloadField>,
    },
    MessageReceived {
        channel_id: &'static str,
        latency_us: u64,
    },
    MessageReceivedStructured {
        channel_id: &'static str,
        latency_us: u64,
        fields: Vec<StructuredPayloadField>,
    },
    /// A structured log / trace event (a `tracing` event mirrored into diagnostics), as
    /// opposed to channel messaging. `target` is the emitting component and `level` the
    /// tracing level name (`"ERROR"`/`"WARN"`/`"INFO"`/`"DEBUG"`/`"TRACE"`); the host maps
    /// `level` to its own severity and may group by `target`. `message` is the event's human
    /// text, `fields` the structured remainder. Distinct from the `Message*` variants so a
    /// log line is never framed as a message receipt.
    Event {
        target: &'static str,
        level: &'static str,
        message: String,
        fields: Vec<StructuredPayloadField>,
    },
}

static GLOBAL_DIAGNOSTICS_TX: OnceLock<DiagnosticSender> = OnceLock::new();
static UNINSTALLED_LOSS: queue::Counters = queue::Counters::new();

thread_local! {
    /// Scoped thread-local injection for isolated callers and tests. The
    /// global sender is used only when the current thread has no override.
    static THREAD_LOCAL_DIAGNOSTICS_TX: std::cell::RefCell<Option<DiagnosticSender>> =
        const { std::cell::RefCell::new(None) };
}

/// Install once at host startup. A second installation returns an error,
/// never pretending to replace the already installed sink. Tests use the
/// explicitly scoped thread-local injection instead.
pub fn install_global_sender(sender: DiagnosticSender) -> Result<(), SenderAlreadyInstalled> {
    GLOBAL_DIAGNOSTICS_TX
        .set(sender)
        .map_err(|_| SenderAlreadyInstalled)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SenderAlreadyInstalled;

/// Thread-local scope, restored on drop. The guard cannot move to another
/// thread. Spawned workers require explicit sender injection or the global sink.
pub struct ThreadSenderGuard {
    previous: Option<DiagnosticSender>,
    _thread_bound: PhantomData<Rc<()>>,
}
pub fn scoped_thread_sender(sender: DiagnosticSender) -> ThreadSenderGuard {
    let previous = THREAD_LOCAL_DIAGNOSTICS_TX.with(|slot| slot.replace(Some(sender)));
    ThreadSenderGuard {
        previous,
        _thread_bound: PhantomData,
    }
}
impl Drop for ThreadSenderGuard {
    fn drop(&mut self) {
        THREAD_LOCAL_DIAGNOSTICS_TX.with(|slot| {
            slot.replace(self.previous.take());
        });
    }
}

pub fn uninstalled_loss() -> EmissionLoss {
    UNINSTALLED_LOSS.snapshot()
}

/// Compatibility producer entry point. Admission outcomes remain observable
/// through the sender's loss counters even when the producer ignores them.
pub fn emit_event(event: DiagnosticEvent) {
    let _ = try_emit_event(event);
}

pub fn try_emit_event(event: DiagnosticEvent) -> EmitOutcome {
    route(event, None)
}

pub fn try_emit_correlated(event: DiagnosticEvent, key: DiagnosticCorrelation) -> EmitOutcome {
    route(event, Some(key))
}

fn route(event: DiagnosticEvent, key: Option<DiagnosticCorrelation>) -> EmitOutcome {
    let sender = THREAD_LOCAL_DIAGNOSTICS_TX
        .with(|slot| slot.borrow().clone())
        .or_else(|| GLOBAL_DIAGNOSTICS_TX.get().cloned());
    match sender {
        Some(sender) => match key {
            Some(key) => sender.try_send_correlated(event, key),
            None => sender.try_send(event),
        },
        None => {
            UNINSTALLED_LOSS.reject(EmissionRejection::NoSink);
            EmitOutcome::Rejected {
                sequence: None,
                reason: EmissionRejection::NoSink,
            }
        },
    }
}

/// Convenience: emit a span exit with measured duration. Mirrors
/// the shell-side `emit_span_duration` helper that's the most
/// common emit pattern in instrumentation code.
pub fn emit_span_duration(name: &'static str, duration_us: u64) {
    emit_event(DiagnosticEvent::Span {
        name,
        phase: SpanPhase::Exit,
        duration_us: Some(duration_us),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_with_no_sender_is_silent() {
        // No global sender installed — must not panic, must not block.
        emit_event(DiagnosticEvent::MessageSent {
            channel_id: "test.no_sender",
            byte_len: 42,
        });
    }

    #[test]
    fn emit_routes_to_thread_local_sender() {
        let (tx, rx) = diagnostic_channel(IngressLimits::default());
        let _scope = scoped_thread_sender(tx);

        emit_event(DiagnosticEvent::MessageSent {
            channel_id: "test.routed",
            byte_len: 7,
        });

        let received = rx.try_drain(1).unwrap().packets.remove(0).event;
        match received {
            DiagnosticEvent::MessageSent {
                channel_id,
                byte_len,
            } => {
                assert_eq!(channel_id, "test.routed");
                assert_eq!(byte_len, 7);
            },
            _ => panic!("wrong variant"),
        }

        // Cleanup.
        THREAD_LOCAL_DIAGNOSTICS_TX.with(|slot| {
            *slot.borrow_mut() = None;
        });
    }

    #[test]
    fn emit_span_duration_helper_produces_exit_span() {
        let (tx, rx) = diagnostic_channel(IngressLimits::default());
        let _scope = scoped_thread_sender(tx);

        emit_span_duration("test.span", 1234);

        let received = rx.try_drain(1).unwrap().packets.remove(0).event;
        match received {
            DiagnosticEvent::Span {
                name,
                phase,
                duration_us,
            } => {
                assert_eq!(name, "test.span");
                assert_eq!(phase, SpanPhase::Exit);
                assert_eq!(duration_us, Some(1234));
            },
            _ => panic!("wrong variant"),
        }

        THREAD_LOCAL_DIAGNOSTICS_TX.with(|slot| {
            *slot.borrow_mut() = None;
        });
    }
}
