# R3 resource resolution probes, 2026-09-20

Three disposable probes for lane R3 of the
[family composition thesis](../../../../2026-08-12_family_composition_thesis_brief.md).
Native Windows x86_64, `rustc 1.97.1 (8bab26f4f 2026-07-14)`. Real netfetcher at
genet `5ae30cad` (the revision mere, woodshed and turnstone all pin; unchanged
at genet HEAD `9976945058b`), real `iroh-blobs` 0.103.0 (mere-transport's pin),
real ureq 3.4 as `ports/redshank/Cargo.toml` declares it. Every server is an
in-process loopback HTTP server: **no TLS, no real network, no Redshank or
Turnstone code in the loop.** Each `src` is the regression manifest; logs are
beside it. To replay, run Cargo on each manifest with `--locked` from outside
any workspace, with its own target directory.

## R3-A, ranged read on a persona-scoped context: 6 of 6 pass

`ranged/`, `cargo test -- --test-threads=1`, debug profile.

- A plain thread blocking on a shared tokio runtime, the decoder's calling
  shape, reads a probe byte and three scattered 64 KiB ranges: 206 each, exact
  `Content-Range`, exact bytes, `Accept-Encoding: identity` on every request,
  4 requests and 196,609 of 3,000,000 bytes (6.6%, the figure Redshank's own
  reader reports). Destination `audio` over `http://` with no initiator is
  neither blocked nor upgraded.
- A changed validator with `If-Range` comes back as a whole 200 with the new
  ETag, so the caller can see the representation moved.
- A 302 keeps `Range` and identity encoding, and `url_list` ends at the final URL.
- A cookie set by a page fetch on persona A's jar is sent by a later ranged read
  built from a fresh context over the same jar; persona B's ranged read sends
  none and B's jar stays empty.
- A whole body is observed streaming: an 8 KiB first chunk arrives while the
  server still holds half the object, and the largest chunk is about 0.5 MiB.
- **Negative control, pinned:** after a whole GET has cached the object, a
  ranged request in default cache mode never reaches the network and returns
  **200 with all 3,000,000 bytes**. The cache key ignores `Range`. The same
  request as no-store reaches the network and gets its 206. HTTP permits this
  (RFC 9110 section 14.2 lets any server ignore `Range`), so it is not a
  conformance defect, but a caller asking for 64 KiB gets the whole episode and
  a reader that insists on 206 fails. Until netfetcher's cache answers such a
  request with the stored range, a ranged caller sends no-store. Recorded here,
  not changed.

## R3-B, held and partial episodes in iroh's store: 3 of 3 pass

`held/`, debug profile plus one release timing.

- A streamed import (`add_stream`) yields no hash and nothing addressable while
  bytes are still arriving; the blob exists only after the last byte. **A
  progressive HTTP download therefore cannot be played out of the store while it
  downloads.** The address is the BLAKE3 of all bytes, which a feed never gives.
- A complete blob in the fs store survives shutdown and reopen, and a blocking
  `Read + Seek` bridge from a plain thread does 200 scattered seek-and-read of
  64 KiB correctly: 9.5 ms each in debug, **2.8 ms each in release**. Slow next
  to a bare file, far above what audio needs.
- **Negative, pinned:** the store's reader refuses `SeekFrom::End`, which
  demuxers use. The bridge translates it from the size in the blob's bitfield.
- With the hash known up front, as when a peer or the resident already holds the
  episode, a partial blob reads its present range, returns an error (not a wait,
  not zeros) for an absent one, and serves the far range once the rest is
  imported, through the same reader.

## R3-C, standalone cost

`cost/`, release profile, cold builds in separate target directories, three runs
each of a probe plus 300 scattered 64 KiB reads against a keep-alive server that
counts connections. Latency is loopback and excludes TLS.

| Variant | Median per read | 95th | Connections for 301 requests | Cold release build | Binary |
|---|---|---|---|---|---|
| ureq free function, **what Redshank calls today** | 1.50 ms | 1.70 ms | **301** | 15 s | 2.2 MB |
| ureq, one shared agent | 0.10 to 0.12 ms | 0.18 to 0.20 ms | 1 | same build | same |
| netfetcher, `hyper-transport` only | 0.12 to 0.14 ms | 0.19 to 0.22 ms | 1 | 47 s | 5.3 MB |
| netfetcher, default features (adds HTTP/3, WebSocket) | 0.13 to 0.14 ms | 0.20 to 0.22 ms | 1 | 59 s | 6.7 MB |

- **Side finding in Redshank itself:** `ureq::get` runs on a use-once agent
  (`ureq-3.4.0/src/lib.rs:615-620`), so `redshank-playback`'s range reader opens
  a new connection for every 64 KiB chunk. Over `https` that is also a TLS
  handshake per chunk, which this loopback probe does not show. One shared agent
  removes it, with no change of client.
- Lockfile delta against `woodshed/ports/redshank/Cargo.lock` (737 packages),
  by crate name (`cost/lock_delta.json`): dropping ureq removes 6 crates (ureq,
  ureq-proto, base64, httparse, utf8-zero, webpki-roots). Netfetcher with
  `hyper-transport` alone adds 24 (hyper and its utilities, h2, the compression
  codecs, cookie, psl, netfetcher); default features add 47 (quinn, h3,
  tungstenite and friends on top). The "new versions" rows in the JSON come from
  a fresh resolve taking newer patch releases and would unify in a real lock;
  they are not a cost.
- What the simple client skips is by construction, not measured here: it has no
  access to the persona's cookie jar, HSTS or Alt-Svc state, the host's HTTP
  cache, its redirect limit, or its transport seam.

## What this does not show

No HTTPS, no real CDN behaviour, no Symphonia decoding through either bridge, no
Redshank or Turnstone integration, no resident process boundary, no process-kill
recovery of the fs store, no wasm. R3-C's latencies are loopback numbers and say
nothing about a WAN.
