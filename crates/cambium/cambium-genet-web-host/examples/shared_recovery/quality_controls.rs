// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Finite actual host/Core controls. Compiled only by current-frame-controls.
use super::*;
use cambium_rootstock::{BrowserFrameProgress, BrowserWorkCounts, ProducedTexture, ProducerContext,
    ProducerSemantics, SceneProducer, TextureProducer};
use genet_render_host::{AcceptedBrowserFrame, BrowserFrameCommands, BrowserFrameStartError,
    BrowserFrameSubmission, RasterizeFailure};
use std::{any::Any, sync::atomic::Ordering};
use tabard_workshop::ReaderSpecimen;

#[wasm_bindgen(inline_js = r#"
export function qc_pop_three_owned_scopes(device) {
  const promises = [device.popErrorScope(),device.popErrorScope(),device.popErrorScope()];
  return Promise.allSettled(promises).then(rows => rows.map(row => ({
    status:row.status, cleanNull:row.status === 'fulfilled' && row.value === null
  })));
}
export function qc_raf_turn(timeout) {
  return new Promise((resolve,reject) => {
    let id;
    const timer = setTimeout(() => { cancelAnimationFrame(id); reject(new Error('finite RAF deadline')); },timeout);
    id = requestAnimationFrame(value => { clearTimeout(timer); resolve(value); });
  });
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)] fn qc_pop_three_owned_scopes(device:&JsValue) -> Result<js_sys::Promise,JsValue>;
    #[wasm_bindgen(catch)] fn qc_raf_turn(timeout:f64) -> Result<js_sys::Promise,JsValue>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fault { Clean, Validation, RejectedScope, RoundCap, SurfaceValidation, SurfaceRejectedScope }
