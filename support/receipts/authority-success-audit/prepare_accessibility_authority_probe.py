"""Extract current pure native/browser lowering and native request mapping, unchanged."""
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='accessibility_authority_probe'
native=(ROOT/'crates/cambium/cambium-winit-a11y/src/lib.rs').read_text()
browser=(ROOT/'crates/cambium/cambium-genet-web-host/src/mirror.rs').read_text()
start=native.index('pub fn project_tree<');end=native.index('\n#[cfg(test)]\nmod dpi_tests')
projection=native[start:end]
start=native.index('    fn map_parts(');end=native.index('\n    }',start)+6
mapping=native[start:end]
start=browser.index('use std::collections::HashMap;');end=browser.index('\n#[cfg(test)]\nmod tests;')
web=browser[start:end]
prelude='''extern crate cambium;
use std::{collections::HashMap,rc::Rc,cell::RefCell,sync::Arc};
use accesskit::{Action,ActionData,Affine,NodeId as A11yNodeId,Rect,Role,TreeUpdate};
use cambium_rootstock::{A11yAction,A11yRequest,ProducedAction,ProducerRegistry,ProducerRole,ProducerSemantics,OwnedLayout,WindowDom,document_projection,Host,HostState,HostHooks,HostOptions,HostWake,HostWindow,Runner};
use genet_scripted_dom::{ScriptedDom,NodeId};
use layout_dom_api::{LayoutDom,LocalName,Namespace,NodeKind};
use sprigging::LeafRegistry;
use cambium::{AnyView,GenetCtx,GenetElement,el,button,custom_leaf};
struct Mapping {action_map:HashMap<A11yNodeId,NodeId>,produced_map:HashMap<A11yNodeId,ProducedAction>}
'''
tests='''
#[derive(Default)]struct State{pane:bool,hidden:bool,clicks:usize}
type Child=Box<dyn AnyView<State,(),GenetCtx,GenetElement>>;
type Logic=fn(&State)->Child;
fn view(s:&State)->Child{
    let target=button("Target",|s:&mut State,_|s.clicks+=1).attr("id","target").attr("style",if s.hidden{"display:none;"}else{"display:block;width:100px;height:40px;"});
    if s.pane {Box::new(el("div",(el("div",()).attr("style","height:300px;"),target)).attr("id","pane").attr("style","display:block;width:280px;height:120px;overflow:auto;"))}
    else {Box::new(el("div",(el("div",()).attr("style","height:300px;"),target,el("div",()).attr("style","height:500px;"))))}
}
struct Window;
impl HostWindow for Window{
    fn request_redraw(&self){} fn inner_size(&self)->(u32,u32){(400,200)} fn scale_factor(&self)->f64{1.}
    fn set_ime_allowed(&self,_:bool){} fn set_ime_cursor_area(&self,_:f64,_:f64,_:f64,_:f64){}
}
fn host(pane:bool)->Host<State,Logic,Child>{
    let mut s=HostState::new();s.shared.sheet="body { margin:0; }".into();s.window=Some(Box::new(Window));
    s.runner=Some(Runner::new(Rc::new(RefCell::new(ScriptedDom::new())),view as Logic,State{pane,..Default::default()}));
    let wake=HostWake::new(s.wake_pending.clone(),Arc::new(||{}));
    let mut host=Host::new(HostOptions::default(),None,HostHooks::inert(),s,wake);
    host.set_surface_size(400.,200.);host.relayout(400.,200.);host
}
fn by_id(dom:&ScriptedDom,n:NodeId,id:&str)->Option<NodeId>{
    if dom.attribute(n,&Namespace::from(""),&LocalName::from("id"))==Some(id){return Some(n)}
    dom.dom_children(n).find_map(|c|by_id(dom,c,id))
}
fn compare_scroll(pane:bool){
    let mut h=host(pane);h.pointer_moved(20.,20.);h.wheel(0.,150.);
    let dom=h.s.runner.as_ref().unwrap().dom();let dom=dom.borrow();let n=by_id(&dom,dom.document(),"target").unwrap();
    let layout=h.s.layout.as_ref().unwrap();let painted=layout.painted_rect(&*dom,n).unwrap();
    let neutral=document_projection(&WindowDom::document(&dom),layout,None);
    let neutral=neutral.nodes().iter().find(|row|row.id.get()==dom.opaque_id(n)).unwrap().bounds.unwrap();
    let (native,_,_)=project_tree_with_actions(&*dom,layout,&mut LeafRegistry::new(),&mut ProducerRegistry::new(),None);
    let native=native.nodes.iter().find(|(id,_)|id.0==dom.opaque_id(n)).unwrap().1.bounds().unwrap();
    println!("pane={pane} painted={painted:?} neutral={neutral:?} native={native:?}");
    assert!((painted.1-150.).abs()<0.01);assert!((neutral.y-painted.1).abs()<0.01);
    assert!((native.y0-300.).abs()<0.01);assert!((native.y0-f64::from(painted.1)).abs()>149.);
}
#[test]fn viewport_scroll_native_and_neutral_bounds_disagree(){compare_scroll(false)}
#[test]fn nested_scroll_native_and_neutral_bounds_disagree(){compare_scroll(true)}
#[test]fn hidden_node_missing_from_tree_remains_mappable_and_activates(){
    let mut h=host(false);h.s.runner.as_mut().unwrap().update(|s|s.hidden=true);h.relayout(400.,200.);
    let request={
        let dom=h.s.runner.as_ref().unwrap().dom();let dom=dom.borrow();let n=by_id(&dom,dom.document(),"target").unwrap();let id=A11yNodeId(dom.opaque_id(n));
        let (tree,action_map,produced_map)=project_tree_with_actions(&*dom,h.s.layout.as_ref().unwrap(),&mut LeafRegistry::new(),&mut ProducerRegistry::new(),None);
        assert!(!tree.nodes.iter().any(|(candidate,_)|*candidate==id));assert!(action_map.contains_key(&id));
        Mapping{action_map,produced_map}.map_parts(Action::Click,None,id).unwrap()
    };
    h.apply_a11y_requests(&[request]);assert_eq!(h.s.runner.as_ref().unwrap().state().clicks,1);
}
#[test]fn leaf_author_name_differs_between_native_and_browser_lowering(){
    use layout_dom_api::LayoutDomMut;
    struct Leaf;
    impl sprigging::Leaf for Leaf{
        fn measure(&mut self,_:sprigging::SizeHint,_:sprigging::SizeHint)->sprigging::Size{sprigging::Size{width:100.,height:40.}}
        fn paint(&mut self,_:&mut sprigging::PaintCx<'_>){}
        fn accessibility(&mut self,n:&mut accesskit::Node){n.set_label("Leaf name")}
    }
    let mut dom=ScriptedDom::new();let root=dom.document();dom.set_inner_html(root,r#"<custom-leaf key="7" aria-label="Author name" style="display:block;width:100px;height:40px;"></custom-leaf>"#);
    let layout=OwnedLayout::new(&dom,&[""],400.,200.,&[],&Default::default());
    let mut leaves=LeafRegistry::new();leaves.insert(7,Box::new(Leaf));
    let (tree,_)=project_tree(&dom,&layout,&mut leaves,&mut ProducerRegistry::new(),None);
    assert!(tree.nodes.iter().any(|(_,n)|n.label()==Some("Leaf name")));
    let neutral=document_projection(&WindowDom::document(&dom),&layout,None);
    let mirror=web_mirror::plan(&neutral,1.,|_,name| (name==Some("Author name")).then(||web_mirror::LeafSemantics{role:None,name:Some("Leaf name".into()),..Default::default()}));
    fn find(ns:&[web_mirror::MirrorNode])->bool{ns.iter().any(|n|n.attrs.iter().any(|(k,v)|*k=="aria-label" && v=="Author name") || n.text.as_deref()==Some("Author name") || find(&n.children))}
    assert!(find(&mirror));
}
'''
with (HERE/(NAME+'.rs')).open('x',encoding='utf-8') as f:f.write(prelude+'\nimpl Mapping {\n'+mapping+'\n}\n'+projection+'\nmod web_mirror {\n'+web+'\n}\n'+tests)
runner=(HERE/'run_native_authority_v2_probe.py').read_text().replace('native_authority_v2','accessibility_authority').replace('native-authority-v2','accessibility-authority')
runner=runner.replace("'paint_list_render','web_time']", "'paint_list_render','web_time','accesskit']")
runner=runner.replace("'crates/cambium/cambium-genet-winit-host']", "'crates/cambium/cambium-genet-winit-host','crates/cambium/cambium-winit-a11y','crates/cambium/cambium-genet-web-host']")
runner=runner.replace("sum(line.startswith('test ') and line.endswith(' ... ok') for line in ran.stdout.splitlines())", "int(re.search(r'test result: ok\\. (\\d+) passed;',ran.stdout).group(1)) if ran.returncode == 0 else 0")
runner=runner.replace('Original current Cambium/Rootstock over retained exact suppliers. Unchanged extracted native window-control and T4 pair/style functions exercised with real ScriptedDom, neutral accessibility projection and OwnedLayout. No whole native-host build, OS native hit test, GPU pixels or fresh current supplier graph. Extraction fixture does not implement the functions under test.', 'Original current Cambium/Rootstock over retained exact suppliers plus accesskit. Unchanged pure native accessibility projection, native request mapping, and browser mirror lowerings exercised with actual current DOM/Host/layout/neutral projection. Mapping fixture supplies only storage for the unchanged original mapper. No whole native/browser adapter build, OS accessibility action or GPU pixels, or fresh current supplier graph.')
with (HERE/('run_'+NAME+'.py')).open('x',encoding='utf-8') as f:f.write(runner)
assert projection in (HERE/(NAME+'.rs')).read_text() and mapping in (HERE/(NAME+'.rs')).read_text() and web in (HERE/(NAME+'.rs')).read_text()
print('Prepared unchanged accessibility function extractions')
