// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::do_fetch_ua_with_context_observed;

#[tokio::test]
async fn http_redirect_reports_final_url_and_complete_body_before_return() {
    let mut server = mockito::Server::new_async().await;
    let redirect = server
        .mock("GET", "/start")
        .with_status(302)
        .with_header("location", "/final")
        .expect(1)
        .create_async()
        .await;
    let final_response = server
        .mock("GET", "/final")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body("redirected page")
        .expect(1)
        .create_async()
        .await;
    let context = netfetcher::FetchContext::permissive();
    let mut events = Vec::new();
    let mut reported = Vec::new();
    let fetched = do_fetch_ua_with_context_observed(
        &format!("{}/start", server.url()),
        64,
        None,
        &context,
        |response_url, content_type, bytes| {
            events.push("complete-body");
            reported.push((
                response_url.to_string(),
                content_type.map(str::to_owned),
                bytes.to_vec(),
            ));
        },
    )
    .await
    .expect("redirected HTTP page fetch");

    assert_eq!(fetched.bytes, b"redirected page");
    assert_eq!(
        reported,
        vec![(
            format!("{}/final", server.url()),
            Some("text/html; charset=utf-8".to_owned()),
            b"redirected page".to_vec(),
        )]
    );
    events.push("terminal-outcome");
    assert_eq!(events, ["complete-body", "terminal-outcome"]);
    redirect.assert_async().await;
    final_response.assert_async().await;
}

#[tokio::test]
async fn http_body_cap_failure_does_not_report_complete_source() {
    let mut server = mockito::Server::new_async().await;
    let response = server
        .mock("GET", "/large")
        .with_status(200)
        .with_body("larger than cap")
        .expect(1)
        .create_async()
        .await;
    let context = netfetcher::FetchContext::permissive();
    let mut progress = Vec::new();
    let result = do_fetch_ua_with_context_observed(
        &format!("{}/large", server.url()),
        4,
        None,
        &context,
        |response_url, content_type, bytes| {
            progress.push((
                response_url.to_string(),
                content_type.map(str::to_owned),
                bytes.to_vec(),
            ));
        },
    )
    .await;

    assert!(result.unwrap_err().contains("4-byte cap"));
    assert!(
        progress.is_empty(),
        "partial bytes are not a completed source"
    );
    response.assert_async().await;
}

#[tokio::test]
async fn html_attachment_is_returned_without_streaming_as_a_document() {
    let mut server = mockito::Server::new_async().await;
    let response = server
        .mock("GET", "/download")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_header("content-disposition", "attachment; filename=page.html")
        .with_body("downloaded bytes")
        .expect(1)
        .create_async()
        .await;
    let context = netfetcher::FetchContext::permissive();
    let mut progress = Vec::new();
    let fetched = do_fetch_ua_with_context_observed(
        &format!("{}/download", server.url()),
        64,
        None,
        &context,
        |response_url, content_type, bytes| {
            progress.push((
                response_url.to_string(),
                content_type.map(str::to_owned),
                bytes.to_vec(),
            ));
        },
    )
    .await
    .expect("HTTP attachment fetch");

    assert_eq!(fetched.bytes, b"downloaded bytes");
    assert_eq!(
        fetched.content_disposition.as_deref(),
        Some("attachment; filename=page.html")
    );
    assert!(
        progress.is_empty(),
        "attachments are not streamed documents"
    );
    response.assert_async().await;
}

#[tokio::test]
async fn nonredirect_http_success_reports_final_url_and_body_once() {
    let mut server = mockito::Server::new_async().await;
    let response = server
        .mock("GET", "/page")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("plain page")
        .expect(1)
        .create_async()
        .await;
    let requested_url = format!("{}/page", server.url());
    let context = netfetcher::FetchContext::permissive();
    let mut progress = Vec::new();
    let fetched = do_fetch_ua_with_context_observed(
        &requested_url,
        64,
        None,
        &context,
        |response_url, content_type, bytes| {
            progress.push((
                response_url.to_string(),
                content_type.map(str::to_owned),
                bytes.to_vec(),
            ));
        },
    )
    .await
    .expect("HTTP page fetch");

    assert_eq!(fetched.bytes, b"plain page");
    assert_eq!(progress.len(), 1);
    assert_eq!(progress[0].0, requested_url);
    assert_eq!(progress[0].1.as_deref(), Some("text/plain"));
    assert_eq!(progress[0].2, b"plain page");
    response.assert_async().await;
}
