// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The blocking fetch handle: what a host builds once and hands to anything
//! that needs bytes on its own thread (a decoder, a downloader, a feed reader).
//! Design and decisions: `design_docs/mere_docs/implementation_strategy/2026-09-20_ranged_fetch_plan.md`.

use std::io;
use std::sync::Arc;
use std::time::Duration;

use netfetcher::{
    AltSvcStore, CacheMode, CookieRecord, CookieStore, FetchContext, HstsStore, HttpCache,
    InMemoryAltSvc, InMemoryCookieJar, InMemoryHsts, InMemoryHttpCache, Request, Response,
    SameSiteContext, StoredResponse, Transport,
};
use tokio::runtime::Runtime;
use url::Url;

/// A byte range of a remote object. `end` is inclusive; `None` reads to the end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub start: u64,
    pub end: Option<u64>,
}

/// What came back with a body: where it really came from and how to recognise
/// the same representation later.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Facts {
    /// The address after redirects.
    pub final_url: String,
    pub content_type: Option<String>,
    /// The server's declared length of the body it sent, when it gave one.
    pub content_length: Option<u64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

/// One honoured range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeReply {
    pub bytes: Vec<u8>,
    /// Offset of `bytes[0]` in the object.
    pub start: u64,
    /// Length of the whole object.
    pub total: u64,
    pub facts: Facts,
}

/// A whole body held in memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Body {
    pub bytes: Vec<u8>,
    pub facts: Facts,
}

#[derive(Debug)]
pub enum FetchError {
    /// The address did not parse or is not something this handle fetches.
    BadAddress(String),
    /// No response: connection, TLS, policy block or timeout.
    Unreachable(String),
    /// A response, but not a usable one.
    Status(u16),
    /// The server answered a ranged request with the whole object.
    RangeIgnored,
    /// `if_range` no longer names the object: the representation changed.
    Changed,
    /// A 206 whose `Content-Range` is missing, malformed or not what was asked.
    BadRange(String),
    /// The body is larger than the caller's limit.
    TooLarge { limit: u64 },
    /// The body stopped or failed partway.
    Body(io::Error),
    /// The caller's sink refused bytes.
    Sink(io::Error),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadAddress(why) => write!(f, "bad address: {why}"),
            Self::Unreachable(why) => write!(f, "unreachable: {why}"),
            Self::Status(status) => write!(f, "HTTP {status}"),
            Self::RangeIgnored => f.write_str("the server does not honour byte ranges"),
            Self::Changed => f.write_str("the representation changed"),
            Self::BadRange(why) => write!(f, "bad range reply: {why}"),
            Self::TooLarge { limit } => write!(f, "body exceeds the {limit}-byte limit"),
            Self::Body(error) => write!(f, "body failed: {error}"),
            Self::Sink(error) => write!(f, "sink failed: {error}"),
        }
    }
}

impl std::error::Error for FetchError {}

/// Blocking fetches under one host policy. Call from a plain thread, never from
/// inside an async runtime.
pub trait Fetch: Send + Sync {
    /// One byte range. With `if_range` (an ETag or Last-Modified from an earlier
    /// reply), a changed object is [`FetchError::Changed`] rather than wrong bytes.
    fn read_range(
        &self,
        url: &str,
        range: Range,
        if_range: Option<&str>,
    ) -> Result<RangeReply, FetchError>;

    /// A whole body of at most `limit` bytes, for small things such as a feed.
    fn read_all(&self, url: &str, accept: Option<&str>, limit: u64) -> Result<Body, FetchError>;

    /// A whole body streamed into `sink`, never held whole. `limit` bounds it.
    fn read_into(
        &self,
        url: &str,
        sink: &mut dyn io::Write,
        limit: Option<u64>,
    ) -> Result<Facts, FetchError>;
}

/// The stores a host owns and every fetch in one scope shares. What a scope is
/// (a persona, a session, a graph) is the host's business, not this crate's.
#[derive(Clone)]
pub struct Stores {
    pub cookies: Arc<dyn CookieStore>,
    pub cache: Arc<dyn HttpCache>,
    pub hsts: Arc<dyn HstsStore>,
    pub alt_svc: Arc<dyn AltSvcStore>,
    /// The wire every fetch in this scope goes through: where a proxy, an
    /// onion or garlic lane, or a test double plugs in. `None` is netfetcher's
    /// default, direct HTTP.
    pub transport: Option<Arc<dyn Transport>>,
}

impl Stores {
    /// Process-lifetime stores; a host with durable ones supplies its own.
    pub fn in_memory() -> Self {
        Self {
            cookies: Arc::new(InMemoryCookieJar::new()),
            cache: Arc::new(InMemoryHttpCache::new()),
            hsts: Arc::new(InMemoryHsts::new()),
            alt_svc: Arc::new(InMemoryAltSvc::new()),
            transport: None,
        }
    }

