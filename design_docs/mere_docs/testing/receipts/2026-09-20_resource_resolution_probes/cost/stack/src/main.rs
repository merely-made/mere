//! R3-C: the same ranged read through netfetcher, blocking from the main thread.

use std::{sync::atomic::Ordering, time::Instant};

use cost_server::{CHUNK, Server, object, offsets, summarize};
use netfetcher::{CacheMode, Destination, FetchContext, Request};

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let context = FetchContext::permissive();
    let server = Server::start();
    let data = object();
    let url = url::Url::parse(&server.url()).unwrap();
    let fetch = |range: String| {
        let mut request = Request::get(url.clone()).with_destination(Destination::Audio);
        request.headers.push(("range".to_owned(), range));
        request.cache = CacheMode::NoStore;
        let response = runtime.block_on(netfetcher::fetch(request, &context));
        assert_eq!(response.status, 206);
        runtime.block_on(response.bytes()).unwrap()
    };
    assert_eq!(fetch("bytes=0-0".to_owned()).len(), 1);
    let mut micros = Vec::new();
    for offset in offsets() {
        let started = Instant::now();
        let bytes = fetch(format!("bytes={offset}-{}", offset + CHUNK - 1));
        micros.push(started.elapsed().as_micros());
        assert_eq!(&bytes[..], &data[offset..offset + CHUNK]);
    }
    let (median, p95) = summarize(micros);
    println!(
        "{{\"variant\":\"{}\",\"median_us\":{median},\"p95_us\":{p95},\"connections\":{},\"requests\":{}}}",
        env!("CARGO_PKG_NAME"),
        server.connections.load(Ordering::Relaxed),
        server.requests.load(Ordering::Relaxed)
    );
}
