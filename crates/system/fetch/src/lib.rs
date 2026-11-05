// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Off-UI-thread fetching (S2.2b).
//!
//! netfetcher's WHATWG Fetch is async (tokio/hyper), so it runs on a worker
//! runtime, never the UI thread. Delivery is **model 2** (channel + bare wake):
//! a completed fetch pushes a [`FetchOutcome`] over an `mpsc` channel and wakes
//! the winit loop via an `EventLoopProxy<()>`; the host drains the channel in
//! `user_event` and updates its per-URL content cache. The winit user-event type
//! stays trivial, so persistence (S3) and sync (S5) can push their own typed
//! channels through the same wake without fighting over one event enum.
//!
//! S2.2b-i carries the decoded body as text and renders it plainly; content-type
//! routing to the nematic + genet engines is S2.2b-ii.
//!
//! Smolweb: the same actor also speaks the small web. A page request is routed by
//! scheme, http(s) runs through netfetcher (WHATWG Fetch) and gemini / gopher /
//! finger / spartan / nex / guppy run through the [`errand`] transport
//! crate. Either way the result is the same [`Fetched`] (decoded body +
//! content-type), so the render side (nematic engines) is unchanged.
//! Subresources use the same scheme routing and Gemini trust store as pages,
//! while retaining raw bytes for images and other binary media.
//! Titan uploads and Spartan query/data submissions require explicit Submit,
//! including when a read redirects to a submission target.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use netfetcher::{CookieRecord, CookieStore, InMemoryCookieJar, SameSiteContext};
use zeroize::Zeroizing;

/// Host-supplied durable trust storage for Gemini-style TLS.
#[cfg(feature = "smolweb")]
pub use errand::TofuStore as SmolwebTofuStore;

mod policy;
pub use policy::ClientFetchPolicy;

/// Host-side cap on a fetched page body (§A5): a hard ceiling enforced *while*
/// streaming the body, so a malicious or runaway server cannot OOM the host by
/// sending an unbounded response. Generous for a real page; a script `net.fetch`
/// passes a tighter cap (the body is copied into the guest's mem-quota'd memory).
const PAGE_BODY_CAP: usize = 64 * 1024 * 1024;

/// The fetch vocabulary lives in `page-load` so a controller can speak it
/// without linking this actor's transport. Re-exported unchanged.
pub use page_load::{
    FetchFailure, FetchOutcome, FetchRequestId, Fetched, PageProgress, next_fetch_request_id,
};

/// One explicit small-web mutation. Kept separate from page fetches so the
/// actor cannot deduplicate, redirect-follow, or retry a write as if it were a
/// read.
pub enum SmolwebSubmission {
    Titan {
        url: String,
        body: Vec<u8>,
        mime: String,
        token: Option<String>,
        identity: Option<GeminiClientIdentity>,
    },
    Spartan {
        url: String,
        body: Vec<u8>,
    },
}

impl SmolwebSubmission {
    pub fn url(&self) -> &str {
        match self {
            Self::Titan { url, .. } | Self::Spartan { url, .. } => url,
        }
    }
}

/// A write response. Redirects are returned to the host rather than followed,
/// because following them inside the write operation would erase the boundary
/// between the mutation receipt and the subsequent read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubmissionAnswer {
    Success(Fetched),
    Redirect(String),
}

pub struct SubmissionOutcome {
    pub request: u64,
    pub url: String,
    pub result: Result<SubmissionAnswer, FetchFailure>,
}

/// A fetched subresource: raw bytes for an absolute URL (page CSS via
/// `<link>`, an `<img>`, ...). Carried as bytes, not text, since media is
/// binary. Failures are delivered too so a host can clear pending state and
/// retain its fallback presentation.
pub struct SubresourceOutcome {
    pub url: String,
    /// The actor always completes a requested subresource so hosts can clear
    /// their correlation state even when transport or decoding fails.
    pub result: Result<Vec<u8>, String>,
}

