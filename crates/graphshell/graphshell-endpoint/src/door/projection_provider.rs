// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Native application-side runner for a projection offer.
use super::projection_offer::{
    ProviderAnswer, ProviderCall, ViewerContext, Work, WorkOperation, WorkOutcome, WorkResult,
};
use crate::{
    CompleteEndpoint, IntentSink, PresentationSource, ProjectionNoticeSource, ProjectionSource,
    ResumableProjectionSource, dispatch_common,
};
use djinn::door::{
    app_admission::AppRouteId,
    app_client::{AppClientError, AppConnection},
};
use std::{collections::BTreeMap, fmt::Display, time::Duration};

/// Serve until the host withdraws the lease or the connection fails. The factory
/// creates a separate endpoint for each admitted viewer. Its limit is the lower
/// of the host limit and `own_max_sessions`, which must be positive. Context
/// values are host assertions, not credentials. Dropping the runner on
/// disconnect drops all instances. In-flight synchronous work cannot be interrupted;
/// after cancellation its worker finishes and drops its endpoint instances.
/// Do not cancel an in-progress native read and
/// then reuse the connection: native framing is not cancellation safe.
pub async fn serve<E, F>(
    mut connection: AppConnection,
    route: AppRouteId,
    label: String,
    own_max_sessions: usize,
    mut factory: F,
) -> Result<(), AppClientError>
where
    E: CompleteEndpoint + ResumableProjectionSource + ProjectionNoticeSource + Send + 'static,
    <E as ProjectionSource>::Error: Display,
    <E as PresentationSource>::Error: Display,
    <E as IntentSink>::Error: Display,
    <E as ResumableProjectionSource>::Error: Display,
    <E as ProjectionNoticeSource>::Error: Display,
    F: FnMut(&ViewerContext) -> Result<E, String> + Send + 'static,
{
    let answer: ProviderAnswer = connection
        .call(ProviderCall::Offer { route, label })
        .await?;
    let limit = match answer {
        ProviderAnswer::Registered { max_sessions } if max_sessions > 0 && own_max_sessions > 0 => {
            max_sessions.min(own_max_sessions)
        },
        _ => {
            return Err(AppClientError::InvalidResponse(
                "expected provider registration with a positive session limit".into(),
            ));
        },
    };
    let mut sessions = BTreeMap::new();
    let mut completed = None;
    let mut last_work_id = 0;
    loop {
        let answer: ProviderAnswer = connection
            .call(ProviderCall::Next {
                completed: completed.take(),
            })
            .await?;
        match answer {
            ProviderAnswer::Work { work } => {
                if work.id <= last_work_id {
                    return Err(AppClientError::InvalidResponse(
                        "provider work id is not increasing".into(),
                    ));
                }
                last_work_id = work.id;
                // Only one blocking job exists per provider. It owns the map
                // while executing, so cancellation drops it after the job ends.
                let (new_sessions, new_factory, result) = tokio::task::spawn_blocking(move || {
                    let result = execute(work, &mut sessions, limit, &mut factory);
                    (sessions, factory, result)
                })
                .await
                .map_err(|_| AppClientError::InvalidResponse("provider worker failed".into()))?;
                sessions = new_sessions;
                factory = new_factory;
                completed = Some(result)
            },
            ProviderAnswer::Idle => tokio::time::sleep(Duration::from_millis(50)).await,
            ProviderAnswer::Withdrawn => return Ok(()),
            ProviderAnswer::Registered { .. } => {
                return Err(AppClientError::InvalidResponse(
                    "unexpected provider registration".into(),
                ));
            },
        }
    }
}

