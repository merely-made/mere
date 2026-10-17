"""Extract unchanged private source functions for focused real DOM/layout checks."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
NAME='native_authority_probe'
dec=(ROOT/'crates/cambium/cambium-genet-winit-host/src/decorations.rs').read_text()
t4=(ROOT/'crates/cambium/cambium-genet-winit-host/examples/t4_receipt.rs').read_text()
def function(source,name):
    start=source.index('fn '+name)
    return source[start:source.index('\n}',start)+2]
fragments=[function(dec,'window_control_node'),*(function(t4,n) for n in ['css_rgb','leaf_style','plain_style','pair_view'])]
prelude='''extern crate cambium;
use cambium::{AnyView, GenetCtx, GenetElement, custom_leaf, el};
use cambium_rootstock::{OwnedLayout, Runner, WindowDom, document_projection};
use genet_scripted_dom::{ScriptedDom, NodeId};
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace};
use std::{rc::Rc, cell::RefCell};
struct St;
type Child = Box<dyn AnyView<St, (), GenetCtx, GenetElement>>;
#[derive(Clone,Copy)] enum LayerKind {Clip,Opacity}
const LEAF_RGB:(u8,u8,u8)=(51,102,204);
const LEAF_W:u32=64; const LEAF_H:u32=48;
'''
tests='''
fn by_id(dom:&ScriptedDom,node:NodeId,wanted:&str)->Option<NodeId>{
    if dom.attribute(node,&Namespace::from(""),&LocalName::from("id"))==Some(wanted){return Some(node)}
    dom.dom_children(node).find_map(|child|by_id(dom,child,wanted))
}
fn compare_name(markup:&str,wanted:&str,lookup:Option<&str>){
    let mut dom=ScriptedDom::new(); let root=dom.document(); dom.set_inner_html(root,markup);
    let button=by_id(&dom,root,"control").unwrap();
    let layout=OwnedLayout::new(&dom,&[""],400.,200.,&[],&Default::default());
    let projection=document_projection(&WindowDom::document(&dom),&layout,None);
    let node=projection.nodes().iter().find(|n|n.id.get()==dom.opaque_id(button)).unwrap();
    assert_eq!(node.name.as_deref(),Some(wanted));
    let actual=lookup.and_then(|name|window_control_node(&dom,root,name));
    if lookup.is_some(){assert_eq!(actual,Some(button));}
    else{assert_eq!(window_control_node(&dom,root,wanted),None);}
}
#[test]fn referenced_name_is_projected_but_native_lookup_misses_it(){
    compare_name(r#"<span id="caption">Maximize</span><button id="control" aria-labelledby="caption"></button>"#,"Maximize",None);
}
#[test]fn text_name_is_projected_but_native_lookup_misses_it(){
    compare_name(r#"<button id="control">Maximize</button>"#,"Maximize",None);
}
#[test]fn_superseded_raw_label_is_still_the_native_target(){
    compare_name(r#"<span id="caption">Agrandir</span><button id="control" aria-label="Maximize" aria-labelledby="caption">Maximize</button>"#,"Agrandir",Some("Maximize"));
}
#[test]fn t4_bare_control_paints_at_overriding_local_offset(){
    let dom=Rc::new(RefCell::new(ScriptedDom::new()));
    let logic=|_:&St|pair_view(LayerKind::Clip,101,4.,4.);
    let _runner=Runner::new(dom.clone(),logic,St);
    let dom=dom.borrow();
    fn find(dom:&ScriptedDom,n:NodeId)->Option<NodeId>{
        if dom.attribute(n,&Namespace::from(""),&LocalName::from("style")).is_some_and(|s|s.contains("left:616px")){return Some(n)}
        dom.dom_children(n).find_map(|c|find(dom,c))
    }
    let bare=find(&dom,dom.document()).unwrap();
    let layout=OwnedLayout::new(&dom,&[""],960.,340.,&[],&Default::default());
    let rect=layout.painted_rect(&dom,bare).unwrap();
    println!("actual bare control rect: {rect:?}");
    assert!((rect.0-4.).abs()<0.01 && (rect.1-4.).abs()<0.01);
    assert!((rect.0-616.).abs()>600.);
}
'''.replace('#[test]fn_superseded','#[test]fn superseded')
probe=HERE/(NAME+'.rs')
with probe.open('x',encoding='utf-8') as f:f.write(prelude+'\n\n'.join(fragments)+tests)
runner=(HERE/'run_rootstock_receipt_suite_v3_probe.py').read_text()
runner=runner.replace('rootstock_receipt_suite_v3', 'native_authority').replace('rootstock-receipt-suite-v3','native-authority')
runner=runner.replace("'crates/cambium/cambium-rootstock']", "'crates/cambium/cambium-rootstock','crates/cambium/cambium-genet-winit-host']")
runner=runner.replace("[str(binary),'--test-threads=1']", "[str(binary),'--test-threads=1','--nocapture']")
runner=runner.replace('Original current no-optional-feature Cambium and Rootstock compiled fresh over retained exact suppliers. Three original current public integration suites: capture owner, hover routing, and prepared-frame fake-driver receipts; no native/GPU/optional embedding or fresh full-current dependency graph. Compilation is not source-reading coverage.', 'Original current Cambium/Rootstock over retained exact suppliers. Unchanged extracted native window-control and T4 pair/style functions exercised with real ScriptedDom, neutral accessibility projection and OwnedLayout. No whole native-host build, OS native hit test, GPU pixels or fresh current supplier graph. Extraction fixture does not implement the functions under test.')
with (HERE/('run_'+NAME+'.py')).open('x',encoding='utf-8') as f:f.write(runner)
assert all(fragment in probe.read_text() for fragment in fragments)
print('Prepared five unchanged source function extractions; no production edits')
