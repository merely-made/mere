// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One explicitly admitted crawler request, with a streaming byte cap.
//!
//! The transport receives one GET. Redirects and HTTP failures remain observed
//! statuses for the caller to admit or refuse. No browser session, cookie jar,
//! cache, HTTP/3 fallback, automatic retry, or redirect traversal participates.

use futures_util::StreamExt;
use netfetcher::{Method, Transport, WireRequest};
use url::Url;

use crate::CRAWLER_USER_AGENT;

/// The observed status and bounded successful response from one request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrawlerResponse {
    pub status: u16,
    pub content_type: Option<String>,
    pub content_disposition: Option<String>,
    /// Successful response bytes. Non-success bodies are dropped without read.
    pub bytes: Vec<u8>,
}

/// Acquisition failures retain bytes already received for the caller's budget.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CrawlerFetchError {
    InvalidUrl(String),
    UnsupportedScheme(String),
    CredentialsInUrl,
    Network {
        message: String,
        received_bytes: usize,
    },
    ResponseLimit {
        max_bytes: usize,
        received_bytes: usize,
    },
    UnsupportedEncoding(String),
}

impl CrawlerFetchError {
    /// Bytes seen before failure, including a chunk which crossed the cap.
    pub fn received_bytes(&self) -> usize {
        match self {
            Self::Network { received_bytes, .. } | Self::ResponseLimit { received_bytes, .. } => {
                *received_bytes
            },
            _ => 0,
        }
    }
}

impl std::fmt::Display for CrawlerFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl(error) => write!(f, "bad crawler URL: {error}"),
            Self::UnsupportedScheme(scheme) => write!(f, "unsupported crawler scheme: {scheme}"),
            Self::CredentialsInUrl => f.write_str("crawler URL contains credentials"),
            Self::Network { message, .. } => write!(f, "crawler network error: {message}"),
            Self::ResponseLimit { max_bytes, .. } => {
                write!(f, "crawler response exceeds the {max_bytes}-byte cap")
            },
            Self::UnsupportedEncoding(encoding) => {
                write!(
                    f,
                    "crawler requested identity encoding; received {encoding}"
                )
            },
        }
    }
}

impl std::error::Error for CrawlerFetchError {}

/// Acquire one HTTP(S) response with the native retry-free crawler transport.
///
/// Only identity-encoded successful bodies are read. HTTP statuses (including
/// 404/410 for an absent robots file) remain distinct from acquisition failures.
/// The cap bounds accepted response bytes; an incoming transport chunk may cross
/// it, in which case its observed length is reported and the stream is dropped.
pub async fn fetch_crawler_capped(
    url: &str,
    max_bytes: usize,
) -> Result<CrawlerResponse, CrawlerFetchError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let transport = native_transport::CrawlerTransport::shared()?;
        fetch_crawler_capped_with_transport(url, max_bytes, transport).await
    }
    #[cfg(target_arch = "wasm32")]
    {
        fetch_crawler_capped_with_transport(url, max_bytes, &netfetcher::NoTransport).await
    }
}