struct Control {
    fault:Cell<Fault>, epoch:Cell<u64>, source_calls:Cell<u64>, scale:Cell<f32>,
    surface_control_drops:Cell<u64>, surface_control_drop_borrow_safe:Cell<bool>,
    request_in_frame:Cell<bool>, request_after_frame:Cell<bool>,
    early_pop:RefCell<Option<JsFuture>>, producer:RefCell<std::rc::Weak<RefCell<ControlledProducer>>>,
}
impl Control {
    fn new() -> Self { Self { fault:Cell::new(Fault::Clean),epoch:Cell::new(1),source_calls:Cell::new(0),
        surface_control_drops:Cell::new(0),surface_control_drop_borrow_safe:Cell::new(true),
        scale:Cell::new(1.0),request_in_frame:Cell::new(false),request_after_frame:Cell::new(false),early_pop:RefCell::new(None),producer:RefCell::new(std::rc::Weak::new()) } }
    fn select(&self, fault:Fault) -> Result<(),String> {
        self.epoch.set(self.epoch.get().checked_add(1).ok_or("finite fixture epoch exhausted")?);
        self.fault.set(fault); Ok(())
    }
}
// The original Workshop owns this Rc. The fixture wrapper injects failures,
// and its clean callback calls the actual ReaderSpecimen implementation.
struct ControlledReader {
    original:Rc<RefCell<ReaderSpecimen>>, control:Rc<Control>,
    seen:Cell<(usize,u64,u64)>, revision:Cell<u64>,
}
fn reader_revision(source:&ControlledReader) -> u64 {
    let identity=(Rc::as_ptr(&source.original) as usize,source.original.borrow().revision(),source.control.epoch.get());
    if source.seen.get()!=identity {
        source.seen.set(identity);
        // This finite fixture has fewer than 64 deliberate mutations.
        source.revision.set(source.revision.get().checked_add(1).expect("bounded fixture source revision"));
    }
    source.revision.get()
}
fn reader_frame(source:&mut ControlledReader,w:u32,h:u32) -> Scene {
    source.control.source_calls.set(source.control.source_calls.get()+1);
    if source.control.fault.get()!=Fault::RoundCap { return source.original.borrow_mut().frame(w,h); }
    let mut scene=Scene::new(w,h);
    let mut path=netrender::scene::ScenePath::new(); path.move_to(8.,12.).line_to(24.,12.);
    scene.push_shape_stroked(path,[0.25,0.5,0.75,1.],40_000_000.0/source.control.scale.get());
    let netrender::SceneOp::Shape(shape)=&mut scene.ops[0] else { unreachable!() };
    shape.stroke.as_mut().unwrap().cap=netrender::scene::SceneStrokeCap::Round;
    scene
}
struct ControlledProducer { inner:SceneProducer<ControlledReader>, control:Rc<Control> }
impl TextureProducer for ControlledProducer {
    fn render(&mut self,cx:&ProducerContext<'_>)->Option<ProducedTexture>{ self.inner.render(cx) }
    fn render_checked(&mut self,cx:&ProducerContext<'_>)->Result<Option<ProducedTexture>,RasterizeFailure>{ self.inner.render_checked(cx) }
    fn supports_deferred_acceptance(&self)->bool{ self.inner.supports_deferred_acceptance() }
    fn prepared_is_current(&self)->bool{ self.inner.prepared_is_current() }
    fn prepare_checked(&mut self,cx:&ProducerContext<'_>,commands:&BrowserFrameCommands<'_>)
        ->Result<Option<ProducedTexture>,RasterizeFailure> {
        self.control.scale.set(cx.frame.layout_scale);
        let output=self.inner.prepare_checked(cx,commands)?;
        if output.is_some() {
            match self.control.fault.get() {
                Fault::Validation => {
                    // Real WebGPU validation: copying eight bytes between
                    // four-byte buffers. No synthetic Vello error is supplied.
                    let make=||commands.device().create_buffer(&wgpu::BufferDescriptor {
                        label:Some("finite QC invalid-copy control"),size:4,
                        usage:wgpu::BufferUsages::COPY_SRC|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false,
                    });
                    let from=make();let to=make();
                    let mut encoder=commands.device().create_command_encoder(&Default::default());
                    encoder.copy_buffer_to_buffer(&from,0,&to,0,8); commands.queue().submit([encoder.finish()]);
                },
                Fault::RejectedScope => {
                    // Consume exactly the owning raw scopes; its actual seal
                    // subsequently pops an empty stack and must preserve real
                    // JS rejection rather than treating it as resolved null.
                    let device=commands.device().as_webgpu().expect("actual WebGPU required");
                    let promise=qc_pop_three_owned_scopes(device.as_ref()).map_err(|_| injected_api_error())?;
                    *self.control.early_pop.borrow_mut()=Some(JsFuture::from(promise));
                },
                _=>{},
            }
        }
        Ok(output)
    }
    fn commit_prepared(&mut self){self.inner.commit_prepared()}
    fn commit_prepared_retiring(&mut self)->Option<Box<dyn Any>>{self.inner.commit_prepared_retiring()}
    fn abort_prepared(&mut self){self.inner.abort_prepared()}
    fn take_prepared(&mut self)->Option<Box<dyn Any>>{self.inner.take_prepared()}
    fn take_suspended_resources(&mut self)->Option<Box<dyn Any>>{self.inner.take_suspended_resources()}
    fn suspend(&mut self){self.inner.suspend()}
    fn retire(&mut self){self.inner.retire()}
    fn semantics(&mut self)->Option<ProducerSemantics>{self.inner.semantics()}
}
// Installed only after initial private work has drained. The actual host
// invokes this synchronously inside phase 2, not in producer preparation.
struct SurfaceControlDropProbe { on_drop:Box<dyn Fn()> }
impl Drop for SurfaceControlDropProbe { fn drop(&mut self){(self.on_drop)();} }
fn install_surface_controls(mount:&BrowserMount,control:Rc<Control>)->Result<(),String> {
    let host=mount.host();let weak=Rc::downgrade(&host);let dropped=control.clone();
    let probe=SurfaceControlDropProbe { on_drop:Box::new(move || {
        dropped.surface_control_drops.set(dropped.surface_control_drops.get()+1);
        if let Some(host)=weak.upgrade() {
            if host.try_borrow_mut().is_err(){dropped.surface_control_drop_borrow_safe.set(false);}
        }
    }) };

    let mut callback=Some(Box::new(move |point:cambium_rootstock::BrowserSurfaceControlPoint,commands:&BrowserFrameCommands<'_>| {
        let _keep_probe=&probe;
        if point!=cambium_rootstock::BrowserSurfaceControlPoint::AfterCompose { return Ok(()); }
        match control.fault.get() {
            Fault::SurfaceValidation => {
                let make=||commands.device().create_buffer(&wgpu::BufferDescriptor {
                    label:Some("finite actual phase-2 invalid-copy control"),size:4,
                    usage:wgpu::BufferUsages::COPY_SRC|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false,
                });
                let from=make();let to=make();
                let mut encoder=commands.device().create_command_encoder(&Default::default());
                encoder.copy_buffer_to_buffer(&from,0,&to,0,8);
                commands.queue().submit([encoder.finish()]);
            },
            Fault::SurfaceRejectedScope => {
                let device=commands.device().as_webgpu().ok_or_else(injected_api_error)?;
                let promise=qc_pop_three_owned_scopes(device.as_ref()).map_err(|_|injected_api_error())?;
                *control.early_pop.borrow_mut()=Some(JsFuture::from(promise));
            },
            _=>{},
        }
        Ok(())
    }) as cambium_rootstock::BrowserSurfaceControlFn);
    {
        let host=mount.host();let mut host=host.borrow_mut();
        host.set_browser_surface_control(&mut callback)
            .map_err(|e|format!("actual phase-2 diagnostic hook admission: {e:?}"))?;
    }
    // Previous callback (if any) and its resources retire outside Host RefMut.
    drop(callback);
    Ok(())
}
fn injected_api_error()->RasterizeFailure {
    RasterizeFailure::ProviderError(Arc::new(netrender::VelloError::WgpuErrorFromScope(wgpu::Error::Internal {
        source:Box::new(std::io::Error::other("fixture raw API threw before scope rejection control")),
        description:"fixture raw API threw before scope rejection control".into(),
    })))
}
fn controlled_hooks(control:Rc<Control>) -> HostHooks<WorkshopState,Logic,WorkshopView> {
    let mut ordinary=hooks(); let mut previews=PreviewBindings::default();
    let mut bound:Option<Rc<RefCell<ControlledReader>>>=None;
    let after_control=control.clone();
    ordinary.after_frame=Box::new(move |ctx| {
        if after_control.request_after_frame.replace(false) {
            if let Some(window)=ctx.window { window.request_redraw(); }
        }
    });
    ordinary.frame=Box::new(move |ctx| {
        if control.request_in_frame.get() { if let Some(window)=ctx.window {window.request_redraw();} }
        if bound.is_none() {
            let source=Rc::new(RefCell::new(ControlledReader {
                original:ctx.runner.state().reader_preview(),control:control.clone(),seen:Cell::new((0,0,0)),revision:Cell::new(0),
            }));
            let inner=SceneProducer::new(source.clone(),0x7163_7265_6164_6572,reader_frame,reader_revision,|s| {
                Some(ProducerSemantics { role:Some(cambium_rootstock::ProducerRole::Image),
                    name:Some(s.original.borrow().accessible_name().into()),children:Vec::new() })
            });
            let producer=Rc::new(RefCell::new(ControlledProducer {inner,control:control.clone()}));
            *control.producer.borrow_mut()=Rc::downgrade(&producer);
            ctx.producers.register(READER_LEAF_KEY,producer,&[])
                .expect("fixture owns reserved reader slot before ordinary registration");
            bound=Some(source);
        }
        bound.as_ref().unwrap().borrow_mut().original=ctx.runner.state().reader_preview();
        // Actual production bindings still supply graph/stylesheet and observe
        // the same normal reader source. They never replace the fixture slot.
        previews.register(ctx.runner.state(),ctx.leaves,ctx.producers);
        false
    }); ordinary
}
fn work(mount:&BrowserMount)->Result<BrowserWorkCounts,String> {
    let counts=mount.host().borrow().browser_work_counts().ok_or("work observation not enabled")?;
    if counts.saturated{return Err("work observation saturated".into())} Ok(counts)
}
fn work_json(counts:BrowserWorkCounts)->Value {
    json!({"relayout":counts.relayout,"accessibility":counts.accessibility,"preparation":counts.preparation,
        "surface_submission":counts.surface_submission,"surface_acquired":counts.surface_acquired,
        "surface_present_call":counts.surface_present_call,"surface_skipped":counts.surface_skipped,
        "surface_quality_accepted":counts.surface_completion_accepted,"surface_completion_refused":counts.surface_completion_refused,
        "publication":counts.publication,"saturated":counts.saturated})
}
fn identities(mount:&BrowserMount)->Result<Value,String> {
    let host=mount.host().borrow();
    let images:Result<Vec<_>,String>=KEYS.into_iter().map(|key|host.s.shared.producers.image_identity(key)
        .map(|image|image_json(key,image)).ok_or("accepted producer identity unavailable".into())).collect();
    let runner=host.s.runner.as_ref().ok_or("actual runner unavailable")?;
    Ok(json!({"images":images?,"sequence":host.s.presentation_sequence,
        "viewport":host.s.window.as_ref().map(|window|window.inner_size()),
        "reader_rc":format!("{:p}",Rc::as_ptr(&runner.state().reader_preview())),
        "stylesheet_rc":format!("{:p}",Rc::as_ptr(&runner.state().stylesheet_preview())),
        "theme":runner.state().draft_theme(),"mode":runner.state().mode_key()}))
}
fn admit_copies(mounts:&[BrowserMount],roots:&[&Frame],paired:bool)->Result<usize,String>{
    let factor=if paired {2}else{1};let mut total=0usize;
    for root in roots {
        total=frame_bytes(root.width,root.height)?.checked_mul(factor).and_then(|bytes|total.checked_add(bytes))
            .ok_or("copied root byte admission overflow")?;
    }
    for mount in mounts {
        let host=mount.host().borrow();
        for key in KEYS {
            let image=host.s.shared.producers.image_identity(key).ok_or("copy plan source missing")?;
            total=frame_bytes(image.size[0],image.size[1])?.checked_mul(factor).and_then(|bytes|total.checked_add(bytes))
                .ok_or("copied source byte admission overflow")?;
        }
    }
    if total>RETAINED_LIMIT{return Err("planned complete copied images exceed unchanged 128 MiB limit".into())}Ok(total)
}

async fn idle(mounts:&[BrowserMount])->Result<(),String>{
    wait(||Ok(mounts.iter().all(|mount| {
        let host=mount.host().borrow(); !host.has_pending_browser_frame()
            && (mount.lifetime().status().phase==MountPhase::Disposed || host.s.hidden
                || !mount.window().pending_frame_flag().load(Ordering::Relaxed))
    }))).await
}
async fn raf_turns(count:usize)->Result<(),String>{
    let deadline=now()?+5000.;
    for _ in 0..count {
        let remaining=deadline-now()?;if remaining<=0.{return Err("finite RAF control deadline".into())}
        let promise=qc_raf_turn(remaining).map_err(|e|format!("RAF admission: {e:?}"))?;
        JsFuture::from(promise).await.map_err(|e|format!("RAF deadline: {e:?}"))?;
        if now()?>=deadline{return Err("RAF completion after deadline".into())}
    } Ok(())
}
async fn stable_idle(mounts:&[BrowserMount],count:usize)->Result<Value,String>{
    idle(mounts).await?;
    let before:Result<Vec<_>,_>=mounts.iter().map(work).collect();let before=before?;
    let sequences:Vec<_>=mounts.iter().map(|m|m.host().borrow().s.presentation_sequence).collect();
    raf_turns(count).await?;
    for (i,mount) in mounts.iter().enumerate(){
        if work(mount)?!=before[i] || mount.host().borrow().s.presentation_sequence!=sequences[i] {
            return Err("no-animation idle RAF repeated actual host work".into())
        }
    }
    Ok(json!({"raf_callbacks":count,"work":before.into_iter().map(work_json).collect::<Vec<_>>(),"sequences":sequences}))
}
async fn begin(core:&Arc<RenderCore>)->Result<BrowserFrameSubmission,String>{
    let mut frame=None;
    wait(||match core.try_begin_browser_frame(){
        Ok(value)=>{frame=Some(value);Ok(true)},Err(BrowserFrameStartError::Busy)=>Ok(false),
        Err(error)=>Err(format!("actual frame admission: {error:?}")),
    }).await?;frame.ok_or("frame admission absent".into())
}
async fn finish(submission:BrowserFrameSubmission)->Result<Result<AcceptedBrowserFrame,RasterizeFailure>,String>{
    let mut pending=submission.seal().map_err(|e|format!("actual seal: {e:?}"))?;let mut output=None;
    wait(||if let Some(result)=pending.poll(){output=Some(result);Ok(true)}else{Ok(false)}).await?;
    output.ok_or("actual completion absent".into())
}
fn zero(core:&RenderCore)->Result<(),String>{
    let renderer=core.try_renderer().map_err(|e|format!("quality observation not exclusive: {e:?}"))?;
    if renderer.vello_curve_quality_status()!=Some((0,0)){return Err("accepted/retired FIFO not zero count/bytes".into())}Ok(())
}
async fn clean_scope_and_ignored_refusal(core:&Arc<RenderCore>)->Result<Value,String>{
    let mut clean=begin(core).await?;clean.submit(|_|Ok(())).map_err(|e|format!("clean scoped submission: {e:?}"))?;
    let accepted=finish(clean).await?.map_err(|e|format!("clean null scope completion: {e:?}"))?;
    accepted.check_current().map_err(|e|format!("clean acceptance: {e:?}"))?;drop(accepted);zero(core)?;
    let mut scene=Scene::new(32,24);let mut path=netrender::scene::ScenePath::new();
    path.move_to(0.,0.).cubic_to(0.,1e30,1e30,1e30,31.,0.).close();scene.push_shape_filled(path,[1.,0.,0.,1.]);
    let mut refused=begin(core).await?;
    // Intentionally ignore the immediate Result: its actual provider refusal
    // must remain latched and prevent any accepted token after clean scopes.
    let immediate=refused.submit(|commands|commands.rasterize_scaled_for(0x7163_6573_7469,&scene,32,24,
        ColorLoad::Clear(wgpu::Color::TRANSPARENT),1.));
    if !matches!(&immediate,Err(RasterizeFailure::ProviderError(e)) if matches!(&**e,netrender::VelloError::CurveEstimateLimit{..})){
        return Err(format!("actual huge-curve preflight did not refuse precisely: {immediate:?}"))
    }
    drop(immediate);
    let failure=finish(refused).await?;
    let error=match failure{Err(RasterizeFailure::ProviderError(e)) if matches!(&*e,netrender::VelloError::CurveEstimateLimit{..})=>e,
        Ok(accepted)=>{drop(accepted);return Err("ignored submission refusal minted an accepted token".into())},
        Err(error)=>return Err(format!("ignored submission refusal lost precise type: {error:?}"))};
    zero(core)?;if core.device_failure().is_some(){return Err("healthy curve estimate refusal became device loss".into())}
    // Separate actual command-method refusal from the already covered
    // ignored outer submission Result. This closure deliberately returns Ok.
    let mut ignored_command=begin(core).await?;
    let mut immediate_command_error=None;
    ignored_command.submit(|commands| {
        immediate_command_error=commands.rasterize_scaled_for(0x7163_6573_7469,&scene,32,24,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),1.).err();
        Ok(())
    }).map_err(|e|format!("closure-Ok command control failed synchronously: {e:?}"))?;
    if !matches!(&immediate_command_error,Some(RasterizeFailure::ProviderError(e))
        if matches!(&**e,netrender::VelloError::CurveEstimateLimit{..})) {
        return Err(format!("actual ignored command lacked precise provider refusal: {immediate_command_error:?}"))
    }
    let ignored_command_error=match finish(ignored_command).await? {
        Err(RasterizeFailure::ProviderError(e)) if matches!(&*e,netrender::VelloError::CurveEstimateLimit{..})=>e,
        Ok(accepted)=>{drop(accepted);return Err("ignored command refusal with closure Ok minted acceptance".into())},
        Err(error)=>return Err(format!("ignored command refusal lost precise type: {error:?}")),
    };
    zero(core)?;
    if core.device_failure().is_some(){return Err("ignored healthy command refusal became device loss".into())}
    let mut valid=Scene::new(32,24);valid.push_rect(0.,0.,32.,24.,[0.25,0.5,0.75,1.]);
    let (texture,_view)=owned_raster(core,0x7163_6573_7469,&valid,32,24,ColorLoad::Clear(wgpu::Color::TRANSPARENT),1.).await?;
    let started=now()?;let pixels=collect(core.start_rgba8_readback(&texture,32,24)?,started).await?;zero(core)?;
    Ok(json!({"clean_null_scopes":"actual accepted token from strict three-filter and queue ledger",
        "ignored_submission_error":format!("{error:?}"),
        "ignored_command_with_closure_ok":format!("{ignored_command_error:?}"),"counter_count":0,"counter_bytes":0,
        "valid_next_same_core":{"generation":core.generation(),"sha256":digest(&pixels)}}))
}

