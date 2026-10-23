// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

#[test]
fn pure_preflight_errors_do_not_require_a_tokio_reactor() {
    for url in [":not an address", "unsupported://127.0.0.1/"] {
        assert!(
            pollster::block_on(fetch_page_with_policy(url, ClientFetchPolicy::default())).is_err()
        );
    }
    assert!(
        pollster::block_on(fetch_page_with_policy(
            "http://127.0.0.1/",
            ClientFetchPolicy {
                max_page_bytes: 0,
                ..ClientFetchPolicy::default()
            }
        ))
        .unwrap_err()
        .contains("positive")
    );
}

#[tokio::test]
async fn http_retains_its_twenty_redirect_default_independently_of_smolweb() {
    let mut server = mockito::Server::new_async().await;
    let mut redirects = Vec::new();
    for hop in 0..6 {
        redirects.push(
            server
                .mock("GET", format!("/{hop}").as_str())
                .with_status(302)
                .with_header("location", format!("/{}", hop + 1).as_str())
                .expect(1)
                .create_async()
                .await,
        );
    }
    let final_reply = server
        .mock("GET", "/6")
        .with_status(200)
        .with_body("saved")
        .expect(1)
        .create_async()
        .await;
    let policy = ClientFetchPolicy::default();
    assert_eq!(policy.max_redirects, 4);
    assert_eq!(policy.max_http_redirects, 20);
    assert_eq!(
        fetch_page_with_policy(&format!("{}/0", server.url()), policy)
            .await
            .unwrap()
            .bytes,
        b"saved"
    );
    for redirect in redirects {
        redirect.assert_async().await;
    }
    final_reply.assert_async().await;
}

async fn spartan_reply(
    bytes: Vec<u8>,
    hold_open: bool,
) -> (String, tokio::task::JoinHandle<Vec<u8>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(5), async {
            let (stream, _) = listener.accept().await.unwrap();
            let mut stream = BufReader::new(stream);
            let mut line = Vec::new();
            stream.read_until(b'\n', &mut line).await.unwrap();
            let body_len: usize = String::from_utf8_lossy(&line)
                .split_whitespace()
                .last()
                .unwrap()
                .parse()
                .unwrap();
            let mut body = vec![0; body_len];
            stream.read_exact(&mut body).await.unwrap();
            stream
                .get_mut()
                .write_all(b"2 application/octet-stream\r\n")
                .await
                .unwrap();
            stream.get_mut().write_all(&bytes).await.unwrap();
            if hold_open {
                let mut byte = [0];
                assert_eq!(
                    stream.read(&mut byte).await.unwrap(),
                    0,
                    "overflow closes the client before EOF"
                );
            } else {
                stream.get_mut().shutdown().await.unwrap();
            }
            body
        })
        .await
        .expect("bounded Spartan fixture")
    });
    (format!("spartan://{address}/read"), task)
}

#[tokio::test]
async fn page_and_binary_limits_fail_before_an_unfinished_response_can_be_buffered() {
    let policy = ClientFetchPolicy {
        max_page_bytes: 4,
        max_subresource_bytes: 3,
        ..ClientFetchPolicy::default()
    };
    for (binary, limit) in [(false, 4), (true, 3)] {
        for extra in [0, 1] {
            let expected = vec![b'x'; limit + extra];
            let (url, server) = spartan_reply(expected.clone(), extra != 0).await;
            let result = tokio::time::timeout(Duration::from_secs(1), async {
                if binary {
                    fetch_bytes(&url, &Stores::in_memory(), policy).await
                } else {
                    fetch_page_with_policy(&url, policy)
                        .await
                        .map(|page| page.bytes)
                }
            })
            .await
            .expect("body limit does not wait for peer EOF");
            if extra == 0 {
                assert_eq!(result.unwrap(), expected);
            } else {
                assert!(result.is_err());
            }
            assert!(server.await.unwrap().is_empty());
        }
    }
}

