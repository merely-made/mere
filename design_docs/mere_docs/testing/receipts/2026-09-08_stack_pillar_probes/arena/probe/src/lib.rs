//! R2-A disposable probe against pinned, unmodified Genet source.
//! Failures are research findings, not expected passing regressions.
#[cfg(test)]
mod probes {
    use genet_scripted_dom::{NodeId, Pins, ScriptedDom};
    use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace, QualName};

    fn element(dom: &mut ScriptedDom, name: &str) -> NodeId {
        dom.create_element(QualName::new(None, Namespace::from("http://www.w3.org/1999/xhtml"), LocalName::from(name)))
    }

    fn fixture(observing: bool) -> (ScriptedDom, NodeId, NodeId, Pins) {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let parent = element(&mut dom, "div");
        let child = element(&mut dom, "span");
        dom.append_child(root, parent);
        dom.append_child(parent, child);
        let mut pins = Pins::new();
        pins.pin(child);
        dom.set_observing(observing);
        (dom, parent, child, pins)
    }

    #[test]
    fn control_remove_child_retains_pin_until_release() {
        let (mut dom, _, child, mut pins) = fixture(false);
        dom.remove_child(child);
        dom.collect(pins.iter());
        assert!(dom.is_live(child), "ordinary detachment must retain the pin");
        pins.unpin(child);
        dom.collect(pins.iter());
        assert!(!dom.is_live(child), "released detached node must be collectible");
    }

    fn text_replacement(observing: bool) {
        let (mut dom, parent, child, pins) = fixture(observing);
        dom.set_text_content(parent, "replacement");
        let before_collection = dom.is_live(child);
        dom.collect(pins.iter());
        assert!(before_collection && dom.is_live(child), "retained child lost: observing={observing}, live_before_gc={before_collection}, live_after_gc={}", dom.is_live(child));
    }

    #[test]
    fn text_replacement_retains_pin_observers_off() { text_replacement(false); }
    #[test]
    fn text_replacement_retains_pin_observers_on() { text_replacement(true); }

    fn fragment_replacement(observing: bool) {
        let (mut dom, parent, child, pins) = fixture(observing);
        dom.set_inner_html(parent, "<b>replacement</b>");
        let before_collection = dom.is_live(child);
        dom.collect(pins.iter());
        assert!(before_collection && dom.is_live(child), "retained child lost: observing={observing}, live_before_gc={before_collection}, live_after_gc={}", dom.is_live(child));
    }

    #[test]
    fn fragment_replacement_retains_pin_observers_off() { fragment_replacement(false); }
    #[test]
    fn fragment_replacement_retains_pin_observers_on() { fragment_replacement(true); }

    #[test]
    fn foreign_handle_refuses_same_local_slot() {
        let (mut first, _, foreign, _) = fixture(false);
        let (second, _, local, _) = fixture(false);
        assert!(first.is_live(foreign) && second.is_live(local));
        assert!(!second.is_live(foreign), "a foreign arena handle must not resolve to a local node: foreign={foreign:?}, local={local:?}");
        first.collect([foreign]);
    }

    #[test]
    fn control_released_ids_do_not_alias_after_churn() {
        let (mut dom, parent, old, mut pins) = fixture(false);
        dom.remove_child(old);
        pins.unpin(old);
        dom.collect(pins.iter());
        let baseline = dom.live_node_count();
        for _ in 0..1000 {
            let node = element(&mut dom, "i");
            assert_ne!(node, old);
            dom.append_child(parent, node);
            dom.remove_child(node);
            dom.collect([]);
        }
        assert!(!dom.is_live(old));
        assert_eq!(dom.live_node_count(), baseline);
    }
}
