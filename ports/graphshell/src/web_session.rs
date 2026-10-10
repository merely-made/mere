// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Storing the session's changes without holding the host across an await
//! (Scenograph editor plan, SE22).
//!
//! The host lives in an `Rc<RefCell<…>>` that every handler and frame borrows,
//! and IndexedDB writes await. So a store runs in three steps: under the
//! borrow, take the pending batch; without it, write the batch through a clone
//! of the store; under it again, mark the batch stored. One write is in flight
//! at a time, and changes made meanwhile wait for the next batch.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;
use web_sys::Event;

use super::{BrowserHost, root};

enum PreparedStore {
    Ordinary(graphshell::mere_host::Staged),
    Resident(graphshell::resident_host::ResidentIo<muniment::IndexedDbBackend>),
}

pub(super) use graphshell::resident_host::SessionStore;

/// Start a write if changes are pending and none is in flight. Called by the
/// frame pump once it has let go of the host.
pub(super) fn store_pending(state: &Rc<RefCell<BrowserHost>>, elapsed_ms: f64) {
    let attempt = {
        let mut host = state.borrow_mut();
        if host.session_store != SessionStore::Pending || super::web_timing::page_hidden() {
            return;
        }
        if !host.app.host.resident_publication_due(elapsed_ms, false) {
            return;
        }
        host.session_store = SessionStore::Writing;
        host.app.host.begin_session_store_attempt()
    };
    let state = state.clone();
    spawn_local(async move {
        let validation = {
            let mut host = state.borrow_mut();
            if !host.app.host.session_store_attempt_is_current(&attempt) {
                return;
            }
            if host.app.host.resident_settings_need_validation() {
                host.app.host.prepare_resident_settings_validation()
            } else {
                Ok(None)
            }
        };
        match validation {
            Ok(Some(request)) => {
                // This load owns its inputs. The RefCell borrow above has ended.
                let completion = request.load().await;
                let mut host = state.borrow_mut();
                let BrowserHost {
                    app,
                    session_store,
                    session_store_error,
                    ..
                } = &mut *host;
                if !app.host.continue_settings_store(
                    &attempt,
                    completion,
                    session_store,
                    session_store_error,
                ) {
                    host.chrome_dirty = true;
                    drop(host);
                    wake();
                    return;
                }
            },
            Err(error) => {
                let mut host = state.borrow_mut();
                host.session_store = SessionStore::Failed;
                host.session_store_error = error.to_string();
                drop(host);
                wake();
                return;
            },
            Ok(None) => {
                let mut host = state.borrow_mut();
                if host.app.host.resident_settings_need_validation() {
                    // Another load still owns the single-flight slot. Its
                    // completion wakes the pump; this is not a write failure.
                    host.session_store = SessionStore::Pending;
                    return;
                }
            },
        }
        let (staged, store) = {
            let mut host = state.borrow_mut();
            let now_secs = (js_sys::Date::now() / 1_000.0) as u64;
            // Resident preparation retains its batch in the existing host. This
            // branch never creates another writer or grants backend admission.
            let prepared =
                if host.app.host.resident_busy() || host.app.host.resident_settings_pending() {
                    host.app
                        .host
                        .prepare_resident_io(
                            std::time::SystemTime::UNIX_EPOCH
                                + std::time::Duration::from_secs(now_secs),
                        )
                        .map(PreparedStore::Resident)
                        .map_err(|error| error.to_string())
                } else {
                    host.app
                        .host
                        .prepare_store(now_secs)
                        .map(PreparedStore::Ordinary)
                        .map_err(|error| error.to_string())
                };
            match prepared {
                Ok(staged) => {
                    host.session_store = SessionStore::Writing;
                    (staged, host.app.host.store())
                },
                Err(error) => {
                    host.session_store = SessionStore::Failed;
                    host.session_store_error = error.to_string();
                    drop(host);
                    wake();
                    return;
                },
            }
        };
        let written = match &staged {
            PreparedStore::Ordinary(staged) => staged
                .write(&store)
                .await
                .map_err(|error| error.to_string()),
            PreparedStore::Resident(staged) => {
                staged.write().await.map_err(|error| error.to_string())
            },
        };
        {
            let mut host = state.borrow_mut();
            if !host.app.host.session_store_attempt_is_current(&attempt) {
                // Resident I/O retains its exact bytes and completion state in
                // the host; the current attempt can acknowledge or retry it.
                drop(host);
                wake();
                return;
            }
            match written {
                Ok(()) => {
                    match staged {
                        PreparedStore::Ordinary(staged) => host.app.host.staged(staged),
                        PreparedStore::Resident(staged) => {
                            if let Err(error) = host.app.host.install_resident_io(&staged) {
                                host.session_store = SessionStore::Failed;
                                host.session_store_error = error.to_string();
                                host.chrome_dirty = true;
                                drop(host);
                                wake();
                                return;
                            }
                        },
                    }
                    // A change made during the write is still pending.
                    host.session_store = if host.app.host.has_unstored() {
                        SessionStore::Pending
                    } else {
                        SessionStore::Stored
                    };
                },
                Err(error) => {
                    if let PreparedStore::Resident(staged) = &staged {
                        let _ = host.app.host.install_resident_io(staged);
                    }
                    host.session_store = SessionStore::Failed;
                    host.session_store_error = error.to_string();
                },
            }
            host.chrome_dirty = true;
        }
        wake();
    });
}

/// Ask the frame pump for a frame, without touching the host.
fn wake() {
    if let Ok(root) = root()
        && let Ok(event) = Event::new("graphshell-wake")
    {
        let _ = root.dispatch_event(&event);
    }
}

/// Drive the same owned request seam from the existing frame pump.
pub(super) fn refresh_pending(state: &Rc<RefCell<BrowserHost>>, elapsed_ms: f64) {
    let request = {
        let mut host = state.borrow_mut();
        host.app
            .host
            .suspend_resident_refresh(super::web_timing::page_hidden());
        host.app.host.tick_resident_refresh(elapsed_ms);
        host.app.host.prepare_resident_refresh()
    };
    let Ok(Some(request)) = request else {
        return;
    };
    let state = state.clone();
    spawn_local(async move {
        let completion = request.load().await;
        {
            let mut host = state.borrow_mut();
            match host.app.host.install_resident_refresh(completion) {
                Ok(_) | Err(graphshell::resident_host::ResidentHostError::StaleCompletion) => {},
                Err(error) => host.session_store_error = error.to_string(),
            }
            host.chrome_dirty = true;
        }
        wake();
    });
}
