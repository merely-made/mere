// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Exercise actual browser backend choice through the retained host.
#![cfg(target_arch = "wasm32")]
#[path = "../a11y_page/page.rs"]
mod page;
use cambium_genet_web_host::{
    Mounted, mount, mount_with_webgpu_factory, run_static_constructors_once,
};
use cambium_rootstock::{
    HostFont, HostHooks, HostImage, HostOptions, Init, PresentationBackend, PresentationChoice,
};
use page::{Child, Page};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::prelude::*;
type Logic = fn(&Page) -> Child;
thread_local! { static IMAGE_VISIBLE: Cell<bool> = const { Cell::new(false) }; }
fn fixture_page(state: &Page) -> Child {
    if IMAGE_VISIBLE.with(Cell::get) {
        Box::new(cambium::el(
            "div",
            (
                page::page(state),
                cambium::el("img", ())
                    .attr("src", "fixture://owned-image")
                    .attr("style", "width:32px;height:16px;"),
            ),
        ))
    } else {
        page::page(state)
    }
}
#[wasm_bindgen]
pub async fn mount_image_page(canvas_id: String) -> Result<u32, JsValue> {
    IMAGE_VISIBLE.with(|visible| visible.set(true));
    mount_page(canvas_id, "webgl2".into()).await
}
#[wasm_bindgen]
pub fn update_image(id: u32, phase: u32) {
    IMAGE_VISIBLE.with(|visible| visible.set(phase != 2));
    MOUNTS.with(|mounts| {
        let mounts = mounts.borrow();
        let mounted = &mounts[id as usize];
        let mut host = mounted.host().borrow_mut();
        if phase == 2 {
            host.s.shared.images.remove("fixture://owned-image");
        } else {
            host.s.shared.images.insert(
                "fixture://owned-image".into(),
                include_bytes!("blue.png").to_vec(),
            );
        }
        host.s.layout = None;
        cambium_rootstock::HostWindow::request_redraw(mounted.window());
    });
}

