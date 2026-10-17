//! Compare exact built-in schema bytes with their real typed bindings.
use eidetic::{AdapterRuntimeCompat,Hash,ManifestId,MemoryBackend,ModelAdapterManifest,NoFetcher,
    SchemaDefinition,TypedPayload,validate_payload};
use eidetic::models::{OpaqueBlob,OPAQUE_BLOB_SCHEMA_REF,TrainingCorpus};
fn id(b:&[u8])->ManifestId {ManifestId::of_blob(b)}
fn adapter()->ModelAdapterManifest {
    ModelAdapterManifest {name:"synthetic".into(),base_model_ref:id(b"base"),adapter_blob:id(b"weights"),
        adapter_config_blob:id(b"config"),adapter_format:"peft-lora".into(),adapter_format_version:"1".into(),
        runtime_compat:AdapterRuntimeCompat::default(),rank:1,alpha:1.0,target_modules:vec!["q_proj".into()],
        tokenizer_ref:id(b"tokenizer"),prompt_template_hash:Hash::of(b"template"),quantization_assumption:None,
        training_corpus_root:None,training_method:serde_json::json!({"source":"synthetic"}),eval_results:None}
}

#[test]
fn adapter_definition_is_not_parseable_by_its_declared_native_validator() {
    let bytes=include_bytes!("builtin_adapter_schema.json");
    assert_eq!(Hash::of(bytes),ModelAdapterManifest::schema_ref().0.0);
    let definition:SchemaDefinition=serde_json::from_slice(bytes).unwrap();
    let value=adapter();value.validate().unwrap();
    let error=validate_payload(&definition,&value.serialize_to_bytes().unwrap()).unwrap_err();
    assert!(error.to_string().contains("mere-native schema body parse"));
    assert!(error.to_string().contains("unknown variant"));
    println!("OBSERVATION: a domain-valid adapter cannot use its exact declared schema: {error}");
}

#[test]
fn raw_opaque_binding_does_not_conform_to_its_json_object_definition() {
    let bytes=include_bytes!("builtin_opaque_schema.json");
    assert_eq!(Hash::of(bytes),OPAQUE_BLOB_SCHEMA_REF.0.0);
    let definition:SchemaDefinition=serde_json::from_slice(bytes).unwrap();
    let value=OpaqueBlob(b"raw synthetic weights".to_vec());
    let raw=value.serialize_to_bytes().unwrap();assert_eq!(raw,b"raw synthetic weights");
    let error=validate_payload(&definition,&raw).unwrap_err();
    assert!(error.to_string().contains("payload parse (mere-native)"));
    println!("OBSERVATION: the raw opaque binding is rejected by its exact declared object/bytes-string schema: {error}");
}

#[test]
fn current_training_corpus_binding_conforms_to_its_exact_schema() {
    let bytes=include_bytes!("builtin_training_schema.json");
    assert_eq!(Hash::of(bytes),TrainingCorpus::schema_ref().0.0);
    let definition:SchemaDefinition=serde_json::from_slice(bytes).unwrap();
    let value=TrainingCorpus {training_source_codicils:vec![id(b"train")],evaluation_source_codicils:vec![id(b"eval")]};
    validate_payload(&definition,&value.serialize_to_bytes().unwrap()).unwrap();
    println!("CONTROL: the current training corpus binding validates against its own exact schema");
}

#[test]
fn bootstrap_does_not_make_unknown_built_in_definitions_present() {
    pollster::block_on(async {
        let mut store=MemoryBackend::default();eidetic::bootstrap(&mut store).await.unwrap();
        let reference=ModelAdapterManifest::schema_ref();
        assert!(eidetic::load_schema(&mut store,&mut NoFetcher,reference.0).await.unwrap().is_none());
        eidetic::validate_against_schema(&mut store,&mut NoFetcher,reference,b"not JSON").await.unwrap();
        println!("CONTROL: meta bootstrap leaves adapter definition absent; the documented tolerant validator skips its payload rather than proving validity");
    });
}
