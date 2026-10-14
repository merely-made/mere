//! The real CPU host prepares one model/layout frame for either presenter.
use cambium::{AnyView, GenetAppRunner, GenetCtx, GenetElement, el};
use cambium_rootstock::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};
type Child = Box<dyn AnyView<(), (), GenetCtx, GenetElement>>;
type Logic = fn(&()) -> Child;
fn view(_: &()) -> Child {
    Box::new(el("div", "portable host"))
}
struct Window(Rc<Cell<(u32, u32)>>);
impl HostWindow for Window {
    fn request_redraw(&self) {}
    fn inner_size(&self) -> (u32, u32) {
        self.0.get()
    }
    fn scale_factor(&self) -> f64 {
        1.0
    }
    fn set_ime_allowed(&self, _: bool) {}
    fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
}
struct Driver;
impl PresentationDriver for Driver {
    fn backend(&self) -> PresentationBackend {
        PresentationBackend::WebGl2
    }
    fn generation(&self) -> ContextGeneration {
        ContextGeneration(17)
    }
    fn content_mode(&self) -> FrameContentMode {
        FrameContentMode::PortableScene
    }
    fn wgpu_surface(&self) -> Option<&dyn Surface> {
        None
    }
    fn resize(&mut self, _: u32, _: u32) -> Result<(), PresentationFailure> {
        Ok(())
    }
    fn present(
        &mut self,
        _: &PreparedFrame,
    ) -> Result<Option<PresentedFrame>, PresentationFailure> {
        Ok(None)
    }
    fn lose(&mut self) {}
    fn restore(&mut self) -> Result<bool, PresentationFailure> {
        Ok(true)
    }
    fn begin_capture(
        &mut self,
        _: PresentedFrame,
    ) -> Result<Box<dyn PendingPresentationCapture>, CaptureRefusal> {
        unreachable!()
    }
    fn dispose(&mut self) {}
}
#[test]
fn preparation_runs_hook_once_and_defers_zero_extent_without_a_gpu() {
    let mut state = HostState::<(), Logic, Child>::new();
    state.runner = Some(GenetAppRunner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as Logic,
        (),
    ));
    let size = Rc::new(Cell::new((320, 240)));
    state.window = Some(Box::new(Window(size.clone())));
    state.presentation_driver = Some(Box::new(Driver));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let count = Rc::new(Cell::new(0));
    let observed = count.clone();
    let mut hooks = HostHooks::inert();
    hooks.frame = Box::new(move |_| {
        observed.set(observed.get() + 1);
        false
    });
    let mut host = Host::new(HostOptions::default(), None, hooks, state, wake);
    let frame = host.prepare_presentation_frame().unwrap().unwrap();
    assert!(matches!(frame.paint, PreparedPaint::Portable(_)));
    assert_eq!(
        (frame.width, frame.height, frame.generation),
        (320, 240, ContextGeneration(17))
    );
    assert_eq!(frame.sequence, 1);
    assert_eq!(count.get(), 1);
    assert!(host.s.last_redraw_presentation.is_none());
    size.set((0, 0));
    assert!(host.prepare_presentation_frame().unwrap().is_none());
    assert_eq!(count.get(), 2);
    size.set((321, 241));
    let resumed = host.prepare_presentation_frame().unwrap().unwrap();
    assert_eq!(
        (resumed.width, resumed.height, resumed.sequence),
        (321, 241, 1)
    );
    assert_eq!(count.get(), 3);
}

fn leaf_view(_: &()) -> Child {
    Box::new(cambium::custom_leaf(7, 80, 60))
}
struct GpuOnly;
impl TextureProducer for GpuOnly {
    fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
        None
    }
}
#[test]
fn unsupported_leaf_refuses_with_its_key_and_does_not_repeat_a_stamp() {
    let mut state = HostState::<(), Logic, Child>::new();
    state.runner = Some(GenetAppRunner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        leaf_view as Logic,
        (),
    ));
    state.window = Some(Box::new(Window(Rc::new(Cell::new((320, 240))))));
    state.presentation_driver = Some(Box::new(Driver));
    state.shared.producers.register(7, GpuOnly, &[]).unwrap();
    state.presentation_sequence = 9;
    state.last_redraw_presentation = Some(PresentedFrame {
        host: state.presentation_host,
        sequence: 9,
        width: 320,
        height: 240,
        layout_scale: 1.,
        context_generation: ContextGeneration(17),
    });
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    let failure = match host.prepare_presentation_frame() {
        Err(failure) => failure,
        Ok(_) => panic!("GPU-only producer must explicitly refuse portable presentation"),
    };
    assert_eq!(failure.backend, PresentationBackend::WebGl2);
    assert_eq!(failure.generation, ContextGeneration(17));
    assert_eq!(failure.producer_key, Some(7));
    assert!(host.s.last_redraw_presentation.is_none());
    assert_eq!(host.s.presentation_sequence, 9);
    assert!(host.s.shared.producers.contains(7));
}

