
#![allow(dead_code)]
use ::std::{error::Error,fs::OpenOptions,io,io::Write,path::{Path,PathBuf},sync::{Arc,atomic::{AtomicBool,Ordering}}};
use tokio::sync::Mutex;
use ::std::time::Duration;
#[derive(serde::Serialize,serde::Deserialize)]struct ControllerCounter{last_used:u64}
static READS: ::std::sync::OnceLock<::std::sync::Barrier> = ::std::sync::OnceLock::new();
static FIRST_DONE:(::std::sync::Mutex<bool>,::std::sync::Condvar)=(::std::sync::Mutex::new(false),::std::sync::Condvar::new());
// Delay only the return of an actual disk read. Both callers have read the
// same existing counter, and B resumes after A has completely committed.
mod std {
 pub use ::std::*;
 pub mod fs {
  pub fn read_to_string(p:impl AsRef<::std::path::Path>)->::std::io::Result<String>{
   let result=::std::fs::read_to_string(p);
   if let Some(barrier)=crate::READS.get(){
    barrier.wait();
    if ::std::thread::current().name()==Some("counter-B"){
     let (lock,wake)=&crate::FIRST_DONE;
     let mut done=lock.lock().unwrap();while !*done {done=wake.wait(done).unwrap();}
    }
   }
   result
  }
  pub use ::std::fs::rename;
 }
}
fn reserve_next_counter(authority_root: &Path) -> Result<u64, Box<dyn Error>> {
    let path = controller_counter_path(authority_root);
    let last_used = match std::fs::read_to_string(&path) {
        Ok(text) => {
            let record: ControllerCounter = serde_json::from_str(&text).map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "invalid first-owner controller counter at {}: {error}",
                        path.display()
                    ),
                )
            })?;
            record.last_used
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => 0,
        Err(error) => return Err(error.into()),
    };
    let next = last_used
        .checked_add(1)
        .ok_or("first-owner controller counter is exhausted")?;
    let temporary = path.with_extension("json.tmp");
    let contents = serde_json::to_vec(&ControllerCounter { last_used: next })?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temporary)?;
    file.write_all(&contents)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&temporary, &path)?;
    Ok(next)
}

fn controller_counter_path(authority_root: &Path) -> PathBuf {
    authority_root.join("first-owner-controller-counter.json")
}

#[derive(Debug)]enum SitedStationError{Stopped,Station(()),Denied}
type Event=();type RemoteAuthRevocationOutcome=();
struct FakeStation;
impl FakeStation{async fn next_event(&mut self)->Option<Event>{::std::future::pending().await}}
#[derive(Clone)]struct Lease(Arc<AtomicBool>);
impl Lease{
 fn close(&self){self.0.store(true,Ordering::SeqCst);}
 fn revoke(&self)->Result<RemoteAuthRevocationOutcome,SitedStationError>{self.close();Ok(())}
}
struct SitedStation{station:Arc<Mutex<Option<FakeStation>>>,lease:Lease,watchdog:tokio::task::JoinHandle<()>}
fn remaining_window(a:u64,b:u64)->Duration{Duration::from_millis(b-a)}
impl SitedStation {
 fn authorize_now(&self)->Result<(u64,u64),SitedStationError>{if self.lease.0.load(Ordering::SeqCst){Err(SitedStationError::Denied)}else{Ok((0,3_600_000))}}
 async fn stop_after_deadline<T>(&self)->Result<T,SitedStationError>{Err(SitedStationError::Denied)}
    pub async fn next_event(&self) -> Result<Event, SitedStationError> {
        let (now_ms, expires_at_ms) = self.authorize_now()?;
        let window = remaining_window(now_ms, expires_at_ms);
        let result = {
            let mut station = self.station.lock().await;
            let station = station.as_mut().ok_or(SitedStationError::Stopped)?;
            tokio::time::timeout(window, station.next_event()).await
        };
        match result {
            Ok(Some(event)) => Ok(event),
            Ok(None) => Err(SitedStationError::Stopped),
            Err(_) => self.stop_after_deadline().await,
        }
    }