    /// A netfetcher context over these stores. Cheap: the stores are shared.
    pub fn context(&self) -> FetchContext {
        let mut context = FetchContext::permissive();
        context.cookies = Box::new(Shared(self.cookies.clone()));
        context.cache = Arc::new(Shared(self.cache.clone()));
        context.hsts = Box::new(Shared(self.hsts.clone()));
        context.alt_svc = Box::new(Shared(self.alt_svc.clone()));
        if let Some(transport) = &self.transport {
            context = context.with_transport(transport.clone());
        }
        context
    }
}

/// netfetcher's context owns boxes; a host's stores are shared. This bridges.
struct Shared<T: ?Sized>(Arc<T>);

impl CookieStore for Shared<dyn CookieStore> {
    fn cookies_for(&self, url: &Url, ctx: SameSiteContext) -> Vec<String> {
        self.0.cookies_for(url, ctx)
    }
    fn records_for(&self, url: &Url, ctx: SameSiteContext) -> Vec<CookieRecord> {
        self.0.records_for(url, ctx)
    }
    fn set_cookie(&self, url: &Url, header: &str) {
        self.0.set_cookie(url, header);
    }
}

impl HttpCache for Shared<dyn HttpCache> {
    fn enabled(&self) -> bool {
        self.0.enabled()
    }
    fn get(&self, key: &str) -> Option<StoredResponse> {
        self.0.get(key)
    }
    fn put(&self, key: &str, entry: StoredResponse) {
        self.0.put(key, entry);
    }
}

impl HstsStore for Shared<dyn HstsStore> {
    fn is_secure(&self, host: &str) -> bool {
        self.0.is_secure(host)
    }
    fn record(&self, host: &str, max_age_secs: u64, include_subdomains: bool) {
        self.0.record(host, max_age_secs, include_subdomains);
    }
}

impl AltSvcStore for Shared<dyn AltSvcStore> {
    fn h3_port(&self, host: &str) -> Option<u16> {
        self.0.h3_port(host)
    }
    fn record_h3(&self, host: &str, port: u16, max_age_secs: u64) {
        self.0.record_h3(host, port, max_age_secs);
    }
    fn clear(&self, host: &str) {
        self.0.clear(host);
    }
}

/// [`Fetch`] over netfetcher: one runtime, one context, one set of stores.
pub struct NetFetch {
    runtime: Arc<Runtime>,
    context: FetchContext,
    /// Longest wait for response headers, and for each body chunk after them.
    patience: Duration,
}

impl NetFetch {
    pub const DEFAULT_PATIENCE: Duration = Duration::from_secs(30);

    /// A handle with a small runtime of its own.
    pub fn new(stores: &Stores) -> io::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("fetch-handle")
            .enable_all()
            .build()?;
        Ok(Self::on(stores, Arc::new(runtime)))
    }

    /// A handle on a runtime the host already runs.
    pub fn on(stores: &Stores, runtime: Arc<Runtime>) -> Self {
        Self {
            runtime,
            context: stores.context(),
            patience: Self::DEFAULT_PATIENCE,
        }
    }

    pub fn with_patience(mut self, patience: Duration) -> Self {
        self.patience = patience;
        self
    }

    /// Send, and wait for headers.
    fn send(&self, request: Request) -> Result<Response, FetchError> {
        let response = self
            .runtime
            .block_on(async {
                tokio::time::timeout(self.patience, netfetcher::fetch(request, &self.context)).await
            })
            .map_err(|_| FetchError::Unreachable("timed out waiting for a response".into()))?;
        if response.is_network_error() {
            return Err(FetchError::Unreachable("network error".into()));
        }
        Ok(response)
    }

    /// Drain a body into `take`, stopping past `limit`.
    fn drain(
        &self,
        mut response: Response,
        limit: Option<u64>,
        mut take: impl FnMut(&[u8]) -> Result<(), FetchError>,
    ) -> Result<(), FetchError> {
        let mut seen = 0_u64;
        loop {
            let chunk = self
                .runtime
                .block_on(async {
                    tokio::time::timeout(self.patience, response.body.next_chunk()).await
                })
                .map_err(|_| FetchError::Body(io::ErrorKind::TimedOut.into()))?;
            let Some(chunk) = chunk else { return Ok(()) };
            let chunk = chunk.map_err(FetchError::Body)?;
            seen += chunk.len() as u64;
            if let Some(limit) = limit
                && seen > limit
            {
                return Err(FetchError::TooLarge { limit });
            }
            take(&chunk)?;
        }
    }
}

