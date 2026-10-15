// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Output-only actual acquired-surface fixture proposal; uncompiled/unapplied.
use super::*;
use cambium_rootstock::{BrowserAcquiredCaptureFence,BrowserAcquiredCaptureFrame,BrowserSurfaceCaptureFn};
use genet_render_host::{SurfaceCaptureBudget,SurfaceCaptureLayout,SurfaceCaptureReservation,SurfaceReadbackFailure};
use cambium_genet_web_host::mount_with_core_future_and_readback_diagnostics;
mod artifact_store;

struct Reference { capture:Capture, credit:SurfaceCaptureReservation }
struct Armed {
    callback_count:Rc<Cell<usize>>, outside_borrow:Rc<Cell<bool>>,
    sequence_at_delivery:Rc<Cell<Option<u64>>>,
    result:Rc<RefCell<Option<Result<BrowserAcquiredCaptureFence,SurfaceReadbackFailure>>>>,
}
fn arm(mount:&BrowserMount,budget:&Arc<SurfaceCaptureBudget>)->Result<Armed,String>{
    let callback_count=Rc::new(Cell::new(0));let outside_borrow=Rc::new(Cell::new(true));
    let result=Rc::new(RefCell::new(None));let out=result.clone();let count=callback_count.clone();let outside=outside_borrow.clone();
    let host=Rc::downgrade(mount.host());let sequence_at_delivery=Rc::new(Cell::new(None));let delivered_sequence=sequence_at_delivery.clone();
    let mut callback:Option<BrowserSurfaceCaptureFn>=Some(Box::new(move |value|{
        count.set(count.get()+1);
        outside.set(outside.get() && host.upgrade().is_some_and(|h|h.try_borrow_mut().is_ok()));
        delivered_sequence.set(host.upgrade().and_then(|h|h.try_borrow().ok().map(|h|h.s.presentation_sequence)));
        *out.borrow_mut()=Some(value);
    }));
    Application::request_browser_surface_capture(mount.host(),budget.clone(),&mut callback)
        .map_err(|e|format!("actual surface request refused: {e:?}"))?;
    if callback.is_some(){return Err("successful arm did not take callback owner".into())}
    Ok(Armed{callback_count,outside_borrow,sequence_at_delivery,result})
}
async fn accepted(armed:Armed)->Result<BrowserAcquiredCaptureFrame,String>{
    let started=now()?;
    wait(||{if now()?-started>=5000.{return Err("capture delivery original operation exceeded five seconds".into())}Ok(armed.result.borrow().is_some())}).await?;
    let mut fence=armed.result.borrow_mut().take().ok_or("capture callback absent")?
        .map_err(|e|format!("actual surface callback refusal: {e:?}"))?;
    let mut result=None;
    wait(||{
        if now()?-started>=5000.{return Err("capture map exceeded operation admission".into())}
        // No Host/source/STORE borrow survives this actual map poll.
        if let Some(value)=fence.poll(){result=Some(value);Ok(true)}else{Ok(false)}
    }).await?;
    let frame=result.ok_or("mapped actual surface absent")?.map_err(|e|format!("actual acquired map refusal: {e:?}"))?;
    frame.check_current().map_err(|e|format!("actual acquired delivery stale: {e:?}"))?;
    if armed.callback_count.get()!=1 || !armed.outside_borrow.get(){return Err("capture callback replayed or ran under Host borrow".into())}
    if !matches!(fence.poll(),Some(Err(SurfaceReadbackFailure::AlreadyCollected))){return Err("accepted fence replayed pixels".into())}
    let m=frame.frame().metadata();
    if m.quality_phase!=2 || m.observed_width!=frame.presentation().width || m.observed_height!=frame.presentation().height
        || frame.frame().rgba().len()!=frame_bytes(m.observed_width,m.observed_height)? {return Err("actual acquired extent/phase mismatch".into())}
    Ok(frame)
}
fn phase_observation(o:&genet_render_host::BrowserFrameQualityObservation)->Result<Value,String>{
    let rows=o.rows.as_ref().ok_or("actual scope rows absent")?;
    if o.actual_row_count!=4 || rows.len()!=4 || o.wrapper_rejected || !o.zero_classic_fifo_observed {return Err("actual phase scope/queue/FIFO not accepted".into())}
    for (row,filter) in rows.iter().zip(["validation","internal","out-of-memory","queue"]){
        if row.filter.as_deref()!=Some(filter)||row.status.as_deref()!=Some("fulfilled")||row.error!=Some(false){return Err("actual phase ledger contains rejection/error".into())}
    }
    Ok(json!({"frame":o.frame.diagnostic_label(),"generation":o.generation,"phase":o.phase,
        "actual_row_count":o.actual_row_count,"wrapper_rejected":o.wrapper_rejected,"zero_classic_fifo_observed":o.zero_classic_fifo_observed,
        "rows":rows.iter().map(|r|json!({"filter":r.filter,"status":r.status,"error":r.error,"message":r.message})).collect::<Vec<_>>() }))
}
fn acquired_receipt(f:&BrowserAcquiredCaptureFrame)->Result<Value,String>{
    let d=f.diagnostics();let m=f.frame().metadata();let a=d.phase1.as_ref().ok_or("phase1 diagnostics absent")?;let b=d.phase2.as_ref().ok_or("phase2 diagnostics absent")?;
    if a.phase!=1 || b.phase!=2 || a.frame!=m.frame || b.frame!=m.frame || d.frame!=m.frame
        || a.generation!=m.generation || b.generation!=m.generation || d.generation!=m.generation {return Err("phase/Core/frame diagnostics differ".into())}
    let c=&m.configuration;
    Ok(json!({"kind":"actual-acquired-final-surface","presentation":presentation(f.presentation()),"frame":m.frame.diagnostic_label(),
        "generation":m.generation,"quality_phase":m.quality_phase,"host_epoch":d.epoch,"owned_surface_stamp":format!("{:?}",d.surface),
        "configuration":{"advertised_usage":c.advertised_usage(),"requested_usage":c.requested_usage(),"format":format!("{:?}",c.format()),
        "base_view_format":format!("{:?}",c.view_format()),"color_space":format!("{:?}",c.color_space()),"alpha_mode":format!("{:?}",c.alpha_mode()),"extent":c.extent()},
        "observed":{"usage":m.observed_usage,"format":m.observed_format,"width":m.observed_width,"height":m.observed_height},
        "layout":{"row_bytes":m.layout.row_bytes,"padded_row_bytes":m.layout.padded_row_bytes,"padded_bytes":m.layout.padded_bytes,
        "raw_bytes":m.layout.rgba_bytes,"reservation_bytes":m.layout.reservation_bytes},"normalization":m.normalization,
        "sources":d.sources.iter().map(|s|image_json(s.local_key,s.image)).collect::<Vec<_>>(),"phase1":phase_observation(a)?,"phase2":phase_observation(b)?,
        "sha256":digest_bytes(f.frame().rgba()),"callback_once_outside_host_borrow":true,"repeat_poll":"AlreadyCollected"}))
}
fn observed_phase(o:&genet_render_host::BrowserFrameQualityObservation)->Value {
    json!({"frame":o.frame.diagnostic_label(),"generation":o.generation,"phase":o.phase,"actual_row_count":o.actual_row_count,
        "wrapper_rejected":o.wrapper_rejected,"zero_classic_fifo_observed":o.zero_classic_fifo_observed,
        "rows":o.rows.as_ref().map(|rows|rows.iter().map(|r|json!({"filter":r.filter,"status":r.status,"error":r.error,"message":r.message})).collect::<Vec<_>>())})
}
fn credits(budget:&Arc<SurfaceCaptureBudget>,w:u32,h:u32)->Result<(SurfaceCaptureReservation,SurfaceCaptureReservation),String>{
    let layout=SurfaceCaptureLayout::checked(w,h,u32::MAX,BYTES_LIMIT as u64).map_err(|e|format!("reference aligned layout refused: {e:?}"))?;
    let raw=budget.reserve(layout.rgba_bytes as u64).map_err(|e|format!("reference raw credit refused: {e:?}"))?;
    let padded=budget.reserve(layout.padded_bytes).map_err(|e|format!("reference buffer credit refused: {e:?}"))?;
    Ok((raw,padded))
}
/// Retained root reference: the old by-ref callback only clones the immutable
/// prepared view. Actual composition, mapping and teardown run outside Host.
async fn reference_root(mount:&BrowserMount,budget:&Arc<SurfaceCaptureBudget>)->Result<Reference,String>{
    let observed=Rc::new(RefCell::new(None));let copied=Rc::new(RefCell::new(None));
    let weak={let h=mount.host().borrow();Arc::downgrade(h.s.shared.render_core.as_ref().ok_or("reference Core absent")?)};
    {
        let mut host=mount.host().borrow_mut();let out=observed.clone();
        host.s.pending_presentation_observer=Some(Box::new(move|ctx,p|*out.borrow_mut()=Some(seal(ctx,p))));
        let out=copied.clone();
        host.s.pending_stamped_capture=Some(Box::new(move|result|{
            *out.borrow_mut()=Some(result.map_err(|e|e.to_string()).map(|c|(c.presentation,c.view.clone())));
        }));
    }
    mount.window().request_redraw();wait(||Ok(copied.borrow().is_some())).await?;
    let (p,view)=copied.borrow_mut().take().ok_or("reference callback absent")??;
    let authority=observed.borrow_mut().take().ok_or("reference seal absent")??;
    if authority["presentation"]!=presentation(p){return Err("reference presentation mismatch".into())}
    let (credit,padded)=credits(budget,p.width,p.height)?;
    let generation={let core=weak.upgrade().ok_or("reference Core retired before target admission")?;core.generation()};
    let (target_credit,_target_operation)=artifact_store::hold_target(budget,weak.clone(),generation,None,frame_bytes(p.width,p.height)? as u64)?;
    let started=now()?;let mut submission=None;
    wait(||{
        if now()?-started>=5000.{return Err("reference owner wait exceeded original admission deadline".into())}
        let core=weak.upgrade().ok_or("reference Core retired")?;
        match core.try_begin_browser_frame(){Ok(s)=>{submission=Some(s);Ok(true)},Err(genet_render_host::BrowserFrameStartError::Busy)=>Ok(false),Err(e)=>Err(format!("reference owner refused: {e:?}"))}}).await?;
    let mut submission=submission.ok_or("reference owner absent")?;
    if now()?-started>=5000.{return Err("reference submission exceeded original admission deadline".into())}
    let target=submission.submit(|commands|{
        let target=commands.device().create_texture(&wgpu::TextureDescriptor{label:Some("retained root reference, not acquired canvas"),
            size:wgpu::Extent3d{width:p.width,height:p.height,depth_or_array_layers:1},mip_level_count:1,sample_count:1,
            dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8Unorm,
            usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC,view_formats:&[]});
        let target_view=target.create_view(&Default::default());
        commands.compose_external_texture(&view,&target_view,wgpu::TextureFormat::Rgba8Unorm,p.width,p.height,
            netrender::ExternalTexturePlacement::new([0.,0.,p.width as f32,p.height as f32]))?;
        Ok(target)
    }).map_err(|e|format!("reference composition refused: {e:?}"))?;
    drop(view);let mut pending=submission.seal().map_err(|e|format!("reference seal refused: {e:?}"))?;let mut accepted=None;
    wait(||{if now()?-started>=5000.{return Err("reference original admission deadline".into())}
        if let Some(value)=pending.poll(){accepted=Some(value);Ok(true)}else{Ok(false)}}).await?;
    let accepted=accepted.ok_or("reference quality absent")?.map_err(|e|format!("reference quality refused: {e:?}"))?;
    accepted.check_current().map_err(|e|format!("reference quality stale: {e:?}"))?;
    let readback={let core=weak.upgrade().ok_or("reference Core retired")?;
        if now()?-started>=5000.{return Err("reference map start exceeded original admission deadline".into())}
        core.start_rgba8_readback(&target,p.width,p.height)?};
    drop(accepted);drop(target);
    let frame=collect(readback,started).await?;drop(padded);
    // The submitted target/view are gone and accepted map completed. Keep its
    // conservative GPU credit until actual Core closure, even on later errors.
    artifact_store::target_mapped(target_credit,false)?;
    {let core=weak.upgrade().ok_or("reference Core retired at final success edge")?;
        if core.generation()!=generation || core.loss_notice().is_some(){return Err("reference Core lost at final success edge".into())}
        let host=mount.host().try_borrow().map_err(|_|"reference Host borrowed at final success edge")?;
        if host.s.hidden || !host.s.shared.render_core.as_ref().is_some_and(|c|Arc::ptr_eq(c,&core))
            || host.s.last_redraw_presentation.map(presentation)!=Some(presentation(p)) || host.s.presentation_sequence!=p.sequence {
            return Err("reference Core/presentation changed at final success edge".into())
        }
        let runner=host.s.runner.as_ref().ok_or("reference runner absent at final success edge")?;let state=runner.state();
        let authority_now=json!({"theme":state.draft_theme(),"mode":state.mode_key(),"dirty":state.is_dirty(),"status":state.status(),
            "focus":format!("{:?}",runner.focus()),"zoom":host.ui_zoom(),"graph_selected":state.selected_graph_node(),
            "name_text":state.text_field("name").map(|f|f.text()),"name_selection":format!("{:?}",state.text_field("name").map(|f|f.caret_selection()))});
        let images=KEYS.into_iter().map(|key|{
            if host.s.shared.producers.error(key).is_some(){return Err("reference producer refused at final success edge")}
            host.s.shared.producers.image_identity(key).map(|i|image_json(key,i)).ok_or("reference source absent at final success edge")
        }).collect::<Result<Vec<_>,_>>()?;
        if authority["authority"]!=authority_now || authority["images"]!=json!(images)
            || authority["runner"]!=json!(format!("{:p}",runner))
            || authority["reader_source"]!=json!(format!("{:p}",Rc::as_ptr(&state.reader_preview())))
            || authority["stylesheet_source"]!=json!(format!("{:p}",Rc::as_ptr(&state.stylesheet_preview()))) {
            return Err("reference retained source snapshot changed at final success edge".into())
        }
    }
    if now()?-started>=5000.{return Err("reference final success exceeded original admission deadline".into())}
    Ok(Reference{capture:Capture{frame,seal:authority},credit})
}
/// Actual registered producer pixels, not a reconstructed source document.
/// Only weak Core survives each asynchronous quality/map turn.
async fn reference_source(mount:&BrowserMount,key:u64,index:u64,budget:&Arc<SurfaceCaptureBudget>)->Result<Reference,String>{
    let (image,weak)={let h=mount.host().borrow();
        let image=h.s.shared.producers.image_identity(key).ok_or("registered source absent")?;
        if h.s.shared.producers.error(key).is_some(){return Err("registered source refused".into())}
        let core=h.s.shared.render_core.as_ref().ok_or("source Core absent")?;
        if image.generation!=core.generation(){return Err("registered source stale".into())}
        (image,Arc::downgrade(core))};
    let [w,h]=image.size;let (credit,padded)=credits(budget,w,h)?;
    // Four fixture-private keys never name an application/provider resource.
    if index>=4{return Err("private diagnostic source index outside reserved fixture range".into())}
    let private_tile_key=0x6163_7164_7372_6300+index;
    let (target_credit,_target_operation)=artifact_store::hold_target(budget,weak.clone(),image.generation,Some(private_tile_key),frame_bytes(w,h)? as u64)?;
    let source_started=now()?;let mut scene=Scene::new(w,h);
    scene.push_image_full(0.,0.,w as f32,h as f32,[0.,0.,1.,1.],[1.;4],image.renderer_key,0,NO_CLIP);
    let mut submission=None;wait(||{
        if now()?-source_started>=5000.{return Err("source owner wait exceeded original admission deadline".into())}
        let core=weak.upgrade().ok_or("source Core retired")?;
        match core.try_begin_browser_frame(){Ok(s)=>{submission=Some(s);Ok(true)},Err(genet_render_host::BrowserFrameStartError::Busy)=>Ok(false),Err(e)=>Err(format!("source owner refused: {e:?}"))}
    }).await?;
    let mut submission=submission.ok_or("source owner absent")?;
    if now()?-source_started>=5000.{return Err("source submission exceeded original admission deadline".into())}
    let target=submission.submit(|c|c.rasterize_scaled_for(private_tile_key,&scene,w,h,ColorLoad::Clear(wgpu::Color::TRANSPARENT),1.))
        .map_err(|e|format!("actual source raster refused: {e:?}"))?;
    let mut pending=submission.seal().map_err(|e|format!("source seal refused: {e:?}"))?;
    let mut accepted=None;wait(||{if now()?-source_started>=5000.{return Err("source reference original admission deadline".into())}
        if let Some(value)=pending.poll(){accepted=Some(value);Ok(true)}else{Ok(false)}}).await?;
    let accepted=accepted.ok_or("source acceptance absent")?.map_err(|e|format!("source quality refused: {e:?}"))?;
    accepted.check_current().map_err(|e|format!("source completion stale: {e:?}"))?;
    let readback={let core=weak.upgrade().ok_or("source Core retired after quality")?;
        if now()?-source_started>=5000.{return Err("source map start exceeded original admission deadline".into())}
        core.start_rgba8_readback(&target.0,w,h)?};
    drop(accepted);drop(target);
    let frame=collect(readback,source_started).await?;drop(padded);
    artifact_store::target_mapped(target_credit,false)?;
    // Remove only this fixture's private tile lowering after accepted map,
    // outside Host borrow. The original operation deadline still applies.
    // A sibling frame's owned scopes/FIFO must finish before legacy access.
    let mut tile_removed=false;
    wait(||{
        if now()?-source_started>=5000.{return Err("private source retirement exceeded original reference deadline".into())}
        let core=weak.upgrade().ok_or("source Core retired before private tile retirement")?;
        if core.generation()!=image.generation || core.loss_notice().is_some(){return Err("source Core lost before private tile retirement".into())}
        {let host=mount.host().try_borrow().map_err(|_|"source Host borrowed during retirement currentness check")?;
            if host.s.hidden || !host.s.shared.render_core.as_ref().is_some_and(|c|Arc::ptr_eq(c,&core))
                || host.s.shared.producers.image_identity(key).map(|i|image_json(key,i))!=Some(image_json(key,image)) {
                return Err("source identity changed before private tile retirement".into())
            }
        }
        let access=match core.try_renderer(){Ok(access)=>access,Err(genet_render_host::RasterizeFailure::DeviceLost(e))=>return Err(format!("source retirement lost: {e:?}")),Err(_)=>return Ok(false)};
        tile_removed=access.invalidate_surface_tiles(private_tile_key);
        if !tile_removed{return Err("private diagnostic tile ownership was not found for retirement".into())}
        drop(access);drop(core);
        if now()?-source_started>=5000.{return Err("private source retirement completed after original reference deadline".into())}
        Ok(true)
    }).await?;
    artifact_store::target_mapped(target_credit,tile_removed)?;
    let mut source_seal=image_json(key,image);
    source_seal["diagnostic_target"]=json!({"private_tile_key":private_tile_key,"extra_target_bytes":frame_bytes(w,h)?,
        "accepted_map_completed":true,"private_tile_retired_after_map":tile_removed,"credit_release":"actual Core closure"});
    {let core=weak.upgrade().ok_or("source Core retired at final success edge")?;
        if core.generation()!=image.generation || core.loss_notice().is_some(){return Err("source Core lost at final success edge".into())}
        let host=mount.host().try_borrow().map_err(|_|"source Host borrowed at final success edge")?;
        if host.s.hidden || !host.s.shared.render_core.as_ref().is_some_and(|c|Arc::ptr_eq(c,&core))
            || host.s.shared.producers.error(key).is_some()
            || host.s.shared.producers.image_identity(key).map(|i|image_json(key,i))!=Some(image_json(key,image)) {
            return Err("source identity/Core changed at final success edge".into())
        }
    }
    if now()?-source_started>=5000.{return Err("source final success exceeded original admission deadline".into())}
    Ok(Reference{capture:Capture{frame,seal:source_seal},credit})
}
fn retain_reference(name:String,r:Reference)->Result<Value,String>{
    artifact_store::retain(name,artifact_store::Artifact::Reference{frame:r.capture.frame,seal:r.capture.seal,_credit:r.credit})
}
pub(super) fn export(name:&str)->Result<js_sys::Uint8Array,String>{artifact_store::export(name)}
pub(super) fn compare_admission(a:&str,b:&str)->Result<(),String>{artifact_store::compare_admission(a,b)}
pub(super) fn release_work(){artifact_store::release_work()}
pub(super) fn clear()->u64{artifact_store::clear()}

