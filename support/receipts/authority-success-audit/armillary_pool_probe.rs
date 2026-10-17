// Read-only audit probe. Compile the unchanged production module directly.
#[path = "../../../crates/armillary/src/pool.rs"]
mod pool;

use pool::Pool;
use std::sync::mpsc;
use std::time::Duration;

const WAIT: Duration = Duration::from_millis(150);

fn actor_job(
    pool: &Pool,
    id: usize,
    started: mpsc::Sender<usize>,
    done: mpsc::Sender<usize>,
) -> mpsc::Sender<()> {
    let (release, released) = mpsc::channel();
    pool.submit(Box::new(move || {
        started.send(id).unwrap();
        let _ = released.recv();
        done.send(id).unwrap();
    }));
    release
}

#[test]
fn audit_queued_actors_can_share_an_unreserved_idle_worker_and_strand() {
    let mut observed = None;
    for attempt in 0..32 {
        let pool = Pool::new();
        let (warm, warmed) = mpsc::channel();
        pool.submit(Box::new(move || warm.send(()).unwrap()));
        warmed.recv_timeout(WAIT).unwrap();
        // Match the existing production reuse test's allowance for idle entry.
        std::thread::sleep(Duration::from_millis(50));
        let (start, started) = mpsc::channel();
        let (finish, finished) = mpsc::channel();
        let first = actor_job(&pool, 0, start.clone(), finish.clone());
        let second = actor_job(&pool, 1, start, finish);
        let initial = started.recv_timeout(WAIT).unwrap();
        let other = started.recv_timeout(WAIT);
        let workers = pool.workers();
        first.send(()).unwrap();
        second.send(()).unwrap();
        // Both jobs are released, including a queued one, before any assertion.
        finished.recv_timeout(WAIT).unwrap();
        finished.recv_timeout(WAIT).unwrap();
        if other.is_err() {
            assert_eq!(workers, 1);
            let later = started.recv_timeout(WAIT).unwrap();
            assert_ne!(initial, later);
            observed = Some((attempt, workers));
            break;
        }
    }
    assert!(observed.is_some(), "the bounded attempts did not observe starvation");
    println!("Idle-worker race observed at {observed:?}: second actor starts only after first is released; both jobs subsequently finish.");
}

#[test]
fn audit_busy_worker_control_starts_both_actors_before_release() {
    let pool = Pool::new();
    let (start, started) = mpsc::channel();
    let (finish, finished) = mpsc::channel();
    let first = actor_job(&pool, 0, start.clone(), finish.clone());
    assert_eq!(started.recv_timeout(WAIT).unwrap(), 0);
    let second = actor_job(&pool, 1, start, finish);
    let second_started = started.recv_timeout(WAIT);
    first.send(()).unwrap();
    second.send(()).unwrap();
    finished.recv_timeout(WAIT).unwrap();
    finished.recv_timeout(WAIT).unwrap();
    assert_eq!(second_started.unwrap(), 1);
    assert_eq!(pool.workers(), 2);
    println!("Busy-worker control starts both actors on two workers before either is released.");
}