struct Armed {
    paints:Rc<RefCell<Vec<Result<paint_list_api::PaintEnvelope,CaptureRefusal>>>>,
    copied:Rc<RefCell<Vec<Result<Result<(PresentedFrame,f64,PendingFrame),String>,CaptureRefusal>>>>,
    observed:Rc<RefCell<Vec<Result<Value,String>>>>,
}
fn arm(mount:&BrowserMount)->Armed {
    let paints=Rc::new(RefCell::new(Vec::new()));let copied=Rc::new(RefCell::new(Vec::new()));
    let observed=Rc::new(RefCell::new(Vec::new()));
    let mut host=mount.host().borrow_mut();
    let out=paints.clone();host.s.pending_paint_capture=Some(Box::new(move |result|out.borrow_mut().push(result)));
    let out=copied.clone();host.s.pending_stamped_capture=Some(Box::new(move |result| {
        out.borrow_mut().push(result.map(|capture| {
            let presentation=capture.presentation;
            frame_bytes(presentation.width,presentation.height)?;
            Ok((presentation,now()?,start_frame_readback(capture.surface,capture.view,
                presentation.width,presentation.height)?))
        }));
    }));
    let out=observed.clone();host.s.pending_presentation_observer=Some(Box::new(move |ctx,p|out.borrow_mut().push(seal(ctx,p))));
    Armed {paints,copied,observed}
}
fn envelope_receipt(envelope:&paint_list_api::PaintEnvelope, seal:&Value,p:PresentedFrame)->Result<Value,String>{
    if envelope.viewport.width!=(p.width as f32/p.layout_scale) as i32
        || envelope.viewport.height!=(p.height as f32/p.layout_scale) as i32 {
        return Err("paint envelope does not match presented logical viewport".into())
    }
    let mut draws=Vec::new();
    for command in &envelope.commands {
        if let paint_list_api::PaintCmd::DrawExternalTexture(draw)=command {
            let image=seal["images"].as_array().ok_or("presentation image ledger missing")?.iter()
                .find(|image|image["local_key"]==draw.texture_key.to_string()).ok_or("paint resource absent from accepted seal")?;
            if draw.content_generation!=image["stamp"].as_u64(){return Err("paint uses foreign/stale accepted source stamp".into())}
            draws.push(json!({"local_key":draw.texture_key.to_string(),"stamp":draw.content_generation}));
        }
    }
    if draws.is_empty(){return Err("actual capture lacks a visible preview command".into())}
    // Laid-out but fully paint-culled slots need not emit a draw. Every emitted
    // reference must be present/stamped; separate sources() copies both full
    // physical producer images even when the viewport clips one of them.
    Ok(json!({"viewport":[envelope.viewport.width,envelope.viewport.height],"generation":envelope.generation,
        "command_count":envelope.commands.len(),"font_count":envelope.fonts.len(),"image_count":envelope.images.len(),"external_draws":draws,
        "source":"actual intact envelope prepared for this accepted target; no reconstructed scene or command removal"}))
}
async fn accepted_capture(armed:&Armed)->Result<(Capture,Value),String>{
    wait(||Ok(!armed.paints.borrow().is_empty()&&!armed.copied.borrow().is_empty())).await?;
    if armed.paints.borrow().len()!=1||armed.copied.borrow().len()!=1||armed.observed.borrow().len()!=1{
        return Err("one-shot callback count differs from exactly one accepted presentation".into())
    }
    let envelope=armed.paints.borrow_mut().pop().ok_or("paint result absent")?.map_err(|e|format!("paint refused: {e:?}"))?;
    let (p,started,pending)=armed.copied.borrow_mut().pop().ok_or("stamped result absent")?.map_err(|e|format!("stamped refused: {e:?}"))??;
    let seal=armed.observed.borrow_mut().pop().ok_or("accepted seal absent")??;
    if seal["presentation"]!=presentation(p){return Err("paint/pixel observer presentation mismatch".into())}
    let receipt=envelope_receipt(&envelope,&seal,p)?;
    let frame=collect(pending,started).await?;
    Ok((Capture{frame,seal},receipt))
}
async fn submit_unarmed(mounts:&[BrowserMount],index:usize)->Result<Value,String>{
    idle(mounts).await?;
    let before=identities(&mounts[index])?;
    let mut submitted=false;
    wait(|| {
        let mount=&mounts[index];let mut host=mount.host().borrow_mut();
        if host.s.pending_paint_capture.is_some()||host.s.pending_stamped_capture.is_some(){return Err("unarmed submission already has callback".into())}
        if host.has_submitted_browser_frame(){submitted=true;return Ok(true)}
        let _=mount.window().take_pending_frame();
        let progress=host.redraw_browser_turn(None);
        if matches!(progress,BrowserFrameProgress::Refused){return Err(format!("unarmed actual host refused: {:?}",host.s.last_raster_failure))}
        submitted=host.has_submitted_browser_frame();
        // Direct fixture use of the exact host seam acknowledges this initial
        // synchronous preparation. The separate RAF controls exercise adapter
        // request ownership rather than relying on this direct route.
        if submitted {let _=mount.window().take_pending_frame();}
        Ok(submitted)
    }).await?;
    if !submitted{return Err("actual private frame was never submitted".into())}
    if identities(&mounts[index])?!=before{return Err("unaccepted submission changed public image/stamp/presentation".into())}
    Ok(before)
}
async fn late_capture(mounts:&[BrowserMount])->Result<Value,String>{
    let before=submit_unarmed(mounts,0).await?;
    let prepared=work(&mounts[0])?;
    let armed=arm(&mounts[0]);mounts[0].window().request_redraw();
    let (copied,envelope)=accepted_capture(&armed).await?;
    let after=identities(&mounts[0])?;
    if after["sequence"].as_u64()!=before["sequence"].as_u64().and_then(|value|value.checked_add(1)){
        return Err("missing-envelope target presented before its replacement".into())
    }
    if work(&mounts[0])?.preparation<=prepared.preparation{return Err("late arm did not cause fresh preparation".into())}
    let idle=stable_idle(mounts,6).await?;
    if !armed.paints.borrow().is_empty()||!armed.copied.borrow().is_empty()||!armed.observed.borrow().is_empty(){
        return Err("late one-shot callback replayed after collection".into())
    }
    Ok(json!({"before":before,"after":after,"prepared_before_arm":work_json(prepared),"envelope":envelope,
        "accepted_pixels_sha256":digest(&copied.frame),"no_animation_idle":idle,"callback_counts":{"paint":1,"stamped":1,"presentation":1}}))
}
async fn negative(mounts:&[BrowserMount],control:&Rc<Control>,fault:Fault)->Result<Value,String>{
    idle(mounts).await?;
    let baseline=capture(&mounts[0]).await?;
    let planned_bytes=admit_copies(&mounts[..1],&[&baseline.frame],true)?;
    let source_before=sources(&mounts[0],0).await?;
    idle(mounts).await?;
    let before=identities(&mounts[0])?;let core=mounts[0].host().borrow().s.shared.render_core.clone().ok_or("core")?;
    let work_before=work(&mounts[0])?;
    control.select(fault)?;let armed=arm(&mounts[0]);mounts[0].window().request_redraw();
    wait(||Ok(!armed.paints.borrow().is_empty()&&!armed.copied.borrow().is_empty())).await?;
    if armed.paints.borrow().len()!=1||armed.copied.borrow().len()!=1||!armed.observed.borrow().is_empty(){
        return Err("refused frame published or did not refuse captures exactly once".into())
    }
    let paint=armed.paints.borrow_mut().pop().unwrap().err().ok_or("fault paint unexpectedly accepted")?;
    let copied=armed.copied.borrow_mut().pop().unwrap().err().ok_or("fault target unexpectedly accepted")?;
    let precise=|error:&CaptureRefusal|match error {
        CaptureRefusal::ProviderError{generation,error,..} if *generation==core.generation()=>match (fault,&**error){
            (Fault::RoundCap,netrender::VelloError::CurveFlatteningLimit)=>true,
            (Fault::Validation | Fault::SurfaceValidation,netrender::VelloError::WgpuErrorFromScope(wgpu::Error::Validation{..}))=>true,
            (Fault::RejectedScope | Fault::SurfaceRejectedScope,netrender::VelloError::WgpuErrorFromScope(wgpu::Error::Internal{description,..}))=>description.contains("actual GPU scope/queue promise rejected"),
            _=>false,
        },_=>false,
    };
    if !precise(&paint)||!precise(&copied){return Err(format!("wrong actual {fault:?} refusal: paint={paint:?}; copied={copied:?}"))}
    if identities(&mounts[0])?!=before{return Err("negative changed accepted image/stamp/presentation/application identity".into())}
    idle(mounts).await?;zero(&core)?;
    let work_refused=work(&mounts[0])?;
    let phase2=matches!(fault,Fault::SurfaceValidation | Fault::SurfaceRejectedScope);
    let expected=if phase2 {1}else{0};
    if work_refused.surface_submission.checked_sub(work_before.surface_submission)!=Some(expected)
        || work_refused.surface_acquired.checked_sub(work_before.surface_acquired)!=Some(expected)
        || work_refused.surface_present_call.checked_sub(work_before.surface_present_call)!=Some(expected)
        || work_refused.surface_completion_refused.checked_sub(work_before.surface_completion_refused)!=Some(expected)
        || work_refused.surface_completion_accepted!=work_before.surface_completion_accepted
        || work_refused.surface_skipped!=work_before.surface_skipped
        || work_refused.publication!=work_before.publication {
        return Err("actual negative did not bind precise phase-2 branch counts without publication".into())
    }
    let refused_idle=stable_idle(mounts,6).await?;
    let early=if matches!(fault,Fault::RejectedScope | Fault::SurfaceRejectedScope) {
        let mut future=control.early_pop.borrow_mut().take().ok_or("actual raw pop proof missing")?;let mut rows=None;
        wait(|| {let mut cx=TaskContext::from_waker(Waker::noop());match Pin::new(&mut future).poll(&mut cx){
            Poll::Pending=>Ok(false),Poll::Ready(value)=>{rows=Some(value.map_err(|e|format!("early pop proof: {e:?}"))?);Ok(true)}
        }}).await?;
        let rows=js_sys::Array::from(&rows.ok_or("early proof absent")?);
        if rows.length()!=3||rows.iter().any(|row|js_sys::Reflect::get(&row,&JsValue::from_str("cleanNull")).ok().and_then(|v|v.as_bool())!=Some(true)){
            return Err("actual resolved-null scopes were not distinct from later empty-stack rejection".into())}
        json!({"actual_owned_pop_count":3,"fulfilled_clean_null_count":3,"later_empty_stack_rejection_observed":true})
    }else{Value::Null};
    if core.device_failure().is_some(){return Err("healthy provider refusal triggered device loss".into())}
    control.select(Fault::Clean)?;
    let recovered=capture(&mounts[0]).await?;
    let source_after=sources(&mounts[0],0).await?;
    let source_pixels:Result<Vec<_>,String>=(0..2).map(|slot|compare_pixels(&source_before[slot].frame,&source_after[slot].frame)).collect();
    idle(mounts).await?;zero(&core)?;
    let current=mounts[0].host().borrow().s.shared.render_core.clone().ok_or("core after healthy refusal")?;
    if !Arc::ptr_eq(&current,&core){return Err("healthy refusal replaced the actual Core".into())}
    Ok(json!({"fault":format!("{fault:?}"),"paint_error":format!("{paint:?}"),"stamped_error":format!("{copied:?}"),
        "before":before,"work_before":work_json(work_before),"work_refused":work_json(work_refused),"refused_idle":refused_idle,"unaccepted_publication_unchanged":true,"strict_promise_proof":early,"planned_copy_bytes":planned_bytes,
        "retired_quality_count":0,"retired_quality_bytes":0,"valid_next_same_core":{"generation":current.generation(),
        "root_pixels":compare_pixels(&baseline.frame,&recovered.frame)?,"complete_source_pixels":source_pixels?}}))
}
async fn hidden_work(mounts:&[BrowserMount],control:&Rc<Control>)->Result<Value,String>{
    idle(mounts).await?;mounts[0].host().borrow_mut().set_hidden(true);mounts[0].window().request_redraw();
    raf_turns(2).await?;let before=work(&mounts[0])?;let sequence=mounts[0].host().borrow().s.presentation_sequence;
    let calls=control.source_calls.get();raf_turns(12).await?;
    if work(&mounts[0])?!=before||mounts[0].host().borrow().s.presentation_sequence!=sequence||control.source_calls.get()!=calls{
        return Err("healthy hidden owed request repeated actual layout/a11y/preparation/source work".into())}
    if !mounts[0].window().pending_frame_flag().load(Ordering::Relaxed){return Err("hidden owed flag was lost".into())}
    mounts[0].host().borrow_mut().set_hidden(false);let visible=capture(&mounts[0]).await?;
    let after=work(&mounts[0])?;if after.preparation<=before.preparation{return Err("unhide did not prepare an actual frame".into())}
    let idle=stable_idle(mounts,6).await?;
    let pending_before=submit_unarmed(mounts,0).await?;
    mounts[0].host().borrow_mut().set_hidden(true);
    wait(||Ok(!mounts[0].host().borrow().has_pending_browser_frame())).await?;
    let retired_work=work(&mounts[0])?;let retired_calls=control.source_calls.get();
    raf_turns(12).await?;
    if work(&mounts[0])?!=retired_work||control.source_calls.get()!=retired_calls
        || mounts[0].host().borrow().s.presentation_sequence!=pending_before["sequence"].as_u64().ok_or("hidden sequence")? {
        return Err("hide-while-pending did not retire once without hidden work/presentation".into())}
    let core=mounts[0].host().borrow().s.shared.render_core.clone().ok_or("hidden core")?;
    let mut completion=begin(&core).await?;completion.submit(|_|Ok(())).map_err(|e|format!("hidden retirement completion: {e:?}"))?;
    let accepted=finish(completion).await?.map_err(|e|format!("hidden retired scope/FIFO: {e:?}"))?;drop(accepted);zero(&core)?;
    mounts[0].host().borrow_mut().set_hidden(false);let returned=capture(&mounts[0]).await?;
    Ok(json!({"hidden_raf_callbacks":12,"before":work_json(before),"after":work_json(after),"source_calls_during_hidden":calls,
        "unhide_pixels_sha256":digest(&visible.frame),"idle":idle,"hide_while_pending":{"before":pending_before,
        "stable_hidden_work":work_json(retired_work),"source_calls":retired_calls,"raf_callbacks":12,
        "returned_pixels":compare_pixels(&visible.frame,&returned.frame)?,"retired_quality_count":0,"retired_quality_bytes":0}}))
}
async fn stale(mounts:&[BrowserMount])->Result<Value,String>{
    let before=submit_unarmed(mounts,0).await?;let armed=arm(&mounts[0]);
    {
        let mut host=mounts[0].host().borrow_mut();focus_attribute(&mut host,"aria-label","Primary hue")?;
        let press=KeyPress::named(NamedKey::ArrowRight);host.key(&press);host.key_released(&press.key);
    }
    size(mounts[0].window().canvas(),960,780)?;mounts[0].window().request_redraw();
    let (capture,envelope)=accepted_capture(&armed).await?;
    if capture.seal["authority"]["theme"]==before["theme"]{return Err("actual stale control did not edit author-owned theme".into())}
    if Some(u64::from(capture.frame.width))==before["viewport"][0].as_u64()
        && Some(u64::from(capture.frame.height))==before["viewport"][1].as_u64(){return Err("stale target used pre-resize extent".into())}
    let after=identities(&mounts[0])?;
    if after["sequence"].as_u64()!=before["sequence"].as_u64().and_then(|value|value.checked_add(1)){
        return Err("stale pending frame was presented before fresh source/resize".into())}
    Ok(json!({"before":before,"after":after,"envelope":envelope,"pixels_sha256":digest(&capture.frame)}))
}