fn parse(url: &str) -> Result<Url, FetchError> {
    let parsed = Url::parse(url).map_err(|error| FetchError::BadAddress(error.to_string()))?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        other => Err(FetchError::BadAddress(format!(
            "unsupported scheme {other}"
        ))),
    }
}

fn header<'a>(response: &'a Response, name: &str) -> Option<&'a str> {
    response
        .headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn facts(response: &Response, requested: &Url) -> Facts {
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
fn content_range(value: &str) -> Option<(u64, u64, u64)> {
    let (span, total) = value.strip_prefix("bytes ")?.split_once('/')?;
    let (start, end) = span.split_once('-')?;
    let parsed: (u64, u64, u64) = (start.parse().ok()?, end.parse().ok()?, total.parse().ok()?);
    (parsed.0 <= parsed.1 && parsed.1 < parsed.2).then_some(parsed)
}

impl Fetch for NetFetch {
    fn read_range(
        &self,
        url: &str,
        range: Range,
        if_range: Option<&str>,
    ) -> Result<RangeReply, FetchError> {
        let requested = parse(url)?;
        let mut request = Request::get(requested.clone());
        let span = match range.end {
            Some(end) => format!("bytes={}-{end}", range.start),
            None => format!("bytes={}-", range.start),
        };
        request.headers.push(("range".to_owned(), span));
        if let Some(validator) = if_range {
            request
                .headers
                .push(("if-range".to_owned(), validator.to_owned()));
        }
        // Never the HTTP cache: once a whole object is stored, netfetcher answers
        // a ranged request with all of it (pinned in this crate's tests).
        request.cache = CacheMode::NoStore;

        let response = self.send(request)?;
        match response.status {
            206 => {},
            200 if if_range.is_some() => return Err(FetchError::Changed),
            200 => return Err(FetchError::RangeIgnored),
            status => return Err(FetchError::Status(status)),
        }
        let (start, end, total) = header(&response, "content-range")
            .and_then(content_range)
            .ok_or_else(|| FetchError::BadRange("missing or malformed Content-Range".into()))?;
        let wanted_end = range.end.map_or(total - 1, |end| end.min(total - 1));
        if start != range.start || end != wanted_end {
            return Err(FetchError::BadRange(format!(
                "asked for {}-{wanted_end}, got {start}-{end}",
                range.start
            )));
        }
        let facts = facts(&response, &requested);
        let expected = end - start + 1;
        let mut bytes = Vec::with_capacity(expected as usize);
        self.drain(response, Some(expected), |chunk| {
            bytes.extend_from_slice(chunk);
            Ok(())
        })?;
        if bytes.len() as u64 != expected {
            return Err(FetchError::Body(io::ErrorKind::UnexpectedEof.into()));
        }
        Ok(RangeReply {
            bytes,
            start,
            total,
            facts,
        })
    }

    fn read_all(&self, url: &str, accept: Option<&str>, limit: u64) -> Result<Body, FetchError> {
        let requested = parse(url)?;
        let mut request = Request::get(requested.clone());
        if let Some(accept) = accept {
            request
                .headers
                .push(("accept".to_owned(), accept.to_owned()));
        }
        let response = self.send(request)?;
        if !(200..300).contains(&response.status) {
            return Err(FetchError::Status(response.status));
        }
        let facts = facts(&response, &requested);
        if facts.content_length.is_some_and(|length| length > limit) {
            return Err(FetchError::TooLarge { limit });
        }
        let mut bytes = Vec::new();
        self.drain(response, Some(limit), |chunk| {
            bytes.extend_from_slice(chunk);
            Ok(())
        })?;
        Ok(Body { bytes, facts })
    }

    fn read_into(
        &self,
        url: &str,
        sink: &mut dyn io::Write,
        limit: Option<u64>,
    ) -> Result<Facts, FetchError> {
        let requested = parse(url)?;
        let mut request = Request::get(requested.clone());
        // Identity, so the sink receives the object's own bytes and the declared
        // length is the length written.
        request
            .headers
            .push(("accept-encoding".to_owned(), "identity".to_owned()));
        // A download is custody, not a cache entry.
        request.cache = CacheMode::NoStore;
        let response = self.send(request)?;
        if !(200..300).contains(&response.status) {
            return Err(FetchError::Status(response.status));
        }
        let facts = facts(&response, &requested);
        if let (Some(limit), Some(length)) = (limit, facts.content_length)
            && length > limit
        {
            return Err(FetchError::TooLarge { limit });
        }
        self.drain(response, limit, |chunk| {
            sink.write_all(chunk).map_err(FetchError::Sink)
        })?;
        Ok(facts)
    }
}

#[cfg(test)]
#[path = "handle_tests.rs"]
mod tests;
