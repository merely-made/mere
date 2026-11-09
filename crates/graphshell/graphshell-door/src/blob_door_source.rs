// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A Murm [`BlobSource`] over a blob door session (cycle breaks P2 step 3).
//!
//! A supervised application that holds no blob store serves its scope's
//! blobs to peers by answering the transport's three questions through the
//! resident's blob custody route: `allows` is the route's `allows` call,
//! `children` is a verified whole read of the hash sequence, and `export_bao`
//! is `bao-open` followed by the chunks of the verified read. The design and
//! its security properties are in the domain contract, "Serving through the
//! door".
//!
//! Give the source its own door session: a verified read holds the session
//! while its chunks are copied out, and the application's custody calls should
//! not wait behind a peer's download.

use std::io;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::Mutex;
use transport::blob_source::{BaoReader, BlobSource, chunk_ranges_to_intervals};
use transport::{BlobHash, BlobScope};

use crate::blob_door::{BaoRead, BlobDoorClient, BlobDoorError, ChunkInterval};

/// Serves one application's scopes from the resident's custody.
pub struct DoorBlobSource {
    client: Arc<Mutex<BlobDoorClient>>,
}

impl DoorBlobSource {
    /// Take a dedicated door session. The source answers for every scope the
    /// application owns; the provider names the scope it serves.
    pub fn new(client: BlobDoorClient) -> Arc<Self> {
        Arc::new(Self {
            client: Arc::new(Mutex::new(client)),
        })
    }

    /// Bind `scope` on this source's session, so the resident's ownership
    /// check admits the calls made for it.
    pub async fn bind_scope(&self, scope: BlobScope) -> Result<u64, BlobDoorError> {
        self.client.lock().await.bind_scope(scope.to_bytes()).await
    }
}

fn other(error: BlobDoorError) -> io::Error {
    io::Error::other(error.to_string())
}

struct DoorBao {
    client: Arc<Mutex<BlobDoorClient>>,
    read: BaoRead,
}

#[async_trait]
impl BaoReader for DoorBao {
    async fn next(&mut self) -> io::Result<Option<Bytes>> {
        let mut client = self.client.lock().await;
        match client.bao_next(&mut self.read).await.map_err(other)? {
            Some(piece) => Ok(Some(Bytes::from(piece))),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl BlobSource for DoorBlobSource {
    async fn allows(&self, scope: BlobScope, peer: &[u8; 32], hash: BlobHash) -> bool {
        self.client
            .lock()
            .await
            .allows(scope.to_bytes(), *peer, hash.to_bytes())
            .await
            .unwrap_or(false)
    }

    async fn children(&self, hash: BlobHash) -> io::Result<Vec<BlobHash>> {
        let bytes = self
            .client
            .lock()
            .await
            .read(hash.to_bytes())
            .await
            .map_err(other)?;
        if bytes.len() % 32 != 0 {
            return Err(io::Error::other("not a hash sequence"));
        }
        Ok(bytes
            .chunks_exact(32)
            .map(|child| BlobHash::from_bytes(child.try_into().expect("32 bytes")))
            .collect())
    }

    async fn export_bao(
        &self,
        scope: BlobScope,
        peer: &[u8; 32],
        hash: BlobHash,
        ranges: transport::blob_source::ChunkRanges,
    ) -> io::Result<Box<dyn BaoReader>> {
        let intervals = chunk_ranges_to_intervals(&ranges)
            .into_iter()
            .map(|(start, end)| ChunkInterval { start, end })
            .collect();
        let read = self
            .client
            .lock()
            .await
            .bao_open(scope.to_bytes(), *peer, hash.to_bytes(), intervals)
            .await
            .map_err(other)?;
        Ok(Box::new(DoorBao {
            client: Arc::clone(&self.client),
            read,
        }))
    }
}