async fn redraw_ownership(mounts:&[BrowserMount],control:&Rc<Control>)->Result<Value,String>{
    control.request_in_frame.set(true);
    let before=submit_unarmed(mounts,0).await?;let prepared=work(&mounts[0])?;
    mounts[0].window().request_redraw();
    JsFuture::from(js_sys::Promise::resolve(&JsValue::NULL)).await.map_err(|e|format!("request microtask: {e:?}"))?;
    if !mounts[0].host().borrow().has_submitted_browser_frame(){return Err("second request was not while actual frame pending".into())}
    mounts[0].window().request_redraw();idle(mounts).await?;
    let after=identities(&mounts[0])?;
    if after["sequence"].as_u64()!=before["sequence"].as_u64().and_then(|v|v.checked_add(2))
        || work(&mounts[0])?.preparation!=prepared.preparation+1 {
        return Err("late requests were lost or did not coalesce to exactly one fresh preparation".into())
    }
    let quiet=stable_idle(mounts,6).await?;control.request_in_frame.set(false);
    let before_callbacks=mounts[0].host().borrow().s.presentation_sequence;
    control.request_after_frame.set(true);let after_frame_capture=capture(&mounts[0]).await?;idle(mounts).await?;
    if mounts[0].host().borrow().s.presentation_sequence!=before_callbacks+2{
        return Err("AfterFrame redraw was acknowledged away or replayed".into())}
    let callback_count=Rc::new(Cell::new(0));let errors=Rc::new(RefCell::new(Vec::new()));
    let count=callback_count.clone();let failure=errors.clone();let window=mounts[0].window().clone();
    mounts[0].host().borrow_mut().s.pending_capture=Some(Box::new(move |result|match result{
        Ok(_)=>{count.set(count.get()+1);window.request_redraw();},Err(error)=>failure.borrow_mut().push(error),
    }));
    let before_callback=mounts[0].host().borrow().s.presentation_sequence;
    let callback_capture=capture(&mounts[0]).await?;idle(mounts).await?;
    if callback_count.get()!=1||!errors.borrow().is_empty()||mounts[0].host().borrow().s.presentation_sequence!=before_callback+2{
        return Err("capture callback's real follow-up request was lost/replayed".into())}
    let final_idle=stable_idle(mounts,6).await?;
    Ok(json!({"before":before,"after":after,"prepared_before_late_requests":work_json(prepared),"two_late_requests_one_followup":true,
        "preparation_hook_request_idle":quiet,"after_frame_pixels_sha256":digest(&after_frame_capture.frame),
        "capture_callback_count":callback_count.get(),"capture_callback_pixels_sha256":digest(&callback_capture.frame),"final_idle":final_idle}))
}
async fn loss_pending(mounts:&[BrowserMount])->Result<Value,String>{
    idle(mounts).await?;
    let before_a=capture(&mounts[0]).await?;let before_b=capture(&mounts[1]).await?;
    let planned_bytes=admit_copies(mounts,&[&before_a.frame,&before_b.frame],true)?;
    let source_a=sources(&mounts[0],0).await?;let source_b=sources(&mounts[1],1).await?;
    let before=submit_unarmed(mounts,0).await?;let armed=arm(&mounts[0]);
    mounts[1].host().borrow_mut().set_hidden(true);
    let core=mounts[0].host().borrow().s.shared.render_core.clone().ok_or("loss core")?;
    let generation=core.generation();let weak=Arc::downgrade(&core);let owner=core.shared_recovery();
    core.device().destroy();drop(core);
    wait(|| {
        if let RecoveryStatus::Failed{error,..}=owner.status(){return Err(format!("actual async replacement failed: {error}"))}
        Ok(mounts.iter().all(|m| {
            let h=m.host().borrow();h.s.shared.render_core.as_ref().is_some_and(|c|c.generation()!=generation)
                &&h.s.surface.is_some()&&!h.s.needs_recovery_surface
        }))
    }).await?;
    if armed.paints.borrow().len()!=1||armed.copied.borrow().len()!=1||!armed.observed.borrow().is_empty(){
        return Err("pending loss published or did not refuse both one-shots once".into())}
    let paint=armed.paints.borrow_mut().pop().unwrap().err().ok_or("loss paint unexpectedly accepted")?;
    let copied=armed.copied.borrow_mut().pop().unwrap().err().ok_or("loss target unexpectedly accepted")?;
    let notice=match (&paint,&copied){
        (CaptureRefusal::DeviceLost(a),CaptureRefusal::DeviceLost(b)) if a==b&&a.generation==generation&&a.reason==wgpu::DeviceLostReason::Destroyed=>a.clone(),
        _=>return Err(format!("loss did not override pending scope/counter outcomes precisely: {paint:?}; {copied:?}")),
    };
    if mounts[0].host().borrow().s.presentation_sequence!=before["sequence"].as_u64().ok_or("sequence")?{
        return Err("old pending loss target was presented".into())}
    if weak.upgrade().is_some(){return Err("pending scope/counter frame retained old Core after replacement".into())}
    let replacement=owner.current().map_err(|status|format!("replacement: {status:?}"))?;
    if !mounts[1].host().borrow().s.hidden{return Err("recovery changed hidden host authority".into())}
    for mount in mounts {
        if !Arc::ptr_eq(mount.host().borrow().s.shared.render_core.as_ref().ok_or("adopted core")?,&replacement){
            return Err("hosts did not adopt the same actual replacement".into())}
    }
    mounts[1].host().borrow_mut().set_hidden(false);
    let after_a=capture(&mounts[0]).await?;let after_b=capture(&mounts[1]).await?;
    let source_a_after=sources(&mounts[0],0).await?;let source_b_after=sources(&mounts[1],1).await?;
    let producer_pixels:Result<Vec<_>,String>=(0..2).map(|slot|Ok(json!({"slot":slot,
        "a":compare_source(&source_a[slot],&source_a_after[slot])?,"b":compare_source(&source_b[slot],&source_b_after[slot])?}))).collect();
    idle(mounts).await?;zero(&replacement)?;
    // Keep the exact already copied frames for independent driver byte checks.
    // Only CPU Vec ownership moves here; no additional raster/readback occurs.
    let root_a = compare(&before_a,&after_a)?;
    let root_b = compare(&before_b,&after_b)?;
    let producer_pixels = producer_pixels?;
    let mut artifacts = vec![
        ("first_before".to_owned(),before_a.frame),
        ("first_after".to_owned(),after_a.frame),
        ("second_before".to_owned(),before_b.frame),
        ("second_after".to_owned(),after_b.frame),
    ];
    for (prefix,phase,copies) in [
        ("first","before",source_a),("first","after",source_a_after),
        ("second","before",source_b),("second","after",source_b_after),
    ] {
        for (slot,capture) in copies.into_iter().enumerate() {
            artifacts.push((format!("{prefix}_{}_{phase}",if slot==0 {"reader"} else {"stylesheet"}),capture.frame));
        }
    }
    let mut bytes = 0usize;
    let mut names = std::collections::BTreeSet::new();
    let mut ledger = Vec::new();
    for (name,frame) in &artifacts {
        let expected = frame_bytes(frame.width,frame.height)?;
        if frame.rgba.len()!=expected || !names.insert(name.clone()) {
            return Err("QC retained raw artifact shape/name differs from actual frame".into());
        }
        bytes = bytes.checked_add(expected).ok_or("QC retained raw artifact byte overflow")?;
        if bytes>RETAINED_LIMIT {return Err("QC retained artifacts exceed unchanged 128 MiB admission".into())}
        ledger.push(json!({"name":name,"width":frame.width,"height":frame.height,"bytes":expected,"sha256":digest(frame)}));
    }
    if artifacts.len()!=12 || bytes!=planned_bytes {
        return Err("QC actual 12 artifact bytes differ from admitted complete-source plan".into());
    }
    let artifact_names:Vec<_> = artifacts.iter().map(|(name,_)|name.clone()).collect();
    PIXELS.with(|pixels| -> Result<(),String> {
        let mut pixels = pixels.borrow_mut();
        if !pixels.is_empty(){return Err("QC one-shot artifact map was already occupied".into())}
        for (name,frame) in artifacts {pixels.insert(name,frame);}
        Ok(())
    })?;
    Ok(json!({"loss_generation":generation,"replacement_generation":replacement.generation(),"notice":format!("{notice:?}"),
        "pending_callback_refusals":2,"planned_copy_bytes":planned_bytes,"old_core_released":true,"shared_replacement_arc":true,"factory_count_inferred":false,
        "root_a":root_a,"root_b":root_b,"complete_producer_pixels":producer_pixels,
        "artifact_names":artifact_names,"raw_artifacts":ledger,"retained_artifact_bytes":bytes,
        "artifact_scope":"exact existing pending-loss root/Reader/Stylesheet copies moved into PIXELS; no extra GPU, crop, clone or reconstruction"}))
}
async fn dispose_pending(mounts:&[BrowserMount],control:&Rc<Control>)->Result<Value,String>{
    let before=submit_unarmed(mounts,0).await?;let armed=arm(&mounts[0]);
    let core=mounts[1].host().borrow().s.shared.render_core.clone().ok_or("healthy sibling core")?;
    let generation=core.generation();let lifetime=mounts[0].lifetime().clone();
    if !lifetime.dispose(){return Err("actual active mount did not begin disposal".into())}
    wait(||Ok(lifetime.status().phase==MountPhase::Disposed)).await?;clean(lifetime.status())?;
    if control.surface_control_drops.get()!=1 || !control.surface_control_drop_borrow_safe.get() {
        return Err("retained disposed host did not release diagnostic callback once outside borrow".into())
    }
    if armed.paints.borrow().len()!=1||armed.copied.borrow().len()!=1||!armed.observed.borrow().is_empty(){
        return Err("pending disposal did not refuse captures once without presentation".into())}
    if !matches!(armed.paints.borrow_mut().pop().unwrap(),Err(CaptureRefusal::HostDisposed))
        || !matches!(armed.copied.borrow_mut().pop().unwrap(),Err(CaptureRefusal::HostDisposed)) {
        return Err("pending disposal lost typed HostDisposed".into())}
    if mounts[0].host().borrow().s.presentation_sequence!=before["sequence"].as_u64().ok_or("sequence")?{
        return Err("disposed pending target reached presentation".into())}
    let after=work(&mounts[0])?;raf_turns(6).await?;
    if work(&mounts[0])?!=after{return Err("disposed listener/RAF performed more host work".into())}
    let healthy=capture(&mounts[1]).await?;
    let current=mounts[1].host().borrow().s.shared.render_core.clone().ok_or("sibling current core")?;
    if generation!=current.generation()||!Arc::ptr_eq(&core,&current)||current.device_failure().is_some(){
        return Err("disposing A replaced/lost healthy B Core".into())}
    let mut scene=Scene::new(32,24);scene.push_rect(0.,0.,32.,24.,[0.25,0.5,0.75,1.]);
    let (texture,_)=owned_raster(&current,0x7163_6469_7370,&scene,32,24,ColorLoad::Clear(wgpu::Color::TRANSPARENT),1.).await?;
    let pixels=collect(current.start_rgba8_readback(&texture,32,24)?,now()?).await?;zero(&current)?;
    Ok(json!({"before":before,"lifetime":mount_status(lifetime.status()),"host_disposed_callbacks":2,"presentation_callbacks":0,
        "disposed_work":work_json(after),"healthy_same_core_generation":generation,"healthy_root_sha256":digest(&healthy.frame),
        "healthy_owned_frame_sha256":digest(&pixels),"retired_quality_count":0,"retired_quality_bytes":0}))
}

