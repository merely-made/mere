#![cfg(feature = "cue")]
use inker::webcue::*;
use inker::{SessionSpawnRequest, SurfaceEngineRegistry};
use pelt_core::{PeltController, PeltControllerConfig};
use std::time::Duration;
const HTML: &str = "<!doctype html><html><body style='margin:0;background:#ff0000'><input value='hello'><p>shared retained contract</p></body></html>";
struct Clock;
impl pelt_core::PeltClock for Clock {
    fn now_ms(&self) -> f64 {
        0.
    }
}
fn controller() -> PeltController<netrender::Scene> {
    PeltController::new(
        pelt_core::native::sessions(),
        SurfaceEngineRegistry::new(),
        PeltControllerConfig::from_request(
            inker::routing::ENGINE_GENET_LIVERY,
            SessionSpawnRequest::new("https://held.example/")
                .with_body(HTML)
                .with_content_type("text/html")
                .with_viewport(128, 96),
        ),
        Clock,
    )
    .unwrap()
}
#[test]
fn retained_genet_receives_ordered_requests_input_and_retirement() {
    let mut controller = controller();
    let surface = controller.cue_surface_mut();
    let scope = surface.scope();
    assert!(matches!(
        surface.input(Input::Scroll {
            x: 0.,
            y: 0.,
            dx: 1.,
            dy: 1.,
            modifiers: SessionModifiers {
                control: true,
                ..Default::default()
            }
        }),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        surface.input(Input::Scroll {
            x: f32::NAN,
            y: 0.,
            dx: 0.,
            dy: 1.,
            modifiers: SessionModifiers::default()
        }),
        Err(Error::Invalid(_))
    ));
    assert_eq!(
        surface.capabilities().features().count(),
        Feature::ALL.len()
    );
    assert!(surface.capabilities().feature(Feature::Ime).permits());
    assert!(
        !surface
            .capabilities()
            .feature(Feature::ScriptResult)
            .permits()
    );
    assert!(matches!(
        surface.request(RequestId(1), Command::Script("1".into())),
        Err(Error::Unsupported(_))
    ));
    surface
        .input(Input::Document(SessionInput::Focus(true)))
        .unwrap();
    assert!(matches!(
        surface.poll_event().unwrap().event,
        Event::DocumentInput(_)
    ));
    surface
        .input(Input::Document(SessionInput::Ime(SessionIme::Enabled)))
        .unwrap();
    assert!(matches!(
        surface.poll_event().unwrap().event,
        Event::DocumentInput(_)
    ));
    let key = surface.request(RequestId(1), Command::Zoom(1.25)).unwrap();
    surface.advance(Duration::ZERO).unwrap();
    assert!(
        matches!(surface.poll_event().unwrap(),Envelope { scope:s,event:Event::Completed { request,result:Ok(Reply::Zoom(_)) },.. } if s==scope && request==key)
    );
    assert!(matches!(
        surface.request(RequestId(1), Command::Zoom(1.)),
        Err(Error::RequestOrder)
    ));
    let a = surface
        .request(
            RequestId(2),
            Command::Navigate {
                address: "https://held.example/next".into(),
            },
        )
        .unwrap();
    let b = surface.request(RequestId(3), Command::Reload).unwrap();
    surface.retire().unwrap();
    for expected in [a, b] {
        assert!(
            matches!(surface.poll_event().unwrap().event,Event::Completed{request,result:Err(Error::Retired)} if request==expected)
        );
    }
    assert!(matches!(
        surface.input(Input::Document(SessionInput::Text("late".into()))),
        Err(Error::Retired)
    ));
}
#[test]
fn semantic_publication_retires_on_input_and_replacement_changes_scope() {
    let mut controller = controller();
    let old_scope = controller.cue_surface().scope();
    let surface = controller.cue_surface_mut();
    surface
        .request(RequestId(1), Command::SetSemantics(true))
        .unwrap();
    surface.advance(Duration::ZERO).unwrap();
    while surface.poll_event().is_some() {}
    surface.try_frame(128, 96).unwrap();
    let (publication, projection) = (0..256)
        .filter_map(|_| surface.poll_event())
        .find_map(|envelope| match envelope.event {
            Event::Semantics {
                publication,
                payload: SemanticPayload::Snapshot(projection),
            } => Some((publication, projection)),
            _ => None,
        })
        .expect("prepared semantics");
    surface
        .input(Input::Document(SessionInput::Focus(false)))
        .unwrap();
    let node = projection
        .nodes()
        .iter()
        .find(|n| !n.actions.is_empty())
        .unwrap();
    let action = DocumentA11yActionRequest {
        revision: projection.revision(),
        target: node.id,
        action: node.actions[0],
        data: None,
    };
    assert!(matches!(
        surface.semantic_action(SemanticAction::Document {
            publication,
            action
        }),
        Err(Error::Stale)
    ));
    assert!(
        matches!(surface.poll_event().unwrap().event,Event::SemanticsRetired(p) if p==publication)
    );
    let cancelled = surface.request(RequestId(2), Command::Reload).unwrap();
    controller.open(
        SessionSpawnRequest::new("https://held.example/replaced")
            .with_body(HTML)
            .with_content_type("text/html"),
    );
    assert_ne!(old_scope, controller.cue_surface().scope());
    assert!((0..256).filter_map(|_|controller.poll_cue_event()).any(|event| matches!(event.event,Event::Completed{request,result:Err(Error::Retired)} if request==cancelled)));
}
#[test]
fn hidden_retained_input_refuses_mutation_while_commands_still_advance() {
    let mut controller = controller();
    controller.try_frame(128, 96).unwrap();
    let projection = controller.session().accessibility_projection().unwrap();
    let field = projection
        .nodes()
        .iter()
        .find(|node| node.value.as_deref() == Some("hello"))
        .unwrap();
    assert!(
        controller
            .session_mut()
            .dispatch_accessibility_action(&DocumentA11yActionRequest {
                revision: projection.revision(),
                target: field.id,
                action: inker::DocumentA11yAction::Focus,
                data: None,
            })
    );
    let surface = controller.cue_surface_mut();
    surface.set_visible(false).unwrap();
    assert!(matches!(
        surface.input(Input::Document(SessionInput::Text("hidden".into()))),
        Err(Error::Invalid(_))
    ));
    assert!(matches!(
        surface.dispatch(SessionInput::Text("hidden".into())),
        Err(Error::Invalid(_))
    ));
    assert!(
        surface.poll_event().is_none(),
        "refused input must not emit a result"
    );
    let request = surface.request(RequestId(1), Command::Reload).unwrap();
    surface.advance(Duration::ZERO).unwrap();
    assert!(matches!(
        surface.poll_event().unwrap().event,
        Event::NavigationRequested { .. }
    ));
    assert!(
        matches!(surface.poll_event().unwrap().event, Event::Completed { request: actual, result: Ok(Reply::Dispatched) } if actual == request)
    );
    surface.set_visible(true).unwrap();
    surface.try_frame(128, 96).unwrap();
    let projection = surface.as_ref().accessibility_projection().unwrap();
    assert!(
        projection
            .nodes()
            .iter()
            .any(|node| node.value.as_deref() == Some("hello"))
    );
    surface
        .input(Input::Document(SessionInput::Focus(true)))
        .unwrap();
    assert!(matches!(
        surface.poll_event().unwrap().event,
        Event::DocumentInput(_)
    ));
    surface.set_visible(false).unwrap();
    surface.retire().unwrap();
    assert!(matches!(
        surface.input(Input::Document(SessionInput::Focus(true))),
        Err(Error::Retired)
    ));
    assert!(matches!(
        surface.dispatch(SessionInput::Focus(true)),
        Err(Error::Retired)
    ));
}

