// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Fetch tests.

use super::*;

#[test]
fn page_request_ids_are_unique_and_cancellation_is_typed() {
    let first = next_fetch_request_id();
    let second = next_fetch_request_id();
    assert!(second > first);
    assert_eq!(FetchFailure::Cancelled.to_string(), "cancelled");
}

#[test]
fn fetchable_for_http_and_smolweb_schemes() {
    assert!(is_fetchable("http://example.com"));
    assert!(is_fetchable("https://example.com"));
    assert!(is_fetchable("gemini://capsule.example/"));
    assert!(is_fetchable("gopher://example.org/"));
    assert!(is_fetchable("finger://example.org/alice"));
    assert!(is_fetchable("spartan://example.org/"));
    assert!(is_fetchable("nex://nightfall.city/"));
    assert!(is_fetchable("guppy://mozz.us/"));
    assert!(is_fetchable("titan://capsule.example/page"));
    assert!(!is_fetchable("mere://welcome"));
    assert!(!is_fetchable("about:blank"));
}

#[test]
fn smolweb_content_type_tags_fixed_schemes_and_passes_others_through() {
    fn resp_for(url: &url::Url, mime: &str) -> errand::Response {
        errand::Response {
            url: url.clone(),
            status: errand::Status::Success,
            raw_status: None,
            meta: mime.to_string(),
            body: Vec::new(),
        }
    }

    let finger = url::Url::parse("finger://example.org/alice").unwrap();
    assert_eq!(
        smolweb_content_type(&finger, &resp_for(&finger, "text/plain")),
        "text/x-finger"
    );

    let nex = url::Url::parse("nex://nightfall.city/").unwrap();
    assert_eq!(
        smolweb_content_type(&nex, &resp_for(&nex, "")),
        "application/x-nex"
    );

    let guppy = url::Url::parse("guppy://mozz.us/").unwrap();
    assert_eq!(
        smolweb_content_type(&guppy, &resp_for(&guppy, "text/gemini")),
        "application/x-guppy"
    );

    let titan = url::Url::parse("titan://capsule.example/page").unwrap();
    assert_eq!(
        smolweb_content_type(&titan, &resp_for(&titan, "text/gemini")),
        "application/x-titan"
    );

    let gem = url::Url::parse("gemini://capsule.example/").unwrap();
    let gem_resp = errand::Response {
        url: gem.clone(),
        status: errand::Status::Success,
        raw_status: Some(20),
        meta: "text/gemini; charset=utf-8".into(),
        body: Vec::new(),
    };
    assert_eq!(
        smolweb_content_type(&gem, &gem_resp),
        "text/gemini; charset=utf-8"
    );
}

#[test]
fn scroll_success_keeps_complete_mime_without_inventing_native_metadata() {
    let url = url::Url::parse("scroll://source.test/page").unwrap();
    let mime = "text/scroll;charset=utf-8;lang=fr";
    let response = errand::Response {
        url: url.clone(),
        status: errand::Status::Success,
        raw_status: Some(27),
        meta: mime.to_string(),
        body: b"##### Native\r\n".to_vec(),
    };
    assert_eq!(smolweb_content_type(&url, &response), mime);
    assert_eq!(response.raw_status, Some(27));
    assert_eq!(response.body, b"##### Native\r\n");
}

#[test]
fn smolweb_input_preserves_prompt_target_and_sensitivity() {
    let target = url::Url::parse("gemini://capsule.example/search").unwrap();
    let response = |code| errand::Response {
        url: target.clone(),
        status: errand::Status::Input,
        raw_status: Some(code),
        meta: "Search the capsule".into(),
        body: Vec::new(),
    };

    assert_eq!(
        smolweb_input_failure(&target, &response(10)),
        FetchFailure::InputRequired {
            url: target.to_string(),
            prompt: "Search the capsule".into(),
            sensitive: false,
        }
    );
    assert_eq!(
        smolweb_input_failure(&target, &response(11)),
        FetchFailure::InputRequired {
            url: target.to_string(),
            prompt: "Search the capsule".into(),
            sensitive: true,
        }
    );
}

#[test]
fn smolweb_trace_address_omits_query_and_fragment() {
    assert_eq!(
        url_without_query("gemini://capsule.example/search?secret%20answer#part"),
        "gemini://capsule.example/search"
    );
    assert_eq!(
        url_without_query("https://user:password@example.test/normal;path?secret#part"),
        "https://example.test/normal;path"
    );
}

