// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::time::Duration;

/// Caller-owned acquisition budgets. Each operation takes one snapshot; its
/// timeout covers the complete redirect chain and body, rather than each hop.
/// Bodies which exceed a cap fail instead of returning a successful prefix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientFetchPolicy {
    pub max_page_bytes: usize,
    pub max_subresource_bytes: usize,
    pub max_submission_bytes: usize,
    pub max_submission_response_bytes: usize,
    /// Smolweb request line/header allowance, excluding an explicitly submitted
    /// body. This does not constrain netfetcher's HTTP request headers.
    pub max_request_header_bytes: usize,
    /// Smolweb response line allowance. Native protocol ceilings still apply.
    pub max_response_header_line_bytes: usize,
    /// Smolweb redirects after the initial request; zero permits one request.
    pub max_redirects: u32,
    /// HTTP redirects, preserving netfetcher's existing default separately.
    pub max_http_redirects: u32,
    pub timeout: Duration,
}

impl Default for ClientFetchPolicy {
    fn default() -> Self {
        Self {
            max_page_bytes: super::PAGE_BODY_CAP,
            max_subresource_bytes: 32 * 1024 * 1024,
            max_submission_bytes: super::PAGE_BODY_CAP,
            max_submission_response_bytes: super::PAGE_BODY_CAP,
            max_request_header_bytes: 4096,
            max_response_header_line_bytes: 4096,
            // The old five-iteration loop admitted five requests, not six.
            max_redirects: 4,
            max_http_redirects: 20,
            timeout: Duration::from_secs(30),
        }
    }
}

impl ClientFetchPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.timeout.is_zero()
            || self.max_page_bytes == 0
            || self.max_subresource_bytes == 0
            || self.max_submission_bytes == 0
            || self.max_submission_response_bytes == 0
            || self.max_request_header_bytes == 0
            || self.max_response_header_line_bytes == 0
        {
            return Err("client fetch byte limits and timeout must be positive".into());
        }
        Ok(())
    }

    #[cfg(feature = "smolweb")]
    pub(crate) fn smolweb_options(
        self,
        max_body_bytes: usize,
        timeout: Duration,
        submitted_body_bytes: usize,
    ) -> Result<errand::FetchOptions, String> {
        let max_request_bytes = self
            .max_request_header_bytes
            .checked_add(submitted_body_bytes)
            .ok_or_else(|| "client request byte limit overflow".to_string())?;
        Ok(errand::FetchOptions {
            timeout,
            max_request_bytes,
            max_header_line_bytes: self.max_response_header_line_bytes,
            max_body_bytes,
        })
    }
}