/// Advance only the real Host state machine. The copied predicate must prove
/// phase2 is genuinely pending; missing this finite window refuses the control.
async fn phase2_pending(mount:&BrowserMount)->Result<cambium_rootstock::BrowserSurfaceCapturePending,String>{
    let mut observed=None;
    wait(||{
        let (pending,retired)={let mut h=mount.host().borrow_mut();
            let _=mount.window().take_pending_frame();
            h.redraw_browser_turn(None);
            (h.browser_surface_capture_pending(),h.take_browser_retirements())};
        drop(retired); // Delivery/resource destructors may reenter Host safely.
        if let Some(p)=pending {
            if p.stage==cambium_rootstock::BrowserSurfaceCaptureStage::SurfaceCompletion && p.request_armed && p.proposed_sequence.is_some(){observed=Some(p);return Ok(true)}
        }
        Ok(false)
    }).await?;
    observed.ok_or("actual phase2 pending predicate absent".into())
}
pub(super) async fn prepublication_source_resize_control(mount:&BrowserMount,budget:&Arc<SurfaceCaptureBudget>)->Result<Value,String>{
    let before=mount.host().borrow().s.presentation_sequence;
    let theme_before=serde_json::to_value(mount.host().borrow().s.runner.as_ref().ok_or("source runner absent")?.state().draft_theme()).map_err(|e|e.to_string())?;
    let armed=arm(mount,budget)?;let original=phase2_pending(mount).await?;
    {let mut h=mount.host().borrow_mut();focus_attribute(&mut h,"aria-label","Primary hue")?;
        let press=KeyPress::named(NamedKey::ArrowRight);h.key(&press);h.key_released(&press.key);}
    // Real owned CSS box changes; WebWindow observes its current physical
    // extent and the normal RAF adapter reconfigures the same canvas.
    size(mount.window().canvas(),960,780)?;mount.window().request_redraw();
    let frame=accepted(armed).await?;let m=frame.frame().metadata();
    let theme_after=serde_json::to_value(mount.host().borrow().s.runner.as_ref().ok_or("source runner absent after edit")?.state().draft_theme()).map_err(|e|e.to_string())?;
    if m.frame==original.frame || frame.presentation().sequence!=before.checked_add(1).ok_or("sequence overflow")?
        || (m.observed_width==original.width&&m.observed_height==original.height)||theme_before==theme_after {
        return Err("stale phase2 request was published/replayed or lost actual source/extent edit".into())
    }
    let mut receipt=json!({"original_frame":original.frame.diagnostic_label(),"original_generation":original.generation,"original_stage":"SurfaceCompletion",
        "original_surface_stamp":format!("{:?}",original.surface),"original_extent":[original.width,original.height],"sequence_before":before,
        "theme_before":theme_before,"theme_after":theme_after,"prepublication_request_preserved":true,"only_fresh_sequence_published":true,
        "visible_rollback_claim":false,"accepted_fresh":acquired_receipt(&frame)?});
    receipt["artifact"]=artifact_store::retain("stale_source_resize_fresh".into(),artifact_store::Artifact::Acquired(frame))?;Ok(receipt)
}
/// Called immediately after the real phase2 predicate in the same JS turn.
/// No synthetic map result or fake loss token is manufactured.
pub(super) async fn pending_loss_control(mounts:&[BrowserMount],budget:&Arc<SurfaceCaptureBudget>)->Result<Value,String>{
    let armed=arm(&mounts[0],budget)?;let pending=phase2_pending(&mounts[0]).await?;
    let before_sequence=mounts[0].host().borrow().s.presentation_sequence;
    let (old,device)={let h=mounts[0].host().borrow();let c=h.s.shared.render_core.as_ref().ok_or("pending loss Core absent")?;
        (Arc::downgrade(c),c.device().clone())};
    device.destroy();drop(device); // Actual loss may notify/reenter Host.
    wait(||Ok(armed.result.borrow().is_some())).await?;
    let error=armed.result.borrow_mut().take().ok_or("loss callback absent")?.err().ok_or("lost phase2 returned fence")?;
    if !matches!(&error,SurfaceReadbackFailure::DeviceLost(n) if n.generation==pending.generation&&n.reason==wgpu::DeviceLostReason::Destroyed){return Err(format!("wrong pending phase2 loss refusal: {error:?}"))}
    if armed.callback_count.get()!=1||!armed.outside_borrow.get(){return Err("pending loss callback replayed/borrowed".into())}
    if armed.sequence_at_delivery.get()!=Some(before_sequence){return Err("lost phase2 published before refusal delivery".into())}
    wait(||Ok(mounts.iter().all(|m|{let h=m.host().borrow();h.s.shared.render_core.as_ref().is_some_and(|c|c.generation()!=pending.generation)&&h.s.surface.is_some()&&!h.s.needs_recovery_surface}))).await?;
    if old.upgrade().is_some(){return Err("phase2 readback kept lost Core alive".into())}
    let next=accepted(arm(&mounts[0],budget)?).await?;
    let mut result=json!({"actual_pending_frame":pending.frame.diagnostic_label(),"stage":"SurfaceCompletion","generation":pending.generation,
        "surface_stamp":format!("{:?}",pending.surface),"proposed_sequence":pending.proposed_sequence,"refusal":format!("{error:?}"),
        "callback_count":1,"outside_host_borrow":true,"unaccepted_publication_unchanged":true,"old_core_released":true,"valid_next":acquired_receipt(&next)?});
    result["artifact"]=artifact_store::retain("pending_loss_fresh".into(),artifact_store::Artifact::Acquired(next))?;Ok(result)
}
/// Post-publication map is deliberately unpolled until real hide/dispose.
/// Accepted immutable pixels already delivered by another request remain valid.
pub(super) async fn postpublication_lifetime_control(mounts:&[BrowserMount],budget:&Arc<SurfaceCaptureBudget>,dispose:bool)->Result<Value,String>{
    let armed=arm(&mounts[0],budget)?;wait(||Ok(armed.result.borrow().is_some())).await?;
    let mut fence=armed.result.borrow_mut().take().ok_or("published fence callback absent")?.map_err(|e|format!("fence refused before lifetime stimulus: {e:?}"))?;
    let sequence=mounts[0].host().borrow().s.presentation_sequence;
    let generation=fence.diagnostics().generation;let identity=fence.diagnostics().frame.diagnostic_label();
    if dispose {mounts[0].lifetime().dispose();wait(||Ok(mounts[0].lifetime().status().phase==MountPhase::Disposed)).await?;clean(mounts[0].lifetime().status())?;}
    else {mounts[0].host().borrow_mut().set_hidden(true);}
    let error=match fence.poll(){Some(Err(e))=>e,_=>return Err("unpolled stale fence delivered pixels".into())};
    let expected=if dispose{genet_render_host::SurfaceCaptureCancellation::Disposed}else{genet_render_host::SurfaceCaptureCancellation::Hidden};
    if !matches!(&error,SurfaceReadbackFailure::Cancelled(reason) if std::mem::discriminant(reason)==std::mem::discriminant(&expected)){return Err(format!("wrong lifetime refusal: {error:?}"))}
    if !matches!(fence.poll(),Some(Err(SurfaceReadbackFailure::AlreadyCollected))){return Err("map refusal replayed".into())}
    if armed.callback_count.get()!=1||!armed.outside_borrow.get()||mounts[0].host().borrow().s.presentation_sequence!=sequence{return Err("postpublication refusal replayed callback or rolled back/published sequence".into())}
    if !dispose{mounts[0].host().borrow_mut().set_hidden(false);}
    let sibling=accepted(arm(&mounts[1],budget)?).await?;
    if sibling.frame().metadata().generation!=generation{return Err("healthy hide/dispose replaced shared Core".into())}
    let mut result=json!({"frame":identity,"original_generation":generation,"already_published_sequence":sequence,"reason":format!("{error:?}"),
        "callback_count":1,"repeat_poll":"AlreadyCollected","historical_publication_not_rolled_back":true,"healthy_sibling_same_core":acquired_receipt(&sibling)?});
    result["artifact"]=artifact_store::retain(if dispose{"dispose_healthy_sibling"}else{"unhide_healthy_sibling"}.into(),artifact_store::Artifact::Acquired(sibling))?;Ok(result)
}