async fn rebind(mounts:&[BrowserMount],control:&Rc<Control>)->Result<Value,String>{
    let before=submit_unarmed(mounts,0).await?;let armed=arm(&mounts[0]);
    let original=mounts[0].host().borrow().s.runner.as_ref().ok_or("rebind runner")?.state().reader_preview();
    let new=Rc::new(RefCell::new(ControlledReader {original:original.clone(),control:control.clone(),
        seen:Cell::new((0,0,0)),revision:Cell::new(0)}));
    let producer=control.producer.borrow().upgrade().ok_or("actual registered shared producer handle")?;
    if !producer.borrow_mut().inner.set_source(new.clone()){return Err("actual SceneProducer did not rebind to new source Rc".into())}
    mounts[0].window().request_redraw();
    let (captured,envelope)=accepted_capture(&armed).await?;
    let after=identities(&mounts[0])?;
    if after["reader_rc"]!=before["reader_rc"]||after["theme"]!=before["theme"]
        || after["sequence"].as_u64()!=before["sequence"].as_u64().and_then(|v|v.checked_add(1)) {
        return Err("source binding rebind lost app authority or presented old pending frame".into())}
    if !Rc::ptr_eq(&new.borrow().original,&original){return Err("fixture rebind substituted an authored Reader source".into())}
    if after["images"][0]["stamp"].as_u64()<=before["images"][0]["stamp"].as_u64(){return Err("rebound accepted source failed to advance stamp".into())}
    idle(mounts).await?;
    Ok(json!({"before":before,"after":after,"envelope":envelope,"source_binding":"actual SceneProducer.set_source(new Rc); original Workshop Reader Rc retained",
        "new_adapter_source_rc":format!("{:p}",Rc::as_ptr(&new)),"pixels_sha256":digest(&captured.frame)}))
}

