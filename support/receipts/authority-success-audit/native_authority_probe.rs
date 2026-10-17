extern crate cambium;
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
fn window_control_node<D>(dom: &D, root: NodeId, accessible_label: &str) -> Option<NodeId>
where
    D: LayoutDom<NodeId = NodeId>,
{
    fn find<D>(dom: &D, node: NodeId, button: &LocalName, accessible_label: &str) -> Option<NodeId>
    where
        D: LayoutDom<NodeId = NodeId>,
    {
        let role = dom.attribute(node, &Namespace::from(""), &LocalName::from("role"));
        let label = dom.attribute(node, &Namespace::from(""), &LocalName::from("aria-label"));
        let button_element = dom
            .element_name(node)
            .is_some_and(|name| &name.local == button);
        if (button_element || role.is_some_and(|value| value.eq_ignore_ascii_case("button")))
            && label.is_some_and(|value| value == accessible_label)
        {
            return Some(node);
        }
        dom.dom_children(node)
            .find_map(|child| find(dom, child, button, accessible_label))
    }

    find(dom, root, &LocalName::from("button"), accessible_label)
}

fn css_rgb(r: u8, g: u8, b: u8) -> String {
    format!("rgb({r}, {g}, {b})")
}

fn leaf_style(w: u32, h: u32, dx: f32, dy: f32) -> String {
    format!("display:block;position:absolute;left:{dx}px;top:{dy}px;width:{w}px;height:{h}px;")
}

fn plain_style(dx: f32, dy: f32, css: &str) -> String {
    format!(
        "display:block;position:absolute;left:{dx}px;top:{dy}px;width:{LEAF_W}px;height:{LEAF_H}px;background-color:{css};"
    )
}

fn pair_view(kind: LayerKind, key: u64, dx: f32, dy: f32) -> Child {
    let css = css_rgb(LEAF_RGB.0, LEAF_RGB.1, LEAF_RGB.2);
    let (aw, ah, extra) = match kind {
        LayerKind::Clip => (40u32, 30u32, "overflow:hidden;"),
        LayerKind::Opacity => (90u32, 70u32, "opacity:0.5;"),
    };
    let ancestor_style = |left: u32| {
        format!(
            "display:block;position:absolute;left:{left}px;top:16px;width:{aw}px;height:{ah}px;{extra}"
        )
    };
    let leaf_col = el(
        "div",
        custom_leaf::<St, ()>(key, LEAF_W, LEAF_H)
            .attr("style", leaf_style(LEAF_W, LEAF_H, dx, dy)),
    )
    .attr("style", ancestor_style(16));
    let reference_col = el(
        "div",
        el("div", ()).attr("style", plain_style(dx, dy, &css)),
    )
    .attr("style", ancestor_style(316));
    let bare_col = el("div", ()).attr(
        "style",
        format!(
            "display:block;position:absolute;left:{}px;top:16px;{}",
            616,
            plain_style(dx, dy, &css)
        ),
    );
    Box::new(
        el("div", (leaf_col, reference_col, bare_col))
            .attr("style", "position:relative;width:900px;height:300px;"),
    )
}
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
#[test]fn superseded_raw_label_is_still_the_native_target(){
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