/// One Gemini client certificate, assigned to exactly one capsule origin.
///
/// The host mints the material from its identity layer. The actor enforces the
/// host+effective-port scope on every redirect, so a certificate selected for
/// one capsule is never presented to another. Private bytes are shared rather
/// than copied between effects and commands, and zeroized when the last owner
/// drops.
#[derive(Clone)]
pub struct GeminiClientIdentity {
    host: String,
    port: u16,
    certificate_der: Arc<[u8]>,
    private_key_pkcs8_der: Arc<Zeroizing<Vec<u8>>>,
}

impl std::fmt::Debug for GeminiClientIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GeminiClientIdentity")
            .field("origin", &self.origin())
            .field("certificate", &"[redacted]")
            .field("private_key", &"[redacted]")
            .finish()
    }
}

impl PartialEq for GeminiClientIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
            && self.port == other.port
            && self.certificate_der.as_ref() == other.certificate_der.as_ref()
            && self.private_key_pkcs8_der.as_slice() == other.private_key_pkcs8_der.as_slice()
    }
}

impl Eq for GeminiClientIdentity {}

impl GeminiClientIdentity {
    pub fn new(
        capsule_url: &str,
        certificate_der: Vec<u8>,
        private_key_pkcs8_der: Vec<u8>,
    ) -> Result<Self, String> {
        let url = url::Url::parse(capsule_url).map_err(|error| error.to_string())?;
        if !matches!(url.scheme(), "gemini" | "titan") {
            return Err(
                "Gemini-family client identity requires a gemini:// or titan:// capsule"
                    .to_string(),
            );
        }
        let host = url
            .host_str()
            .ok_or_else(|| "Gemini capsule has no host".to_string())?
            .to_ascii_lowercase();
        if certificate_der.is_empty() || private_key_pkcs8_der.is_empty() {
            return Err("Gemini client identity material is empty".to_string());
        }
        Ok(Self {
            host,
            port: url.port().unwrap_or(1965),
            certificate_der: Arc::from(certificate_der),
            private_key_pkcs8_der: Arc::new(Zeroizing::new(private_key_pkcs8_der)),
        })
    }

    pub fn origin(&self) -> String {
        let host = if self.host.contains(':') && !self.host.starts_with('[') {
            format!("[{}]", self.host)
        } else {
            self.host.clone()
        };
        if self.port == 1965 {
            format!("gemini://{host}")
        } else {
            format!("gemini://{host}:{}", self.port)
        }
    }

    pub fn certificate_der(&self) -> &[u8] {
        self.certificate_der.as_ref()
    }

    #[cfg(feature = "smolweb")]
    fn applies_to(&self, url: &url::Url) -> bool {
        matches!(url.scheme(), "gemini" | "titan")
            && url
                .host_str()
                .is_some_and(|host| host.eq_ignore_ascii_case(&self.host))
            && url.port().unwrap_or(1965) == self.port
    }

    #[cfg(feature = "smolweb")]
    fn errand_view(&self) -> errand::GeminiClientIdentity<'_> {
        errand::GeminiClientIdentity {
            certificate_der: self.certificate_der.as_ref(),
            private_key_pkcs8_der: self.private_key_pkcs8_der.as_slice(),
        }
    }
}

/// Per-URL content state behind the focused-node card.
#[derive(Clone, Debug)]
pub enum ContentState {
    /// A fetch is in flight.
    Loading,
    /// Fetched content, ready to render.
    Ready(Fetched),
    /// The fetch failed; the reason renders on the card.
    Failed(String),
}

impl ContentState {
    /// A small tag distinguishing the states, folded into the card's cache key so
    /// a `Loading → Ready/Failed` transition re-renders the card (terminal states
    /// don't change again, so the tag alone suffices — no body hash needed).
    pub fn tag(state: Option<&ContentState>) -> u8 {
        match state {
            None => 0,
            Some(ContentState::Loading) => 1,
            Some(ContentState::Ready(_)) => 2,
            Some(ContentState::Failed(_)) => 3,
        }
    }
}

