// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Fetch a page at the default page body cap ([`PAGE_BODY_CAP`]). The page-load and
/// crawl paths use this; the script `net.fetch` path calls [`fetch_page_capped`] with
/// a tighter ceiling. `pub(crate)` so the content actor reuses the routing.
pub async fn fetch_page(url: &str) -> Result<Fetched, String> {
    fetch_page_capped(url, PAGE_BODY_CAP).await
}

/// Fetch a page, routing by scheme: smolweb schemes go through [`errand`], every
/// other (http(s)) address through netfetcher's WHATWG Fetch. `max_bytes` caps the
/// body while acquiring it, including Errand's buffered protocol paths.
pub async fn fetch_page_capped(url: &str, max_bytes: usize) -> Result<Fetched, String> {
    fetch_page_with_policy(
        url,
        ClientFetchPolicy {
            max_page_bytes: max_bytes,
            ..ClientFetchPolicy::default()
        },
    )
    .await
}

/// Fetch with host-selected budgets, including one whole-operation deadline.
pub async fn fetch_page_with_policy(
    url: &str,
    policy: ClientFetchPolicy,
) -> Result<Fetched, String> {
    fetch_page_interactive_capped(
        url,
        policy.max_page_bytes,
        None,
        session_stores(),
        policy,
        |_, _, _| {},
    )
    .await
    .map_err(|error| error.to_string())
}

/// Fetch one page while preserving protocol responses that require host
/// participation. This is actor-facing: ordinary utility callers retain the
/// terminal `Result<Fetched, String>` contract above.
pub(super) async fn fetch_page_interactive_capped<F>(
    url: &str,
    max_bytes: usize,
    identity: Option<&GeminiClientIdentity>,
    stores: &Stores,
    policy: ClientFetchPolicy,
    mut on_progress: F,
) -> Result<Fetched, FetchFailure>
where
    F: FnMut(&url::Url, Option<&str>, &[u8]),
{
    policy.validate().map_err(FetchFailure::Failed)?;
    preflight_read(url)?;
    let operation = async {
        #[cfg(not(feature = "smolweb"))]
        let _ = (identity, &mut on_progress);
        #[cfg(feature = "smolweb")]
        if let Some(scheme) = scheme_of(url).and_then(errand::Scheme::parse) {
            {
                let log_url = url_without_query(url);
                tracing::info!(url = %log_url, ?scheme, "smolweb fetch");
                let mut streamed_bytes = 0_usize;
                let result = smolweb_fetch(
                    url,
                    identity,
                    max_bytes,
                    policy,
                    |response_url, content_type, bytes| {
                        streamed_bytes = streamed_bytes.saturating_add(bytes.len());
                        if streamed_bytes <= max_bytes {
                            on_progress(response_url, content_type, bytes);
                        }
                    },
                )
                .await
                .and_then(|fetched| {
                    if fetched.bytes.len() > max_bytes {
                        Err(FetchFailure::Failed(format!(
                            "response exceeds the {max_bytes}-byte cap"
                        )))
                    } else {
                        Ok(fetched)
                    }
                });
                match &result {
                    Ok(fetched) => tracing::info!(
                        url = %log_url,
                        content_type = ?fetched.content_type,
                        bytes = fetched.bytes.len(),
                        "smolweb ok",
                    ),
                    Err(error) => tracing::warn!(url = %log_url, %error, "smolweb failed"),
                }
                return result;
            }
        }
        do_fetch_ua_with_context_observed(
            url,
            max_bytes,
            None,
            &stores
                .context()
                .with_redirect_limit(policy.max_http_redirects),
            |response_url, content_type, bytes| {
                on_progress(response_url, content_type, bytes);
            },
        )
        .await
        .map_err(FetchFailure::Failed)
    };
    tokio::time::timeout(policy.timeout, operation)
        .await
        .map_err(|_| FetchFailure::Failed(request_timeout_message(policy.timeout)))?
}

pub(super) fn request_timeout_message(timeout: Duration) -> String {
    format!("request timed out after {}ms", timeout.as_millis())
}

pub(super) fn preflight_read(url: &str) -> Result<(), FetchFailure> {
    let parsed =
        url::Url::parse(url).map_err(|error| FetchFailure::Failed(format!("bad URL: {error}")))?;
    #[cfg(feature = "smolweb")]
    if is_smolweb(url) {
        if let Err(error) = admit_smolweb_read(&parsed) {
            let log_url = url_without_query(url);
            tracing::info!(url = %log_url, scheme = %parsed.scheme(), "smolweb fetch");
            tracing::warn!(url = %log_url, %error, "smolweb failed");
            return Err(error);
        }
        return Ok(());
    }
    if matches!(parsed.scheme(), "http" | "https") {
        Ok(())
    } else {
        Err(FetchFailure::Failed(format!(
            "unsupported fetch scheme: {}",
            parsed.scheme()
        )))
    }
}

#[cfg(feature = "smolweb")]
pub(super) fn admit_smolweb_read(current: &url::Url) -> Result<(), FetchFailure> {
    match current.scheme() {
        "titan" => Err(FetchFailure::Failed(
            "Titan uploads require explicit Submit".into(),
        )),
        "spartan" if current.query().is_some() => Err(FetchFailure::Failed(
            "Spartan submissions require explicit Submit".into(),
        )),
        _ => Ok(()),
    }
}

#[cfg(feature = "smolweb")]
pub(super) fn url_without_query(raw: &str) -> String {
    url::Url::parse(raw)
        .map(|mut parsed| {
            // Diagnostic copy only: never retain URL credentials or Titan's
            // upload parameters in a read-attempt trace.
            let _ = parsed.set_password(None);
            let _ = parsed.set_username("");
            if parsed.scheme() == "titan"
                && let Some((path, _)) = parsed.path().split_once(';')
            {
                let path = path.to_string();
                parsed.set_path(&path);
            }
            parsed.set_query(None);
            parsed.set_fragment(None);
            parsed.to_string()
        })
        .unwrap_or_else(|_| "<invalid-url>".to_string())
}

/// Fetch a page without the browser session's cookie jar or other installed
/// authenticated HTTP state. Effect providers use this path so resolving a
/// transclusion cannot silently borrow the user's browsing authority.
///
/// Smolweb requests use errand's process-level trust store. A host that admits
/// Gemini should install one explicitly with
/// [`install_in_memory_smolweb_tofu`] or its own durable store.
pub async fn fetch_page_anonymous_capped(url: &str, max_bytes: usize) -> Result<Fetched, String> {
    fetch_page_anonymous_with_policy(
        url,
        ClientFetchPolicy {
            max_page_bytes: max_bytes,
            ..ClientFetchPolicy::default()
        },
    )
    .await
}

pub async fn fetch_page_anonymous_with_policy(
    url: &str,
    policy: ClientFetchPolicy,
) -> Result<Fetched, String> {
    policy.validate()?;
    preflight_read(url).map_err(|error| error.to_string())?;
    if is_smolweb(url) {
        return fetch_page_with_policy(url, policy).await;
    }
    let cx = netfetcher::FetchContext::permissive().with_redirect_limit(policy.max_http_redirects);
    tokio::time::timeout(
        policy.timeout,
        do_fetch_ua_with_context(url, policy.max_page_bytes, None, &cx),
    )
    .await
    .map_err(|_| request_timeout_message(policy.timeout))?
}