#[test]
fn atomic_input_backpressure_does_not_mutate_the_retained_receiver() {
    let mut controller = controller();
    controller.try_frame(128, 96).unwrap();
    let projection = controller.session().accessibility_projection().unwrap();
    let field = projection
        .nodes()
        .iter()
        .find(|node| node.value.as_deref() == Some("hello"))
        .unwrap();
    assert!(
        controller
            .session_mut()
            .dispatch_accessibility_action(&DocumentA11yActionRequest {
                revision: projection.revision(),
                target: field.id,
                action: inker::DocumentA11yAction::Focus,
                data: None
            })
    );
    let surface = controller.cue_surface_mut();
    let mut admitted = 0;
    loop {
        match surface.input(Input::Document(SessionInput::Focus(true))) {
            Ok(()) => admitted += 1,
            Err(Error::Backpressure) => break,
            Err(error) => panic!("unexpected input failure: {error}"),
        }
    }
    assert!(admitted > 0 && admitted <= 256);
    assert!(matches!(
        surface.input(Input::Document(SessionInput::Text("overflow".into()))),
        Err(Error::Backpressure)
    ));
    assert_eq!((0..).map_while(|_| surface.poll_event()).count(), admitted);
    surface.try_frame(128, 96).unwrap();
    let projection = surface.as_ref().accessibility_projection().unwrap();
    assert!(
        projection
            .nodes()
            .iter()
            .any(|node| node.value.as_deref() == Some("hello"))
    );
    assert!(!projection.nodes().iter().any(|node| {
        node.value
            .as_deref()
            .is_some_and(|value| value.contains("overflow"))
    }));
}

