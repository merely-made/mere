// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{ACTIVE_CALLS, INGRESS_ALPN, IngressError, QUEUED_CALLS, RecordIngress};
use muniment::Backend;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::task::{JoinHandle, JoinSet};
use transport::{Alpn, BoundedAcceptHandle, P2pandaTransport};

/// An owning service lifetime. Shutdown cancels/joins active calls and closes
/// its queue before the resident closes the shared endpoint or mesh store.
pub struct IngressHandle<B: Backend + Clone + Send + Sync + 'static> {
    service: Arc<RecordIngress<B>>,
    transport: Arc<P2pandaTransport>,
    queue: BoundedAcceptHandle,
    stop: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<Result<(), IngressError>>>,
}

impl<B: Backend + Clone + Send + Sync + 'static> RecordIngress<B> {
    pub async fn mount(
        self: Arc<Self>,
        transport: Arc<P2pandaTransport>,
    ) -> Result<IngressHandle<B>, IngressError> {
        let queue = transport
            .register_bounded_alpn(
                Alpn::new(INGRESS_ALPN),
                QUEUED_CALLS,
                self.config.call_timeout,
            )
            .await?;
        let (stop, stopped) = oneshot::channel();
        let service = self.clone();
        let network = transport.clone();
        let task = tokio::spawn(async move { service.run(network, stopped).await });
        Ok(IngressHandle {
            service: self,
            transport,
            queue,
            stop: Some(stop),
            task: Some(task),
        })
    }

    async fn run(
        self: Arc<Self>,
        transport: Arc<P2pandaTransport>,
        mut stop: oneshot::Receiver<()>,
    ) -> Result<(), IngressError> {
        let mut tasks = JoinSet::new();
        let result = loop {
            tokio::select! {
                biased;
                _ = &mut stop => break Ok(()),
                done = tasks.join_next(), if !tasks.is_empty() => {
                    // Per-call framing/refusal/timeout failures release their
                    // permits and do not terminate the owning resident service.
                    if done.is_some_and(|r| r.is_err_and(|e| e.is_panic())) {
                        break Err(IngressError::Task("session task panicked".into()));
                    }
                },
                incoming = transport.accept_observed(Alpn::new(INGRESS_ALPN)), if tasks.len() < ACTIVE_CALLS => {
                    let (accepted, arrived) = match incoming { Ok(a) => a, Err(e) => break Err(e.into()) };
                    let service = self.clone();
                    tasks.spawn(async move { service.serve(accepted, arrived).await });
                },
            }
        };
        transport.stop_accepting(&Alpn::new(INGRESS_ALPN));
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
        result
    }
}

impl<B: Backend + Clone + Send + Sync + 'static> IngressHandle<B> {
    pub fn service(&self) -> &RecordIngress<B> {
        &self.service
    }
    pub fn queue_stats(&self) -> transport::BoundedAcceptStats {
        self.queue.stats()
    }
    pub async fn shutdown(mut self) -> Result<(), IngressError> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(task) = self.task.take() {
            task.await
                .map_err(|e| IngressError::Task(e.to_string()))??;
        }
        Ok(())
    }
}

impl<B: Backend + Clone + Send + Sync + 'static> Drop for IngressHandle<B> {
    fn drop(&mut self) {
        self.transport.stop_accepting(&Alpn::new(INGRESS_ALPN));
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}
