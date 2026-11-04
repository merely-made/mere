// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Whether disclosed scene and resource data may survive process exit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheRetention {
    #[default]
    MemoryOnly,
    EncryptedPersistent,
    Exportable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachePolicy {
    pub retention: CacheRetention,
    pub expires_at_ms: Option<u64>,
    pub purge_on_revocation: bool,
}

/// A content-addressed resource request scoped to the disclosing session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRequest {
    pub session: ProjectionSession,
    pub resource: ContentHash,
}

/// Independently transferred bytes. Clients verify the address before caching.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceResponse {
    pub session: ProjectionSession,
    pub resource: ContentHash,
    pub bytes: Vec<u8>,
}

/// Read part of a resource, for resources too large for one carrier frame.
///
/// Deliberately not a range on [`ResourceRequest`]. That reply's `resource`
/// is the address *of its bytes*, checked by
/// [`has_valid_address`](ResourceResponse::has_valid_address); a partial reply
/// would make that field either wrong or ambiguous. Chunks address themselves
/// separately instead, so both checks stay honest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceChunkRequest {
    pub session: ProjectionSession,
    /// The whole resource being read, not this chunk.
    pub resource: ContentHash,
    pub offset: u64,
    /// Most bytes this client will accept back. `0` lets the endpoint choose;
    /// any value is clamped to [`MAX_RESOURCE_CHUNK_BYTES`]. A zero-length
    /// reply below the end of a resource would strand a client in a loop, so
    /// the endpoint never sends one.
    pub length: u32,
}

/// One addressed slice of a resource.
///
/// Carries two hashes because they answer different questions: `resource`
/// says which whole thing this belongs to, and `chunk` says these particular
/// bytes arrived intact. A client verifies `chunk` per frame and the assembled
/// `resource` once at the end.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceChunkResponse {
    pub session: ProjectionSession,
    pub resource: ContentHash,
    pub offset: u64,
    /// Length of the whole resource, so a client can size its buffer and know
    /// when it is finished without waiting for a sentinel frame.
    pub total_len: u64,
    pub chunk: ContentHash,
    /// Base64 (standard alphabet, padded). See [`MAX_RESOURCE_CHUNK_BYTES`].
    pub bytes: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssemblyError {
    /// The frame failed its own `chunk` address, or was not valid base64.
    CorruptChunk,
    /// The frame did not begin where the previous one ended.
    OutOfOrder { expected: u64, found: u64 },
    /// Every byte arrived, but the whole does not match `resource`.
    WrongResource,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self {
            retention: CacheRetention::MemoryOnly,
            expires_at_ms: None,
            purge_on_revocation: true,
        }
    }
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CorruptChunk => write!(formatter, "resource chunk failed its own address"),
            Self::OutOfOrder { expected, found } => {
                write!(formatter, "expected chunk at {expected}, received {found}")
            },
            Self::WrongResource => {
                write!(
                    formatter,
                    "assembled bytes do not match the resource address"
                )
            },
        }
    }
}

impl ResourceResponse {
    pub fn new(session: ProjectionSession, bytes: Vec<u8>) -> Self {
        let resource = ContentHash::of(&bytes);
        Self {
            session,
            resource,
            bytes,
        }
    }

    pub fn has_valid_address(&self) -> bool {
        ContentHash::of(&self.bytes) == self.resource
    }

    /// Whether this is the content-addressed reply to the exact request.
    pub fn matches_request(&self, request: &ResourceRequest) -> bool {
        self.session == request.session
            && self.resource == request.resource
            && self.has_valid_address()
    }
}

impl ResourceChunkResponse {
    /// Cut one reply out of a whole resource already in hand.
    ///
    /// The default path for any endpoint that cannot seek. An offset past the
    /// end yields an empty final chunk rather than an error, so a client that
    /// raced a shrinking resource stops instead of retrying forever.
    pub fn slice(whole: &ResourceResponse, offset: u64, length: u32) -> Self {
        Self::from_slice(
            whole.session.clone(),
            whole.resource,
            &whole.bytes,
            offset,
            length,
        )
    }

    /// Cut one reply from bytes held by reference.
    ///
    /// The form to use when the whole resource is already in hand: taking it
    /// as a slice means serving N chunks copies the resource once, not N
    /// times.
    pub fn from_slice(
        session: ProjectionSession,
        resource: ContentHash,
        whole: &[u8],
        offset: u64,
        length: u32,
    ) -> Self {
        let total_len = whole.len() as u64;
        let start = offset.min(total_len) as usize;
        let want = if length == 0 {
            MAX_RESOURCE_CHUNK_BYTES
        } else {
            length.min(MAX_RESOURCE_CHUNK_BYTES)
        } as usize;
        let end = start.saturating_add(want).min(whole.len());
        let slice = &whole[start..end];
        Self {
            session,
            resource,
            offset: start as u64,
            total_len,
            chunk: ContentHash::of(slice),
            bytes: base64_encode(slice),
        }
    }

    /// The raw bytes, if this frame is intact. `None` when the payload is not
    /// valid base64 or does not match `chunk`.
    pub fn decode(&self) -> Option<Vec<u8>> {
        let bytes = base64_decode(&self.bytes)?;
        (ContentHash::of(&bytes) == self.chunk).then_some(bytes)
    }

    /// Whether this frame reaches the end of the resource.
    pub fn is_final(&self, decoded_len: usize) -> bool {
        self.offset.saturating_add(decoded_len as u64) >= self.total_len
    }
}

impl ResourceAssembly {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bytes received so far; also the offset of the next chunk to request.
    pub fn received(&self) -> u64 {
        self.bytes.len() as u64
    }

    /// Take one reply. `Ok(Some(bytes))` means the resource is complete and
    /// verified; `Ok(None)` means keep going from [`received`](Self::received).
    pub fn accept(
        &mut self,
        response: &ResourceChunkResponse,
    ) -> Result<Option<Vec<u8>>, AssemblyError> {
        let decoded = response.decode().ok_or(AssemblyError::CorruptChunk)?;
        if response.offset != self.received() {
            return Err(AssemblyError::OutOfOrder {
                expected: self.received(),
                found: response.offset,
            });
        }
        self.bytes.extend_from_slice(&decoded);
        if !response.is_final(decoded.len()) {
            return Ok(None);
        }
        if ContentHash::of(&self.bytes) != response.resource {
            return Err(AssemblyError::WrongResource);
        }
        Ok(Some(std::mem::take(&mut self.bytes)))
    }
}
