//! Research adapter over real Muniment/redb, not a production custody API.
use muniment::{Backend, BlobStore, Hash, RedbBackend, WriteOp};

struct Custody {
    store: BlobStore<RedbBackend>,
}

impl Custody {
    fn new(backend: RedbBackend) -> Self { Self { store: BlobStore::new(backend) } }
    fn key(hash: Hash, class: &str, owner: &str) -> String {
        format!("probe/ref/{hash}/{class}/{owner}")
    }
    async fn claim(&mut self, hash: Hash, class: &str, owner: &str) {
        assert!(self.store.has(&hash).await.unwrap());
        self.store.backend().put(&Self::key(hash, class, owner), b"1").await.unwrap();
    }
    async fn transfer(&mut self, hash: Hash, from: (&str, &str), to: (&str, &str)) {
        let from = Self::key(hash, from.0, from.1);
        let to = Self::key(hash, to.0, to.1);
        assert!(self.store.backend().get(&from).await.unwrap().is_some());
        if from == to { return; }
        self.store.backend().apply(&[
            WriteOp::Put { key: to, value: b"1".to_vec() },
            WriteOp::Delete { key: from },
        ]).await.unwrap();
    }
    async fn release(&mut self, hash: Hash, class: &str, owner: &str) {
        self.store.backend().delete(&Self::key(hash,class,owner)).await.unwrap();
    }
    // Caller serializes all custody commands through this owner. Backend apply
    // alone does not atomically cover this read plus delete.
    async fn apply_collection(&mut self, hash: Hash) -> bool {
        if !self.store.backend().list(&format!("probe/ref/{hash}/")).await.unwrap().is_empty() {
            return false;
        }
        self.store.backend().delete(&format!("blob/{hash}")).await.unwrap();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owner_specific_transfer_and_reopen_preserve_equal_blob() {
        pollster::block_on(async {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("custody.redb");
            let hash;
            {
                let mut c = Custody::new(RedbBackend::open(&path).unwrap());
                hash = c.store.put(b"same captured bytes").await.unwrap();
                assert_eq!(hash, c.store.put(b"same captured bytes").await.unwrap());
                assert_eq!(c.store.backend().list("blob/").await.unwrap().len(),1);
                c.claim(hash,"capture","a").await;
                c.claim(hash,"capture","b").await;
                c.transfer(hash,("capture","a"),("recycle","a")).await;
                c.release(hash,"capture","b").await;
                assert!(!c.apply_collection(hash).await);
            }
            let mut c = Custody::new(RedbBackend::open(&path).unwrap());
            assert_eq!(c.store.get(&hash).await.unwrap().as_deref(),Some(b"same captured bytes".as_slice()));
            assert!(c.store.backend().get(&Custody::key(hash,"recycle","a")).await.unwrap().is_some());
            c.transfer(hash,("recycle","a"),("capture","a")).await;
            c.transfer(hash,("capture","a"),("capture","a")).await;
            assert!(!c.apply_collection(hash).await);
            c.release(hash,"capture","a").await;
            assert!(c.apply_collection(hash).await);
            assert!(!c.store.has(&hash).await.unwrap());
        });
    }

    #[test]
    fn stale_collection_proposal_rechecks_new_owner() {
        pollster::block_on(async {
            let dir=tempfile::tempdir().unwrap();
            let mut c=Custody::new(RedbBackend::open(dir.path().join("stale.redb")).unwrap());
            let hash=c.store.put(b"claimed after proposal").await.unwrap();
            let proposal=hash;
            assert!(c.store.backend().list(&format!("probe/ref/{hash}/")).await.unwrap().is_empty());
            c.claim(hash,"capture","new").await;
            assert!(!c.apply_collection(proposal).await);
            assert!(c.store.has(&hash).await.unwrap());
        });
    }

    #[test]
    fn negative_control_atomic_delete_batch_does_not_recheck_owner() {
        pollster::block_on(async {
            let dir=tempfile::tempdir().unwrap();
            let mut c=Custody::new(RedbBackend::open(dir.path().join("broken.redb")).unwrap());
            let hash=c.store.put(b"new owner cannot save stale batch").await.unwrap();
            let proposed=WriteOp::Delete {key:format!("blob/{hash}")};
            c.claim(hash,"capture","new").await;
            c.store.backend().apply(&[proposed]).await.unwrap();
            assert!(!c.store.has(&hash).await.unwrap(),"negative control must expose loss");
            assert!(c.store.backend().get(&Custody::key(hash,"capture","new")).await.unwrap().is_some());
        });
    }
}
