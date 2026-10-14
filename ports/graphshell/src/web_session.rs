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

pub(super) use graphshell::resident_host::SessionStore;

/// Start a write if changes are pending and none is in flight. Called by the
/// frame pump once it has let go of the host.
pub(super) fn store_pending(state: &Rc<RefCell<BrowserHost>>, elapsed_ms: f64) {
    store(state, elapsed_ms, false);
}
pub(super) fn save_now(state: &Rc<RefCell<BrowserHost>>) {
    {
        let mut host = state.borrow_mut();
        if host.session_store == SessionStore::Writing {
            return;
        }
        host.session_store = SessionStore::Pending;
        host.app.host.suspend_resident_refresh(false);
    }
    store(state, 0.0, true);
}
pub(super) fn flush_pending(state: &Rc<RefCell<BrowserHost>>) {
    {
        let mut host = state.borrow_mut();
        if host.app.host.has_unstored() && host.session_store != SessionStore::Writing {
            host.session_store = SessionStore::Pending;
        }
        if host.session_store == SessionStore::Pending
            || host.session_store == SessionStore::Writing
        {
            // Settings validation is part of the retained save. Suspend only
            // after its terminal acknowledgement, even if frames stop.
            host.app.host.suspend_resident_refresh(false);
        } else {
            host.app.host.detach_resident_refresh();
        }
    }
    store(state, 0.0, true);
}
fn store(state: &Rc<RefCell<BrowserHost>>, elapsed_ms: f64, flush: bool) {
    let attempt = {
        let mut host = state.borrow_mut();
        if host.session_store != SessionStore::Pending
            || (!flush && super::web_timing::page_hidden())
        {
            return;
        }
        if !host.app.host.resident_publication_due(elapsed_ms, flush) {
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
                let current_attempt = host.app.host.session_store_attempt_is_current(&attempt);
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
                    if current_attempt && host.session_store == SessionStore::Failed {
                        host.publication_read_error = None;
                    }
                    host.chrome_dirty = true;
                    drop(host);
                    wake();
                    return;
                }
            },
            Err(error) => {
                let mut host = state.borrow_mut();
                host.session_store = SessionStore::Failed;
                host.publication_read_error = None;
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
        let staged = {
            let mut host = state.borrow_mut();
            if host.app.host.pending_publication() {
                host.app.host.notice_resident_refresh();
                host.session_store = SessionStore::Stored;
                host.chrome_dirty = true;
                drop(host);
                wake();
                return;
            }
            let now_secs = (js_sys::Date::now() / 1_000.0) as u64;
            match host.app.host.prepare_owned_store(now_secs) {
                Ok(staged) => staged,
                Err(error) => {
                    host.session_store = SessionStore::Failed;
                    host.publication_read_error = None;
                    host.session_store_error = error.to_string();
                    drop(host);
                    wake();
                    return;
                },
            }
        };
        let written = staged.write().await;
        {
            let mut host = state.borrow_mut();
            if !host.app.host.session_store_attempt_is_current(&attempt) {
                // Resident I/O retains its exact bytes and completion state in
                // the host; the current attempt can acknowledge or retry it.
                drop(host);
                wake();
                return;
            }
            match host.app.host.install_owned_store(staged) {
                Ok(true) => {
                    host.session_store = if host.app.host.has_unstored() {
                        SessionStore::Pending
                    } else {
                        SessionStore::Stored
                    };
                    host.publication_read_error = None;
                    host.session_store_error.clear();
                },
                Ok(false) => {
                    host.session_store = SessionStore::Failed;
                    host.publication_read_error = None;
                    host.session_store_error = written
                        .err()
                        .unwrap_or_else(|| "Applied but unstored; retry Save".into());
                },
                Err(error) => {
                    host.session_store = SessionStore::Failed;
                    host.publication_read_error = None;
                    host.session_store_error = error.to_string();
                },
            }
            if super::web_timing::page_hidden() {
                host.app.host.detach_resident_refresh();
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

/// A retry may finish through the frame pump or the public Demand control.
/// Both capture the error identity before yielding and retire only that error.
fn publication_recovery(host: &BrowserHost) -> Option<(Rc<()>, String)> {
    host.app
        .host
        .pending_publication()
        .then(|| host.publication_read_error.clone())
        .flatten()
}

fn publish_reference_reader(
    host: &mut BrowserHost,
    recovering_error: Option<&(Rc<()>, String)>,
) -> Result<(), String> {
    let selected = host.canvas.selected_members();
    let synced = match host.sync_canvas(&selected, false) {
        Ok(()) => true,
        Err(error) => {
            host.publication_read_error = None;
            host.session_store_error = error;
            false
        },
    };
    let mounted = match host.app.mount_local() {
        Ok(_) => true,
        Err(error) => {
            host.publication_read_error = None;
            host.session_store_error = error.to_string();
            false
        },
    };
    if synced
        && mounted
        && !host.app.host.pending_publication()
        && recovering_error.is_some_and(|(identity, message)| {
            host.publication_read_error
                .as_ref()
                .is_some_and(|(current, _)| Rc::ptr_eq(identity, current))
                && host.session_store_error == *message
        })
    {
        host.session_store_error.clear();
        host.publication_read_error = None;
    }
    if synced && mounted {
        Ok(())
    } else {
        Err(host.session_store_error.clone())
    }
}

/// Drive the same owned request seam from the existing frame pump.
pub(super) fn refresh_pending(state: &Rc<RefCell<BrowserHost>>, elapsed_ms: f64) {
    let (request, recovering_error) = {
        let mut host = state.borrow_mut();
        let suspend =
            super::web_timing::page_hidden() && host.session_store != SessionStore::Writing;
        host.app.host.suspend_resident_refresh(suspend);
        host.app.host.tick_resident_refresh(elapsed_ms);
        let recovering_error = publication_recovery(&host);
        (host.app.host.prepare_resident_refresh(), recovering_error)
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
                Ok(_) => {
                    let _ = publish_reference_reader(&mut host, recovering_error.as_ref());
                },
                Err(graphshell::resident_host::ResidentHostError::StaleCompletion) => {},
                Err(error) => {
                    let message = error.to_string();
                    host.publication_read_error = host
                        .app
                        .host
                        .pending_publication()
                        .then(|| (Rc::new(()), message.clone()));
                    host.session_store_error = message;
                },
            }
            host.chrome_dirty = true;
        }
        if super::web_timing::page_hidden() {
            flush_pending(&state);
        }
        wake();
    });
}

pub(super) fn demand_from_form(state: &Rc<RefCell<BrowserHost>>) {
    use wasm_bindgen::JsCast;
    let result = (|| {
        let input: web_sys::HtmlInputElement = super::element("resident-demand")?
            .dyn_into()
            .map_err(|_| "demand input")?;
        let id = uuid::Uuid::parse_str(input.value().trim())
            .map_err(|e| format!("invalid stable ID: {e}"))?;
        let stratum: web_sys::HtmlSelectElement = super::element("resident-stratum")?
            .dyn_into()
            .map_err(|_| "demand stratum")?;
        let address = if stratum.value() == "resource" {
            graphshell::web_resident::GraphAddress::Resource(id)
        } else {
            graphshell::web_resident::GraphAddress::Surface(id)
        };
        let mut host = state.borrow_mut();
        let recovering_error = publication_recovery(&host);
        host.app
            .host
            .prepare_resident_demand(address)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "presentation refresh is busy; save dirty settings first".to_string())
            .map(|request| (request, address, recovering_error))
    })();
    match result {
        Err(error) => {
            let mut host = state.borrow_mut();
            host.product_status = error;
            host.chrome_dirty = true;
            wake();
        },
        Ok((request, address, recovering_error)) => {
            let state = state.clone();
            spawn_local(async move {
                let completion = request.load().await;
                let mut host = state.borrow_mut();
                host.product_status = match host.app.host.install_resident_refresh(completion) {
                    Ok(_) => match publish_reference_reader(&mut host, recovering_error.as_ref()) {
                        Ok(()) => host.app.host.observation_text(address),
                        Err(error) => error,
                    },
                    Err(error) => error.to_string(),
                };
                host.chrome_dirty = true;
                drop(host);
                wake();
            });
        },
    }
}
