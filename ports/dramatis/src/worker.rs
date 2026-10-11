// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One serial client worker; never block the UI while Djinn asks for consent.
use crate::{Reply, Request};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;
pub struct Worker {
    tx: Option<mpsc::Sender<Request>>,
    rx: mpsc::Receiver<(Reply, bool)>,
    join: Option<thread::JoinHandle<()>>,
}
impl Worker {
    pub fn spawn(endpoint: String, wake: Arc<dyn Fn() + Send + Sync>) -> Self {
        let (tx, requests) = mpsc::channel();
        let (replies, rx) = mpsc::channel();
        let join = thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("client runtime");
            let mut next = Request::Refresh;
            let mut owned_request = false;
            loop {
                let reply = runtime.block_on(crate::execute_at(&endpoint, next));
                if replies.send((reply, owned_request)).is_err() {
                    break;
                }
                wake();
                next = match requests.recv_timeout(Duration::from_secs(3)) {
                    Ok(request) => {
                        owned_request = true;
                        request
                    },
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        owned_request = false;
                        Request::Refresh
                    },
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                };
            }
        });
        Self {
            tx: Some(tx),
            rx,
            join: Some(join),
        }
    }
    pub fn send(&self, request: Request) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(request);
        }
    }
    pub fn drain(&self) -> Vec<(Reply, bool)> {
        self.rx.try_iter().collect()
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.tx.take();
        // A native prompt belongs to Djinn, so closing this app must not wait
        // for its dialog. The client thread owns no application state on disk.
        self.join.take();
    }
}
