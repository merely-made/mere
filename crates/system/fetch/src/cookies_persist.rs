// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Durable, persona-keyed persistence of the session jar (`persona-cookies`).

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, OnceLock};

use eidetic::Store;
use netfetcher::{CookieRecord, SameSite};
use pandect::PersonaId;
use serde::{Deserialize, Serialize};

use super::{COOKIES_DIRTY, session_jar};

/// One persisted cookie. A serde mirror of netfetcher's `CookieRecord` (whose
/// `same_site` is the `cookie` crate's enum, not directly serde-friendly), encoded as
/// JSON in its own per-cookie blob in the durable store.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PersistedCookie {
    name: String,
    value: String,
    domain: String,
    host_only: bool,
    path: String,
    secure: bool,
    http_only: bool,
    /// 0 = Strict, 1 = Lax, 2 = None; absent = unspecified.
    same_site: Option<u8>,
    /// Absolute expiry in Unix seconds; absent = session cookie.
    expires: Option<f64>,
}

impl PersistedCookie {
    fn from_record(r: CookieRecord) -> Self {
        Self {
            name: r.name,
            value: r.value,
            domain: r.domain,
            host_only: r.host_only,
            path: r.path,
            secure: r.secure,
            http_only: r.http_only,
            same_site: r.same_site.map(|s| match s {
                SameSite::Strict => 0,
                SameSite::Lax => 1,
                SameSite::None => 2,
            }),
            expires: r.expires,
        }
    }

    fn into_record(self) -> CookieRecord {
        CookieRecord {
            name: self.name,
            value: self.value,
            domain: self.domain,
            host_only: self.host_only,
            path: self.path,
            secure: self.secure,
            http_only: self.http_only,
            same_site: match self.same_site {
                Some(0) => Some(SameSite::Strict),
                Some(1) => Some(SameSite::Lax),
                Some(2) => Some(SameSite::None),
                _ => None,
            },
            expires: self.expires,
        }
    }
}

/// A cookie's durable identity: `(domain, path, name)`, the RFC 6265 tuple that
/// uniquely names a cookie. The unit of incremental persistence.
type CookieKey = (String, String, String);

/// A mirror of what is currently written to the durable store, per persona, so a
/// persist writes only the cookies that *changed* (incremental) rather than the whole
/// jar each time. Populated on [`load_cookies`] and kept in step by [`persist_cookies`].
pub(crate) fn last_persisted() -> &'static Mutex<HashMap<CookieKey, PersistedCookie>> {
    static LAST: OnceLock<Mutex<HashMap<CookieKey, PersistedCookie>>> = OnceLock::new();
    LAST.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn cookie_key_of(c: &PersistedCookie) -> CookieKey {
    (c.domain.clone(), c.path.clone(), c.name.clone())
}

/// The durable-store blob key for one cookie under `persona`. Keyed by persona so one
/// persona's session never bleeds into another (the native session store's hard
/// partition; v0 is the single default persona, but the key already partitions), then
/// by the hex-encoded `(domain, path, name)` identity (hex so a path's `/` can't
/// collide with the key's own separators).
pub(crate) fn cookie_blob_key(persona: PersonaId, key: &CookieKey) -> String {
    let identity = format!("{}\u{0}{}\u{0}{}", key.0, key.1, key.2);
    let hex: String = identity.bytes().map(|b| format!("{b:02x}")).collect();
    format!("{}{hex}", cookie_prefix(persona))
}

pub(crate) fn cookie_prefix(persona: PersonaId) -> String {
    format!("cookies/{}/", persona.0)
}

/// Persist the shared jar to the durable `store` for `persona`, writing only the
/// cookies that changed since the last persist and deleting blobs for cookies that are
/// gone (expired / cleared). Dirty-gated, so a fetch that set no cookies is a no-op.
/// Each cookie is its own small blob, so one `Set-Cookie` writes one blob, not the
/// whole jar. Runs the async store to completion on the calling (UI) thread, like the
/// rest of meerkat's eidetic use.
pub fn persist_cookies(store: &mut dyn Store, persona: PersonaId) {
    if !COOKIES_DIRTY.swap(false, Ordering::Relaxed) {
        return;
    }
    let current: HashMap<CookieKey, PersistedCookie> = session_jar()
        .all_records()
        .into_iter()
        .map(PersistedCookie::from_record)
        .map(|c| (cookie_key_of(&c), c))
        .collect();

    let mut shadow = match last_persisted().lock() {
        Ok(shadow) => shadow,
        Err(_) => return,
    };
    let mut any_failed = false;

    // Writes: new or changed cookies only.
    for (key, cookie) in &current {
        if shadow.get(key) == Some(cookie) {
            continue;
        }
        match serde_json::to_vec(cookie) {
            Ok(bytes) => {
                match pollster::block_on(store.put(&cookie_blob_key(persona, key), &bytes)) {
                    Ok(()) => {
                        shadow.insert(key.clone(), cookie.clone());
                    },
                    Err(err) => {
                        tracing::warn!(%err, "cookie persist: save failed");
                        any_failed = true;
                    },
                }
            },
            Err(err) => {
                tracing::warn!(%err, "cookie persist: serialize failed");
                any_failed = true;
            },
        }
    }

    // Deletes: cookies that were persisted but are gone from the jar now.
    let removed: Vec<CookieKey> = shadow
        .keys()
        .filter(|k| !current.contains_key(*k))
        .cloned()
        .collect();
    for key in removed {
        match pollster::block_on(store.delete(&cookie_blob_key(persona, &key))) {
            Ok(_) => {
                shadow.remove(&key);
            },
            Err(err) => {
                tracing::warn!(%err, "cookie persist: delete failed");
                any_failed = true;
            },
        }
    }

    // A failed write/delete leaves the shadow out of step; re-arm so the next change
    // retries it rather than the failure being silently dropped.
    if any_failed {
        COOKIES_DIRTY.store(true, Ordering::Relaxed);
    }
}

/// Load `persona`'s persisted cookies from the durable `store` into the shared jar at
/// startup, so a login survives an app restart, and seed the persisted-shadow so the
/// first persist after load does not rewrite everything. A persona with no stored
/// cookies (first run) leaves the jar empty.
pub fn load_cookies(store: &mut dyn Store, persona: PersonaId) {
    let keys = match pollster::block_on(store.list(&cookie_prefix(persona))) {
        Ok(keys) => keys,
        Err(err) => {
            tracing::warn!(%err, "cookie load: iter_keys failed");
            return;
        },
    };
    let mut records = Vec::with_capacity(keys.len());
    let mut shadow = match last_persisted().lock() {
        Ok(shadow) => shadow,
        Err(_) => return,
    };
    for key in keys {
        match pollster::block_on(store.get(&key)) {
            Ok(Some(bytes)) => match serde_json::from_slice::<PersistedCookie>(&bytes) {
                Ok(cookie) => {
                    shadow.insert(cookie_key_of(&cookie), cookie.clone());
                    records.push(cookie.into_record());
                },
                Err(err) => tracing::warn!(%err, "cookie load: deserialize failed"),
            },
            Ok(None) => {},
            Err(err) => tracing::warn!(%err, "cookie load: read failed"),
        }
    }
    drop(shadow);
    session_jar().load_records(records);
}