pub(super) async fn execute(first_id:&str,second_id:&str,run:&str)->Result<Value,String>{
    let started=now()?;let first=canvas(first_id)?;let second=canvas(second_id)?;
    if first==second{return Err("two independent actual canvases required".into())}
    size(&first,1280,1100)?;size(&second,1280,1100)?;
    let options=||HostOptions { title:"Current-frame quality control".into(),..HostOptions::default() };
    let core=Arc::new(RenderCore::boot_async((options().netrender)()).await?);
    let adapter=adapter_identity(&core);let initial_generation=core.generation();let control=Rc::new(Control::new());
    let mut mounts=Vec::new();
    for (index,(canvas,name,mode)) in [(first,"QC-first-draft",Mode::Light),(second,"QC-second-draft",Mode::Dark)].into_iter().enumerate(){
        let state=state(name,mode)?;
        let hook=if index==0{controlled_hooks(control.clone())}else{hooks()};
        let mount=mount_with_render_core(canvas,options(),core.clone(),move |_,_,_|Init {
            state,logic:Box::new(workshop_view) as Logic,sheet:workshop_stylesheet(),
            fonts:generic_fonts().into_iter().map(|(family,bytes)|HostFont {family:Some(family),bytes}).collect(),images:Vec::new(),
        },hook).await?;
        mount.host().borrow_mut().enable_browser_work_observation();
        LIFETIMES.with(|l|l.borrow_mut().push(mount.lifetime().clone()));mounts.push(mount);
    }
    idle(&mounts).await?;
    install_surface_controls(&mounts[0],control.clone())?;
    phase(run,"qc_full_production_baseline");
    let baseline_a=capture(&mounts[0]).await?;let baseline_b=capture(&mounts[1]).await?;
    let planned_bytes=admit_copies(&mounts,&[&baseline_a.frame,&baseline_b.frame],false)?;
    let full_a=sources(&mounts[0],0).await?;let full_b=sources(&mounts[1],1).await?;
    if (0..2).any(|slot|full_a[slot].seal["renderer_key"]==full_b[slot].seal["renderer_key"]){
        return Err("independent full producers alias physical image keys".into())}
    let baseline=json!({"a":baseline_a.seal,"b":baseline_b.seal,"planned_copy_bytes":planned_bytes,
        "root_sha256":[digest(&baseline_a.frame),digest(&baseline_b.frame)],
        "complete_source_sha256":[full_a.iter().map(|c|digest(&c.frame)).collect::<Vec<_>>(),full_b.iter().map(|c|digest(&c.frame)).collect::<Vec<_>>() ]});
    drop(baseline_a);drop(baseline_b);drop(full_a);drop(full_b);
    idle(&mounts).await?;zero(&core)?;
    phase(run,"qc_actual_null_and_ignored_refusal");let scopes=clean_scope_and_ignored_refusal(&core).await?;
    phase(run,"qc_late_paint_capture");let late=late_capture(&mounts).await?;
    phase(run,"qc_late_redraw_no_animation");let redraw=redraw_ownership(&mounts,&control).await?;
    let mut negatives=Vec::new();
    for fault in [Fault::Validation,Fault::RejectedScope,Fault::RoundCap,Fault::SurfaceValidation,Fault::SurfaceRejectedScope]{
        phase(run,&format!("qc_actual_{fault:?}"));negatives.push(negative(&mounts,&control,fault).await?);
    }
    phase(run,"qc_hidden_layout_a11y_idle");let hidden=hidden_work(&mounts,&control).await?;
    phase(run,"qc_stale_input_resize");let stale=stale(&mounts).await?;
    phase(run,"qc_actual_source_rebind");let rebind=rebind(&mounts,&control).await?;
    // No extra strong old Core may mask the pending-loss retirement proof.
    drop(core);
    phase(run,"qc_actual_pending_loss_shared_replacement");let loss=loss_pending(&mounts).await?;
    let artifact_names=loss["artifact_names"].clone();
    phase(run,"qc_pending_disposal_healthy_sibling");let disposal=dispose_pending(&mounts,&control).await?;
    let final_core=mounts[1].host().borrow().s.shared.render_core.clone().ok_or("final owned core")?;
    idle(&mounts).await?;zero(&final_core)?;
    let cleanup=dispose_lifetimes().await?;
    // This fixture owns initial acquisition and the sole replacement. Only
    // after both actual mount lifetimes are disposed may it close that device.
    let final_generation=final_core.generation();final_core.device().destroy();
    wait(||match final_core.loss_notice(){
        Some(notice) if notice.generation==final_generation&&notice.reason==wgpu::DeviceLostReason::Destroyed=>Ok(true),
        Some(notice)=>Err(format!("final owned device closure mismatch: {notice:?}")),None=>Ok(false),
    }).await?;drop(final_core);
    Ok(json!({"status":"pass","run":run,"elapsed_ms":now()?-started,"initial_generation":initial_generation,"initial_adapter":adapter,
        "artifact_names":artifact_names,"baseline":baseline,"actual_scopes_and_ignored_refusal":scopes,"late_capture":late,"redraw_ownership":redraw,"actual_negatives":negatives,
        "hidden_work":hidden,"stale_source_resize":stale,"actual_source_rebind":rebind,"pending_loss":loss,"pending_disposal":disposal,"cleanup":cleanup,
        "final_owned_device_closed_after_mount_cleanup":final_generation,"per_operation_deadline_ms":5000,"renderer_thresholds":{"edge_rgba":32,"edge_alpha":16,"interior_rgba":3,"interior_alpha":2,"mean":1},
        "recovery_pixel_thresholds":{"rgb":3,"alpha":2,"mean_absolute":1},
        "current_frame_async_quality_qualified":false,"current_frame_async_error_scopes_qualified":false,
        "scope":"executed assertions of actual owned prepared producer/staging/shadow/offscreen-root acceptance; final canvas clear/blit scope excluded; qualification is reviewed separately"}))
}
