//! R3-B: held and partial episodes in iroh's blob store, read the way a decoder
//! reads: blocking `Read + Seek` from its own thread.
//! This file is the regression manifest. Nothing here is a proposed API.

#![cfg(test)]

use std::{
    io::{self, Read, Seek, SeekFrom},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use bytes::Bytes;
use iroh_blobs::{
    Hash,
    api::{Store, blobs::{AddProgressItem, BlobReader}},
    protocol::{ChunkRanges, ChunkRangesExt},
    store::{fs::FsStore, mem::MemStore},
};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt},
    runtime::Runtime,
};
use tokio_stream::{StreamExt, wrappers::ReceiverStream};

const OBJECT_BYTES: usize = 3_000_000;
const CHUNK: usize = 64 * 1024;

fn object() -> Vec<u8> {
    (0..OBJECT_BYTES).map(|index| (index % 251) as u8).collect()
}

fn runtime() -> Arc<Runtime> {
    Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap(),
    )
}

/// The decoder-side bridge: Symphonia wants `Read + Seek`; the store's reader is
/// async. One `block_on` per call, from a thread that is not a runtime worker.
struct HeldSource {
    runtime: Arc<Runtime>,
    reader: BlobReader,
    /// From the store's bitfield. The reader itself refuses `SeekFrom::End`.
    size: u64,
}

impl Read for HeldSource {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.runtime.block_on(self.reader.read(output))
    }
}

impl Seek for HeldSource {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let from = match from {
            SeekFrom::End(offset) => SeekFrom::Start(
                self.size
                    .checked_add_signed(offset)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek before start"))?,
            ),
            other => other,
        };
        self.runtime.block_on(self.reader.seek(from))
    }
}

#[test]
fn a_streamed_import_has_no_identity_until_its_last_byte() {
    let runtime = runtime();
    let data = object();
    runtime.block_on(async {
        let store = MemStore::new();
        let (sender, receiver) = tokio::sync::mpsc::channel::<io::Result<Bytes>>(64);
        let progress = store.add_stream(ReceiverStream::new(receiver)).await;
        let mut progress = Box::pin(progress.stream().await);

        // Half the episode has arrived, as in a download in flight.
        for chunk in data[..OBJECT_BYTES / 2].chunks(CHUNK) {
            sender.send(Ok(Bytes::copy_from_slice(chunk))).await.unwrap();
        }
        let early = tokio::time::timeout(Duration::from_millis(500), async {
            loop {
                match progress.next().await {
                    Some(AddProgressItem::Done(tag)) => break Some(tag.hash()),
                    Some(_) => continue,
                    None => break None,
                }
            }
        })
        .await;
        assert!(early.is_err(), "no hash may exist while bytes are still arriving");
        // Nothing addressable exists yet: the address is the hash of all bytes.
        assert!(!store.has(Hash::new(&data)).await.unwrap());

        for chunk in data[OBJECT_BYTES / 2..].chunks(CHUNK) {
            sender.send(Ok(Bytes::copy_from_slice(chunk))).await.unwrap();
        }
        drop(sender);
        let tag = loop {
            match progress.next().await {
                Some(AddProgressItem::Done(tag)) => break tag,
                Some(AddProgressItem::Error(error)) => panic!("import failed: {error}"),
                Some(_) => continue,
                None => panic!("import ended without a tag"),
            }
        };
        assert_eq!(tag.hash(), Hash::new(&data));
        assert!(store.has(tag.hash()).await.unwrap());
    });
    println!("R3-B import: a streamed import is unaddressable until complete; a download in flight cannot be played from the store");
}

