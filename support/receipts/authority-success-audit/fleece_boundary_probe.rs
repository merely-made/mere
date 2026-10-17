//! Audit observations over public APIs. Passing tests characterize current defects.
use fleece::clip::{ClipRect, attach_cropped_visual, fragment_from_text};
use fleece::crawl::*;
use fleece::eidetic_bridge::*;
use image::ImageEncoder;
use std::{cell::RefCell, time::Duration};

fn crawl_sitemap(xml: &str) -> (CrawlSummary, Vec<CrawlRequest>) {
    let requests = RefCell::new(Vec::new());
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let summary = runtime.block_on(run_crawl(
        "https://s.test/", CrawlPolicy { seed_sitemap: true, polite_delay: Duration::ZERO, ..Default::default() },
        |request: CrawlRequest| {
            let response = match request.kind {
                CrawlRequestKind::Robots => Err(CrawlFetchError::NotFound { received_bytes: 0 }),
                CrawlRequestKind::Sitemap => Ok(fetch::Fetched::text(Some("application/xml".into()), xml)),
                CrawlRequestKind::Page => Ok(fetch::Fetched::text(Some("text/html".into()), "<p>page</p>")),
            };
            requests.borrow_mut().push(request);
            async move { response }
        }, || false, |_,_,_,_| Vec::<()>::new(), |_| {},
    ));
    (summary, requests.into_inner())
}

#[test]
fn sitemap_xml_entities_are_sent_as_url_characters() {
    let (summary,requests) = crawl_sitemap("<urlset><url><loc>https://s.test/search?a=1&amp;b=2</loc></url></urlset>");
    assert_eq!(summary.fetched, 2);
    assert_eq!(summary.stop, CrawlStop::FrontierExhausted);
    assert!(requests.iter().any(|r| r.url == "https://s.test/search?a=1&amp;b=2"));
    assert!(!requests.iter().any(|r| r.url == "https://s.test/search?a=1&b=2"));
    let (cdata,cdata_requests) = crawl_sitemap("<urlset><url><loc><![CDATA[https://s.test/search?a=1&b=2]]></loc></url></urlset>");
    assert_eq!(cdata.fetched,1);
    assert!(!cdata_requests.iter().any(|r|r.url.contains("search")));
    println!("XML entity spelling reached Page request; correct URL absent; equivalent CDATA loc omitted; stop={:?}", summary.stop);
}

fn crop(rect: ClipRect) -> (bool, Option<(u32,u32)>) {
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png).write_image(&vec![255;100*100*4],100,100,image::ExtendedColorType::Rgba8).unwrap();
    let mut fragment = fragment_from_text("https://s.test/",None,"selected",None,vec![]);
    fragment.rect = Some(rect);
    let attached = attach_cropped_visual(&mut fragment,&png,(100,100));
    (attached,fragment.visual.map(|v|(v.width,v.height)))
}

#[test]
fn crop_reports_success_outside_image_and_overcaptures_partial_rect() {
    let inside = crop(ClipRect{x:10.,y:10.,width:20.,height:20.});
    assert_eq!(inside,(true,Some((20,20))));
    let partial = crop(ClipRect{x:-50.,y:10.,width:100.,height:20.});
    assert_eq!(partial,(true,Some((100,20)))); // geometric intersection is 50 x 20
    let outside = crop(ClipRect{x:200.,y:10.,width:30.,height:20.});
    assert_eq!(outside,(true,Some((1,20)))); // geometric intersection is empty
    println!("crop control={inside:?}; partial={partial:?} (expected 50x20); outside={outside:?} (expected empty)");
}

fn record(text: &str, scope: &str, capture_hash: eidetic::Hash) -> FleeceAnnotationRecord {
    let dom = genet_static_dom::StaticDocument::parse(&format!("<main><p>{text}</p></main>"));
    let document = fleece::extract_document(&dom);
    let anchor = fleece::anchor_for_range(&document.page.text, fleece::TextPositionSelector {start:0,end:5}, document.contract.quote_context).unwrap();
    let evidence = CaptureEvidenceV1::new(scope,capture_hash,eidetic::Timestamp(7),Some("text/html".into()),CaptureDomMode::Rendered,None,None).unwrap();
    FleeceAnnotationRecord::from_fleece(CaptureIdentity::from_evidence(evidence).unwrap(),&document,&anchor).unwrap()
}

#[test]
fn annotation_interchange_id_aliases_distinct_valid_rendered_targets() {
    let hash = eidetic::Hash::of(b"same acquired representation with caller-owned rendering");
    let first = record("Alpha","https://s.test/a",hash);
    let second = record("Bravo","https://s.test/a",hash);
    let other_scope = record("Alpha","https://s.test/b",hash);
    for r in [&first,&second,&other_scope] {r.validate_integrity().unwrap();r.annotation_json_ld().unwrap();}
    assert_ne!(first.target.source.id,second.target.source.id);
    assert_ne!(first.target.scope,other_scope.target.scope);
    assert_eq!(first.annotation.id,second.annotation.id);
    assert_eq!(first.annotation.id,other_scope.annotation.id);
    let control = record("Alpha","https://s.test/a",eidetic::Hash::of(b"other capture"));
    assert_ne!(first.annotation.id,control.annotation.id);
    assert_eq!(first,record("Alpha","https://s.test/a",hash));
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(async {
        let mut store = eidetic::MemoryBackend::new();
        bootstrap_fleece_annotation_schema(&mut store).await.unwrap();
        let a = save_fleece_annotation(&mut store,&first,7).await.unwrap();
        let b = save_fleece_annotation(&mut store,&second,7).await.unwrap();
        assert_ne!(a,b); // Eidetic's content-addressed records stay separate.
        assert_eq!(load_fleece_annotation(&mut store,a).await.unwrap().unwrap(),first);
        assert_eq!(load_fleece_annotation(&mut store,b).await.unwrap().unwrap(),second);
        println!("distinct canonical targets and source scopes share annotation id {}; Eidetic records stay distinct",first.annotation.id);
    });
}