#[test]
#[ignore = "requires a real wgpu adapter; run explicitly"]
fn two_genet_producers_render_on_one_host_device_without_sharing_quality_state() {
    use pelt_core::cue::PeltTextureSource;
    use wgpu_cue::{GpuHost, Rendered, TextureSource};
    let core = genet_render_host::RenderCore::boot(netrender::NetrenderOptions {
        enable_vello: true,
        tile_cache_size: Some(256),
        ..Default::default()
    })
    .unwrap();
    let host = GpuHost::new(core.device().clone(), core.queue().clone()).unwrap();
    let mut a =
        PeltTextureSource::new(controller(), core.renderer().wgpu_device.core.clone()).unwrap();
    let mut b =
        PeltTextureSource::new(controller(), core.renderer().wgpu_device.core.clone()).unwrap();
    assert_ne!(a.scope(), b.scope());
    let viewport = Viewport {
        width: 128,
        height: 96,
        scale: 1.,
    };
    let cx = host.context(viewport).unwrap();
    assert!(matches!(a.render(&cx, true).unwrap(), Rendered::Pending));
    assert!(matches!(b.render(&cx, true).unwrap(), Rendered::Pending));
    // The shell may render independently and consume its own quality result.
    // Neither producer consults that renderer's observation queue.
    let scene = controller().try_frame(128, 96).unwrap();
    core.try_rasterize_scaled_for(
        99,
        &scene,
        128,
        96,
        netrender::ColorLoad::Clear(wgpu_cue::wgpu::Color::TRANSPARENT),
        1.,
    )
    .unwrap();
    for producer in [&mut a, &mut b] {
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        let frame = loop {
            match producer.render(&cx, true).unwrap() {
                Rendered::Changed(frame) => break frame,
                _ => {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "frame did not become ready"
                    );
                    std::thread::sleep(Duration::from_millis(1));
                },
            }
        };
        assert_eq!(frame.stamp.scope, producer.scope());
        assert_eq!(frame.device_epoch, host.epoch());
        let rgba = core.read_rgba8_texture(&frame.texture, 128, 96).unwrap();
        let pixel = &rgba.rgba[((80 * 128 + 8) * 4)..((80 * 128 + 8) * 4 + 4)];
        assert!(
            pixel[0] > 240 && pixel[1] < 10 && pixel[2] < 10 && pixel[3] > 240,
            "unexpected pixel {pixel:?}"
        );
    }
    let resized = host
        .context(Viewport {
            width: 160,
            height: 100,
            scale: 1.25,
        })
        .unwrap();
    assert!(matches!(
        a.render(&resized, true).unwrap(),
        Rendered::Pending
    ));
    a.retire().unwrap();
    assert!(matches!(a.render(&resized, true), Err(Error::Retired)));
}