async fn setup_two(first_id:&str,second_id:&str)->Result<(Vec<BrowserMount>,Arc<genet_render_host::SharedRenderCore>),String>{
    if first_id==second_id{return Err("two independent canvas IDs required".into())}
    let options=||HostOptions{title:"Actual acquired-surface control batch".into(),..HostOptions::default()};
    let core=Arc::new(RenderCore::boot_async((options().netrender)()).await?);let owner=core.shared_recovery();let mut mounts=Vec::new();
    for (id,name,mode) in [(first_id,"Acquired-first",Mode::Light),(second_id,"Acquired-second",Mode::Dark)]{
        let canvas=canvas(id)?;size(&canvas,1280,1100)?;let state=state(name,mode)?;let ready=core.clone();
        let mount=mount_with_core_future_and_readback_diagnostics(canvas,options(),MountLifetime::new(),async move{Ok(ready)},
            move|_,_,_|Init{state,logic:Box::new(workshop_view) as Logic,sheet:workshop_stylesheet(),
            fonts:generic_fonts().into_iter().map(|(family,bytes)|HostFont{family:Some(family),bytes}).collect(),images:Vec::new()},hooks()).await?;
        mount.host().borrow_mut().enable_browser_work_observation();LIFETIMES.with(|l|l.borrow_mut().push(mount.lifetime().clone()));mounts.push(mount);
    }
    drop(core);Ok((mounts,owner))
}
async fn close_group(mounts:&[BrowserMount],owner:Arc<genet_render_host::SharedRenderCore>)->Result<Value,String>{
    let cleanup=dispose_lifetimes().await?;
    for m in mounts {let h=m.host().borrow();if h.s.surface.is_some()||h.s.shared.render_core.is_some()||h.s.shared.render_recovery.is_some(){return Err("disposed host GPU bindings retained".into())}}
    let core=owner.current().map_err(|e|format!("closing Core unavailable: {e:?}"))?;
    let generation=core.generation();let weak=Arc::downgrade(&core);core.device().destroy();drop(core);
    wait(||{let core=weak.upgrade().ok_or("closing notice witness retired")?;Ok(core.loss_notice().is_some_and(|n|n.generation==generation&&n.reason==wgpu::DeviceLostReason::Destroyed))}).await?;
    drop(owner);if weak.upgrade().is_some(){return Err("closed fixture group retained Core".into())}
    Ok(json!({"mount_cleanup":cleanup,"owned_generation":generation,"destroyed_notice_observed":true,"rust_core_released":true,"physical_driver_reset":false}))
}
async fn prepublication_dispose_control(mount:&BrowserMount,budget:&Arc<SurfaceCaptureBudget>)->Result<Value,String>{
    let armed=arm(mount,budget)?;let pending=phase2_pending(mount).await?;let sequence=mount.host().borrow().s.presentation_sequence;
    mount.lifetime().dispose();wait(||Ok(mount.lifetime().status().phase==MountPhase::Disposed&&armed.result.borrow().is_some())).await?;clean(mount.lifetime().status())?;
    let error=armed.result.borrow_mut().take().ok_or("disposed pending callback absent")?.err().ok_or("disposed pending phase2 returned fence")?;
    if !matches!(&error,SurfaceReadbackFailure::Cancelled(genet_render_host::SurfaceCaptureCancellation::Disposed))
        ||armed.callback_count.get()!=1||!armed.outside_borrow.get()||armed.sequence_at_delivery.get()!=Some(sequence){return Err(format!("wrong prepublication disposal completion: {error:?}"))}
    let counts_before=counts(mount)?;let until=now()?+5000.;
    for _ in 0..6{let remaining=until-now()?;if remaining<=0.{return Err("disposed six RAF deadline".into())}
        JsFuture::from(acquired_control_raf(remaining).map_err(|e|format!("disposed RAF admission: {e:?}"))?).await.map_err(|e|format!("disposed RAF deadline: {e:?}"))?;}
    if now()?>=until||counts(mount)?!=counts_before||mount.host().borrow().s.presentation_sequence!=sequence{return Err("disposed phase2 retained an active RAF or published".into())}
    Ok(json!({"actual_pending_frame":pending.frame.diagnostic_label(),"stage":"SurfaceCompletion","generation":pending.generation,"refusal":format!("{error:?}"),
        "callback_count":1,"outside_host_borrow":true,"sequence_unchanged":true,"actual_raf_callbacks":6,"work_unchanged":true,"lifetime":mount_status(mount.lifetime().status())}))
}
fn admission_borrow_control(mount:&BrowserMount,budget:&Arc<SurfaceCaptureBudget>)->Result<Value,String>{
    let count=Rc::new(Cell::new(0));let outside=Rc::new(Cell::new(true));
    let probe=HookDrop{weak:Rc::downgrade(mount.host()),count:count.clone(),outside:outside.clone()};
    let mut callback:Option<BrowserSurfaceCaptureFn>=Some(Box::new(move|_|{drop(probe);}));
    let result={let _held=mount.host().borrow_mut();Application::request_browser_surface_capture(mount.host(),budget.clone(),&mut callback)};
    if result!=Err(cambium_rootstock::BrowserSurfaceCaptureRequestFailure::HostBorrowed)||callback.is_none()||count.get()!=0{return Err("borrowed request consumed/dropped caller callback".into())}
    drop(callback);if count.get()!=1||!outside.get(){return Err("refused caller callback did not retire once outside borrow".into())}
    Ok(json!({"actual_request_refusal":"HostBorrowed","caller_slot_retained":true,"callback_drop_count":1,"drop_outside_host_borrow":true}))
}
async fn explicit_retirement_controls(mount:&BrowserMount,budget:&Arc<SurfaceCaptureBudget>)->Result<Value,String>{
    let mut cases=Vec::new();
    for explicit_cancel in [true,false]{
        let before=budget.reserved_bytes();let armed=arm(mount,budget)?;wait(||Ok(armed.result.borrow().is_some())).await?;
        let fence=armed.result.borrow_mut().take().ok_or("retirement fence callback absent")?.map_err(|e|format!("retirement request refused: {e:?}"))?;
        if budget.reserved_bytes()<=before{return Err("real pending map had no padded/raw credit".into())}
        let frame=fence.diagnostics().frame.diagnostic_label();
        let refusal=if explicit_cancel{Some(fence.cancel(genet_render_host::SurfaceCaptureCancellation::Explicit))}else{drop(fence);None};
        if explicit_cancel&&!matches!(&refusal,Some(SurfaceReadbackFailure::Cancelled(genet_render_host::SurfaceCaptureCancellation::Explicit))){return Err(format!("wrong actual explicit map cancellation: {refusal:?}"))}
        if budget.reserved_bytes()!=before||armed.callback_count.get()!=1||!armed.outside_borrow.get(){return Err("move-only pending cancel/drop leaked credit or replayed callback".into())}
        cases.push(json!({"frame":frame,"operation":if explicit_cancel{"cancel Explicit"}else{"drop unpolled fence"},"callback_count":1,
            "refusal":refusal.map(|e|format!("{e:?}")),"credit_before":before,"credit_after":budget.reserved_bytes(),"retired_outside_host_borrow":true,
            "GPU_map_completion_state_unobserved":true,"physical_VRAM_free_claim":false}));
    }
    let healthy=accepted(arm(mount,budget)?).await?;let valid=acquired_receipt(&healthy)?;
    let artifact=artifact_store::retain("cancel_drop_valid_next".into(),artifact_store::Artifact::Acquired(healthy))?;
    Ok(json!({"cases":cases,"valid_next":valid,"artifact":artifact,"map_error_injection_qualified":false}))
}
pub(super) async fn execute_lifetime(first:&str,second:&str,run:&str)->Result<Value,String>{
    let started=now()?;let budget=Arc::new(SurfaceCaptureBudget::default());artifact_store::initialize(budget.clone())?;
    let (mounts,owner)=setup_two(first,second).await?;
    let initial_adapter={let c=owner.current().map_err(|e|format!("initial Core absent: {e:?}"))?;adapter_identity(&c)};
    let borrowed_request=admission_borrow_control(&mounts[0],&budget)?;
    let explicit_retirement=explicit_retirement_controls(&mounts[0],&budget).await?;
    let source_resize=prepublication_source_resize_control(&mounts[0],&budget).await?;
    let hidden=postpublication_lifetime_control(&mounts,&budget,false).await?;
    let loss=pending_loss_control(&mounts,&budget).await?;
    let disposed=postpublication_lifetime_control(&mounts,&budget,true).await?;
    let pending_disposed=prepublication_dispose_control(&mounts[1],&budget).await?;
    let replacement_adapter={let c=owner.current().map_err(|e|format!("replacement Core absent: {e:?}"))?;adapter_identity(&c)};
    let closure=close_group(&mounts,owner).await?;
    if now()?-started>=60000.{return Err("lifetime batch original sixty-second bound exceeded".into())}
    Ok(json!({"status":"pass","run":run,"batch":"stale-lifetime","elapsed_ms":now()?-started,"borrowed_request":borrowed_request,"explicit_retirement":explicit_retirement,"source_resize":source_resize,"hidden":hidden,"pending_loss":loss,"disposed":disposed,"pending_disposed":pending_disposed,
        "closure":closure,"initial_adapter":initial_adapter,"replacement_adapter":replacement_adapter,"reserved_bytes":artifact_store::reserved()?,"per_operation_deadline_ms":5000,"physical_driver_reset":false,"os_accessibility_actions":false,
        "transient_surface_skip_qualified":false,"raw_artifacts":[explicit_retirement["artifact"],source_resize["artifact"],hidden["artifact"],loss["artifact"],disposed["artifact"]]}))
}