struct ObservedDriver {
    calls: Rc<Cell<usize>>,
    outcome: Rc<Cell<u8>>,
}
impl PresentationDriver for ObservedDriver {
    fn backend(&self) -> PresentationBackend {
        PresentationBackend::WebGl2
    }
    fn generation(&self) -> ContextGeneration {
        ContextGeneration(17)
    }
    fn content_mode(&self) -> FrameContentMode {
        FrameContentMode::PortableScene
    }
    fn wgpu_surface(&self) -> Option<&dyn Surface> {
        None
    }
    fn resize(&mut self, _: u32, _: u32) -> Result<(), PresentationFailure> {
        Ok(())
    }
    fn present(
        &mut self,
        frame: &PreparedFrame,
    ) -> Result<Option<PresentedFrame>, PresentationFailure> {
        self.calls.set(self.calls.get() + 1);
        match self.outcome.get() {
            0 => Ok(Some(frame.presentation_stamp())),
            1 => Ok(None),
            3 => {
                let mut stamp = frame.presentation_stamp();
                stamp.context_generation = ContextGeneration(16);
                Ok(Some(stamp))
            },
            _ => Err(PresentationFailure {
                backend: self.backend(),
                generation: self.generation(),
                producer_key: None,
                message: "healthy provider refusal".into(),
                native: None,
            }),
        }
    }
    fn lose(&mut self) {}
    fn restore(&mut self) -> Result<bool, PresentationFailure> {
        Ok(true)
    }
    fn begin_capture(
        &mut self,
        _: PresentedFrame,
    ) -> Result<Box<dyn PendingPresentationCapture>, CaptureRefusal> {
        unreachable!()
    }
    fn dispose(&mut self) {}
}

#[test]
fn shared_redraw_observes_only_successful_driver_frames_and_refuses_failed_capture() {
    let mut state = HostState::<(), Logic, Child>::new();
    state.runner = Some(GenetAppRunner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as Logic,
        (),
    ));
    state.window = Some(Box::new(Window(Rc::new(Cell::new((320, 240))))));
    let calls = Rc::new(Cell::new(0));
    let outcome = Rc::new(Cell::new(0));
    state.presentation_driver = Some(Box::new(ObservedDriver {
        calls: calls.clone(),
        outcome: outcome.clone(),
    }));
    let observed = Rc::new(Cell::new(0));
    let counts = observed.clone();
    state.pending_presentation_observer = Some(Box::new(move |_, stamp| {
        assert_eq!(stamp.sequence, 1);
        counts.set(counts.get() + 1);
    }));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.redraw();
    assert_eq!(calls.get(), 1);
    assert_eq!(observed.get(), 1);
    assert_eq!(host.s.last_redraw_presentation.unwrap().sequence, 1);
    outcome.set(1);
    host.redraw();
    assert_eq!(calls.get(), 2);
    assert!(host.s.last_redraw_presentation.is_none());
    assert_eq!(host.s.presentation_sequence, 1);
    let capture_refused = Rc::new(Cell::new(false));
    let refusal = capture_refused.clone();
    host.s.pending_paint_capture = Some(Box::new(move |result| {
        assert!(matches!(
            result,
            Err(CaptureRefusal::PresentationError {
                backend: PresentationBackend::WebGl2,
                ..
            })
        ));
        refusal.set(true);
    }));
    outcome.set(2);
    host.redraw();
    assert_eq!(calls.get(), 3);
    assert!(capture_refused.get());
    assert!(host.s.last_redraw_presentation.is_none());
    assert_eq!(host.s.presentation_sequence, 1);
    outcome.set(0);
    host.redraw();
    assert_eq!(host.s.last_redraw_presentation.unwrap().sequence, 2);
    assert_eq!(
        calls.get(),
        4,
        "ordinary provider errors retain the installed route"
    );
}

#[test]
fn driver_cannot_commit_a_stamp_from_another_context() {
    let mut state = HostState::<(), Logic, Child>::new();
    state.runner = Some(GenetAppRunner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as Logic,
        (),
    ));
    state.window = Some(Box::new(Window(Rc::new(Cell::new((320, 240))))));
    let calls = Rc::new(Cell::new(0));
    state.presentation_driver = Some(Box::new(ObservedDriver {
        calls: calls.clone(),
        outcome: Rc::new(Cell::new(3)),
    }));
    let observed = Rc::new(Cell::new(false));
    let observation = observed.clone();
    state.pending_presentation_observer = Some(Box::new(move |_, _| observation.set(true)));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.redraw();
    assert_eq!(calls.get(), 1);
    assert!(!observed.get());
    assert!(host.s.last_redraw_presentation.is_none());
    assert_eq!(host.s.presentation_sequence, 0);
}
