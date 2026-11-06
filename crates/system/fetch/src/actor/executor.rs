// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn build_runtime() -> Result<Runtime, String> {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|_| "could not build fetch runtime".into())
}

pub(super) fn run_jobs(
    runtime: &Runtime,
    shared: &Arc<Shared>,
    mut next: impl FnMut() -> Option<Job>,
    stores: Stores,
    policy: ClientFetchPolicy,
) {
    let tasks = ResidentTasks::new();
    while let Some(mut job) = next() {
        if shared.is_closed() {
            drop(job);
        } else {
            let _entered = runtime.enter();
            job.spawned = true;
            tasks.spawn(job.run(stores.clone(), policy));
        }
    }
    shared.close();
    let tasks = runtime.block_on(tasks.close_and_join());
    *shared.report.lock().expect("fetch report mutex poisoned") = Some(FetchActorReport {
        tasks,
        admitted: shared.admitted.load(Ordering::Relaxed),
        discarded_before_spawn: shared.discarded_before_spawn.load(Ordering::Relaxed),
        overloaded: shared.overloaded.load(Ordering::Relaxed),
        queue_timeouts: shared.queue_timeouts.load(Ordering::Relaxed),
    });
}

pub(super) async fn execute(
    command: FetchCommand,
    stores: Stores,
    policy: ClientFetchPolicy,
    completion: Arc<Completion>,
) -> FetchUpdate {
    match command {
        FetchCommand::Page {
            request,
            url,
            identity,
        } => {
            let request_url = url.clone();
            let progress = completion.clone();
            let result = fetch_page_interactive_capped(
                &url,
                policy.max_page_bytes,
                identity.as_ref(),
                &stores,
                policy,
                move |response_url, content_type, bytes| {
                    if !progress.terminal.load(Ordering::Acquire) {
                        progress.out.emit(FetchUpdate::PageProgress(PageProgress {
                            request,
                            url: request_url.clone(),
                            response_url: response_url.to_string(),
                            content_type: content_type.map(str::to_string),
                            bytes: bytes.to_vec(),
                        }));
                    }
                },
            )
            .await;
            FetchUpdate::Page(FetchOutcome {
                request,
                url,
                result,
            })
        },
        FetchCommand::Subresource(url) => {
            let result = fetch_bytes(&url, &stores, policy).await;
            FetchUpdate::Subresource(SubresourceOutcome { url, result })
        },
        FetchCommand::Favicon { request, url } => FetchUpdate::Favicon(FaviconOutcome {
            request,
            result: fetch_bytes(&url, &stores, policy).await,
        }),
        FetchCommand::Submit {
            request,
            submission,
        } => {
            let url = submission.url().into();
            #[cfg(feature = "smolweb")]
            let result = submit_smolweb(submission, policy).await;
            #[cfg(not(feature = "smolweb"))]
            let result = Err(FetchFailure::Failed(
                "small-web submissions are not built into this host".into(),
            ));
            FetchUpdate::Submission(SubmissionOutcome {
                request,
                url,
                result,
            })
        },
        FetchCommand::CancelPage { .. } => unreachable!("controls bypass work admission"),
    }
}
