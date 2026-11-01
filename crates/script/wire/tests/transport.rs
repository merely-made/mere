// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Both transports carry whole frames, end cleanly when the consumer hangs
//! up, and refuse a frame over their cap.

use std::io;

use script_wire::{Serve, Transport};

fn echo(frame: &[u8]) -> io::Result<Vec<u8>> {
    Ok(frame.iter().rev().copied().collect())
}

fn exercise(mut client: impl Transport, server: impl Serve + Send + 'static, max: usize) {
    let served = std::thread::spawn(move || server.serve(echo));
    assert_eq!(client.call(vec![1, 2, 3]).unwrap(), vec![3, 2, 1]);
    assert_eq!(client.call(Vec::new()).unwrap(), Vec::<u8>::new());
    assert_eq!(client.call(vec![9; max]).unwrap(), vec![9; max]);
    drop(client);
    served
        .join()
        .unwrap()
        .expect("a hang-up ends the channel cleanly");
}

#[cfg(any(unix, windows))]
#[test]
fn pipes_carry_frames() {
    let (client, server) = script_wire::pipe::pair(1 << 16).unwrap();
    exercise(client, server, 1 << 16);
}

#[cfg(any(unix, windows))]
#[test]
fn pipes_refuse_an_oversized_frame() {
    let (mut client, server) = script_wire::pipe::pair(16).unwrap();
    let served = std::thread::spawn(move || server.serve(echo));
    assert!(client.call(vec![0; 17]).is_err());
    assert!(served.join().unwrap().is_err());
}

#[cfg(feature = "ipc")]
mod ipc {
    use super::*;
    use ipc_channel::ipc::channel;
    use script_wire::ipc::{Server, pair};

    #[test]
    fn ipc_carries_frames() {
        let (client, server) = pair(1 << 16).unwrap();
        exercise(client, server, 1 << 16);
    }

    /// N1 hands a provider its end over a channel: the passed end serves.
    #[test]
    fn a_passed_end_serves() {
        let (client, server) = pair(1 << 16).unwrap();
        let (tx, rx) = channel::<Server>().unwrap();
        tx.send(server).unwrap();
        let server = rx.recv().unwrap();
        exercise(client, server, 1 << 16);
    }

    #[test]
    fn ipc_refuses_an_oversized_frame() {
        let (mut client, server) = pair(16).unwrap();
        let served = std::thread::spawn(move || server.serve(echo));
        assert!(client.call(vec![0; 17]).is_err());
        assert!(served.join().unwrap().is_err());
    }
}
