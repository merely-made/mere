// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The session's cookies as portable records for an engine flip (`flip`).

use netfetcher::{CookieStore, SameSite, SameSiteContext};

use super::session_jar;

/// The shared session's cookies for `url`, as portable records for a verso flip.
/// Reads the jar's structured same-site cookies (a flip is a same-origin top-level
/// navigation) and maps each to a [`inker::flip::api::Cookie`], carrying `Domain` / `Path` /
/// `Secure` / `HttpOnly` / `SameSite` / expiry faithfully (the lossless structured
/// read, native session store plan §5). `Partitioned` is not tracked by the jar yet.
pub fn session_cookies_for(url: &str) -> Vec<inker::flip::api::Cookie> {
    let Ok(parsed) = url::Url::parse(url) else {
        return Vec::new();
    };
    session_jar()
        .records_for(&parsed, SameSiteContext::same_site())
        .into_iter()
        .map(|r| inker::flip::api::Cookie {
            name: r.name,
            value: r.value,
            domain: r.domain,
            path: r.path,
            secure: r.secure,
            http_only: r.http_only,
            same_site: r.same_site.map(map_same_site),
            expires: r.expires,
            partitioned: false,
        })
        .collect()
}

/// netfetcher's `SameSite` (the `cookie` crate's) to verso's engine-agnostic one.
pub(crate) fn map_same_site(same_site: SameSite) -> inker::flip::api::SameSite {
    match same_site {
        SameSite::Strict => inker::flip::api::SameSite::Strict,
        SameSite::Lax => inker::flip::api::SameSite::Lax,
        SameSite::None => inker::flip::api::SameSite::None,
    }
}