/// Whether `url` is a network address meerkat fetches (http(s) or a smolweb
/// scheme), vs a synthesized `mere://` page or another non-network scheme.
pub fn is_fetchable(url: &str) -> bool {
    match scheme_of(url) {
        Some(scheme) => scheme == "http" || scheme == "https" || is_smolweb(url),
        None => false,
    }
}

/// Whether `url` names a small-web scheme this build can fetch. Always false
/// without the `smolweb` feature, so such an address fails as an ordinary fetch.
fn is_smolweb(url: &str) -> bool {
    #[cfg(feature = "smolweb")]
    return scheme_of(url).and_then(errand::Scheme::parse).is_some();
    #[cfg(not(feature = "smolweb"))]
    return {
        let _ = url;
        false
    };
}

/// The scheme of a `scheme://…` URL, if it has an authority component. Returns
/// `None` for schemeless or `scheme:opaque` forms (e.g. `about:blank`), which are
/// never network-fetched.
fn scheme_of(url: &str) -> Option<&str> {
    url.split_once("://").map(|(scheme, _)| scheme)
}

/// A command to the fetch actor.
#[cfg(feature = "actor")]
pub enum FetchCommand {
    /// Fetch `url` as a page document (decoded body as text).
    Page {
        request: FetchRequestId,
        url: String,
        identity: Option<GeminiClientIdentity>,
    },
    /// Abort one exact page request. Other nodes loading the same URL continue.
    CancelPage { request: FetchRequestId },
    /// Fetch the subresource at the (already absolute) `url` as raw bytes.
    Subresource(String),
    /// Fetch the favicon at `url` (already absolute) as raw bytes. `request`
    /// preserves completion identity when the same page asks again before an
    /// older answer lands. Carried separately from `Subresource` so favicon
    /// bytes reach the graph, not the content actors' render stores.
    Favicon {
        request: FetchRequestId,
        url: String,
    },
    /// Perform one user-confirmed smolweb write exactly once.
    Submit {
        request: u64,
        submission: SmolwebSubmission,
    },
}

/// An update from the fetch actor: one completed page, subresource, or favicon fetch.
#[cfg(feature = "actor")]
pub enum FetchUpdate {
    PageProgress(PageProgress),
    Page(FetchOutcome),
    Subresource(SubresourceOutcome),
    /// One terminal favicon result. Failure remains typed so hosts can retire
    /// their exact pending request while keeping their existing tile unchanged.
    Favicon(FaviconOutcome),
    Submission(SubmissionOutcome),
}

/// A terminal favicon completion. The fetch actor reports both success and
/// failure because request bookkeeping must finish even when enrichment is
/// deliberately silent in the UI.
pub struct FaviconOutcome {
    pub request: FetchRequestId,
    pub result: Result<Vec<u8>, String>,
}

#[cfg(feature = "actor")]
mod actor;
#[cfg(feature = "actor")]
pub use actor::{
    FetchActor, FetchActorPolicy, FetchActorReport, spawn_fetcher, spawn_fetcher_with,
    spawn_fetcher_with_admission, spawn_fetcher_with_policy,
};

/// Pin Gemini certificates for the lifetime of this process. This is stronger
/// than errand's permissive default, but it deliberately makes no restart
/// durability claim; a host with durable trust state should install its own
/// [`errand::TofuStore`] instead.
#[cfg(feature = "smolweb")]
pub fn install_in_memory_smolweb_tofu() {
    install_smolweb_tofu(Arc::new(errand::InMemoryTofu::new()));
}

/// Install a host-owned Gemini trust store for every smolweb request in this
/// process. The host keeps the concrete store so certificate-change approval
/// can replace one pin before retrying the refused request.
#[cfg(feature = "smolweb")]
pub fn install_smolweb_tofu(store: Arc<dyn SmolwebTofuStore>) {
    errand::set_trust_store(store);
}

