// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The process-wide cookie jar. Durable per-persona persistence and the flip
//! export are features, in `cookies_persist.rs` and `cookies_flip.rs`.

use super::*;

/// The process-wide HTTP cookie jar: one persistent RFC 6265bis session shared by
/// every fetch, so a `Set-Cookie` on one page is still set on the next (logins
/// survive navigation) and the verso flip can carry the session into a
/// compatibility-view WebView. Without this each fetch built a throwaway jar, so no
/// session ever persisted. v1 is a single unpartitioned jar; per-origin / per-persona
/// partitioning + eidetic durability are later threads (the native session store
/// plan). The jar is `Send + Sync` (a `Mutex` inside), so the fetch worker and the UI
/// thread share the one `Arc`.
pub fn session_jar() -> &'static Arc<InMemoryCookieJar> {
    static JAR: OnceLock<Arc<InMemoryCookieJar>> = OnceLock::new();
    JAR.get_or_init(|| Arc::new(InMemoryCookieJar::new()))
}

/// Adapts the `Arc`-shared [`session_jar`] to netfetcher's `CookieStore` seam, which
/// takes an owned `Box<dyn CookieStore>` per context. Every per-fetch context's box
/// holds a clone of the one `Arc`, so they all read and write the same jar.
struct SharedJar(Arc<InMemoryCookieJar>);

impl CookieStore for SharedJar {
    fn cookies_for(&self, url: &url::Url, ctx: SameSiteContext) -> Vec<String> {
        self.0.cookies_for(url, ctx)
    }
    fn records_for(&self, url: &url::Url, ctx: SameSiteContext) -> Vec<CookieRecord> {
        self.0.records_for(url, ctx)
    }
    fn set_cookie(&self, url: &url::Url, set_cookie_header: &str) {
        self.0.set_cookie(url, set_cookie_header);
        // Mark the jar dirty so the next `persist_cookies` writes it through.
        COOKIES_DIRTY.store(true, Ordering::Relaxed);
    }
}

/// Set whenever a cookie is stored, cleared by [`persist_cookies`]. Lets the durable
/// write skip when nothing changed since the last persist (most fetches set no
/// cookies). Process-global, matching the single [`session_jar`].
pub(crate) static COOKIES_DIRTY: AtomicBool = AtomicBool::new(false);

/// Mark the jar dirty so the next [`persist_cookies`] writes it through. For cookie
/// writes that bypass `SharedJar` — e.g. a script's `document.cookie` via the scripted
/// rung's cookie provider, which uses the raw [`session_jar`]. (Render ladder 2c.)
pub fn mark_cookies_dirty() {
    COOKIES_DIRTY.store(true, Ordering::Relaxed);
}

/// A `FetchContext` whose cookie jar is the shared [`session_jar`]; the other seams
/// keep the permissive in-memory defaults for now. Built per fetch (cheap: the jar
/// inside is shared by `Arc`), so cookies accumulate across navigations.
pub(crate) fn session_context() -> netfetcher::FetchContext {
    let mut cx = netfetcher::FetchContext::permissive();
    cx.cookies = Box::new(SharedJar(session_jar().clone()));
    cx
}

#[cfg(feature = "persona-cookies")]
#[path = "cookies_persist.rs"]
mod persist;
#[cfg(feature = "persona-cookies")]
pub use persist::*;

#[cfg(feature = "flip")]
#[path = "cookies_flip.rs"]
mod flip;
#[cfg(feature = "flip")]
pub use flip::*;
