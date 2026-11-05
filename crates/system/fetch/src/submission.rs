// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

// A write is only ever asked for through the actor.
#[cfg(all(feature = "smolweb", feature = "actor"))]
pub(super) async fn submit_smolweb(
    submission: SmolwebSubmission,
    policy: ClientFetchPolicy,
) -> Result<SubmissionAnswer, FetchFailure> {
    policy.validate().map_err(FetchFailure::Failed)?;
    if match &submission {
        SmolwebSubmission::Titan { body, .. } | SmolwebSubmission::Spartan { body, .. } => {
            body.len() > policy.max_submission_bytes
        },
    } {
        return Err(FetchFailure::Failed(format!(
            "submission exceeds the {}-byte cap",
            policy.max_submission_bytes
        )));
    }

    let (url, response) = match submission {
        SmolwebSubmission::Titan {
            url,
            body,
            mime,
            token,
            identity,
        } => {
            let options = policy
                .smolweb_options(
                    policy.max_submission_response_bytes,
                    policy.timeout,
                    body.len(),
                )
                .map_err(FetchFailure::Failed)?;
            let parsed = url::Url::parse(&url)
                .map_err(|error| FetchFailure::Failed(format!("bad URL: {error}")))?;
            if parsed.scheme() != "titan" {
                return Err(FetchFailure::Failed(
                    "Titan submission requires a titan:// URL".to_string(),
                ));
            }
            let exchange = async {
                match identity
                    .as_ref()
                    .filter(|identity| identity.applies_to(&parsed))
                {
                    Some(identity) => {
                        errand::titan_upload_with_identity_and_options(
                            &parsed,
                            &body,
                            &mime,
                            token.as_deref(),
                            identity.errand_view(),
                            &options,
                        )
                        .await
                    },
                    None => {
                        errand::titan_upload_with_options(
                            &parsed,
                            &body,
                            &mime,
                            token.as_deref(),
                            &options,
                        )
                        .await
                    },
                }
            };
            let response = tokio::time::timeout(policy.timeout, exchange)
                .await
                .map_err(|_| FetchFailure::Failed("submission timed out".to_string()))?
                .map_err(|error| smolweb_transport_failure(&parsed, error))?;
            (parsed, response)
        },
        SmolwebSubmission::Spartan { url, body } => {
            let options = policy
                .smolweb_options(
                    policy.max_submission_response_bytes,
                    policy.timeout,
                    body.len(),
                )
                .map_err(FetchFailure::Failed)?;
            let parsed = url::Url::parse(&url)
                .map_err(|error| FetchFailure::Failed(format!("bad URL: {error}")))?;
            if parsed.scheme() != "spartan" {
                return Err(FetchFailure::Failed(
                    "Spartan submission requires a spartan:// URL".to_string(),
                ));
            }
            let response = tokio::time::timeout(
                policy.timeout,
                errand::spartan_submit_with_options(&parsed, &body, &options),
            )
            .await
            .map_err(|_| FetchFailure::Failed("submission timed out".to_string()))?
            .map_err(|error| smolweb_transport_failure(&parsed, error))?;
            (parsed, response)
        },
    };
    smolweb_submission_answer(&url, response)
}

#[cfg(all(feature = "smolweb", feature = "actor"))]
pub(super) fn smolweb_submission_answer(
    request_url: &url::Url,
    response: errand::Response,
) -> Result<SubmissionAnswer, FetchFailure> {
    match response.status {
        errand::Status::Success => {
            let content_type = smolweb_content_type(request_url, &response);
            let bytes = response.body;
            let body = String::from_utf8_lossy(&bytes).into_owned();
            Ok(SubmissionAnswer::Success(Fetched {
                content_type: Some(content_type),
                content_disposition: None,
                bytes,
                body,
            }))
        },
        errand::Status::Redirect => {
            let target = request_url
                .join(&response.meta)
                .map_err(|error| FetchFailure::Failed(format!("bad redirect target: {error}")))?;
            if request_url.scheme() == "spartan"
                && (target.scheme() != "spartan"
                    || target.host_str() != request_url.host_str()
                    || target.port_or_known_default() != request_url.port_or_known_default())
            {
                return Err(FetchFailure::Failed(
                    "Spartan redirect crossed its request origin".to_string(),
                ));
            }
            Ok(SubmissionAnswer::Redirect(target.to_string()))
        },
        errand::Status::Input => Err(smolweb_input_failure(request_url, &response)),
        errand::Status::CertRequired => Err(FetchFailure::ClientCertificateRequired {
            url: request_url.to_string(),
            prompt: response.meta,
            code: response.raw_status,
        }),
        errand::Status::Failure => Err(FetchFailure::Failed(if response.meta.is_empty() {
            "submission failed".to_string()
        } else {
            response.meta
        })),
    }
}
