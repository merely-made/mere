// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A host's publication of a session's graph, joined to the commit that
//! writes it (engine split D257). The session decides when to publish and
//! carries the writes; the host's [`PublicationHook`] decides what a
//! publication is. The kernel graph's hook is the addressable store's.

use std::future::Future;
use std::pin::Pin;

use super::*;

/// The store a hook reads and stages against: the session's own backend,
/// lent for the call.
#[cfg(not(target_arch = "wasm32"))]
pub struct SessionBackend<'a>(&'a (dyn Backend + Sync + 'a));
/// The store a hook reads and stages against: the session's own backend,
/// lent for the call.
#[cfg(target_arch = "wasm32")]
pub struct SessionBackend<'a>(&'a (dyn Backend + 'a));

#[cfg(not(target_arch = "wasm32"))]
impl<'a> SessionBackend<'a> {
    pub(super) fn of<B: Backend + Sync>(backend: &'a B) -> Self {
        Self(backend)
    }
}

#[cfg(target_arch = "wasm32")]
impl<'a> SessionBackend<'a> {
    pub(super) fn of<B: Backend>(backend: &'a B) -> Self {
        Self(backend)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl Backend for SessionBackend<'_> {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.0.get(key).await
    }

    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        self.0.put(key, bytes).await
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.0.delete(key).await
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        self.0.list(prefix).await
    }

    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
        self.0.scan(start, end).await
    }

    async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
        self.0.apply(ops).await
    }

    async fn transact(&self, f: muniment::TransactFn) -> Result<(), StoreError> {
        self.0.transact(f).await
    }

    fn single_owner(&self) -> bool {
        self.0.single_owner()
    }
}

/// The writes a publication adds to its commit, decided inside the commit's
/// transaction from what the store holds there.
#[cfg(not(target_arch = "wasm32"))]
pub type PublicationWrites =
    Box<dyn FnOnce(&dyn muniment::TransactionReader) -> Vec<WriteOp> + Send>;
/// See the native [`PublicationWrites`]; wasm drops the `Send` bound.
#[cfg(target_arch = "wasm32")]
pub type PublicationWrites = Box<dyn FnOnce(&dyn muniment::TransactionReader) -> Vec<WriteOp>>;

/// The generation a store holds, read by a hook.
#[cfg(not(target_arch = "wasm32"))]
pub type PublishedGeneration<'a> =
    Pin<Box<dyn Future<Output = Result<String, SessionError>> + Send + 'a>>;
/// See the native [`PublishedGeneration`]; wasm drops the `Send` bound.
#[cfg(target_arch = "wasm32")]
pub type PublishedGeneration<'a> = Pin<Box<dyn Future<Output = Result<String, SessionError>> + 'a>>;

/// One publication of a graph, prepared without storage.
pub struct Publication {
    /// What the publication names the graph as.
    pub generation: String,
    /// Its writes, which keep their place ahead of a caller's in the commit.
    pub writes: PublicationWrites,
}

/// How a host publishes its graph type. Each takes the session's graph, its
/// store and its root container.
pub struct PublicationHook<G: SessionGraph> {
    /// Refuse a graph that cannot be published whole.
    pub check: fn(&G) -> Result<(), SessionError>,
    /// The publication of the graph, prepared without storage.
    pub prepare: fn(&G, SessionBackend<'_>, Uuid) -> Result<Publication, SessionError>,
    /// The generation a cold reopen of the graph publishes as, which an
    /// operation revision names.
    pub revision: fn(&G, SessionBackend<'_>, Uuid) -> Result<String, SessionError>,
    /// The generation the store holds for the container now.
    pub published: for<'a> fn(SessionBackend<'a>, Uuid) -> PublishedGeneration<'a>,
}

impl<B: Backend, G: SessionGraph> GraphSession<B, G> {
    /// Publish through `hook` (engine split D257). The kernel graph's
    /// constructors install the addressable store's; another graph's
    /// sessions publish only once their host installs one.
    pub fn with_publication(mut self, hook: &'static PublicationHook<G>) -> Self {
        self.hooks.publication = Some(hook);
        self
    }

    fn publication_hook(&self) -> Result<&'static PublicationHook<G>, SessionError> {
        self.hooks.publication.ok_or_else(|| {
            SessionError::Validation("this session's graph has no publication hook".into())
        })
    }

    /// The refusals before any publication: an unqualified writer, and what
    /// the hook refuses (engine split D171).
    pub(crate) fn check_publishable(&self) -> Result<(), SessionError> {
        let hook = self.publication_hook()?;
        if self.placement.is_none() {
            return Err(SessionError::Validation(
                "addressable publication requires qualified recorded placement".into(),
            ));
        }
        (hook.check)(&self.graph)
    }
}

impl<B: LegacyTransactionBackend, G: SessionGraph> GraphSession<B, G> {
    /// This session graph's publication, prepared without storage, with the
    /// refusals of [`check_publishable`](Self::check_publishable).
    pub(crate) fn prepare_publication(&self) -> Result<Publication, SessionError> {
        self.check_publishable()?;
        let hook = self.publication_hook()?;
        (hook.prepare)(
            &self.graph,
            SessionBackend::of(self.slots.backend()),
            *self.manifest.root_graph_id.as_uuid(),
        )
    }

    /// The generation a cold reopen of this graph publishes as.
    pub(crate) fn publication_revision(&self) -> Result<String, SessionError> {
        let hook = self.publication_hook()?;
        (hook.revision)(
            &self.graph,
            SessionBackend::of(self.slots.backend()),
            *self.manifest.root_graph_id.as_uuid(),
        )
    }

    /// The generation the store holds for this session's container.
    pub(crate) async fn published_generation(&self) -> Result<String, SessionError> {
        let hook = self.publication_hook()?;
        (hook.published)(
            SessionBackend::of(self.slots.backend()),
            *self.manifest.root_graph_id.as_uuid(),
        )
        .await
    }
}
