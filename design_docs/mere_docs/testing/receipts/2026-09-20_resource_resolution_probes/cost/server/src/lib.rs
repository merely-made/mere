//! A keep-alive byte-range server that counts connections and requests, so a
//! client's connection reuse is visible. Std only.

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

pub const OBJECT_BYTES: usize = 3_000_000;
pub const CHUNK: usize = 64 * 1024;
pub const READS: usize = 300;

pub fn object() -> Vec<u8> {
    (0..OBJECT_BYTES).map(|index| (index % 251) as u8).collect()
}

/// The scattered offsets every variant reads, in the same order.
pub fn offsets() -> impl Iterator<Item = usize> {
    (0..READS).map(|step| (step * 7_919 * 13) % (OBJECT_BYTES - CHUNK))
}

pub struct Server {
    pub address: SocketAddr,
    pub connections: Arc<AtomicUsize>,
    pub requests: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
}

impl Server {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let connections = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let data = Arc::new(object());
        {
            let (connections, requests, stop) = (connections.clone(), requests.clone(), stop.clone());
            thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            connections.fetch_add(1, Ordering::Relaxed);
                            stream.set_nonblocking(false).unwrap();
                            stream.set_nodelay(true).unwrap();
                            let (data, requests) = (data.clone(), requests.clone());
                            thread::spawn(move || serve(stream, &data, &requests));
                        },
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(1));
                        },
                        Err(error) => panic!("cost server failed: {error}"),
                    }
                }
            });
        }
        Self {
            address,
            connections,
            requests,
            stop,
        }
    }

    pub fn url(&self) -> String {
        format!("http://{}/episode.mp3", self.address)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn serve(mut stream: TcpStream, data: &[u8], requests: &AtomicUsize) {
    let mut pending = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        // One request per header block; these requests carry no bodies.
        let end = loop {
            if let Some(end) = pending.windows(4).position(|window| window == b"\r\n\r\n") {
                break end + 4;
            }
            match stream.read(&mut buffer) {
                Ok(0) | Err(_) => return,
                Ok(count) => pending.extend_from_slice(&buffer[..count]),
            }
        };
        let head = String::from_utf8_lossy(&pending[..end]).into_owned();
        pending.drain(..end);
        requests.fetch_add(1, Ordering::Relaxed);
        let range = head
            .lines()
            .filter_map(|line| line.split_once(':'))
            .find_map(|(name, value)| name.eq_ignore_ascii_case("range").then(|| value.trim().to_owned()))
            .and_then(|value| value.strip_prefix("bytes=").map(str::to_owned))
            .and_then(|value| {
                let (start, end) = value.split_once('-')?;
                Some((start.parse::<usize>().ok()?, end.parse::<usize>().ok()?))
            });
        let Some((start, end)) = range else {
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
            continue;
        };
        let end = end.min(data.len() - 1);
        let body = &data[start..=end];
        let head = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {start}-{end}/{}\r\nContent-Type: audio/mpeg\r\nETag: \"v1\"\r\nConnection: keep-alive\r\n\r\n",
            body.len(),
            data.len()
        );
        if stream.write_all(head.as_bytes()).is_err() || stream.write_all(body).is_err() {
            return;
        }
    }
}

/// Median and 95th percentile of per-read latencies, in microseconds.
pub fn summarize(mut micros: Vec<u128>) -> (u128, u128) {
    micros.sort_unstable();
    (micros[micros.len() / 2], micros[micros.len() * 95 / 100])
}
