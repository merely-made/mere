/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

#[test]
fn script_and_cambium_listeners_on_one_node_interleave_by_registration_order() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let events = Rc::new(RefCell::new(Registry::new()));
    let mut ctx = GenetCtx::with_events(dom.clone(), events.clone());
    let node = {
        let mut d = dom.borrow_mut();
        let node = d.create_element(html_qual("button"));
        let doc = d.document();
        d.insert_before(doc, node, None);
        node
    };
    let at = TargetKey::Node(node);
    let path = |n: u64| vec![ViewId::new(n)];

    // Cambium, script, Cambium, script: one list, in this order.
    ctx.register_click(node, path(1), false);
    events
        .borrow_mut()
        .add(at, script_listener("click", 10, false));
    ctx.register_click(node, path(2), false);
    events
        .borrow_mut()
        .add(at, script_listener("click", 11, false));

    let ran = Rc::new(RefCell::new(Vec::new()));
    let route = build_path(&dom.borrow(), at, None, "click", true);
    let init = EventInit {
        event_type: "click".to_owned(),
        bubbles: true,
        stop_propagation: false,
        stop_immediate: false,
        canceled: false,
        activation_event: false,
    };
    dispatch(&events, route, &init, &mut Recorder(ran.clone()));

    let owners: Vec<Owner> = ran.borrow().iter().map(|(owner, _)| *owner).collect();
    assert_eq!(
        owners,
        [CAMBIUM_OWNER, Owner::SCRIPT, CAMBIUM_OWNER, Owner::SCRIPT]
    );
    // Each Cambium token resolves to the view path it was registered with.
    let paths: Vec<Option<Vec<ViewId>>> = ran
        .borrow()
        .iter()
        .filter(|(owner, _)| *owner == CAMBIUM_OWNER)
        .map(|(_, token)| ctx.listeners().path(*token).map(<[ViewId]>::to_vec))
        .collect();
    assert_eq!(paths, [Some(path(1)), Some(path(2))]);
}

#[test]
fn a_runner_walk_runs_a_shared_registrys_script_listeners_in_place() {
    let log = log();
    let (_, events, mut runner) = start(
        {
            let log = log.clone();
            move |capture: &bool| {
                on_key(el::<_, bool, ()>("button", "+"), {
                    let log = log.clone();
                    move |_: &mut bool, _: KeyEvent| log.borrow_mut().push("cambium".into())
                })
                .capture(*capture)
            }
        },
        false,
    );
    runner.set_foreign_invoker(Some(Box::new(Script(log.clone()))));
    let button = runner.root();
    runner.set_focus(Some(button));
    events.borrow_mut().add(
        TargetKey::Node(button),
        script_listener("keydown", 1, false),
    );

    runner.dispatch_key(key());
    assert_eq!(
        seen(&log),
        ["cambium", "script 1"],
        "registered in this order"
    );

    // Re-registering Cambium's listener (a rebuild that moves it to the capture
    // pass and back) puts it after the script's: the shared list is ordered by
    // registration, not by owner.
    log.borrow_mut().clear();
    runner.update(|capture| *capture = true);
    runner.update(|capture| *capture = false);
    runner.dispatch_key(key());
    assert_eq!(seen(&log), ["script 1", "cambium"]);
}

#[test]
fn script_listeners_in_the_walk_wrap_cambiums_by_capture_and_bubble() {
    let log = log();
    let (dom, events, mut runner) = start(
        {
            let log = log.clone();
            move |_: &()| {
                on_click(
                    el::<_, (), ()>(
                        "div",
                        on_click(el::<_, (), ()>("button", "+"), note(&log, "cambium")),
                    ),
                    note(&log, "cambium parent"),
                )
            }
        },
        (),
    );
    runner.set_foreign_invoker(Some(Box::new(Script(log.clone()))));
    let button = first_named(&dom, runner.root(), "button");
    let div = first_named(&dom, runner.root(), "div");
    events
        .borrow_mut()
        .add(TargetKey::Node(div), script_listener("click", 1, true));
    events
        .borrow_mut()
        .add(TargetKey::Node(div), script_listener("click", 2, false));

    runner.dispatch_click(button, click());

    assert_eq!(
        seen(&log),
        ["script 1", "cambium", "cambium parent", "script 2"],
        "capture on the div, the button, then the div's bubble in registration order \
         (Cambium's parent listener registered at build, before the script's)"
    );
}

#[test]
fn a_registry_shared_with_no_foreign_invoker_skips_other_owners() {
    let log = log();
    let (_, events, mut runner) = start(
        {
            let log = log.clone();
            move |_: &()| on_click(el::<_, (), ()>("button", "+"), note(&log, "cambium"))
        },
        (),
    );
    events.borrow_mut().add(
        TargetKey::Node(runner.root()),
        script_listener("click", 1, false),
    );

    runner.dispatch_click(runner.root(), click());

    assert_eq!(seen(&log), ["cambium"]);
}

#[test]
fn cambium_listeners_register_and_unregister_in_the_events_registry() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut ctx = GenetCtx::new(dom.clone());
    let events = ctx.events();
    assert!(
        !Rc::ptr_eq(&events, &GenetCtx::new(dom.clone()).events()),
        "each context makes its own registry unless one is handed in"
    );
    let node = dom.borrow_mut().create_element(html_qual("button"));
    let at = TargetKey::Node(node);
    let path = vec![ViewId::new(7)];

    ctx.register_click(node, path.clone(), false);
    ctx.register_key(node, path.clone(), true, true);
    ctx.register_focus(node, path.clone());
    assert!(events.borrow().has_type("click"));
    assert!(events.borrow().has_type("keydown"));
    assert!(events.borrow().has_type("focus") && events.borrow().has_type("blur"));
    assert!(ctx.has_click(node) && ctx.has_key(node) && ctx.is_focusable(node));
    assert_eq!(events.borrow().snapshot(at, "keydown").len(), 1);

    // Registering the same path again replaces it: no duplicate in the list.
    ctx.register_click(node, path.clone(), true);
    let clicks = events.borrow().snapshot(at, "click");
    assert_eq!(clicks.len(), 1);
    assert!(clicks[0].capture, "the phase updated in place");

    ctx.unregister_click(node, &path);
    ctx.unregister_key(node, &path);
    ctx.unregister_focus(node);
    assert!(!ctx.has_click(node) && !ctx.has_key(node) && !ctx.is_focusable(node));
    assert!(
        !events.borrow().has_listeners(at),
        "nothing left in the registry"
    );
    assert!(!events.borrow().has_type("click"));
}
