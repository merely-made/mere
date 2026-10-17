//! Real manifest resolver cache lifecycle with synthetic fetcher, no network.
use eidetic::{BlobFetcher,BlobManifest,BlobSource,Hash,ManifestId,MemoryBackend,NoFetcher,
    PrivacyClass,ProvenanceRecord,SchemaRef,Store,Timestamp,TrustEnvelope};
use eidetic::manifest::resolve_blob;
const BYTES:&[u8]=b"synthetic immutable payload";
#[derive(Default)]
struct Fetcher {calls:usize,offline:bool}
#[async_trait::async_trait(?Send)]
impl BlobFetcher for Fetcher {
    async fn fetch(&mut self,_source:&BlobSource)->eidetic::Result<Option<Vec<u8>>> {
        self.calls+=1;
        if self.offline {Err(eidetic::Error::new("synthetic offline"))} else {Ok(Some(BYTES.to_vec()))}
    }
}
fn cache_key()->String {format!("blob:{}",Hash::of(BYTES).to_hex())}
fn manifest(sources:Vec<BlobSource>)->BlobManifest {
    BlobManifest {id:ManifestId::of_blob(BYTES),schema:SchemaRef::from_id(ManifestId::of_blob(b"synthetic schema")),
        content_hash:Hash::of(BYTES),byte_size:BYTES.len() as u64,created_at:Timestamp(1),last_accessed:None,
        sources,privacy:PrivacyClass::LocalOnly,provenance:ProvenanceRecord::self_imported("synthetic fetcher"),
        trust:TrustEnvelope::self_asserted(),schema_metadata:serde_json::Value::Null,
        manifest_version:BlobManifest::CURRENT_VERSION}
}
fn remote()->BlobSource {BlobSource::Https {url:"https://synthetic.invalid/blob".into()}}

#[test]
fn fetched_cache_is_not_reused_without_a_matching_declared_local_source() {
    pollster::block_on(async {
        for sources in [vec![remote()],vec![BlobSource::Local{key:"blob:other-missing-key".into()},remote()]] {
            let mut store=MemoryBackend::default();let mut fetcher=Fetcher::default();let m=manifest(sources);
            assert_eq!(resolve_blob(&mut store,&mut fetcher,&m).await.unwrap(),BYTES);
            assert_eq!(fetcher.calls,1);
            assert_eq!(store.get(&cache_key()).await.unwrap().unwrap(),BYTES);
            fetcher.offline=true;
            assert!(resolve_blob(&mut store,&mut fetcher,&m).await.unwrap_err().to_string().contains("synthetic offline"));
            assert_eq!(fetcher.calls,2);
            assert_eq!(store.get(&cache_key()).await.unwrap().unwrap(),BYTES);
        }
        println!("OBSERVATION: successful remote resolution writes valid canonical cache bytes, but the next offline resolution fails without reading them for remote-only or nonmatching-local manifests");
    });
}

#[test]
fn matching_canonical_local_source_reuses_the_same_cache() {
    pollster::block_on(async {
        let mut store=MemoryBackend::default();let mut fetcher=Fetcher::default();
        let m=manifest(vec![BlobSource::Local{key:cache_key()},remote()]);
        assert_eq!(resolve_blob(&mut store,&mut fetcher,&m).await.unwrap(),BYTES);
        fetcher.offline=true;
        assert_eq!(resolve_blob(&mut store,&mut fetcher,&m).await.unwrap(),BYTES);
        assert_eq!(fetcher.calls,1);
        println!("CONTROL: the canonical Local source makes the warm read succeed without another fetch");
    });
}

#[test]
fn corrupt_declared_cache_bytes_are_rejected() {
    pollster::block_on(async {
        let mut store=MemoryBackend::default();store.put(&cache_key(),b"different bytes").await.unwrap();
        let m=manifest(vec![BlobSource::Local{key:cache_key()}]);
        assert!(resolve_blob(&mut store,&mut NoFetcher,&m).await.unwrap_err().to_string().contains("content hash mismatch"));
        println!("CONTROL: a declared cache hit still verifies its bytes against the manifest hash");
    });
}
