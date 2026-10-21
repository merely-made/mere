// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Independent synchronous consumer of the immutable published execution API.
use std::sync::Arc;
fn main() -> Result<(), armillary::ActorError> {
    let pool = armillary::Pool::new();
    let (actor, updates) =
        armillary::spawn_on::<u32, u32, _>(&pool, Arc::new(|| {}), |commands, out| {
            let mut sum = 0;
            while let Ok(value) = commands.recv() {
                sum += value;
                out.emit(sum);
            }
        });
    assert!(actor.command(2));
    assert!(actor.command(3));
    actor.join()?;
    assert_eq!(updates.iter().collect::<Vec<_>>(), vec![2, 5]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use armillary::actor::{self, ActorError, ActorHandle};
    use armillary::pool::Pool;
    use std::sync::{Arc, mpsc};
    use std::time::Duration;

    fn blocked_completion(pooled: bool) {
        let (started, entered) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let run = move |_: mpsc::Receiver<()>, _: actor::Emitter<()>| {
            started.send(()).unwrap();
            released.recv().unwrap();
        };
        let pool = Pool::new();
        let (actor, _) = if pooled {
            actor::spawn_on(&pool, Arc::new(|| {}), run)
        } else {
            actor::spawn(Arc::new(|| {}), run)
        };
        entered.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(!actor.is_finished());
        let (closed, observed) = mpsc::channel();
        let joiner = std::thread::spawn(move || closed.send(actor.join()).unwrap());
        let premature = observed.recv_timeout(Duration::from_millis(50));
        release.send(()).unwrap();
        assert!(premature.is_err(), "blocked actor cannot report joined");
        assert_eq!(
            observed.recv_timeout(Duration::from_secs(5)).unwrap(),
            Ok(())
        );
        joiner.join().unwrap();
    }

    #[test]
    fn dedicated_join_waits_for_the_actor_to_exit() {
        blocked_completion(false);
    }

    #[test]
    fn pooled_join_waits_for_the_actor_to_exit() {
        blocked_completion(true);
    }

    #[test]
    fn dedicated_and_pooled_panics_are_reported() {
        let pool = Pool::new();
        let (dedicated, _) = actor::spawn::<(), (), _>(Arc::new(|| {}), |_, _| panic!("control"));
        let (pooled, _) =
            actor::spawn_on::<(), (), _>(&pool, Arc::new(|| {}), |_, _| panic!("control"));
        assert_eq!(dedicated.join(), Err(ActorError::Panicked));
        assert_eq!(pooled.join(), Err(ActorError::Panicked));
    }

    #[test]
    fn actor_handles_retain_the_sync_contract() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ActorHandle<()>>();
        assert_send_sync::<Pool>();
    }

    #[test]
    fn last_pool_drop_preserves_actor_completion_then_retires_the_worker() {
        use std::cell::RefCell;

        struct ExitProbe(mpsc::Sender<()>);
        impl Drop for ExitProbe {
            fn drop(&mut self) {
                let _ = self.0.send(());
            }
        }
        thread_local! {
            static EXIT: RefCell<Option<ExitProbe>> = const { RefCell::new(None) };
        }

        let pool = Pool::new();
        let (exited, exit) = mpsc::channel();
        let (started, ready) = mpsc::channel();
        let (actor, updates) =
            actor::spawn_on::<u32, u32, _>(&pool, Arc::new(|| {}), move |commands, out| {
                EXIT.with(|probe| *probe.borrow_mut() = Some(ExitProbe(exited)));
                started.send(()).unwrap();
                while let Ok(value) = commands.recv() {
                    out.emit(value + 1);
                }
            });
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        drop(pool);
        assert!(actor.command(41));
        actor.join().unwrap();
        assert_eq!(updates.into_iter().collect::<Vec<_>>(), vec![42]);
        exit.recv_timeout(Duration::from_secs(5))
            .expect("last public pool handle must release its worker");
    }
}
