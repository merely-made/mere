//! R3-C: the ranged read on ureq, twice: the free function Redshank calls
//! today (a use-once agent per request) and one shared agent.

use std::{sync::atomic::Ordering, time::Instant};

use cost_server::{CHUNK, Server, object, offsets, summarize};

fn run(label: &str, fetch: impl Fn(&str, &str) -> Vec<u8>) {
    let server = Server::start();
    let data = object();
    let url = server.url();
    assert_eq!(fetch(&url, "bytes=0-0").len(), 1);
    let mut micros = Vec::new();
    for offset in offsets() {
        let started = Instant::now();
        let bytes = fetch(&url, &format!("bytes={offset}-{}", offset + CHUNK - 1));
        micros.push(started.elapsed().as_micros());
        assert_eq!(&bytes[..], &data[offset..offset + CHUNK]);
    }
    let (median, p95) = summarize(micros);
    println!(
        "{{\"variant\":\"{label}\",\"median_us\":{median},\"p95_us\":{p95},\"connections\":{},\"requests\":{}}}",
        server.connections.load(Ordering::Relaxed),
        server.requests.load(Ordering::Relaxed)
    );
}

fn main() {
    run("ureq free function (Redshank today)", |url, range| {
        ureq::get(url)
            .header("Accept-Encoding", "identity")
            .header("Range", range)
            .call()
            .unwrap()
            .body_mut()
            .read_to_vec()
            .unwrap()
    });
    let agent = ureq::Agent::new_with_defaults();
    run("ureq shared agent", move |url, range| {
        agent
            .get(url)
            .header("Accept-Encoding", "identity")
            .header("Range", range)
            .call()
            .unwrap()
            .body_mut()
            .read_to_vec()
            .unwrap()
    });
}