/// Fetch a page as the **crawler**, identifying with [`CRAWLER_USER_AGENT`]. http(s)
/// sends the descriptive bot UA; smolweb routes through errand as usual (errand owns
/// its UA). The crawl actor fetches every page (and robots.txt) through this, so a
/// site sees an honest, contactable bot rather than a masquerading browser.
pub async fn fetch_page_crawler(url: &str) -> Result<Fetched, String> {
    fetch_page_crawler_with_policy(url, ClientFetchPolicy::default()).await
}

pub async fn fetch_page_crawler_with_policy(
    url: &str,
    policy: ClientFetchPolicy,
) -> Result<Fetched, String> {
    policy.validate()?;
    preflight_read(url).map_err(|error| error.to_string())?;
    if is_smolweb(url) {
        return fetch_page_with_policy(url, policy).await;
    }
    let cx = session_stores()
        .context()
        .with_redirect_limit(policy.max_http_redirects);
    tokio::time::timeout(
        policy.timeout,
        do_fetch_ua_with_context(url, policy.max_page_bytes, Some(CRAWLER_USER_AGENT), &cx),
    )
    .await
    .map_err(|_| request_timeout_message(policy.timeout))?
}

mod cookies;
pub use cookies::*;

mod handle;
pub use handle::{
    Body, Facts, Fetch, FetchError, NetFetch, NoFallback, Range, RangeReply, Resources, Stores,
};

/// Drain a streaming [`netfetcher::ResponseBody`] into a buffer, aborting with an
/// error once the accumulated length would exceed `max_bytes` (§A5). Enforced *during*
/// the stream (not after `bytes()` buffers it all), so an unbounded / chunked body
/// cannot OOM the host: the read stops the moment the cap is crossed.
async fn read_capped(
    mut body: netfetcher::ResponseBody,
    max_bytes: usize,
) -> Result<Vec<u8>, String> {
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = body.next_chunk().await {
        let chunk = chunk.map_err(|e| format!("read error: {e}"))?;
        if buf.len().saturating_add(chunk.len()) > max_bytes {
            return Err(format!("response exceeds the {max_bytes}-byte cap"));
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

/// The crawl actor's descriptive User-Agent: a site can allow / block / reach the bot
/// from it (politeness, alongside robots.txt). The `merebot` token matches the
/// `User-agent:` groups `crawl::robots` honors; the `Mozilla/5.0 (compatible; …)`
/// shape is the conventional crawler form (cf. Googlebot).
pub const CRAWLER_USER_AGENT: &str =
    "Mozilla/5.0 (compatible; merebot/0.1; +https://mere.computer/bot)";

async fn do_fetch_ua_with_context(
    url: &str,
    max_bytes: usize,
    user_agent: Option<&str>,
    cx: &netfetcher::FetchContext,
) -> Result<Fetched, String> {
    do_fetch_ua_with_context_observed(url, max_bytes, user_agent, cx, |_, _, _| {}).await
}

/// HTTP page fetch with an optional complete-response observer. The callback
/// runs only after the capped body has been read successfully, and receives
/// netfetcher's actual final response URL rather than the requested address.
async fn do_fetch_ua_with_context_observed<F>(
    url: &str,
    max_bytes: usize,
    user_agent: Option<&str>,
    cx: &netfetcher::FetchContext,
    mut on_complete: F,
) -> Result<Fetched, String>
where
    F: FnMut(&url::Url, Option<&str>, &[u8]),
{
    let parsed = url::Url::parse(url).map_err(|e| format!("bad URL: {e}"))?;
    let mut request = netfetcher::Request::get(parsed);
    if let Some(ua) = user_agent {
        // netfetcher only injects its default UA when none is set, so this wins.
        request
            .headers
            .push(("user-agent".to_owned(), ua.to_owned()));
    }
    let response = netfetcher::fetch(request, cx).await;
    if response.is_network_error() {
        return Err("network error".to_string());
    }
    let status = response.status;
    if !(200..300).contains(&status) {
        return Err(format!("HTTP {status}"));
    }
    let content_type = response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.clone());
    let content_disposition = response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-disposition"))
        .map(|(_, v)| v.clone());
    let final_url = response.url_list.last().cloned();
    let bytes = read_capped(response.body, max_bytes).await?;
    let body = String::from_utf8_lossy(&bytes).into_owned();
    let fetched = Fetched {
        content_type,
        content_disposition,
        bytes,
        body,
    };
    if !page_load::is_download_response(&fetched)
        && let Some(final_url) = final_url.as_ref()
    {
        on_complete(final_url, fetched.content_type.as_deref(), &fetched.bytes);
    }
    Ok(fetched)
}

/// Fetch raw response bytes through the same scheme and trust routing as a page,
/// bounded by the caller's binary resource budget and whole deadline.
#[cfg(feature = "actor")]
async fn fetch_bytes(
    url: &str,
    stores: &Stores,
    policy: ClientFetchPolicy,
) -> Result<Vec<u8>, String> {
    policy.validate()?;
    preflight_read(url).map_err(|error| error.to_string())?;
    let operation = async {
        #[cfg(feature = "smolweb")]
        if is_smolweb(url) {
            let (_final_url, response) = smolweb_fetch_response(
                url,
                None,
                policy.max_subresource_bytes,
                policy,
                &mut |_, _, _| {},
            )
            .await
            .map_err(|error| error.to_string())?;
            if response.body.len() > policy.max_subresource_bytes {
                return Err(format!(
                    "response exceeds the {}-byte cap",
                    policy.max_subresource_bytes
                ));
            }
            return Ok(response.body);
        }

        let parsed = url::Url::parse(url).map_err(|error| format!("bad URL: {error}"))?;
        let cx = stores
            .context()
            .with_redirect_limit(policy.max_http_redirects);
        let response = netfetcher::fetch(netfetcher::Request::get(parsed), &cx).await;
        if response.is_network_error() || !(200..300).contains(&response.status) {
            return Err(format!("subresource request failed ({})", response.status));
        }
        read_capped(response.body, policy.max_subresource_bytes).await
    };
    tokio::time::timeout(policy.timeout, operation)
        .await
        .map_err(|_| request_timeout_message(policy.timeout))?
}

#[cfg(all(test, feature = "smolweb", feature = "actor"))]
mod tests;

#[cfg(all(
    test,
    feature = "smolweb",
    feature = "actor",
    not(target_arch = "wasm32")
))]
mod client_limits_tests;

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "http_progress_tests.rs"]
mod http_progress_tests;

