// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Fetch a smolweb URL through [`errand`], following redirects up to
/// the caller's redirect budget, and fold the response into a [`Fetched`] the nematic engines
/// render. Input and certificate statuses stay typed so the host can continue
/// the protocol conversation; terminal failures remain displayable prose.
#[cfg(feature = "smolweb")]
pub(super) async fn smolweb_fetch<F>(
    url: &str,
    identity: Option<&GeminiClientIdentity>,
    max_bytes: usize,
    policy: ClientFetchPolicy,
    mut on_progress: F,
) -> Result<Fetched, FetchFailure>
where
    F: FnMut(&url::Url, Option<&str>, &[u8]),
{
    let (current, response) =
        smolweb_fetch_response(url, identity, max_bytes, policy, &mut on_progress).await?;
    let content_type = smolweb_content_type(&current, &response);
    let bytes = response.body;
    let body = String::from_utf8_lossy(&bytes).into_owned();
    Ok(Fetched {
        content_type: Some(content_type),
        content_disposition: None,
        bytes,
        body,
    })
}

/// Fetch a successful smolweb response without decoding its body. Page loads
/// and binary subresources share redirect, timeout, certificate, and protocol
/// status handling through this one path.
/// Titan targets and Spartan URLs with any query component (including an empty
/// `?`) require explicit Submit. Reads refuse these before transport at every
/// redirect hop, rather than sending an upload through ordinary acquisition.
#[cfg(feature = "smolweb")]
pub(super) async fn smolweb_fetch_response<F>(
    url: &str,
    identity: Option<&GeminiClientIdentity>,
    max_bytes: usize,
    policy: ClientFetchPolicy,
    on_progress: &mut F,
) -> Result<(url::Url, errand::Response), FetchFailure>
where
    F: FnMut(&url::Url, Option<&str>, &[u8]),
{
    let mut current =
        url::Url::parse(url).map_err(|error| FetchFailure::Failed(format!("bad URL: {error}")))?;
    let deadline = tokio::time::Instant::now()
        .checked_add(policy.timeout)
        .ok_or_else(|| FetchFailure::Failed("client request deadline overflow".into()))?;
    for _ in 0..=policy.max_redirects {
        // These exchanges have submission semantics even with an empty body.
        // Check every redirect before transport, so read acquisition never
        // borrows the authority of the separate explicit Submit command.
        admit_smolweb_read(&current)?;
        let response_url = current.clone();
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return Err(FetchFailure::Failed(request_timeout_message(
                policy.timeout,
            )));
        }
        let options = policy
            .smolweb_options(max_bytes, remaining, 0)
            .map_err(FetchFailure::Failed)?;
        let response = if current.scheme() == "gemini" {
            match identity.filter(|identity| identity.applies_to(&current)) {
                Some(identity) => {
                    errand::fetch_gemini_url_streaming_with_identity_and_options(
                        &current,
                        Some(identity.errand_view()),
                        &options,
                        |chunk| on_progress(&response_url, chunk.content_type, chunk.bytes),
                    )
                    .await
                },
                None => {
                    errand::fetch_gemini_url_streaming_with_options(&current, &options, |chunk| {
                        on_progress(&response_url, chunk.content_type, chunk.bytes)
                    })
                    .await
                },
            }
        } else {
            match identity.filter(|identity| identity.applies_to(&current)) {
                Some(identity) => {
                    errand::fetch_url_with_identity_and_options(
                        &current,
                        Some(identity.errand_view()),
                        &options,
                    )
                    .await
                },
                None => errand::fetch_url_with_options(&current, &options).await,
            }
        }
        .map_err(|error| smolweb_transport_failure(&current, error))?;
        match response.status {
            errand::Status::Success => {
                return Ok((current, response));
            },
            errand::Status::Redirect => {
                current = current.join(&response.meta).map_err(|error| {
                    FetchFailure::Failed(format!("bad redirect target: {error}"))
                })?;
            },
            errand::Status::Input => {
                return Err(smolweb_input_failure(&current, &response));
            },
            errand::Status::CertRequired => {
                return Err(FetchFailure::ClientCertificateRequired {
                    url: current.to_string(),
                    prompt: response.meta,
                    code: response.raw_status,
                });
            },
            errand::Status::Failure => {
                return Err(FetchFailure::Failed(if response.meta.is_empty() {
                    "request failed".to_string()
                } else {
                    response.meta
                }));
            },
        }
    }
    Err(FetchFailure::Failed("too many redirects".to_string()))
}

#[cfg(feature = "smolweb")]
pub(super) fn smolweb_transport_failure(current: &url::Url, error: errand::Error) -> FetchFailure {
    match error {
        errand::Error::CertificateChanged { host, pinned, seen } => {
            FetchFailure::CertificateChanged {
                url: current.to_string(),
                target: host,
                pinned,
                seen,
            }
        },
        error => FetchFailure::Failed(error.to_string()),
    }
}

#[cfg(feature = "smolweb")]
pub(super) fn smolweb_input_failure(
    current: &url::Url,
    response: &errand::Response,
) -> FetchFailure {
    FetchFailure::InputRequired {
        url: current.to_string(),
        prompt: response.meta.clone(),
        // Gemini status 11 is the sensitive-input form. Other protocols
        // currently expose only an ordinary input code.
        sensitive: response.raw_status == Some(11),
    }
}

/// The content-type to render a smolweb response under, in nematic's vocabulary.
/// Most schemes carry their own media type (gemini/spartan `text/gemini`, a gopher
/// menu `application/gopher-menu`, a gopher text file `text/plain`); finger has no
/// type of its own, so it is tagged `text/x-finger` to reach the finger engine.
#[cfg(feature = "smolweb")]
pub(super) fn smolweb_content_type(url: &url::Url, response: &errand::Response) -> String {
    match errand::Scheme::parse(url.scheme()) {
        // Protocols whose content type must be fixed regardless of the response
        // meta field, so the host routes to the correct nematic engine.
        Some(errand::Scheme::Finger) => "text/x-finger".to_string(),
        Some(errand::Scheme::Nex) => "application/x-nex".to_string(),
        Some(errand::Scheme::Guppy) => "application/x-guppy".to_string(),
        Some(errand::Scheme::Titan) => "application/x-titan".to_string(),
        // MIME routing strips parameters only when it compares types. Keep the
        // complete success value here so response language/charset survive the
        // host's existing content_type seam.
        _ => response
            .mime()
            .map_or_else(|| "text/gemini".to_string(), |_| response.meta.clone()),
    }
}
