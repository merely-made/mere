use document_canvas::*;
use inker::{Block,InlineSpan,EngineDocument,DocumentProvenance,DocumentTrustState};
use paint_list_api::PaintList;

fn paragraph(text:&str)->Block {Block::Paragraph {spans:vec![InlineSpan::Text(text.into())]}}
fn doc(blocks:Vec<Block>)->EngineDocument {EngineDocument {
    address:"doc:audit".into(),title:None,content_type:"text/plain".into(),lang:None,
    provenance:DocumentProvenance::default(),trust:DocumentTrustState::Unknown,diagnostics:vec![],
    navigation:Default::default(),blocks}}
fn layout(blocks:Vec<Block>,style:&DocumentStyleSheet)->LaidOutDocument {
    layout_document(&doc(blocks),Viewport::new(640.,480.),style)
}
fn link(url:&str,label:&str)->InlineSpan {InlineSpan::Link {url:url.into(),title:None,
    spans:vec![InlineSpan::Text(label.into())],predicate:None}}

#[test]
fn adjacent_links_share_the_full_run_hitbox_and_first_click_navigates_to_second() {
    let mut style=DocumentStyleSheet::default();style.link_adornment=LinkAdornment::None;
    let packet=layout(vec![Block::Paragraph {spans:vec![link("https://first.test/","FIRST"),link("https://second.test/","SECOND")]}],&style).packet;
    assert_eq!(packet.interactions.len(),2);
    let first=&packet.interactions[0];let second=&packet.interactions[1];
    assert_eq!(first.bounds,second.bounds);
    assert_ne!(first.link_semantics.as_ref().unwrap().identity,second.link_semantics.as_ref().unwrap().identity);
    let RenderedBlockKind::Text {glyph_runs}=&packet.blocks[0].kind else {panic!("text")};
    let run=&glyph_runs[0];let glyph=&run.glyphs[0];
    let x=run.origin.x+glyph.x+glyph.advance*0.5;let y=first.bounds.origin.y+first.bounds.size.height*0.5;
    assert_eq!(packet.link_at(x,y),Some("https://second.test/"));
    let spaced=layout(vec![Block::Paragraph {spans:vec![link("https://first.test/","FIRST"),InlineSpan::Text(" ".into()),link("https://second.test/","SECOND")]}],&style).packet;
    assert_eq!(spaced.interactions[0].bounds,spaced.interactions[1].bounds,"space does not split the shaping run");
    let bold=InlineSpan::Link {url:"https://first.test/".into(),title:None,
        spans:vec![InlineSpan::Strong(vec![InlineSpan::Text("FIRST".into())])],predicate:None};
    let control=layout(vec![Block::Paragraph {spans:vec![bold,InlineSpan::Text(" ".into()),link("https://second.test/","SECOND")]}],&style).packet;
    assert_ne!(control.interactions[0].bounds,control.interactions[1].bounds);
    let r=&control.interactions[0];assert_eq!(control.link_at(r.bounds.origin.x+1.,r.bounds.origin.y+r.bounds.size.height*0.5),Some("https://first.test/"));
    println!("Distinct logical links share the whole shaping-run rectangle; first label glyph resolves second URL, with or without a space. Bold-font boundary control resolves first URL.");
}

#[test]
fn ordered_and_unordered_lists_lose_their_distinction_before_paint() {
    let style=DocumentStyleSheet::default();
    let items=vec![vec![paragraph("first item")],vec![paragraph("second item")]];
    let ordered=layout(vec![Block::List {ordered:true,items:items.clone()}],&style);
    let unordered=layout(vec![Block::List {ordered:false,items}],&style);
    assert_eq!(ordered.packet,unordered.packet);
    let a=paint_list_from_packet(&ordered.packet,&ordered.fonts,&style.colors);
    let b=paint_list_from_packet(&unordered.packet,&unordered.fonts,&style.colors);
    assert_eq!(format!("{:?}",a.commands()),format!("{:?}",b.commands()));
    let changed=layout(vec![Block::List {ordered:true,items:vec![vec![paragraph("changed item")],vec![paragraph("second item")]]}],&style);
    assert_ne!(ordered.packet,changed.packet);
    println!("Changing ordered=true to false leaves complete packet and paint-command Debug dump identical; changed content control differs.");
}