#[tokio::test]
async fn explicit_submit_has_independent_request_and_response_budgets() {
    let policy = ClientFetchPolicy {
        max_submission_bytes: 4,
        max_submission_response_bytes: 3,
        ..ClientFetchPolicy::default()
    };
    for extra in [0, 1] {
        let (url, server) = spartan_reply(vec![b'r'; 3 + extra], extra != 0).await;
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            submit_smolweb(
                SmolwebSubmission::Spartan {
                    url,
                    body: b"sent".to_vec(),
                },
                policy,
            ),
        )
        .await
        .expect("submission response cap does not wait for EOF");
        if extra == 0 {
            let SubmissionAnswer::Success(receipt) = result.unwrap() else {
                panic!("native success");
            };
            assert_eq!(receipt.bytes, b"rrr");
        } else {
            assert!(result.is_err());
        }
        assert_eq!(server.await.unwrap(), b"sent");
    }
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let result = submit_smolweb(
        SmolwebSubmission::Spartan {
            url: format!("spartan://{}/", listener.local_addr().unwrap()),
            body: b"extra".to_vec(),
        },
        policy,
    )
    .await;
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("submission exceeds")
    );
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
    );
}

#[tokio::test]
async fn redirects_share_one_deadline_instead_of_restarting_the_timeout() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(3), async {
            for reply in [
                b"3 /second\r\n".as_slice(),
                b"2 text/plain\r\nsaved".as_slice(),
            ] {
                let (stream, _) = listener.accept().await.unwrap();
                let mut stream = BufReader::new(stream);
                let mut line = String::new();
                stream.read_line(&mut line).await.unwrap();
                tokio::time::sleep(Duration::from_millis(300)).await;
                let _ = stream.get_mut().write_all(reply).await;
                let _ = stream.get_mut().shutdown().await;
            }
        })
        .await
        .expect("both redirect requests reached the fixture");
    });
    let policy = ClientFetchPolicy {
        timeout: Duration::from_millis(500),
        ..ClientFetchPolicy::default()
    };
    let error = fetch_page_with_policy(&format!("spartan://{address}/first"), policy)
        .await
        .unwrap_err();
    assert!(error.contains("timed out"), "{error}");
    server.await.unwrap();
}

#[tokio::test]
async fn configured_redirect_count_and_invalid_limits_refuse_before_extra_contact() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut stream = BufReader::new(stream);
        let mut line = String::new();
        stream.read_line(&mut line).await.unwrap();
        stream.get_mut().write_all(b"3 /second\r\n").await.unwrap();
        stream.get_mut().shutdown().await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(100), listener.accept())
                .await
                .is_err()
        );
    });
    let policy = ClientFetchPolicy {
        max_redirects: 0,
        ..ClientFetchPolicy::default()
    };
    let error = fetch_page_with_policy(&format!("spartan://{address}/first"), policy)
        .await
        .unwrap_err();
    assert!(error.contains("redirect"));
    server.await.unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let policy = ClientFetchPolicy {
        max_page_bytes: 0,
        ..policy
    };
    assert!(
        fetch_page_with_policy(
            &format!("spartan://{}/", listener.local_addr().unwrap()),
            policy
        )
        .await
        .unwrap_err()
        .contains("positive")
    );
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
    );
}

#[tokio::test]
async fn http_page_body_is_inside_the_configured_whole_deadline() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0; 1024];
        stream.read(&mut request).await.unwrap();
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n")
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
    });
    let policy = ClientFetchPolicy {
        timeout: Duration::from_millis(100),
        ..ClientFetchPolicy::default()
    };
    let error = tokio::time::timeout(
        Duration::from_secs(1),
        fetch_page_with_policy(&format!("http://{address}/"), policy),
    )
    .await
    .unwrap()
    .unwrap_err();
    assert!(error.contains("timed out"), "{error}");
    server.await.unwrap();
}
