// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The projection session over the WIT calls, end to end: the call script
//! through `WitCarrier` to `WitServer` over OS pipes and over ipc-channel,
//! once with the provider's end passed over a channel first, as N1 hands it
//! over (D19). Every transcript is the same, and a channel's end closes the
//! sessions it opened.
#![cfg(all(feature = "wit-testkit", any(unix, windows)))]

use chirograph::{
    CapabilityProfile, Carrier, CarrierRequestBody, PresentationCapability, ProtocolVersion,
    SessionOpen,
};
use graphshell_endpoint::wit::testkit::{
    CALLS_BEFORE_REVOCATION, CountdownGate, FixtureEndpoint, Transcript, assert_outcomes, run,
};
use graphshell_endpoint::wit::{Calls, MAX_FRAME, WitCarrier, WitServer, serve};
use script_wire::{Serve, Transport};

type Server = WitServer<FixtureEndpoint, CountdownGate>;

fn server() -> Server {
    WitServer::new(
        FixtureEndpoint::default(),
        CountdownGate::new(CALLS_BEFORE_REVOCATION),
    )
}

/// Run the script over one channel; return its transcript and the server.
fn over(client: impl Transport, end: impl Serve + Send + 'static) -> (Transcript, Server) {
    let provider = std::thread::spawn(move || {
        let mut s = server();
        serve(&mut s, end).expect("the channel ends cleanly");
        s
    });
    let mut carrier = WitCarrier::new(Calls::new(client));
    let transcript = run(&mut carrier);
    drop(carrier);
    (transcript, provider.join().unwrap())
}

fn pipes() -> (Transcript, Server) {
    let (client, end) = script_wire::pipe::pair(MAX_FRAME).unwrap();
    over(client, end)
}

#[test]
fn the_script_runs_over_pipes() {
    let (transcript, served) = pipes();
    assert_outcomes(&transcript);
    // Close dropped the session: the gate heard it once.
    assert_eq!(served.gate.closed.len(), 1);
    assert_eq!(served.open_sessions(), 0);
}

#[cfg(feature = "wit-ipc")]
#[test]
fn ipc_and_a_passed_end_match_pipes() {
    use ipc_channel::ipc::channel;
    use script_wire::ipc::{Server as End, pair};

    let (reference, _) = pipes();
    let (client, end) = pair(MAX_FRAME).unwrap();
    assert_eq!(over(client, end).0, reference);

    let (client, end) = pair(MAX_FRAME).unwrap();
    let (tx, rx) = channel::<End>().unwrap();
    tx.send(end).unwrap();
    assert_eq!(over(client, rx.recv().unwrap()).0, reference);
}

/// D19: a consumer that goes away without closing leaves no session open.
#[test]
fn a_channel_end_closes_its_sessions() {
    let (client, end) = script_wire::pipe::pair(MAX_FRAME).unwrap();
    let provider = std::thread::spawn(move || {
        let mut s = server();
        serve(&mut s, end).unwrap();
        s
    });
    let mut carrier = WitCarrier::new(Calls::new(client));
    for _ in 0..2 {
        let open = CarrierRequestBody::Open(Box::new(SessionOpen {
            version: ProtocolVersion::V1,
            capabilities: CapabilityProfile::new([PresentationCapability::PortableCard]),
        }));
        // A carrier holds one session; the second Open is refused locally.
        let _ = carrier.request(open);
    }
    drop(carrier);
    let served = provider.join().unwrap();
    assert_eq!(served.open_sessions(), 0);
    assert_eq!(served.gate.closed.len(), 1);
}

/// D19: a call in flight when the provider goes reaches the consumer as a
/// disconnect.
#[test]
fn a_provider_that_goes_is_a_disconnect() {
    let (client, end) = script_wire::pipe::pair(MAX_FRAME).unwrap();
    drop(end);
    let mut carrier = WitCarrier::new(Calls::new(client));
    let reply = carrier.request(CarrierRequestBody::Discover);
    assert!(
        reply.as_ref().is_err_and(|e| e.is_disconnected()),
        "{reply:?}"
    );
}