/// Acquire one response using a host-supplied single-hop transport.
///
/// A host must supply a transport that honors [`Transport`]'s one-hop contract.
/// Admission, robots policy, request counting and total-byte accounting belong
/// to the caller; this function enforces this response's cap while reading it.
pub async fn fetch_crawler_capped_with_transport(
    url: &str,
    max_bytes: usize,
    transport: &dyn Transport,
) -> Result<CrawlerResponse, CrawlerFetchError> {
    let url = Url::parse(url).map_err(|error| CrawlerFetchError::InvalidUrl(error.to_string()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(CrawlerFetchError::UnsupportedScheme(
            url.scheme().to_string(),
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(CrawlerFetchError::CredentialsInUrl);
    }
    if max_bytes == 0 {
        return Err(CrawlerFetchError::ResponseLimit {
            max_bytes,
            received_bytes: 0,
        });
    }
    let request = WireRequest {
        url,
        method: Method::Get,
        headers: vec![
            ("user-agent".to_string(), CRAWLER_USER_AGENT.to_string()),
            ("accept-encoding".to_string(), "identity".to_string()),
        ],
        body: None,
        prefer_h3: false,
    };
    let mut response = transport
        .send(request)
        .await
        .ok_or_else(|| CrawlerFetchError::Network {
            message: "request failed".to_string(),
            received_bytes: 0,
        })?;
    let content_type = header(&response.headers, "content-type").map(str::to_string);
    let content_disposition = header(&response.headers, "content-disposition").map(str::to_string);
    if !(200..300).contains(&response.status) {
        return Ok(CrawlerResponse {
            status: response.status,
            content_type,
            content_disposition,
            bytes: Vec::new(),
        });
    }
    if let Some(encoding) = header(&response.headers, "content-encoding") {
        if !encoding.trim().eq_ignore_ascii_case("identity") {
            return Err(CrawlerFetchError::UnsupportedEncoding(encoding.to_string()));
        }
    }
    if let Some(length) =
        header(&response.headers, "content-length").and_then(|length| length.parse::<u64>().ok())
    {
        if length > max_bytes as u64 {
            return Err(CrawlerFetchError::ResponseLimit {
                max_bytes,
                received_bytes: 0,
            });
        }
    }
    let mut bytes = Vec::new();
    let mut received_bytes = 0usize;
    while let Some(chunk) = response.body.next().await {
        let chunk = chunk.map_err(|error| CrawlerFetchError::Network {
            message: error.to_string(),
            received_bytes,
        })?;
        received_bytes = received_bytes.saturating_add(chunk.len());
        if chunk.len() > max_bytes.saturating_sub(bytes.len()) {
            return Err(CrawlerFetchError::ResponseLimit {
                max_bytes,
                received_bytes,
            });
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(CrawlerResponse {
        status: response.status,
        content_type,
        content_disposition,
        bytes,
    })
}

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

#[cfg(not(target_arch = "wasm32"))]
mod native_transport {
    use bytes::Bytes;
    use futures_util::TryStreamExt;
    use http_body_util::{BodyExt, Full};
    use hyper_rustls::HttpsConnector;
    use hyper_util::client::legacy::Client;
    use hyper_util::client::legacy::connect::HttpConnector;
    use hyper_util::rt::TokioExecutor;
    use netfetcher::{RawResponse, Transport, TransportFuture, WireRequest};
    use std::sync::OnceLock;

    use super::CrawlerFetchError;

    /// A separate pool with canceled-request retries explicitly disabled.
    /// TLS uses an explicit ring provider and Mozilla roots, independent of
    /// Netfetcher's global test-only insecure-certificate switch.
    pub(super) struct CrawlerTransport {
        client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    }

    impl CrawlerTransport {
        pub(super) fn shared() -> Result<&'static Self, CrawlerFetchError> {
            static TRANSPORT: OnceLock<Result<CrawlerTransport, String>> = OnceLock::new();
            TRANSPORT
                .get_or_init(Self::new)
                .as_ref()
                .map_err(|message| CrawlerFetchError::Network {
                    message: message.clone(),
                    received_bytes: 0,
                })
        }

        fn new() -> Result<Self, String> {
            let connector = hyper_rustls::HttpsConnectorBuilder::new()
                .with_provider_and_webpki_roots(rustls::crypto::ring::default_provider())
                .map_err(|error| format!("crawler TLS configuration: {error}"))?
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build();
            let client = Client::builder(TokioExecutor::new())
                // Hyper's default retries an unstarted request canceled while
                // reusing a pooled connection. Every crawler attempt has its
                // own admission, so retries must return to the caller instead.
                .retry_canceled_requests(false)
                .build(connector);
            Ok(Self { client })
        }
    }

    impl Transport for CrawlerTransport {
        fn send(&self, request: WireRequest) -> TransportFuture<'_> {
            Box::pin(async move {
                let uri = http::Uri::try_from(request.url.as_str()).ok()?;
                let mut builder = http::Request::builder()
                    .method(request.method.as_str())
                    .uri(uri);
                for (name, value) in request.headers {
                    builder = builder.header(name, value);
                }
                let request = builder
                    .body(Full::new(request.body.unwrap_or_default()))
                    .ok()?;
                let response = self.client.request(request).await.ok()?;
                let status = response.status().as_u16();
                let headers = response
                    .headers()
                    .iter()
                    .filter_map(|(name, value)| {
                        value
                            .to_str()
                            .ok()
                            .map(|value| (name.as_str().to_string(), value.to_string()))
                    })
                    .collect();
                let body = response
                    .into_body()
                    .into_data_stream()
                    .map_err(std::io::Error::other);
                Some(RawResponse {
                    status,
                    headers,
                    body: Box::pin(body),
                })
            })
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use bytes::Bytes;
    use netfetcher::{RawResponse, TransportFuture};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn local_server_sees_one_bot_request_without_cookies() {
        let mut server = mockito::Server::new_async().await;
        let request = server
            .mock("GET", "/body")
            .match_header("user-agent", CRAWLER_USER_AGENT)
            .match_header("accept-encoding", "identity")
            .match_header("cookie", mockito::Matcher::Missing)
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("bounded")
            .expect(1)
            .create_async()
            .await;
        let response = fetch_crawler_capped(&format!("{}/body", server.url()), 7)
            .await
            .unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.bytes, b"bounded");
        request.assert_async().await;
    }

    #[tokio::test]
    async fn local_redirect_is_observed_without_following() {
        let mut server = mockito::Server::new_async().await;
        let first = server
            .mock("GET", "/redirect")
            .with_status(302)
            .with_header("location", "/target")
            .expect(1)
            .create_async()
            .await;
        let target = server
            .mock("GET", "/target")
            .with_status(200)
            .expect(0)
            .create_async()
            .await;
        let response = fetch_crawler_capped(&format!("{}/redirect", server.url()), 8)
            .await
            .unwrap();
        assert_eq!(response.status, 302);
        assert!(response.bytes.is_empty());
        first.assert_async().await;
        target.assert_async().await;
    }

    #[tokio::test]
    async fn local_response_length_is_refused_before_reading() {
        let mut server = mockito::Server::new_async().await;
        let request = server
            .mock("GET", "/large")
            .with_status(200)
            .with_body("over-budget")
            .expect(1)
            .create_async()
            .await;
        let error = fetch_crawler_capped(&format!("{}/large", server.url()), 4)
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            CrawlerFetchError::ResponseLimit { max_bytes: 4, .. }
        ));
        request.assert_async().await;
    }

    struct StreamingTransport {
        calls: AtomicUsize,
        polls: Arc<AtomicUsize>,
        chunks: Mutex<Option<Vec<std::io::Result<Bytes>>>>,
        status: u16,
    }

    impl Transport for StreamingTransport {
        fn send(&self, request: WireRequest) -> TransportFuture<'_> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            assert!(!request.prefer_h3);
            let chunks = self.chunks.lock().unwrap().take().unwrap();
            let polls = self.polls.clone();
            let status = self.status;
            Box::pin(async move {
                Some(RawResponse {
                    status,
                    headers: Vec::new(),
                    body: Box::pin(futures_util::stream::iter(chunks).inspect(move |_| {
                        polls.fetch_add(1, Ordering::Relaxed);
                    })),
                })
            })
        }
    }

    fn streaming_transport(chunks: Vec<std::io::Result<Bytes>>, status: u16) -> StreamingTransport {
        StreamingTransport {
            calls: AtomicUsize::new(0),
            polls: Arc::new(AtomicUsize::new(0)),
            chunks: Mutex::new(Some(chunks)),
            status,
        }
    }

    #[tokio::test]
    async fn chunked_response_stops_at_cap_and_accounts_for_received_chunk() {
        let transport = streaming_transport(
            vec![
                Ok(Bytes::from_static(b"ab")),
                Ok(Bytes::from_static(b"cd")),
                Ok(Bytes::from_static(b"unread")),
            ],
            200,
        );
        let error = fetch_crawler_capped_with_transport("https://example.test/", 3, &transport)
            .await
            .unwrap_err();
        assert_eq!(
            error,
            CrawlerFetchError::ResponseLimit {
                max_bytes: 3,
                received_bytes: 4,
            }
        );
        assert_eq!(transport.calls.load(Ordering::Relaxed), 1);
        assert_eq!(transport.polls.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn exhausted_byte_budget_issues_no_request() {
        let transport = streaming_transport(vec![Ok(Bytes::from_static(b"unread"))], 200);
        let error = fetch_crawler_capped_with_transport("https://example.test/", 0, &transport)
            .await
            .unwrap_err();
        assert_eq!(
            error,
            CrawlerFetchError::ResponseLimit {
                max_bytes: 0,
                received_bytes: 0,
            }
        );
        assert_eq!(transport.calls.load(Ordering::Relaxed), 0);
        assert_eq!(transport.polls.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn failed_stream_keeps_partial_bytes_for_budget_accounting() {
        let transport = streaming_transport(
            vec![
                Ok(Bytes::from_static(b"ab")),
                Err(std::io::Error::other("connection closed")),
                Ok(Bytes::from_static(b"unread")),
            ],
            200,
        );
        let error = fetch_crawler_capped_with_transport("https://example.test/", 8, &transport)
            .await
            .unwrap_err();
        assert_eq!(error.received_bytes(), 2);
        assert!(matches!(error, CrawlerFetchError::Network { .. }));
        assert_eq!(transport.polls.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn missing_robots_and_denied_robots_remain_distinct_without_body_reads() {
        for status in [404, 410, 401, 403] {
            let transport = streaming_transport(vec![Ok(Bytes::from_static(b"unread"))], status);
            let response = fetch_crawler_capped_with_transport(
                "https://example.test/robots.txt",
                3,
                &transport,
            )
            .await
            .unwrap();
            assert_eq!(response.status, status);
            assert!(response.bytes.is_empty());
            assert_eq!(transport.polls.load(Ordering::Relaxed), 0);
        }
    }
}