#[wasm_bindgen(inline_js=r#"
export function acquired_control_pop_three(device) {
  return Promise.allSettled([device.popErrorScope(),device.popErrorScope(),device.popErrorScope()])
    .then(rows=>rows.map(row=>({status:row.status,cleanNull:row.status==='fulfilled'&&row.value===null})));
}
export function acquired_control_raf(timeout) {
  return new Promise((resolve,reject)=>{let id;const timer=setTimeout(()=>{cancelAnimationFrame(id);reject(Error('finite RAF deadline'));},timeout);
    id=requestAnimationFrame(v=>{clearTimeout(timer);resolve(v);});});
}
export function acquired_control_unconfigure(canvas) {
  const context=canvas.getContext('webgpu');
  if(!context||typeof context.unconfigure!=='function')throw Error('owned WebGPU canvas context unavailable');
  context.unconfigure();
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)] fn acquired_control_pop_three(device:&JsValue)->Result<js_sys::Promise,JsValue>;
    #[wasm_bindgen(catch)] fn acquired_control_raf(timeout:f64)->Result<js_sys::Promise,JsValue>;
    #[wasm_bindgen(catch)] fn acquired_control_unconfigure(canvas:&HtmlCanvasElement)->Result<(),JsValue>;
}
#[derive(Clone,Copy,Debug)] enum SurfaceFault { Clean, Validation, RejectedScope, Lost }
fn counts(m:&BrowserMount)->Result<cambium_rootstock::BrowserWorkCounts,String>{
    let c=m.host().borrow().browser_work_counts().ok_or("actual work counters absent")?;
    if c.saturated{return Err("monotonic work counters saturated".into())}Ok(c)
}
async fn idle_six(mounts:&[BrowserMount])->Result<Value,String>{
    wait(||Ok(mounts.iter().all(|m|!m.host().borrow().has_pending_browser_frame()&&!m.window().pending_frame_flag().load(std::sync::atomic::Ordering::Relaxed)))).await?;
    let before:Vec<_>=mounts.iter().map(counts).collect::<Result<_,_>>()?;
    let sequence:Vec<_>=mounts.iter().map(|m|m.host().borrow().s.presentation_sequence).collect();let until=now()?+5000.;
    for _ in 0..6 {let remaining=until-now()?;if remaining<=0.{return Err("original finite six RAF deadline".into())}
        JsFuture::from(acquired_control_raf(remaining).map_err(|e|format!("RAF promise admission: {e:?}"))?).await.map_err(|e|format!("actual RAF timeout: {e:?}"))?;
        if now()?>=until{return Err("actual six RAF completed late".into())}}
    for (i,m) in mounts.iter().enumerate(){if counts(m)?!=before[i]||m.host().borrow().s.presentation_sequence!=sequence[i]{return Err("healthy phase2 refusal manufactured an idle redraw loop".into())}}
    Ok(json!({"actual_raf_callbacks":6,"all_work_counters_and_sequences_unchanged":true}))
}
struct HookDrop {weak:std::rc::Weak<RefCell<Application>>,count:Rc<Cell<u64>>,outside:Rc<Cell<bool>>}
impl Drop for HookDrop {fn drop(&mut self){self.count.set(self.count.get()+1);if let Some(h)=self.weak.upgrade(){if h.try_borrow_mut().is_err(){self.outside.set(false)}}}}
async fn execute_surface_controls(first:&str,second:&str,run:&str,lost_only:bool)->Result<Value,String>{
    let started=now()?;let budget=Arc::new(SurfaceCaptureBudget::default());artifact_store::initialize(budget.clone())?;
    let (mounts,owner)=setup_two(first,second).await?;idle_six(&mounts).await?;
    let initial_adapter={let c=owner.current().map_err(|e|format!("initial Core absent: {e:?}"))?;adapter_identity(&c)};
    let selected=Rc::new(Cell::new(SurfaceFault::Clean));let fault=selected.clone();let buffers=Rc::new(RefCell::new(Vec::new()));let owned=buffers.clone();
    let early=Rc::new(RefCell::new(None));let early_hook=early.clone();let drops=Rc::new(Cell::new(0));let outside=Rc::new(Cell::new(true));
    let unconfigured=Rc::new(Cell::new(false));let loss_once=unconfigured.clone();let canvas=mounts[0].window().canvas().clone();let unconfigure_error=Rc::new(RefCell::new(None));let loss_error=unconfigure_error.clone();
    let probe=HookDrop{weak:Rc::downgrade(mounts[0].host()),count:drops.clone(),outside:outside.clone()};
    let mut hook=Some(Box::new(move|point:cambium_rootstock::BrowserSurfaceControlPoint,commands:&genet_render_host::BrowserFrameCommands<'_>|{
        let _keep=&probe;
        if point==cambium_rootstock::BrowserSurfaceControlPoint::BeforeAcquire&&matches!(fault.get(),SurfaceFault::Lost)&&!loss_once.replace(true){
            if let Err(error)=acquired_control_unconfigure(&canvas){*loss_error.borrow_mut()=Some(format!("actual owned unconfigure threw: {error:?}"));}
        }
        if point!=cambium_rootstock::BrowserSurfaceControlPoint::AfterCompose{return Ok(())}
        match fault.get(){
            SurfaceFault::Validation=>{
                let make=||commands.device().create_buffer(&wgpu::BufferDescriptor{label:Some("actual phase2 acquired fixture invalid-copy"),size:4,
                    usage:wgpu::BufferUsages::COPY_SRC|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});
                let a=make();let b=make();let mut encoder=commands.device().create_command_encoder(&Default::default());encoder.copy_buffer_to_buffer(&a,0,&b,0,8);
                commands.queue().submit([encoder.finish()]);owned.borrow_mut().extend([a,b]);
            },
            SurfaceFault::RejectedScope=>{
                if let Some(device)=commands.device().as_webgpu(){
                    // If this actual API throws, the control fails through its
                    // missing early-pop evidence; no synthetic GPU error exists.
                    if let Ok(promise)=acquired_control_pop_three(device.as_ref()){*early_hook.borrow_mut()=Some(JsFuture::from(promise));}
                }
            },SurfaceFault::Clean|SurfaceFault::Lost=>{},
        }Ok(())
    }) as cambium_rootstock::BrowserSurfaceControlFn);
    {let mut h=mounts[0].host().borrow_mut();h.set_browser_surface_control(&mut hook).map_err(|e|format!("real phase2 hook admission refused: {e:?}"))?;}drop(hook);
    let weak={let h=mounts[0].host().borrow();Arc::downgrade(h.s.shared.render_core.as_ref().ok_or("actual Core absent")?)};
    let mut cases=Vec::new();
    for fault in if lost_only{Vec::new()}else{vec![SurfaceFault::Validation,SurfaceFault::RejectedScope]}{
        let before=counts(&mounts[0])?;let sequence=mounts[0].host().borrow().s.presentation_sequence;
        selected.set(fault);let armed=arm(&mounts[0],&budget)?;
        wait(||Ok(armed.result.borrow().is_some())).await?;
        let error=armed.result.borrow_mut().take().ok_or("phase2 refusal absent")?.err().ok_or("real phase2 fault returned a fence")?;
        if armed.callback_count.get()!=1||!armed.outside_borrow.get()||armed.sequence_at_delivery.get()!=Some(sequence){return Err("phase2 fault replayed callback or published".into())}
        let precise=matches!((&fault,&error),
            (SurfaceFault::Validation,SurfaceReadbackFailure::FrameRejected(genet_render_host::RasterizeFailure::ProviderError(e))) if matches!(&**e,netrender::VelloError::WgpuErrorFromScope(wgpu::Error::Validation{..})))
            || matches!((&fault,&error),(SurfaceFault::RejectedScope,SurfaceReadbackFailure::FrameRejected(genet_render_host::RasterizeFailure::ProviderError(e))) if matches!(&**e,netrender::VelloError::WgpuErrorFromScope(wgpu::Error::Internal{description,..}) if description.contains("actual GPU scope/queue promise rejected")));
        if !precise{return Err(format!("wrong actual phase2 {fault:?} refusal: {error:?}"))}
        let after=counts(&mounts[0])?;
        let diagnostic=mounts[0].host().borrow().browser_surface_capture_observation().ok_or("actual refused phase2 diagnostics absent")?;
        let phase1=diagnostic.phase1.as_ref().ok_or("negative actual phase1 scope rows absent")?;
        let phase2=diagnostic.phase2.as_ref().ok_or("negative actual phase2 scope rows absent")?;
        if phase1.phase!=1||phase2.phase!=2||phase1.frame!=phase2.frame||diagnostic.frame!=phase2.frame{return Err("negative diagnostics crossed actual phases/frame".into())}
        if after.surface_submission.checked_sub(before.surface_submission)!=Some(1)||after.surface_acquired.checked_sub(before.surface_acquired)!=Some(1)
            ||after.surface_present_call.checked_sub(before.surface_present_call)!=Some(1)||after.surface_completion_refused.checked_sub(before.surface_completion_refused)!=Some(1)
            ||after.surface_completion_accepted!=before.surface_completion_accepted||after.publication!=before.publication||after.surface_skipped!=before.surface_skipped{return Err("phase2 fault did not bind exact1/1/1/refused1/publication0".into())}
        // Fault-owned buffers are explicitly retired outside Host borrow.
        let retired=std::mem::take(&mut *buffers.borrow_mut());drop(retired);
        let early_rows=if matches!(fault,SurfaceFault::RejectedScope){
            let mut promise=early.borrow_mut().take().ok_or("actual early-pop promise absent")?;let mut rows=None;
            wait(||{let mut cx=TaskContext::from_waker(Waker::noop());if let Poll::Ready(value)=Pin::new(&mut promise).poll(&mut cx){rows=Some(value.map_err(|e|format!("actual early pop rejected: {e:?}"))?);Ok(true)}else{Ok(false)}}).await?;
            let rows=js_sys::Array::from(&rows.ok_or("actual early rows absent")?);
            if rows.length()!=3||rows.iter().any(|row|js_sys::Reflect::get(&row,&JsValue::from_str("status")).ok().and_then(|v|v.as_string()).as_deref()!=Some("fulfilled")||js_sys::Reflect::get(&row,&JsValue::from_str("cleanNull")).ok().and_then(|v|v.as_bool())!=Some(true)){return Err("early owned scopes were not three real clean-null completions".into())}
            json!({"actual_pop_count":3,"fulfilled_clean_null_count":3,"subsequent_real_empty_stack_rejection":true})
        }else{Value::Null};
        let quiet=idle_six(&mounts).await?;selected.set(SurfaceFault::Clean);
        {let mut h=mounts[0].host().borrow_mut();focus_attribute(&mut h,"aria-label","Primary hue")?;let p=KeyPress::named(NamedKey::ArrowRight);h.key(&p);h.key_released(&p.key);}
        let fresh=accepted(arm(&mounts[0],&budget)?).await?;
        {let h=mounts[0].host().borrow();let original=weak.upgrade().ok_or("healthy original Core retired")?;
            if !Arc::ptr_eq(h.s.shared.render_core.as_ref().ok_or("healthy Core absent")?,&original){return Err("healthy phase2 refusal replaced Core".into())}}
        let valid_next=acquired_receipt(&fresh)?;let artifact=artifact_store::retain(format!("{fault:?}_valid_next"),artifact_store::Artifact::Acquired(fresh))?;
        cases.push(json!({"fault":format!("{fault:?}"),"refusal":format!("{error:?}"),"actual_phase2_counts":{"submission":1,"acquired":1,"present_call":1,"refused":1,"publication":0},
            "phase1":phase_observation(phase1)?,"phase2":observed_phase(phase2),"owned_surface_stamp":format!("{:?}",diagnostic.surface),
            "actual_work_before":format!("{before:?}"),"actual_work_after":format!("{after:?}"),"callback_count":1,"idle_after_refusal":quiet,"early_owned_scope_rows":early_rows,
            "authored_input_retry":true,"valid_next_same_core":valid_next,"artifact":artifact}));
    }
    if lost_only {
    // The exact same canvas is genuinely unconfigured immediately before
    // actual getCurrentTexture. No fabricated SurfaceAcquire value exists.
    selected.set(SurfaceFault::Lost);let before=counts(&mounts[0])?;let sequence=mounts[0].host().borrow().s.presentation_sequence;
    let armed=arm(&mounts[0],&budget)?;let mut skipped=None;
    wait(||{
        let (pending,lost,retired)={let mut h=mounts[0].host().borrow_mut();let _=mounts[0].window().take_pending_frame();h.redraw_browser_turn(None);
            (h.browser_surface_capture_pending(),h.s.needs_recovery_surface&&h.s.surface.is_none(),h.take_browser_retirements())};drop(retired);
        if let Some(p)=pending{if p.stage==cambium_rootstock::BrowserSurfaceCaptureStage::SurfaceSkipped&&lost{skipped=Some(p);return Ok(true)}}
        if armed.callback_count.get()!=0{return Err("actual Lost skip was not observed before fresh completion".into())}Ok(false)
    }).await?;
    if !unconfigured.get()||unconfigure_error.borrow().is_some(){return Err(format!("actual same-canvas unconfigure did not run cleanly: {:?}",unconfigure_error.borrow()))}
    let skipped=skipped.ok_or("actual skipped surface predicate absent")?;let fresh=accepted(armed).await?;let after=counts(&mounts[0])?;
    if fresh.presentation().sequence!=sequence.checked_add(1).ok_or("sequence overflow")?||fresh.frame().metadata().frame==skipped.frame
        ||after.surface_skipped.checked_sub(before.surface_skipped)!=Some(1)||after.publication.checked_sub(before.publication)!=Some(1){return Err("actual Lost skip published stale work or failed exactly one retry".into())}
    {let h=mounts[0].host().borrow();let original=weak.upgrade().ok_or("Lost healthy Core retired")?;if !Arc::ptr_eq(h.s.shared.render_core.as_ref().ok_or("Lost retry Core absent")?,&original){return Err("surface Lost replaced healthy shared Core".into())}}
    let valid_next=acquired_receipt(&fresh)?;let artifact=artifact_store::retain("Lost_valid_next".into(),artifact_store::Artifact::Acquired(fresh))?;
    cases.push(json!({"fault":"Lost","stimulus":"once same owned canvas context.unconfigure in BeforeAcquire; actual getCurrentTexture maps Lost",
        "unconfigure_calls":1,"actual_Lost_detached_surface":true,"actual_skipped_frame":skipped.frame.diagnostic_label(),"actual_stage":"SurfaceSkipped","original_sequence":sequence,
        "request_preserved":true,"one_fresh_publication":true,"healthy_same_core":true,"actual_work_before":format!("{before:?}"),"actual_work_after":format!("{after:?}"),
        "valid_next_same_core":valid_next,"artifact":artifact,"raw_Unavailable_qualified":false}));
    }
    let closure=close_group(&mounts,owner).await?;
    if drops.get()!=1||!outside.get(){return Err("disposed retained Host failed to retire hook exactly once outside borrow".into())}
    if now()?-started>=60000.{return Err("phase2 faults original sixty-second bound exceeded".into())}
    Ok(json!({"status":if lost_only{"completed-unqualified-diagnostic"}else{"pass"},"run":run,"batch":if lost_only{"real-surface-Lost-diagnostic"}else{"real-phase2-faults"},"elapsed_ms":now()?-started,"cases":cases,"closure":closure,
        "initial_adapter":initial_adapter,"hook_drop_count":drops.get(),"hook_drop_outside_borrow":outside.get(),"per_operation_deadline_ms":5000,
        "raw_artifacts":cases.iter().map(|c|c["artifact"].clone()).collect::<Vec<_>>(),"reserved_bytes":artifact_store::reserved()?,"physical_driver_reset":false,"os_accessibility_actions":false}))
}