#[test]
#[ignore = "requires a real wgpu adapter; run explicitly"]
fn shared_content_preserves_routing_and_presents_genet_through_the_neutral_host() {
    use pelt_core::cue::PeltSharedContent;
    use pelt_core::{PeltContent, PeltTileRequest};
    use std::sync::Arc;
    use wgpu_cue::{GpuHost, Imported, NativeImporter, Presenter, RenderContext};
    struct NoNativeImport;
    impl NativeImporter for NoNativeImport {
        fn try_import(&mut self, _: &RenderContext<'_>, _: NativeFrame) -> Result<Imported> {
            panic!("Genet is a host-device producer")
        }
    }
    let core = genet_render_host::RenderCore::boot(netrender::NetrenderOptions {
        enable_vello: true,
        tile_cache_size: Some(256),
        ..Default::default()
    })
    .unwrap();
    let registries = pelt_core::native::registries(
        "cue-test",
        pelt_core::native::route_policy(),
        inker::EngineProfileBinding {
            user_data_dir: String::new(),
        },
    );
    let content = PeltContent::routed(
        registries,
        PeltTileRequest::from_request(
            SessionSpawnRequest::new("https://held.example/")
                .with_body(HTML)
                .with_content_type("text/html")
                .with_viewport(128, 96),
        ),
        Arc::new(|| Box::new(Clock)),
    )
    .unwrap();
    let mut shared =
        PeltSharedContent::from_content(content, core.renderer().wgpu_device.core.clone(), None)
            .unwrap();
    assert_eq!(
        shared.route().active_engine(),
        inker::routing::ENGINE_GENET_LIVERY
    );
    assert_eq!(
        shared.capabilities().presentation,
        PresentationMode::HostDevice
    );
    let host = GpuHost::new(core.device().clone(), core.queue().clone()).unwrap();
    let mut presenter = Presenter::default();
    let viewport = Viewport {
        width: 128,
        height: 96,
        scale: 1.,
    };
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    let previous = loop {
        if let Some(frame) = presenter
            .present(&host, shared.source(), viewport, &mut NoNativeImport)
            .unwrap()
        {
            assert_eq!(frame.stamp.scope, shared.scope());
            let pixels = core.read_rgba8_texture(frame.texture, 128, 96).unwrap();
            assert_eq!(
                &pixels.rgba[(80 * 128 + 8) * 4..(80 * 128 + 8) * 4 + 4],
                &[255, 0, 0, 255]
            );
            break frame.stamp;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "shared presenter timed out"
        );
        std::thread::sleep(Duration::from_millis(1));
    };
    shared
        .rebind_device(core.renderer().wgpu_device.core.clone())
        .unwrap();
    let rebound_host = GpuHost::new(core.device().clone(), core.queue().clone()).unwrap();
    loop {
        if let Some(frame) = presenter
            .present(
                &rebound_host,
                shared.source(),
                viewport,
                &mut NoNativeImport,
            )
            .unwrap()
        {
            assert_eq!(frame.stamp.scope, previous.scope);
            assert!(frame.stamp.content_revision > previous.content_revision);
            assert!(frame.stamp.allocation_epoch > previous.allocation_epoch);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "rebound presenter timed out"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(shared.capabilities().evidence, Evidence::Observed);
    while shared.poll_event().is_some() {}
    let request = shared
        .request(
            RequestId(1),
            Command::Navigate {
                address: "https://held.example/next".into(),
            },
        )
        .unwrap();
    shared.advance(Duration::ZERO).unwrap();
    assert!(matches!(
        shared.poll_event().unwrap().event,
        Event::NavigationRequested { .. }
    ));
    assert!(
        matches!(shared.poll_event().unwrap().event,Event::Completed{request:key,result:Ok(Reply::Dispatched)} if key==request)
    );
}

#[test]
fn canonical_ime_composition_changes_the_same_retained_pelt_value() {
    use inker::DocumentA11yAction;
    let mut controller = controller();
    let surface = controller.cue_surface_mut();
    surface
        .request(RequestId(1), Command::SetSemantics(true))
        .unwrap();
    surface.advance(Duration::ZERO).unwrap();
    while surface.poll_event().is_some() {}
    surface.try_frame(128, 96).unwrap();
    let (publication, projection) = (0..256)
        .filter_map(|_| surface.poll_event())
        .find_map(|event| match event.event {
            Event::Semantics {
                publication,
                payload: SemanticPayload::Snapshot(projection),
            } => Some((publication, projection)),
            _ => None,
        })
        .unwrap();
    let field = projection
        .nodes()
        .iter()
        .find(|node| node.value.as_deref() == Some("hello"))
        .unwrap();
    surface
        .semantic_action(SemanticAction::Document {
            publication,
            action: DocumentA11yActionRequest {
                revision: projection.revision(),
                target: field.id,
                action: DocumentA11yAction::Focus,
                data: None,
            },
        })
        .unwrap();
    for ime in [
        SessionIme::Enabled,
        SessionIme::Preedit {
            text: "世界".into(),
            selection: Some((0, 6)),
        },
        SessionIme::Commit("世界".into()),
    ] {
        surface
            .input(Input::Document(SessionInput::Ime(ime)))
            .unwrap();
    }
    assert!(
        (0..256)
            .filter_map(|_| surface.poll_event())
            .any(|event| matches!(
                event.event,
                Event::DocumentInput(SessionInputResult { editable: true, .. })
            ))
    );
    surface.try_frame(128, 96).unwrap();
    let projection = surface.as_ref().accessibility_projection().unwrap();
    assert!(
        projection.nodes().iter().any(|node| node
            .value
            .as_deref()
            .is_some_and(|value| value.contains("世界"))),
        "IME commit did not reach the owned session"
    );
}

#[test]
#[ignore = "requires a real wgpu adapter; run explicitly"]
fn pool_lends_the_exact_receiver_for_shared_frames_and_revokes_hidden_admission() {
    use pelt_core::cue::PeltSceneRenderer;
    use pelt_core::{
        PeltContent, PeltControllerPool, PeltPlacement, PeltTileRequest, WorkspaceRect,
    };
    use std::sync::Arc;
    use wgpu_cue::{GpuHost, Imported, NativeImporter, Presenter, RenderContext, Source};
    struct NoNative;
    impl NativeImporter for NoNative {
        fn try_import(&mut self, _: &RenderContext<'_>, _: NativeFrame) -> Result<Imported> {
            panic!("retained source must not call native importer")
        }
    }
    let core = genet_render_host::RenderCore::boot(netrender::NetrenderOptions {
        enable_vello: true,
        tile_cache_size: Some(256),
        ..Default::default()
    })
    .unwrap();
    let content = PeltContent::routed(
        pelt_core::native::registries(
            "cue-pool",
            pelt_core::native::route_policy(),
            inker::EngineProfileBinding {
                user_data_dir: String::new(),
            },
        ),
        PeltTileRequest::from_request(
            SessionSpawnRequest::new("https://held.example/")
                .with_body(HTML)
                .with_content_type("text/html")
                .with_viewport(128, 96),
        ),
        Arc::new(|| Box::new(Clock)),
    )
    .unwrap();
    let identity = content.session_identity();
    let scope = content.cue_surface().unwrap().scope();
    let mut pool = PeltControllerPool::<u8, u8, netrender::Scene>::new();
    pool.insert(1, content);
    let placement = PeltPlacement {
        content: 1,
        window: 0,
        rect: WorkspaceRect::new(0., 0., 128., 96.),
        visible: true,
        focused: true,
        scale_factor: 1.,
    };
    pool.set_placements([(7, placement.clone())]);
    let admission = pool.placement_generation(&7).unwrap();
    let mut renderer = PeltSceneRenderer::new(core.renderer().wgpu_device.core.clone()).unwrap();
    let host = GpuHost::new(core.device().clone(), core.queue().clone()).unwrap();
    let mut presenter = Presenter::default();
    let viewport = Viewport {
        width: 128,
        height: 96,
        scale: 1.,
    };
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    loop {
        let controller = pool.cue_content_mut(&7).unwrap().document_mut().unwrap();
        assert_eq!(controller.session_identity(), identity.unwrap());
        let mut source = renderer.bind(controller);
        if let Some(frame) = presenter
            .present(
                &host,
                Source::HostDevice(&mut source),
                viewport,
                &mut NoNative,
            )
            .unwrap()
        {
            assert_eq!(frame.stamp.scope, scope);
            let pixels = core.read_rgba8_texture(frame.texture, 128, 96).unwrap();
            assert_eq!(
                &pixels.rgba[(80 * 128 + 8) * 4..(80 * 128 + 8) * 4 + 4],
                &[255, 0, 0, 255]
            );
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    pool.set_placements([]);
    assert!(!pool.admits(&7, admission));
    assert!(pool.cue_content_mut(&7).is_err());
    assert!(matches!(
        pool.content_mut(&1)
            .unwrap()
            .cue_surface_mut()
            .unwrap()
            .input(Input::Document(SessionInput::Text("late".into()))),
        Err(Error::Invalid(_))
    ));
    pool.set_placements([(7, placement)]);
    assert_ne!(pool.placement_generation(&7), Some(admission));
    assert_eq!(
        pool.cue_content_mut(&7).unwrap().session_identity(),
        identity
    );
    assert_eq!(
        pool.content(&1).unwrap().cue_surface().unwrap().scope(),
        scope
    );
}

#[test]
fn pool_visual_mutations_have_revisions_independent_of_layout_admission() {
    use pelt_core::{
        PeltContent, PeltControllerPool, PeltPlacement, PeltTileRequest, WorkspaceRect,
    };
    use std::sync::Arc;
    let content = PeltContent::routed(
        pelt_core::native::registries(
            "paint-revision",
            pelt_core::native::route_policy(),
            inker::EngineProfileBinding {
                user_data_dir: String::new(),
            },
        ),
        PeltTileRequest::from_request(
            SessionSpawnRequest::new("https://held.example/")
                .with_body(HTML)
                .with_content_type("text/html")
                .with_viewport(128, 96),
        ),
        Arc::new(|| Box::new(Clock)),
    )
    .unwrap();
    let mut pool = PeltControllerPool::<u8, u8, netrender::Scene>::new();
    pool.insert(1, content);
    pool.set_placements([(
        7,
        PeltPlacement {
            content: 1,
            window: 0,
            rect: WorkspaceRect::new(0., 0., 128., 96.),
            visible: true,
            focused: true,
            scale_factor: 1.,
        },
    )]);
    let revision = pool.paint_revision(&1).unwrap();
    while pool.poll_cue_event(&1).is_some() {}
    pool.advance_cue_browser(&1, Duration::ZERO).unwrap();
    assert_eq!(
        pool.paint_revision(&1),
        Some(revision),
        "control drains cannot force a new paint or invalidate semantic admission"
    );
    pool.prepare(&7).unwrap();
    assert_eq!(
        pool.paint_revision(&1),
        Some(revision),
        "layout preparation alone cannot perpetually dirty a pending frame"
    );
    pool.input(
        &7,
        pelt_core::PeltInput::Session(SessionInput::Focus(false)),
    );
    let after_input = pool.paint_revision(&1).unwrap();
    assert!(after_input > revision);
    pool.content_mut(&1)
        .unwrap()
        .document_mut()
        .unwrap()
        .set_page_zoom(1.25)
        .unwrap();
    let after_extension = pool.paint_revision(&1).unwrap();
    assert!(after_extension > after_input);
    pool.invalidate_paint(&1); // Exact due timer work is observed by the host.
    assert!(pool.paint_revision(&1).unwrap() > after_extension);
    pool.remove_content(&1);
    assert!(pool.paint_revision(&1).is_none());
}