thread_local! { static MOUNTS: RefCell<Vec<Mounted<Page,Logic,Child>>> = const { RefCell::new(Vec::new()) }; }
#[wasm_bindgen(start)]
pub fn start() {
    run_static_constructors_once();
    console_error_panic_hook::set_once();
}
#[wasm_bindgen]
pub async fn mount_page(canvas_id: String, choice: String) -> Result<u32, JsValue> {
    let canvas = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(&canvas_id))
        .ok_or_else(|| JsValue::from_str("missing canvas"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let presentation_choice = match choice.as_str() {
        "auto" | "auto-reject" => PresentationChoice::Auto,
        "webgpu" | "webgpu-reject" => PresentationChoice::WebGpu,
        "webgl2" => PresentationChoice::WebGl2,
        _ => return Err(JsValue::from_str("invalid backend")),
    };
    let inits = Rc::new(Cell::new(0));
    let count = inits.clone();
    let options = HostOptions {
        title: "Presentation controls".into(),
        presentation_choice,
        ..Default::default()
    };
    let init = move |_: &dyn cambium_rootstock::HostWindow,
                     _: &cambium_rootstock::WindowCommands,
                     _: &cambium_rootstock::HostWake| {
        count.set(count.get() + 1);
        Init {
            state: Page::default(),
            logic: fixture_page as Logic,
            sheet: page::SHEET.into(),
            fonts: vec![HostFont {
                family: None,
                bytes: include_bytes!(
                    "../../../examples/genet_web_smoke/assets/Roboto-Regular.ttf"
                )
                .to_vec(),
            }],
            images: if IMAGE_VISIBLE.with(Cell::get) {
                vec![HostImage {
                    url: "fixture://owned-image".into(),
                    bytes: include_bytes!("red.png").to_vec(),
                }]
            } else {
                vec![]
            },
        }
    };
    let mounted = if choice.ends_with("-reject") {
        mount_with_webgpu_factory(
            canvas,
            options,
            cambium_genet_web_host::MountLifetime::new(),
            init,
            HostHooks::inert(),
            Box::new(|canvas, _, _, _| {
                let _ = canvas.get_context("webgpu");
                Box::pin(async { Err("injected device refusal after context binding".into()) })
            }),
        )
        .await
    } else {
        mount(canvas, options, init, HostHooks::inert()).await
    }
    .map_err(|error| JsValue::from_str(&error))?;
    mounted
        .window()
        .canvas()
        .set_attribute("data-fixture-init-count", &inits.get().to_string())?;
    Ok(MOUNTS.with(|mounts| {
        let mut mounts = mounts.borrow_mut();
        let id = mounts.len() as u32;
        mounts.push(mounted);
        id
    }))
}
#[wasm_bindgen]
pub fn selected(id: u32) -> String {
    MOUNTS.with(|mounts| {
        let mounts = mounts.borrow();
        let host = mounts[id as usize].host().borrow();
        match host
            .s
            .presentation_driver
            .as_ref()
            .map(|driver| driver.backend())
            .unwrap_or(PresentationBackend::Wgpu)
        {
            PresentationBackend::Wgpu => "webgpu".into(),
            PresentationBackend::WebGl2 => "webgl2".into(),
        }
    })
}
#[wasm_bindgen]
pub fn presses(id: u32) -> u32 {
    MOUNTS.with(|mounts| {
        mounts.borrow()[id as usize]
            .host()
            .borrow()
            .s
            .runner
            .as_ref()
            .unwrap()
            .state()
            .presses
    })
}
#[wasm_bindgen]
pub fn stamp(id: u32) -> String {
    MOUNTS.with(|mounts| {
        format!(
            "{:?}",
            mounts.borrow()[id as usize]
                .host()
                .borrow()
                .s
                .last_redraw_presentation
        )
    })
}

#[wasm_bindgen]
pub fn dispose_mount(id: u32) {
    MOUNTS.with(|mounts| mounts.borrow()[id as usize].dispose());
}

#[wasm_bindgen]
pub fn replace_chooser(id: u32) {
    MOUNTS.with(|mounts| mounts.borrow()[id as usize].host().borrow_mut().s.files = None);
}
#[wasm_bindgen]
pub fn dispose_with_loan(id: u32) {
    MOUNTS.with(|mounts| {
        let mounts = mounts.borrow();
        let mounted = &mounts[id as usize];
        let _loan = mounted.host().borrow_mut();
        mounted.dispose();
    });
}

#[derive(Default)]
struct CaptureProbe {
    pending: Option<Result<cambium_rootstock::CaptureFrame, String>>,
    expected: Option<cambium_rootstock::PresentedFrame>,
    result: Option<Result<(cambium_rootstock::PresentedFrame, cambium_rootstock::Frame), String>>,
}
thread_local! {static CAPTURES: RefCell<Vec<Rc<RefCell<CaptureProbe>>>>=const {RefCell::new(Vec::new())};}
#[wasm_bindgen]
pub fn request_capture(id: u32) -> u32 {
    let probe = Rc::new(RefCell::new(CaptureProbe::default()));
    let result = CAPTURES.with(|captures| {
        let mut captures = captures.borrow_mut();
        let id = captures.len() as u32;
        captures.push(probe.clone());
        id
    });
    MOUNTS.with(|mounts| {
        let mounts = mounts.borrow();
        let mounted = &mounts[id as usize];
        let mut host = mounted.host().borrow_mut();
        let observed = probe.clone();
        host.s.pending_presentation_observer = Some(Box::new(move |_, stamp| {
            observed.borrow_mut().expected = Some(stamp)
        }));
        host.s.pending_stamped_capture = Some(Box::new(move |capture| {
            probe.borrow_mut().pending = Some(capture.map_err(|e| e.to_string()))
        }));
        cambium_rootstock::HostWindow::request_redraw(mounted.window());
    });
    result
}
#[wasm_bindgen]
pub fn poll_capture(id: u32) -> JsValue {
    CAPTURES.with(|captures| {
        let captures = captures.borrow();
        let mut probe = captures[id as usize].borrow_mut();
        if probe.result.is_none() {
            let ready = match probe.pending.as_mut() {
                Some(Ok(job)) => job.poll().map(|r| r.map_err(|e| e.to_string())),
                Some(Err(error)) => Some(Err(error.clone())),
                None => None,
            };
            if let Some(result) = ready {
                probe.pending = None;
                probe.result = Some(result);
            }
        }
        let object = js_sys::Object::new();
        let set = |name: &str, value: JsValue| {
            js_sys::Reflect::set(&object, &name.into(), &value).unwrap();
        };
        set("ready", probe.result.is_some().into());
        if let Some(result) = &probe.result {
            match result {
                Err(error) => set("error", error.clone().into()),
                Ok((stamp, frame)) => {
                    set("stamp", format!("{stamp:?}").into());
                    set("expected", format!("{:?}", probe.expected.unwrap()).into());
                    set("width", frame.width.into());
                    set("height", frame.height.into());
                    set(
                        "rgba",
                        js_sys::Uint8Array::from(frame.rgba.as_slice()).into(),
                    );
                },
            }
        }
        object.into()
    })
}
