// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use crate::surface::{WebSurface, WebWindow};
use cambium_rootstock::{
    PresentationBackend, PresentationChoice, PresentationDriver, backend_attempts,
};
use netrender::NetrenderOptions;
use std::{future::Future, pin::Pin};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::HtmlCanvasElement;

pub type WebGpuBootFuture = Pin<Box<dyn Future<Output = Result<WebSurface, String>>>>;
/// An injectable device/surface attempt, also usable by browser qualification.
pub type WebGpuFactory =
    Box<dyn FnMut(HtmlCanvasElement, u32, u32, NetrenderOptions) -> WebGpuBootFuture>;
pub(crate) struct Selected {
    pub canvas: HtmlCanvasElement,
    pub driver: Option<Box<dyn PresentationDriver>>,
    pub core: Option<std::sync::Arc<genet_render_host::RenderCore>>,
}
pub(crate) fn default_factory() -> WebGpuFactory {
    Box::new(|canvas, w, h, options| Box::pin(WebSurface::boot(canvas, w, h, options)))
}
pub(crate) async fn select(
    mut canvas: HtmlCanvasElement,
    choice: PresentationChoice,
    options: &dyn Fn() -> NetrenderOptions,
    factory: &mut WebGpuFactory,
    wgpu_portable_scene: bool,
) -> Result<Selected, String> {
    let mut failures = Vec::new();
    for (index, backend) in backend_attempts(choice).iter().enumerate() {
        if index > 0 {
            canvas = fresh_canvas(&canvas)?;
        }
        let (w, h) = WebWindow::new(canvas.clone()).physical_size();
        canvas.set_width(w.max(1));
        canvas.set_height(h.max(1));
        match backend {
            PresentationBackend::Wgpu => {
                match factory(canvas.clone(), w.max(1), h.max(1), options()).await {
                    Ok(surface) => {
                        let core = surface.shared_core();
                        let target = canvas.clone();
                        let driver = cambium_rootstock::WgpuDriver::new(
                            Box::new(surface),
                            core.shared_recovery(),
                            Box::new(move |core| {
                                let (w, h) = WebWindow::new(target.clone()).physical_size();
                                WebSurface::from_core(target.clone(), core, w, h).map(|surface| {
                                    Box::new(surface) as Box<dyn cambium_rootstock::Surface>
                                })
                            }),
                        );
                        let driver = if wgpu_portable_scene {
                            portable_wgpu(driver)?
                        } else {
                            driver
                        };
                        return Ok(Selected {
                            canvas,
                            driver: Some(Box::new(driver)),
                            core: Some(core),
                        });
                    },
                    Err(error) => failures.push(format!("WebGPU: {error}")),
                }
            },
            PresentationBackend::WebGl2 => {
                #[cfg(feature = "webgl")]
                match super::webgl::WebGlPresentation::boot(
                    canvas.clone(),
                    netrender::vello_backends::SparseResourceLimits::default(),
                )
                .await
                {
                    Ok(driver) => {
                        return Ok(Selected {
                            canvas,
                            driver: Some(Box::new(driver)),
                            core: None,
                        });
                    },
                    Err(error) => failures.push(format!("WebGL2: {error}")),
                }
                #[cfg(not(feature = "webgl"))]
                failures.push("WebGL2: not compiled into this host".into());
            },
        }
    }
    Err(failures.join("; "))
}
/// The WebGPU route presents the same flattened portable scene as WebGL2, so
/// producer scenes draw in the page pass with no resampled intermediate.
#[cfg(feature = "webgl")]
fn portable_wgpu(
    driver: cambium_rootstock::WgpuDriver,
) -> Result<cambium_rootstock::WgpuDriver, String> {
    Ok(
        driver.with_portable_scene(Box::new(|capture, width, height, scale| {
            crate::portable::materialize(
                capture,
                width,
                height,
                scale,
                netrender::vello_backends::SparseResourceLimits::default(),
            )
            .map_err(|error| (error.source_key, error.message))
        })),
    )
}
#[cfg(not(feature = "webgl"))]
fn portable_wgpu(
    _: cambium_rootstock::WgpuDriver,
) -> Result<cambium_rootstock::WgpuDriver, String> {
    Err("portable WebGPU presentation is not compiled into this host".into())
}
fn fresh_canvas(old: &HtmlCanvasElement) -> Result<HtmlCanvasElement, String> {
    let document = old.owner_document().ok_or("canvas has no document")?;
    let focused = document
        .active_element()
        .is_some_and(|element| element.is_same_node(Some(old)));
    let canvas = old
        .clone_node_with_deep(false)
        .map_err(|e| format!("clone canvas: {e:?}"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| "canvas clone changed type")?;
    old.parent_node()
        .ok_or("canvas is detached")?
        .replace_child(&canvas, old)
        .map_err(|e| format!("replace failed canvas: {e:?}"))?;
    if focused {
        let _ = canvas.focus();
    }
    Ok(canvas)
}
pub(crate) async fn next_frame() -> Result<(), String> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let callback = Closure::once_into_js(move |_: f64| {
            let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
        });
        match web_sys::window()
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("no window"))
            .and_then(|window| window.request_animation_frame(callback.unchecked_ref()))
        {
            Ok(_) => {},
            Err(error) => {
                let _ = reject.call1(&wasm_bindgen::JsValue::NULL, &error);
            },
        }
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(|e| format!("animation turn: {e:?}"))
}