    pub async fn stop(&self) {
        self.lease.close();
        self.watchdog.abort();
        self.station.lock().await.take();
    }

    pub async fn revoke(&self) -> Result<RemoteAuthRevocationOutcome, SitedStationError> {
        let outcome = self.lease.revoke()?;
        self.watchdog.abort();
        self.station.lock().await.take();
        Ok(outcome)
    }


}
#[test]
fn concurrent_counter_reservations_both_succeed_with_same_value(){
 let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("counter-fixture");assert!(!root.exists());::std::fs::create_dir(&root).unwrap();
 ::std::fs::write(controller_counter_path(&root),br#"{"last_used":0}"#).unwrap();
 READS.set(::std::sync::Barrier::new(2)).unwrap();
 let a=root.clone();let first=::std::thread::Builder::new().name("counter-A".into()).spawn(move||{
  let result=reserve_next_counter(&a).map_err(|e|e.to_string());
  let(lock,wake)=&FIRST_DONE;*lock.lock().unwrap()=true;wake.notify_one();result
 }).unwrap();
 let b=root.clone();let second=::std::thread::Builder::new().name("counter-B".into()).spawn(move||reserve_next_counter(&b).map_err(|e|e.to_string())).unwrap();
 let a=first.join().unwrap().unwrap();let b=second.join().unwrap().unwrap();assert_eq!((a,b),(1,1));
 let raw=::std::fs::read(controller_counter_path(&root)).unwrap();assert_eq!(serde_json::from_slice::<ControllerCounter>(&raw).unwrap().last_used,1);
 // B's successful reservation left no remaining record of A's spend.
 ::std::fs::remove_file(controller_counter_path(&root)).unwrap();::std::fs::remove_dir(&root).unwrap();
 println!("counter-A=1 counter-B=1 disk=1; both actual synced writes and renames succeeded");
}
async fn cancellation_case(revoking:bool){
 let owner=Arc::new(SitedStation{station:Arc::new(Mutex::new(Some(FakeStation))),lease:Lease(Arc::new(AtomicBool::new(false))),watchdog:tokio::spawn(::std::future::pending())});
 let waiter=owner.clone();let task=tokio::spawn(async move{waiter.next_event().await});tokio::task::yield_now().await;
 assert!(owner.station.try_lock().is_err(),"actual next_event holds mutex");
 let action=async{if revoking {owner.revoke().await.map(|_|())}else{owner.stop().await;Ok(())}};
 assert!(tokio::time::timeout(Duration::from_millis(10),action).await.is_err());
 assert!(owner.lease.0.load(Ordering::SeqCst));assert!(!task.is_finished());
 assert!(owner.station.try_lock().is_err());
 // Releasing the simulated radio wait is sufficient for the original stop.
 task.abort();let _=task.await;owner.stop().await;assert!(owner.station.lock().await.is_none());
 println!("{} closed lease but could not cancel pending event; explicit task abort releases mutex",if revoking{"revoke"}else{"stop"});
}
#[tokio::test(start_paused=true)]async fn stop_cannot_interrupt_in_flight_event(){cancellation_case(false).await;}
#[tokio::test(start_paused=true)]async fn successful_revoke_cannot_interrupt_in_flight_event(){cancellation_case(true).await;}
#[tokio::test(start_paused=true)]async fn idle_stop_control_completes(){
 let owner=SitedStation{station:Arc::new(Mutex::new(Some(FakeStation))),lease:Lease(Arc::new(AtomicBool::new(false))),watchdog:tokio::spawn(::std::future::pending())};
 owner.stop().await;assert!(owner.station.lock().await.is_none());assert!(owner.lease.0.load(Ordering::SeqCst));
}
