//! Actual public URL-cache replacement semantics; synthetic memory only.
use eidetic::MemoryBackend;
use eidetic::content_store::{StoredContent,save_content,load_content};
const URL:&str="https://synthetic.invalid/cache";
fn content(body:&[u8],kind:Option<&str>)->StoredContent {
    StoredContent {body:body.to_vec(),content_type:kind.map(str::to_owned)}
}

#[test]
fn replacing_a_typed_response_with_none_retains_the_old_content_type() {
    pollster::block_on(async {
        let mut store=MemoryBackend::default();
        save_content(&mut store,URL,&content(b"old HTML",Some("text/html"))).await.unwrap();
        save_content(&mut store,URL,&content(b"new untyped bytes",None)).await.unwrap();
        let loaded=load_content(&mut store,URL).await.unwrap().unwrap();
        assert_eq!(loaded.body,b"new untyped bytes");
        assert_eq!(loaded.content_type.as_deref(),Some("text/html"));
        println!("OBSERVATION: successful replacement body with content_type=None keeps the previous response's text/html metadata");
    });
}

#[test]
fn an_initial_untyped_response_has_no_content_type() {
    pollster::block_on(async {
        let mut store=MemoryBackend::default();
        let value=content(b"untyped bytes",None);
        save_content(&mut store,URL,&value).await.unwrap();
        assert_eq!(load_content(&mut store,URL).await.unwrap().unwrap(),value);
        println!("CONTROL: None on the first write round-trips without metadata");
    });
}

#[test]
fn an_explicit_replacement_content_type_replaces_the_previous_one() {
    pollster::block_on(async {
        let mut store=MemoryBackend::default();
        save_content(&mut store,URL,&content(b"old HTML",Some("text/html"))).await.unwrap();
        let value=content(b"new text",Some("text/plain"));
        save_content(&mut store,URL,&value).await.unwrap();
        assert_eq!(load_content(&mut store,URL).await.unwrap().unwrap(),value);
        println!("CONTROL: an explicit replacement type updates both body and metadata");
    });
}
