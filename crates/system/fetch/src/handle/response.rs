// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn parse(url: &str) -> Result<Url, FetchError> {
    let parsed = Url::parse(url).map_err(|error| FetchError::BadAddress(error.to_string()))?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        other => Err(FetchError::BadAddress(format!(
            "unsupported scheme {other}"
        ))),
    }
}

pub(super) fn header<'a>(response: &'a Response, name: &str) -> Option<&'a str> {
    response
        .headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

pub(super) fn facts(response: &Response, requested: &Url) -> Facts {
    Facts {
        final_url: response.url_list.last().unwrap_or(requested).to_string(),
        content_type: header(response, "content-type")
            .map(|value| value.split(';').next().unwrap_or(value).trim().to_owned()),
        content_length: header(response, "content-length").and_then(|value| value.parse().ok()),
        etag: header(response, "etag").map(str::to_owned),
        last_modified: header(response, "last-modified").map(str::to_owned),
    }
}

/// `bytes START-END/TOTAL`, with the ordering a valid reply must have.
pub(super) fn content_range(value: &str) -> Option<(u64, u64, u64)> {
    let (span, total) = value.strip_prefix("bytes ")?.split_once('/')?;
    let (start, end) = span.split_once('-')?;
    let parsed: (u64, u64, u64) = (start.parse().ok()?, end.parse().ok()?, total.parse().ok()?);
    (parsed.0 <= parsed.1 && parsed.1 < parsed.2).then_some(parsed)
}