#[test]
fn a_held_episode_seeks_from_a_blocking_thread_and_survives_reopen() {
    let runtime = runtime();
    let data = Arc::new(object());
    let directory = tempfile::tempdir().unwrap();

    let hash = runtime.block_on(async {
        let store = FsStore::load(directory.path()).await.unwrap();
        let tag = store.add_bytes(data.as_ref().clone()).with_named_tag("episode").await.unwrap();
        store.shutdown().await.unwrap();
        tag.hash
    });

    // A second process lifetime: the resident reopening its store.
    let store = runtime.block_on(FsStore::load(directory.path())).unwrap();
    let blobs: Store = (*store).clone();
    let (reads, elapsed) = thread::spawn({
        let (runtime, data) = (runtime.clone(), data.clone());
        move || {
            // Negative, pinned 2026-09-20: iroh-blobs 0.103's reader refuses
            // `SeekFrom::End`, which demuxers use, so the bridge translates it.
            let mut raw = blobs.reader(hash);
            assert!(runtime.block_on(raw.seek(SeekFrom::End(0))).is_err());
            let size = runtime.block_on(async { blobs.observe(hash).await }).unwrap().size();
            let mut source = HeldSource {
                runtime,
                reader: blobs.reader(hash),
                size,
            };
            assert_eq!(source.seek(SeekFrom::End(0)).unwrap(), OBJECT_BYTES as u64);
            assert_eq!(source.seek(SeekFrom::End(-1)).unwrap(), OBJECT_BYTES as u64 - 1);
            let mut buffer = vec![0_u8; CHUNK];
            let started = Instant::now();
            let mut reads = 0_u32;
            // A decoder's pattern: scattered seeks, then a chunk each.
            for step in 0..200_u64 {
                let offset = (step * 7_919 * 13) % (OBJECT_BYTES - CHUNK) as u64;
                source.seek(SeekFrom::Start(offset)).unwrap();
                source.read_exact(&mut buffer).unwrap();
                assert_eq!(&buffer[..], &data[offset as usize..offset as usize + CHUNK]);
                reads += 1;
            }
            (reads, started.elapsed())
        }
    })
    .join()
    .unwrap();
    runtime.block_on(store.shutdown()).unwrap();
    println!(
        "R3-B held: {reads} seek+read of {CHUNK} bytes after reopen, {:.0} us each (fs store, {} build)",
        elapsed.as_micros() as f64 / f64::from(reads),
        if cfg!(debug_assertions) { "debug" } else { "release" }
    );
}

#[test]
fn a_partial_blob_serves_present_ranges_and_refuses_absent_ones() {
    let runtime = runtime();
    let data = object();
    runtime.block_on(async {
        // A holder with the whole episode, as a peer or the resident would be.
        let holder = MemStore::new();
        let hash = holder.add_bytes(data.clone()).await.unwrap().hash;

        // A receiver that knows the hash up front and has the first mebibyte.
        let receiver = MemStore::new();
        let head = ChunkRanges::bytes(0..1_048_576_u64);
        let bao = holder.export_bao(hash, head.clone()).bao_to_vec().await.unwrap();
        receiver.import_bao_bytes(hash, head, bao).await.unwrap();
        let bitfield = receiver.observe(hash).await.unwrap();
        assert!(!bitfield.is_complete());
        assert_eq!(bitfield.size(), OBJECT_BYTES as u64);

        let mut buffer = vec![0_u8; CHUNK];
        let mut reader = receiver.reader(hash);
        reader.seek(SeekFrom::Start(500_000)).await.unwrap();
        reader.read_exact(&mut buffer).await.unwrap();
        assert_eq!(&buffer[..], &data[500_000..500_000 + CHUNK]);

        // Negative: a range that has not arrived is an error, not a wait and not zeros.
        let mut reader = receiver.reader(hash);
        reader.seek(SeekFrom::Start(2_000_000)).await.unwrap();
        assert!(reader.read_exact(&mut buffer).await.is_err());

        // The rest arrives; the same reader shape now serves the far range.
        let tail = ChunkRanges::bytes(1_048_576_u64..);
        let bao = holder.export_bao(hash, tail.clone()).bao_to_vec().await.unwrap();
        receiver.import_bao_bytes(hash, tail, bao).await.unwrap();
        assert!(receiver.observe(hash).await.unwrap().is_complete());
        let mut reader = receiver.reader(hash);
        reader.seek(SeekFrom::Start(2_000_000)).await.unwrap();
        reader.read_exact(&mut buffer).await.unwrap();
        assert_eq!(&buffer[..], &data[2_000_000..2_000_000 + CHUNK]);
    });
    println!("R3-B partial: with the hash known up front, present ranges read, absent ranges error, and completion needs no new reader shape");
}
