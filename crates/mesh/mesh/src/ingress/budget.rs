// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::Counters;
use std::io;
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Count all actual I/O, including partial/refused/timed-out frames. Independent
/// read/write caps bound the whole call, not just each individually framed body.
pub(super) struct BudgetStream<'a, S> {
    inner: S,
    counters: &'a Counters,
    read: usize,
    written: usize,
}

impl<'a, S> BudgetStream<'a, S> {
    pub(super) fn new(inner: S, counters: &'a Counters) -> Self {
        Self {
            inner,
            counters,
            read: 0,
            written: 0,
        }
    }
}

impl<S> Drop for BudgetStream<'_, S> {
    fn drop(&mut self) {
        self.counters
            .request_bytes
            .fetch_add(self.read, Ordering::Relaxed);
        self.counters
            .reply_bytes
            .fetch_add(self.written, Ordering::Relaxed);
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for BudgetStream<'_, S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        let remaining = 24 * 1024 - this.read;
        if remaining == 0 {
            return Poll::Ready(Err(io::Error::other("ingress read budget")));
        }
        let limit = remaining.min(buf.remaining());
        let mut limited = ReadBuf::new(buf.initialize_unfilled_to(limit));
        let result = Pin::new(&mut this.inner).poll_read(cx, &mut limited);
        let count = limited.filled().len();
        buf.advance(count);
        this.read += count;
        result
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for BudgetStream<'_, S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if bytes.is_empty() {
            return Poll::Ready(Ok(0));
        }
        let remaining = 8 * 1024 - this.written;
        if remaining == 0 {
            return Poll::Ready(Err(io::Error::other("ingress write budget")));
        }
        match Pin::new(&mut this.inner).poll_write(cx, &bytes[..bytes.len().min(remaining)]) {
            Poll::Ready(Ok(count)) => {
                this.written += count;
                Poll::Ready(Ok(count))
            },
            result => result,
        }
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}
