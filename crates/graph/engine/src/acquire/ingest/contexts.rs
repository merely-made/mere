// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl ContextCache {
    /// An empty cache (the **none** preset): every remote `@context` is refused.
    pub fn new() -> Self {
        Self::default()
    }

    /// The **minimal** preset: only Mere's curated context (`schema:name` /
    /// `schema:keywords` + the `mere:` relation prefix), served at
    /// `https://mere.computer/ns/context`. Tiny; resolves Mere-authored documents
    /// but not arbitrary schema.org pages — those need [`full`](Self::full).
    pub fn minimal() -> Self {
        Self::new().with(MERE_CONTEXT_URL, MERE_MINIMAL_CONTEXT)
    }

    /// The **full** preset (the host default): the minimal context plus the
    /// vendored standard vocabularies (schema.org, ActivityStreams 2.0, Dublin
    /// Core; see `assets/NOTICE`), so an arbitrary remote `@context` for any of
    /// them resolves offline. Each pack is an independent module registered
    /// here; with the `bundled-contexts` feature off this equals
    /// [`minimal`](Self::minimal). A user can also step down to `minimal` /
    /// `new`, or bring their own via [`with`](Self::with).
    pub fn full() -> Self {
        #[allow(unused_mut)]
        let mut cache = Self::minimal();
        #[cfg(feature = "bundled-contexts")]
        {
            cache = cache
                .register(SCHEMA_ORG_URLS, SCHEMA_ORG_CONTEXT)
                .register(ACTIVITYSTREAMS_URLS, ACTIVITYSTREAMS_CONTEXT)
                .register(DUBLIN_CORE_URLS, DUBLIN_CORE_CONTEXT);
        }
        cache
    }

    /// Register `document` as the context served for each of `urls`.
    #[cfg(feature = "bundled-contexts")]
    pub(super) fn register(mut self, urls: &[&str], document: &'static [u8]) -> Self {
        for url in urls {
            self = self.with(*url, document);
        }
        self
    }

    /// Bundle a context document under its URL (builder style).
    pub fn with(mut self, url: impl Into<String>, document: impl Into<Vec<u8>>) -> Self {
        std::sync::Arc::make_mut(&mut self.documents).insert(url.into(), document.into());
        self
    }

    pub(super) fn get(&self, url: &str) -> Option<&[u8]> {
        self.documents.get(url).map(Vec::as_slice)
    }
}

/// Whether `url` is a remote `@context` the bundled packs already cover, so a host
/// need not fetch it before ingest. Always `false` when `bundled-contexts` is off
/// (then `full()` equals `minimal()`, so every remote context must be fetched).
pub fn is_bundled_context(url: &str) -> bool {
    #[cfg(feature = "bundled-contexts")]
    {
        SCHEMA_ORG_URLS.contains(&url)
            || ACTIVITYSTREAMS_URLS.contains(&url)
            || DUBLIN_CORE_URLS.contains(&url)
    }
    #[cfg(not(feature = "bundled-contexts"))]
    {
        let _ = url;
        false
    }
}
