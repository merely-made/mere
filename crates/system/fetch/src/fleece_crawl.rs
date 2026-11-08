// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Fleece's crawl seam over this crate's single-hop crawler transport.
//!
//! Fleece owns the generic gathering runtime; the transport is the host's. This
//! adapter (moved from `mere-fleece` under the engine split plan, P1) lowers
//! [`crate::crawler`] results onto Fleece's [`CrawlFetchError`] receipts.

use fleece::crawl::{CrawlFetchError, CrawlRequest};

use crate::Fetched;

/// Mere's concrete single-hop HTTP adapter for the supplied request seam. It
/// reads within the response cap without ambient browser cookies, redirects or
/// retries. Hosts with their own transports can supply their own adapter.
pub async fn fetch_crawler_request(request: CrawlRequest) -> Result<Fetched, CrawlFetchError> {
    let result = crate::crawler::fetch_crawler_capped(&request.url, request.max_bytes).await;
    lower_crawler_result(result)
}

fn lower_crawler_result(
    result: Result<crate::crawler::CrawlerResponse, crate::crawler::CrawlerFetchError>,
) -> Result<Fetched, CrawlFetchError> {
    let response = result.map_err(|error| match &error {
        crate::crawler::CrawlerFetchError::ResponseLimit { received_bytes, .. } => {
            CrawlFetchError::ResponseLimit {
                received_bytes: *received_bytes,
            }
        },
        _ => CrawlFetchError::Failed {
            received_bytes: error.received_bytes(),
            message: error.to_string(),
        },
    })?;
    match response.status {
        200..=299 => Ok(Fetched {
            content_type: response.content_type,
            content_disposition: response.content_disposition,
            body: String::from_utf8_lossy(&response.bytes).into_owned(),
            bytes: response.bytes,
        }),
        404 | 410 => Err(CrawlFetchError::NotFound {
            received_bytes: response.bytes.len(),
        }),
        status => Err(CrawlFetchError::Failed {
            message: format!("HTTP {status}"),
            received_bytes: response.bytes.len(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_hop_adapter_preserves_absence_failure_status_and_partial_byte_evidence() {
        let response = |status| crate::crawler::CrawlerResponse {
            status,
            content_type: Some("text/plain".into()),
            content_disposition: Some("inline".into()),
            bytes: b"body".to_vec(),
        };
        let success = lower_crawler_result(Ok(response(200))).unwrap();
        assert_eq!(success.body, "body");
        assert_eq!(success.bytes, b"body");
        assert_eq!(success.content_type.as_deref(), Some("text/plain"));
        assert_eq!(success.content_disposition.as_deref(), Some("inline"));
        for status in [404, 410] {
            assert_eq!(
                lower_crawler_result(Ok(response(status))),
                Err(CrawlFetchError::NotFound { received_bytes: 4 })
            );
        }
        for status in [302, 401, 403, 500] {
            assert!(matches!(
                lower_crawler_result(Ok(response(status))),
                Err(CrawlFetchError::Failed {
                    received_bytes: 4,
                    ..
                })
            ));
        }
        assert_eq!(
            lower_crawler_result(Err(crate::crawler::CrawlerFetchError::ResponseLimit {
                max_bytes: 5,
                received_bytes: 7,
            })),
            Err(CrawlFetchError::ResponseLimit { received_bytes: 7 })
        );
        assert_eq!(
            lower_crawler_result(Err(crate::crawler::CrawlerFetchError::Network {
                message: "reset".into(),
                received_bytes: 3,
            }))
            .unwrap_err()
            .received_bytes(),
            3
        );
    }
}