/// One bounded clean/recovery batch. Real phase2 faults and stale lifetime
/// controls are separate exports; they must not be silently inferred here.
pub(super) async fn execute_recovery(first_id:&str,second_id:&str,run:&str)->Result<Value,String>{
    let started=now()?;let budget=Arc::new(SurfaceCaptureBudget::default());artifact_store::initialize(budget.clone())?;
    let options=||HostOptions{title:"Acquired Workshop surface recovery".into(),..HostOptions::default()};
    let core=Arc::new(RenderCore::boot_async((options().netrender)()).await?);let generation=core.generation();
    let initial_adapter=adapter_identity(&core);let old=Arc::downgrade(&core);let owner=core.shared_recovery();let mut mounts=Vec::new();
    for (id,name,mode) in [(first_id,"Acquired-first",Mode::Light),(second_id,"Acquired-second",Mode::Dark)]{
        let canvas=canvas(id)?;size(&canvas,1280,1100)?;let state=state(name,mode)?;let ready=core.clone();
        let life=MountLifetime::new();let mount=mount_with_core_future_and_readback_diagnostics(canvas,options(),life,
            async move{Ok(ready)},move|_,_,_|Init{state,logic:Box::new(workshop_view) as Logic,sheet:workshop_stylesheet(),
            fonts:generic_fonts().into_iter().map(|(family,bytes)|HostFont{family:Some(family),bytes}).collect(),images:Vec::new()},hooks()).await?;
        LIFETIMES.with(|l|l.borrow_mut().push(mount.lifetime().clone()));mounts.push(mount);
    }
    drop(core);let mut ledger=Vec::new();let mut actual=Vec::new();let mut authorities=Vec::new();
    for (phase_name,after) in [("before",false),("after",true)]{
        if after {
            // Hidden sibling has no raster RAF but must detach via shared wake.
            mounts[1].host().borrow_mut().set_hidden(true);
            {let core=owner.current().map_err(|e|format!("destroy Core absent: {e:?}"))?;core.device().destroy();}
            wait(||{if let RecoveryStatus::Failed{error,..}=owner.status(){return Err(format!("shared replacement failed: {error}"))}
                Ok(mounts.iter().all(|m|{let h=m.host().borrow();h.s.shared.render_core.as_ref().is_some_and(|c|c.generation()!=generation)&&h.s.surface.is_some()&&!h.s.needs_recovery_surface}))}).await?;
            if old.upgrade().is_some(){return Err("actual pending surface retained old Core".into())}
            let replacement=owner.current().map_err(|e|format!("replacement absent: {e:?}"))?;
            for m in &mounts {if !Arc::ptr_eq(m.host().borrow().s.shared.render_core.as_ref().ok_or("replacement binding absent")?,&replacement){return Err("siblings adopted distinct replacement Core".into())}}
            drop(replacement);if !mounts[1].host().borrow().s.hidden{return Err("recovery changed hidden authority".into())}
            mounts[1].host().borrow_mut().set_hidden(false);
        }
        for (index,mount) in mounts.iter().enumerate(){
            let prefix=if index==0{"first"}else{"second"};let reference=reference_root(mount,&budget).await?;
            authorities.push(reference.capture.seal.clone());
            ledger.push(retain_reference(format!("{prefix}_root_reference_{phase_name}"),reference)?);
            let frame=accepted(arm(mount,&budget)?).await?;let mut observation=acquired_receipt(&frame)?;observation["artifact_name"]=json!(format!("{prefix}_acquired_{phase_name}"));actual.push(observation);
            ledger.push(artifact_store::retain(format!("{prefix}_acquired_{phase_name}"),artifact_store::Artifact::Acquired(frame))?);
            for (slot,key) in KEYS.into_iter().enumerate(){let r=reference_source(mount,key,(index*2+slot) as u64,&budget).await?;
                ledger.push(retain_reference(format!("{prefix}_{}_reference_{phase_name}",if slot==0{"reader"}else{"stylesheet"}),r)?);}
        }
    }
    // Authority comparison deliberately excludes GPU generation/sequence, but
    // preserves actual runner, both retained document Rc and author state.
    for index in 0..2 {for field in ["authority","runner","reader_source","stylesheet_source"]{
        if authorities[index][field]!=authorities[index+2][field]{return Err(format!("actual recovery changed authority: {field}"))}
    }}
    // The application remains authoritative after actual shared replacement.
    // These transient move-only frames consume the SAME budget and are dropped
    // before exports; they are not additional retained historical artifacts.
    {let mut h=mounts[0].host().borrow_mut();focus_attribute(&mut h,"aria-label","Primary hue")?;}
    let input_before=accepted(arm(&mounts[0],&budget)?).await?;
    let theme_before=serde_json::to_value(mounts[0].host().borrow().s.runner.as_ref().ok_or("input runner absent")?.state().draft_theme()).map_err(|e|e.to_string())?;
    {let mut h=mounts[0].host().borrow_mut();let press=KeyPress::named(NamedKey::ArrowRight);h.key(&press);h.key_released(&press.key);}
    let input_after=accepted(arm(&mounts[0],&budget)?).await?;
    let theme_after=serde_json::to_value(mounts[0].host().borrow().s.runner.as_ref().ok_or("input runner absent")?.state().draft_theme()).map_err(|e|e.to_string())?;
    let before_hash=digest_bytes(input_before.frame().rgba());let after_hash=digest_bytes(input_after.frame().rgba());
    if theme_before==theme_after || before_hash==after_hash{return Err("post-recovery logical input did not change actual author state and acquired canvas pixels".into())}
    let input=json!({"route":"logical production Host key press/release; no OS keyboard/a11y claim","theme_before":theme_before,"theme_after":theme_after,
        "before_sha256":before_hash,"after_sha256":after_hash,"before":acquired_receipt(&input_before)?,"after":acquired_receipt(&input_after)?});
    drop(input_before);drop(input_after);
    let cleanup=dispose_lifetimes().await?;
    for m in &mounts {let h=m.host().borrow();if h.s.surface.is_some()||h.s.shared.render_core.is_some()||h.s.shared.render_recovery.is_some(){return Err("disposed Host retained GPU bindings".into())}}
    let closed=owner.current().map_err(|e|format!("final Core absent: {e:?}"))?;let replacement_adapter=adapter_identity(&closed);
    let closed_generation=closed.generation();let weak=Arc::downgrade(&closed);closed.device().destroy();drop(closed);
    wait(||{let core=weak.upgrade().ok_or("closure witness Core retired before loss observation")?;
        Ok(core.loss_notice().is_some_and(|n|n.generation==closed_generation&&n.reason==wgpu::DeviceLostReason::Destroyed))}).await?;
    drop(owner);if weak.upgrade().is_some(){return Err("closed actual group retained Core".into())}
    if now()?-started>=60000.{return Err("fixture original sixty-second bound exceeded".into())}
    Ok(json!({"status":"pass","run":run,"batch":"clean-recovery","elapsed_ms":now()?-started,"initial_generation":generation,"replacement_generation":closed_generation,
        "initial_adapter":initial_adapter,"replacement_adapter":replacement_adapter,"actual_acquired":actual,"authority_seals":authorities,"raw_artifacts":ledger,"post_recovery_input":input,
        "diagnostic_target_ledger":artifact_store::target_ledger(),"retained_budget_bytes":artifact_store::reserved()?,"shared_budget_limit":RETAINED_LIMIT,"per_frame_limit":BYTES_LIMIT,
        "original_operation_deadline_ms":5000,"renderer_thresholds":{"edge_rgba":32,"edge_alpha":16,"interior_rgba":3,"interior_alpha":2,"mean":1,"reference_neighbor_variation":8},
        "recovery_pixel_thresholds":{"rgb":3,"alpha":2,"mean_absolute":1},"minimum_colors":13,"cleanup":cleanup,
        "old_core_released":true,"shared_replacement_arc":true,"disposed_host_gpu_bindings_released":true,"final_owned_device_destroyed":true,
        "physical_driver_reset":false,"os_accessibility_actions":false,"sustained_frame_or_performance_claim":false,
        "pixel_acceptance":"pending independent driver comparisons and all raw/PNG visual review","phase2_faults_qualified":false,"pending_stale_lifetime_qualified":false}))
}

pub(super) async fn execute_faults(first:&str,second:&str,run:&str)->Result<Value,String>{execute_surface_controls(first,second,run,false).await}
pub(super) async fn execute_lost(first:&str,second:&str,run:&str)->Result<Value,String>{execute_surface_controls(first,second,run,true).await}