#[test]
fn gemini_identity_is_scoped_to_one_capsule_origin() {
    let identity = GeminiClientIdentity::new(
        "gemini://Capsule.Example/account",
        vec![1, 2, 3],
        vec![4, 5, 6],
    )
    .unwrap();
    assert!(identity.applies_to(&url::Url::parse("gemini://capsule.example/private").unwrap()));
    assert!(identity.applies_to(&url::Url::parse("gemini://capsule.example:1965/other").unwrap()));
    assert!(identity.applies_to(&url::Url::parse("titan://capsule.example/upload").unwrap()));
    assert!(!identity.applies_to(&url::Url::parse("gemini://other.example/private").unwrap()));
    assert!(
        !identity.applies_to(&url::Url::parse("gemini://capsule.example:1966/private").unwrap())
    );
    assert!(!identity.applies_to(&url::Url::parse("https://capsule.example/private").unwrap()));
    assert_eq!(identity.origin(), "gemini://capsule.example");
}

#[test]
fn submission_redirect_is_returned_without_becoming_a_fetch() {
    let request = url::Url::parse("titan://capsule.example/upload").unwrap();
    let response = errand::Response {
        url: request.clone(),
        status: errand::Status::Redirect,
        raw_status: Some(30),
        meta: "gemini://capsule.example/posts/1".into(),
        body: Vec::new(),
    };
    assert_eq!(
        smolweb_submission_answer(&request, response),
        Ok(SubmissionAnswer::Redirect(
            "gemini://capsule.example/posts/1".into()
        ))
    );
}

#[test]
fn spartan_submission_redirect_cannot_cross_origin() {
    let request = url::Url::parse("spartan://capsule.example/form").unwrap();
    let response = errand::Response {
        url: request.clone(),
        status: errand::Status::Redirect,
        raw_status: Some(3),
        meta: "spartan://other.example/receipt".into(),
        body: Vec::new(),
    };
    assert!(matches!(
        smolweb_submission_answer(&request, response),
        Err(FetchFailure::Failed(message)) if message.contains("crossed")
    ));
}

#[test]
fn certificate_change_keeps_the_target_and_both_fingerprints_typed() {
    let current = url::Url::parse("gemini://capsule.example:1966/private").unwrap();
    assert_eq!(
        smolweb_transport_failure(
            &current,
            errand::Error::CertificateChanged {
                host: "capsule.example:1966".into(),
                pinned: "11".repeat(32),
                seen: "22".repeat(32),
            },
        ),
        FetchFailure::CertificateChanged {
            url: current.to_string(),
            target: "capsule.example:1966".into(),
            pinned: "11".repeat(32),
            seen: "22".repeat(32),
        }
    );
}

#[test]
fn state_tag_distinguishes_transitions() {
    let ready = ContentState::Ready(Fetched::text(None, ""));
    assert_eq!(ContentState::tag(None), 0);
    assert_eq!(ContentState::tag(Some(&ContentState::Loading)), 1);
    assert_ne!(
        ContentState::tag(Some(&ContentState::Loading)),
        ContentState::tag(Some(&ready)),
        "Loading and Ready re-key the card",
    );
    assert_ne!(
        ContentState::tag(Some(&ready)),
        ContentState::tag(Some(&ContentState::Failed("x".into()))),
    );
}

#[cfg(not(target_arch = "wasm32"))]
mod read_effects {
    use super::*;

    #[derive(Clone, Default)]
    struct TraceCapture(Arc<std::sync::Mutex<String>>);

    impl tracing::Subscriber for TraceCapture {
        fn register_callsite(
            &self,
            _: &'static tracing::Metadata<'static>,
        ) -> tracing::subscriber::Interest {
            tracing::subscriber::Interest::sometimes()
        }

        fn max_level_hint(&self) -> Option<tracing::metadata::LevelFilter> {
            Some(tracing::metadata::LevelFilter::TRACE)
        }

        fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
            true
        }

        fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }

        fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
        fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
        fn enter(&self, _: &tracing::span::Id) {}
        fn exit(&self, _: &tracing::span::Id) {}

        fn event(&self, event: &tracing::Event<'_>) {
            struct Fields(String);

            impl tracing::field::Visit for Fields {
                fn record_debug(
                    &mut self,
                    field: &tracing::field::Field,
                    value: &dyn std::fmt::Debug,
                ) {
                    self.0.push_str(&format!(" {}={value:?}", field.name()));
                }
            }

            let mut fields = Fields(String::new());
            event.record(&mut fields);
            let mut captured = self.0.lock().unwrap();
            captured.push_str(&fields.0);
            captured.push('\n');
        }
    }

    fn unopened_effect_target(scheme: &str, query: &str) -> (std::net::TcpListener, String) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!(
            "{scheme}://{}/submit{query}",
            listener.local_addr().unwrap()
        );
        (listener, url)
    }

    fn assert_uncontacted(listener: &std::net::TcpListener) {
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
    }

    #[test]
    fn refused_titan_read_traces_redact_credentials_and_upload_parameters() {
        const CHILD: &str = "MERE_FETCH_TRACE_CAPTURE_CHILD";
        if std::env::var_os(CHILD).as_deref() != Some(std::ffi::OsStr::new("1")) {
            // Tracing's callsite cache is process-wide. Isolate the scoped
            // subscriber from other tests reading with no subscriber installed.
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "tests::read_effects::refused_titan_read_traces_redact_credentials_and_upload_parameters",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "isolated trace regression failed: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
            return;
        }
        let (listener, base) = unopened_effect_target("titan", "");
        let mut target = url::Url::parse(&base).unwrap();
        target.set_username("fixture-user-secret").unwrap();
        target
            .set_password(Some("fixture-password-secret"))
            .unwrap();
        target.set_path("/upload;token=fixture-path-token-secret");
        target.set_query(Some("fixture-query-secret"));
        target.set_fragment(Some("fixture-fragment-secret"));
        let capture = TraceCapture::default();
        let result = tracing::subscriber::with_default(capture.clone(), || {
            pollster::block_on(fetch_page_capped(target.as_str(), 1024))
        });
        assert_eq!(result, Err("Titan uploads require explicit Submit".into()));
        assert_uncontacted(&listener);
        let captured = capture.0.lock().unwrap();
        assert!(captured.contains("smolweb fetch"));
        assert!(captured.contains("smolweb failed"), "{captured}");
        assert!(captured.contains(&format!(
            "titan://{}/upload",
            listener.local_addr().unwrap()
        )));
        for secret in [
            "fixture-user-secret",
            "fixture-password-secret",
            "fixture-path-token-secret",
            "fixture-query-secret",
            "fixture-fragment-secret",
        ] {
            assert!(!captured.contains(secret));
        }
    }

    /// One real Spartan exchange, recording the complete request before replying.
    async fn spartan_fixture(response: Vec<u8>) -> (String, tokio::task::JoinHandle<Vec<u8>>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("spartan://{}/read", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_secs(5), async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n") {
                    assert!(request.len() < 1024);
                    request.push(stream.read_u8().await.unwrap());
                }
                let length: usize = std::str::from_utf8(&request)
                    .unwrap()
                    .trim_end()
                    .split(' ')
                    .next_back()
                    .unwrap()
                    .parse()
                    .unwrap();
                assert!(length < 1024);
                let header_length = request.len();
                request.resize(header_length + length, 0);
                stream
                    .read_exact(&mut request[header_length..])
                    .await
                    .unwrap();
                stream.write_all(&response).await.unwrap();
                stream.shutdown().await.unwrap();
                request
            })
            .await
            .expect("Spartan fixture timed out")
        });
        (url, task)
    }

    #[tokio::test]
    async fn every_read_entry_refuses_submission_before_connecting() {
        let stores = Stores::in_memory();
        for (scheme, query, message) in [
            ("titan", "", "Titan uploads require explicit Submit"),
            (
                "spartan",
                "?secret%20body",
                "Spartan submissions require explicit Submit",
            ),
            (
                "spartan",
                "?",
                "Spartan submissions require explicit Submit",
            ),
        ] {
            let (listener, url) = unopened_effect_target(scheme, query);
            let mut progress = false;
            assert_eq!(
                fetch_page_interactive_capped(
                    &url,
                    1024,
                    None,
                    &stores,
                    ClientFetchPolicy::default(),
                    |_, _, _| {
                        progress = true;
                    }
                )
                .await,
                Err(FetchFailure::Failed(message.into()))
            );
            assert!(!progress);
            assert_eq!(fetch_page_capped(&url, 1024).await, Err(message.into()));
            assert_eq!(
                fetch_page_anonymous_capped(&url, 1024).await,
                Err(message.into())
            );
            assert_eq!(fetch_page_crawler(&url).await, Err(message.into()));
            assert_eq!(
                fetch_bytes(&url, &stores, ClientFetchPolicy::default()).await,
                Err(message.into())
            );
            assert_uncontacted(&listener);
        }
    }

    #[tokio::test]
    async fn read_redirects_refuse_submission_before_contacting_the_target() {
        for (scheme, query, message) in [
            ("titan", "", "Titan uploads require explicit Submit"),
            (
                "spartan",
                "?secret%20body",
                "Spartan submissions require explicit Submit",
            ),
            (
                "spartan",
                "?",
                "Spartan submissions require explicit Submit",
            ),
        ] {
            let (listener, target) = unopened_effect_target(scheme, query);
            let (url, source) = spartan_fixture(format!("3 {target}\r\n").into_bytes()).await;
            assert_eq!(fetch_page_capped(&url, 1024).await, Err(message.into()));
            assert_eq!(source.await.unwrap(), b"127.0.0.1 /read 0\r\n");
            assert_uncontacted(&listener);
        }
    }

    #[tokio::test]
    async fn read_redirect_to_an_ordinary_spartan_page_keeps_exact_bytes() {
        let body = b"# Read only\r\n\0\xff";
        let mut reply = b"2 application/octet-stream\r\n".to_vec();
        reply.extend_from_slice(body);
        let (target, destination) = spartan_fixture(reply).await;
        let (url, source) = spartan_fixture(format!("3 {target}\r\n").into_bytes()).await;
        let fetched = fetch_page_capped(&url, 1024).await.unwrap();
        assert_eq!(fetched.bytes, body);
        assert_eq!(
            fetched.content_type.as_deref(),
            Some("application/octet-stream")
        );
        assert_eq!(source.await.unwrap(), b"127.0.0.1 /read 0\r\n");
        assert_eq!(destination.await.unwrap(), b"127.0.0.1 /read 0\r\n");
    }

    #[tokio::test]
    async fn explicit_spartan_submit_sends_exact_bytes_and_returns_its_redirect() {
        let (url, server) = spartan_fixture(b"3 /receipt\r\n".to_vec()).await;
        let target = format!("{url}?ignored%20query");
        let body = b"saved\r\n\0\xff".to_vec();
        let result = submit_smolweb(
            SmolwebSubmission::Spartan {
                url: target,
                body: body.clone(),
            },
            ClientFetchPolicy::default(),
        )
        .await
        .unwrap();
        let receipt = url::Url::parse(&url).unwrap().join("/receipt").unwrap();
        assert_eq!(result, SubmissionAnswer::Redirect(receipt.to_string()));
        let mut expected = format!("127.0.0.1 /read {}\r\n", body.len()).into_bytes();
        expected.extend_from_slice(&body);
        assert_eq!(server.await.unwrap(), expected);
    }

    #[tokio::test]
    async fn explicit_titan_submit_preserves_payload_and_capsule_identity() {
        use rustls::pki_types::PrivateKeyDer;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        install_in_memory_smolweb_tofu();
        for body in [b"saved\r\n\0\xff".to_vec(), Vec::new()] {
            let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
            let client = rcgen::generate_simple_self_signed(vec!["author.example".into()]).unwrap();
            let client_der = client.cert.der().to_vec();
            let mut roots = rustls::RootCertStore::empty();
            roots.add(client.cert.der().clone()).unwrap();
            let provider = Arc::new(rustls::crypto::ring::default_provider());
            let verifier = rustls::server::WebPkiClientVerifier::builder_with_provider(
                Arc::new(roots),
                provider.clone(),
            )
            .build()
            .unwrap();
            let config = rustls::ServerConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_client_cert_verifier(verifier)
                .with_single_cert(
                    vec![certificate.cert.der().clone()],
                    PrivateKeyDer::Pkcs8(certificate.key_pair.serialize_der().into()),
                )
                .unwrap();
            let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let target = format!(
                "titan://127.0.0.1:{}/upload",
                listener.local_addr().unwrap().port()
            );
            let identity = GeminiClientIdentity::new(
                &target,
                client_der.clone(),
                client.key_pair.serialize_der(),
            )
            .unwrap();
            let body_length = body.len();
            let server = tokio::spawn(async move {
                tokio::time::timeout(Duration::from_secs(5), async move {
                    let (stream, _) = listener.accept().await.unwrap();
                    let mut tls = acceptor.accept(stream).await.unwrap();
                    let peer = tls.get_ref().1.peer_certificates().unwrap()[0].to_vec();
                    let mut request = Vec::new();
                    while !request.ends_with(b"\r\n") {
                        assert!(request.len() < 1024);
                        request.push(tls.read_u8().await.unwrap());
                    }
                    let header_length = request.len();
                    request.resize(header_length + body_length, 0);
                    tls.read_exact(&mut request[header_length..]).await.unwrap();
                    tls.write_all(b"20 application/octet-stream\r\nsaved\r\n\0\xff")
                        .await
                        .unwrap();
                    tls.shutdown().await.unwrap();
                    (request, peer)
                })
                .await
                .expect("Titan fixture timed out")
            });
            let response = submit_smolweb(
                SmolwebSubmission::Titan {
                    url: target.clone(),
                    body: body.clone(),
                    mime: "application/octet-stream".into(),
                    token: Some("fixture-token".into()),
                    identity: Some(identity),
                },
                ClientFetchPolicy::default(),
            )
            .await
            .unwrap();
            let SubmissionAnswer::Success(receipt) = response else {
                panic!("Titan submission did not return its native success body");
            };
            assert_eq!(receipt.bytes, b"saved\r\n\0\xff");
            let (request, peer) = server.await.unwrap();
            let mut expected = format!(
                "{target};size={body_length};mime=application/octet-stream;token=fixture-token\r\n"
            )
            .into_bytes();
            expected.extend_from_slice(&body);
            assert_eq!(request, expected);
            assert_eq!(peer, client_der);
        }
    }
}