#[cfg(all(feature = "https-blobs", not(target_arch = "wasm32")))]
pub mod https_blob;
#[cfg(all(feature = "iroh-blobs", not(target_arch = "wasm32")))]
pub mod iroh_blob;

pub mod crawler;

#[path = "submission.rs"]
mod submission;

#[cfg(all(feature = "smolweb", feature = "actor"))]
use submission::submit_smolweb;

#[cfg(all(feature = "smolweb", feature = "actor"))]
use submission::smolweb_submission_answer;

#[path = "page.rs"]
mod page;

pub use page::fetch_page;

pub use page::fetch_page_capped;

pub use page::fetch_page_with_policy;

use page::fetch_page_interactive_capped;

use page::request_timeout_message;

use page::preflight_read;

#[cfg(feature = "smolweb")]
use page::admit_smolweb_read;

#[cfg(feature = "smolweb")]
use page::url_without_query;

pub use page::fetch_page_anonymous_capped;

pub use page::fetch_page_anonymous_with_policy;

#[path = "smolweb_response.rs"]
mod smolweb_response;

#[cfg(feature = "smolweb")]
use smolweb_response::smolweb_fetch;

#[cfg(feature = "smolweb")]
use smolweb_response::smolweb_fetch_response;

#[cfg(feature = "smolweb")]
use smolweb_response::smolweb_transport_failure;

#[cfg(feature = "smolweb")]
use smolweb_response::smolweb_input_failure;

#[cfg(feature = "smolweb")]
use smolweb_response::smolweb_content_type;