#[test]
fn nested_block_source_indices_alias_unrelated_top_level_content_or_leave_range() {
    let style=DocumentStyleSheet::default();
    let source=doc(vec![Block::Quote {blocks:vec![paragraph("quoted zero"),paragraph("quoted one")]},paragraph("unrelated outside")]);
    let packet=layout_document(&source,Viewport::new(640.,480.),&style).packet;
    let RenderedBlockKind::Group {children}=&packet.blocks[0].kind else {panic!("group")};
    let target=&children[1];let x=target.bounds.origin.x+1.;let y=target.bounds.origin.y+1.;
    let hit=packet.block_at(x,y).unwrap();assert!(std::ptr::eq(hit,target));assert_eq!(hit.source_block_index,1);
    assert_eq!(source.blocks[hit.source_block_index],paragraph("unrelated outside"));
    assert_ne!(packet.top_level_block(1).unwrap().bounds,hit.bounds,"top-level helper correctly avoids the child");
    let source=doc(vec![paragraph("before"),Block::Quote {blocks:vec![paragraph("quoted")]}]);
    let packet=layout_document(&source,Viewport::new(640.,480.),&style).packet;
    let RenderedBlockKind::Group {children}=&packet.blocks[1].kind else {panic!("group")};
    assert_eq!(children[0].source_block_index,1000);assert!(children[0].source_block_index>=source.blocks.len());
    println!("block_at on quoted child yields source index1 naming unrelated top-level text; quote at index1 emits child1000 outside the source array. Top-level lookup control remains correct.");
}

#[test]
fn paragraph_ignores_its_role_spacing_above() {
    let base=DocumentStyleSheet::default();let ordinary=layout(vec![paragraph("body")],&base).packet;
    let mut changed=base.clone();changed.roles.body.spacing_above=80.;
    assert_eq!(changed.resolve(BlockRole::Body).spacing_above,80.);
    let with_above=layout(vec![paragraph("body")],&changed).packet;assert_eq!(ordinary,with_above);
    changed.roles.body.spacing_below+=80.;let with_below=layout(vec![paragraph("body")],&changed).packet;assert_ne!(ordinary,with_below);
    let mut code=base.clone();code.roles.code.spacing_above=80.;
    let blocks=||vec![Block::CodeBlock {language:None,text:"body".into()}];
    let zero=layout(blocks(),&base).packet;let spaced=layout(blocks(),&code).packet;
    assert!((spaced.blocks[0].bounds.origin.y-zero.blocks[0].bounds.origin.y-80.).abs()<0.01);
    println!("Resolved body spacing_above80 leaves paragraph packet unchanged; body spacing_below and code spacing_above controls change geometry.");
}

#[test]
fn link_adornment_changes_renumber_later_retained_link_identities() {
    let mut marked=DocumentStyleSheet::default();let mut plain=marked.clone();plain.link_adornment=LinkAdornment::None;
    let make=|label:&str|vec![Block::Paragraph {spans:vec![link("https://empty.test/",label),link("https://later.test/","later")]}];
    let identity=|packet:&DocumentRenderPacket|packet.interactions.iter().find(|r|matches!(&r.kind,InteractionKind::Link {url} if url=="https://later.test/"))
        .unwrap().link_semantics.as_ref().unwrap().identity;
    let a=layout(make(""),&marked).packet;let b=layout(make(""),&plain).packet;
    assert_eq!(a.interactions.len(),2);assert_eq!(b.interactions.len(),1);
    assert_ne!(identity(&a),identity(&b));
    let control_a=layout(make("first"),&marked).packet;let control_b=layout(make("first"),&plain).packet;
    assert_eq!(identity(&control_a),identity(&control_b));
    marked.link_adornment=LinkAdornment::Glyph(String::new());
    assert_eq!(identity(&layout(make(""),&marked).packet),identity(&b));
    println!("Same source later link gets a different retained ID when adornment hides an earlier empty-label link; nonempty-label control keeps ID, empty custom glyph repeats the change.");
}