fn execute<E, F>(
    work: Work,
    sessions: &mut BTreeMap<String, E>,
    limit: usize,
    factory: &mut F,
) -> WorkResult
where
    E: CompleteEndpoint + ResumableProjectionSource + ProjectionNoticeSource,
    <E as ProjectionSource>::Error: Display,
    <E as PresentationSource>::Error: Display,
    <E as IntentSink>::Error: Display,
    <E as ResumableProjectionSource>::Error: Display,
    <E as ProjectionNoticeSource>::Error: Display,
    F: FnMut(&ViewerContext) -> Result<E, String>,
{
    let failed = |message: &str| WorkOutcome::Failed {
        message: message.into(),
    };
    let result = match work.operation {
        WorkOperation::Open { context } => {
            if context.session != work.session {
                failed("viewer session mismatch")
            } else if sessions.contains_key(&work.session) {
                failed("viewer session already open")
            } else if sessions.len() >= limit {
                failed("provider session limit reached")
            } else {
                match factory(&context) {
                    Ok(endpoint) => {
                        let descriptor = endpoint.describe();
                        sessions.insert(work.session.clone(), endpoint);
                        WorkOutcome::Opened { descriptor }
                    },
                    Err(message) => WorkOutcome::Failed { message },
                }
            }
        },
        WorkOperation::Request { request } => match sessions.get_mut(&work.session) {
            Some(endpoint) => {
                match dispatch_common(endpoint, request, &mut |endpoint: &mut E, request| {
                    endpoint.resume(request).map_err(|e| e.to_string())
                }) {
                    Ok(response) => WorkOutcome::Response { response },
                    Err(_) => failed("session-plane request belongs to host"),
                }
            },
            None => failed("viewer session is not open"),
        },
        WorkOperation::PollNotice => match sessions.get_mut(&work.session) {
            Some(endpoint) => match endpoint.poll_notice() {
                Ok(notice) => WorkOutcome::Notice { notice },
                Err(e) => WorkOutcome::Failed {
                    message: e.to_string(),
                },
            },
            None => failed("viewer session is not open"),
        },
        WorkOperation::Close => {
            if sessions.remove(&work.session).is_some() {
                WorkOutcome::Closed
            } else {
                failed("viewer session is not open")
            }
        },
    };
    WorkResult {
        id: work.id,
        session: work.session,
        result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProjectionCatalog;
    use chirograph::{
        CarrierNotice, CarrierRequest, CarrierRequestBody, CarrierResponseBody, EndpointDescriptor,
        IntentInvocation, IntentResult, ProjectionRequest, ProjectionSnapshot, ResourceRequest,
        ResourceResponse, ResumeReply, ResumeRequest,
    };
    struct Endpoint(String);
    impl ProjectionCatalog for Endpoint {
        fn describe(&self) -> EndpointDescriptor {
            EndpointDescriptor {
                label: self.0.clone(),
                projections: vec![],
            }
        }
    }
    impl ProjectionSource for Endpoint {
        type Error = String;
        fn snapshot(&mut self, _: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
            Err("unavailable".into())
        }
    }
    impl PresentationSource for Endpoint {
        type Error = String;
        fn resource(&mut self, _: ResourceRequest) -> Result<ResourceResponse, String> {
            Err("unavailable".into())
        }
    }
    impl IntentSink for Endpoint {
        type Error = String;
        fn invoke(&mut self, _: IntentInvocation) -> Result<IntentResult, String> {
            Err("unavailable".into())
        }
    }
    impl ResumableProjectionSource for Endpoint {
        type Error = String;
        fn resume(&mut self, _: ResumeRequest) -> Result<ResumeReply, String> {
            Err("unavailable".into())
        }
    }
    impl ProjectionNoticeSource for Endpoint {
        type Error = String;
        fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, String> {
            Ok(None)
        }
    }
    fn work(id: u64, session: &str, operation: WorkOperation) -> Work {
        Work {
            id,
            session: session.into(),
            operation,
        }
    }
    fn open(session: &str) -> WorkOperation {
        WorkOperation::Open {
            context: ViewerContext {
                session: session.into(),
                subject: [1; 32],
                application: None,
            },
        }
    }
    #[test]
    fn sessions_are_isolated_bounded_and_host_keeps_session_plane() {
        let mut sessions = BTreeMap::new();
        let mut factory = |context: &ViewerContext| Ok(Endpoint(context.session.clone()));
        assert!(matches!(
            execute(work(1, "a", open("a")), &mut sessions, 2, &mut factory).result,
            WorkOutcome::Opened { .. }
        ));
        assert!(matches!(
            execute(work(2, "b", open("b")), &mut sessions, 2, &mut factory).result,
            WorkOutcome::Opened { .. }
        ));
        assert!(matches!(
            execute(work(3, "c", open("c")), &mut sessions, 2, &mut factory).result,
            WorkOutcome::Failed { .. }
        ));
        let request = |body| WorkOperation::Request {
            request: CarrierRequest { id: 41, body },
        };
        let response = execute(
            work(4, "b", request(CarrierRequestBody::Discover)),
            &mut sessions,
            2,
            &mut factory,
        );
        assert!(
            matches!(response.result, WorkOutcome::Response { response: chirograph::CarrierResponse { id: 41, body: Ok(CarrierResponseBody::Descriptor(EndpointDescriptor { label, .. })) } } if label == "b")
        );
        assert!(matches!(
            execute(
                work(5, "a", request(CarrierRequestBody::Close)),
                &mut sessions,
                2,
                &mut factory
            )
            .result,
            WorkOutcome::Failed { .. }
        ));
        assert!(matches!(
            execute(
                work(6, "a", WorkOperation::Close),
                &mut sessions,
                2,
                &mut factory
            )
            .result,
            WorkOutcome::Closed
        ));
        assert!(matches!(
            execute(
                work(7, "a", WorkOperation::PollNotice),
                &mut sessions,
                2,
                &mut factory
            )
            .result,
            WorkOutcome::Failed { .. }
        ));
        assert!(matches!(
            execute(work(8, "c", open("wrong")), &mut sessions, 2, &mut factory).result,
            WorkOutcome::Failed { .. }
        ));
    }
    #[cfg(unix)]
    async fn fixture_connection() -> (AppConnection, tokio::net::UnixStream) {
        use djinn::door::{
            app_admission::{AppHello, AppId},
            frame::{read_native_message_async, write_native_message_async},
            wire::{APP_CONNECT_SCHEMA, AppHostMessage, AppMessage, BrowserChallenge},
        };
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "djinn-provider-{}-{}.sock",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let listener = tokio::net::UnixListener::bind(&path).unwrap();
        let endpoint = path.to_str().unwrap().to_owned();
        let host = tokio::spawn(async move {
            let (mut host, _) = listener.accept().await.unwrap();
            let hello: AppHello = read_native_message_async(&mut host).await.unwrap().unwrap();
            let request = hello.accept().unwrap();
            let challenge = BrowserChallenge {
                schema: djinn::door::wire::BROWSER_CHALLENGE_SCHEMA.into(),
                host_nonce: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into(),
            };
            write_native_message_async(
                &mut host,
                &AppHostMessage::<(), (), ()>::Challenge {
                    challenge: challenge.clone(),
                },
            )
            .await
            .unwrap();
            let connect: AppMessage<(), ()> =
                read_native_message_async(&mut host).await.unwrap().unwrap();
            assert!(
                matches!(connect, AppMessage::Connect { schema, host_nonce, .. } if schema == APP_CONNECT_SCHEMA && host_nonce == challenge.host_nonce)
            );
            write_native_message_async(
                &mut host,
                &AppHostMessage::<(), (), ()>::Connected {
                    app: request.app,
                    session: "fixture".into(),
                },
            )
            .await
            .unwrap();
            host
        });
        let client = AppConnection::open_at(&endpoint, AppId::new("fixture"))
            .await
            .unwrap();
        let host = host.await.unwrap();
        std::fs::remove_file(path).unwrap();
        (client, host)
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn runner_exchanges_work_and_reports_disconnect() {
        use djinn::door::{
            frame::{read_native_message_async, write_native_message_async},
            wire::{AppHostMessage, AppMessage},
        };
        let (connection, mut host) = fixture_connection().await;
        let server = tokio::spawn(async move {
            let first: AppMessage<(), ProviderCall> =
                read_native_message_async(&mut host).await.unwrap().unwrap();
            assert!(matches!(
                first,
                AppMessage::Call {
                    id: 1,
                    call: ProviderCall::Offer { .. }
                }
            ));
            write_native_message_async(
                &mut host,
                &AppHostMessage::<(), (), _>::Answer {
                    id: 1,
                    answer: ProviderAnswer::Registered { max_sessions: 1 },
                },
            )
            .await
            .unwrap();
            let next: AppMessage<(), ProviderCall> =
                read_native_message_async(&mut host).await.unwrap().unwrap();
            assert!(matches!(
                next,
                AppMessage::Call {
                    id: 2,
                    call: ProviderCall::Next { completed: None }
                }
            ));
            write_native_message_async(
                &mut host,
                &AppHostMessage::<(), (), _>::Answer {
                    id: 2,
                    answer: ProviderAnswer::Work {
                        work: work(1, "a", open("a")),
                    },
                },
            )
            .await
            .unwrap();
            let next: AppMessage<(), ProviderCall> =
                read_native_message_async(&mut host).await.unwrap().unwrap();
            assert!(matches!(
                next,
                AppMessage::Call {
                    id: 3,
                    call: ProviderCall::Next {
                        completed: Some(WorkResult {
                            id: 1,
                            result: WorkOutcome::Opened { .. },
                            ..
                        })
                    }
                }
            ));
            // Closing before the next answer makes the runner report disconnect.
        });
        let result = serve(
            connection,
            AppRouteId::new("sample").unwrap(),
            "Sample".into(),
            2,
            |ctx| Ok(Endpoint(ctx.session.clone())),
        )
        .await;
        assert!(result.is_err());
        server.await.unwrap();
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn slow_factory_does_not_block_the_async_runtime() {
        use djinn::door::{
            frame::{read_native_message_async, write_native_message_async},
            wire::{AppHostMessage, AppMessage},
        };
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        let (connection, mut host) = fixture_connection().await;
        let entered = Arc::new(AtomicBool::new(false));
        let timer_ran = Arc::new(AtomicBool::new(false));
        let factory_entered = entered.clone();
        let factory_timer = timer_ran.clone();
        let provider = tokio::spawn(serve(
            connection,
            AppRouteId::new("sample").unwrap(),
            "sample".into(),
            1,
            move |ctx| {
                factory_entered.store(true, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(100));
                if !factory_timer.load(Ordering::SeqCst) {
                    return Err("runtime blocked".into());
                }
                Ok(Endpoint(ctx.session.clone()))
            },
        ));
        let _: AppMessage<(), ProviderCall> =
            read_native_message_async(&mut host).await.unwrap().unwrap();
        write_native_message_async(
            &mut host,
            &AppHostMessage::<(), (), _>::Answer {
                id: 1,
                answer: ProviderAnswer::Registered { max_sessions: 1 },
            },
        )
        .await
        .unwrap();
        let _: AppMessage<(), ProviderCall> =
            read_native_message_async(&mut host).await.unwrap().unwrap();
        write_native_message_async(
            &mut host,
            &AppHostMessage::<(), (), _>::Answer {
                id: 2,
                answer: ProviderAnswer::Work {
                    work: work(1, "a", open("a")),
                },
            },
        )
        .await
        .unwrap();
        let tick = tokio::spawn(async move {
            while !entered.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
            timer_ran.store(true, Ordering::SeqCst);
        });
        let next: AppMessage<(), ProviderCall> =
            read_native_message_async(&mut host).await.unwrap().unwrap();
        assert!(matches!(
            next,
            AppMessage::Call {
                call: ProviderCall::Next {
                    completed: Some(WorkResult {
                        result: WorkOutcome::Opened { .. },
                        ..
                    })
                },
                ..
            }
        ));
        tick.await.unwrap();
        drop(host);
        assert!(provider.await.unwrap().is_err());
    }
}
