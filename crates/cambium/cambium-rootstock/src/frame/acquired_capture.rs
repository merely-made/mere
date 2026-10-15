// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Explicit diagnostics for copies from the actual acquired final surface.
//! Requests and completion delivery are retired outside the Host borrow.
use std::{cell::RefCell, rc::Rc, sync::{Arc,Weak}};
use crate::{Host, HostTree, PresentedFrame, ProducerImageIdentity, BrowserSurfaceStamp};
use crate::meristem_bounds::RootView;
use genet_render_host::{AcceptedSurfaceReadback, AcquiredSurfaceFrame, BrowserFrameQualityObservation,
    RenderCore, SurfaceCaptureBudget, SurfaceCaptureCancellation as Cancel, SurfaceCaptureConfiguration,
    SurfaceCaptureFrameIdentity, SurfaceReadbackFailure as Failure};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum BrowserSurfaceCaptureStage { PreparedResources, SurfaceCompletion, SurfaceSkipped }
/// Copied identities only: no Core, target, buffer, callback, or Host ownership.
#[derive(Clone,Debug)]
pub struct BrowserSurfaceCapturePending {
    pub frame:SurfaceCaptureFrameIdentity,
    pub stage:BrowserSurfaceCaptureStage,
    pub generation:u64,
    pub epoch:u64,
    pub surface:BrowserSurfaceStamp,
    pub width:u32,
    pub height:u32,
    pub proposed_sequence:Option<u64>,
    pub request_armed:bool,
}
#[derive(Clone,Debug)]
pub struct BrowserSurfaceCaptureSource { pub local_key:u64, pub image:ProducerImageIdentity }
#[derive(Clone,Debug)]
pub struct BrowserSurfaceCaptureDiagnostics {
    pub frame:SurfaceCaptureFrameIdentity,
    pub generation:u64,
    pub epoch:u64,
    pub surface:BrowserSurfaceStamp,
    pub configuration:Option<SurfaceCaptureConfiguration>,
    pub phase1:Option<BrowserFrameQualityObservation>,
    pub phase2:Option<BrowserFrameQualityObservation>,
    pub sources:Vec<BrowserSurfaceCaptureSource>,
    pub presentation:Option<PresentedFrame>,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum BrowserSurfaceCaptureRequestFailure { HostBorrowed, AlreadyArmed, Disposed, Hidden, Empty }
pub type BrowserSurfaceCaptureFn=Box<dyn FnOnce(Result<BrowserAcquiredCaptureFence,Failure>)+'static>;
// A successful check is synchronous CPU inspection; false means a current
// application borrow is live, so do not map, cancel, or reenter it on this turn.
type Probe=Rc<dyn Fn(&CaptureHostStamp)->Result<bool,Cancel>>;
pub(crate) struct BrowserSurfaceCaptureRequest {
    pub(crate) identity:Arc<()>,
    pub(crate) budget:Arc<SurfaceCaptureBudget>,
    callback:Option<BrowserSurfaceCaptureFn>,
    probe:Probe,
}
#[derive(Clone)]
pub(crate) struct CaptureHostStamp {
    pub(crate) presentation:PresentedFrame,
    pub(crate) epoch:u64,
    pub(crate) sheet_generation:u64,
    pub(crate) surface:BrowserSurfaceStamp,
    pub(crate) format:wgpu::TextureFormat,
    pub(crate) core:Weak<RenderCore>,
    pub(crate) generation:u64,
    pub(crate) sources:Vec<BrowserSurfaceCaptureSource>,
}
/// Move-only delivery. This guard must be dropped only after Host RefMut ends.
pub(crate) struct BrowserSurfaceCaptureDelivery {
    callback:Option<BrowserSurfaceCaptureFn>,
    result:Option<Result<BrowserAcquiredCaptureFence,Failure>>,
}
impl Drop for BrowserSurfaceCaptureDelivery {
    fn drop(&mut self) {
        if let (Some(callback),Some(result))=(self.callback.take(),self.result.take()) { callback(result); }
    }
}
impl BrowserSurfaceCaptureRequest {
    pub(crate) fn delivery(mut self,result:Result<BrowserAcquiredCaptureFence,Failure>) -> BrowserSurfaceCaptureDelivery {
        BrowserSurfaceCaptureDelivery { callback:self.callback.take(),result:Some(result) }
    }
    pub(crate) fn fence(&self,readback:AcceptedSurfaceReadback,stamp:CaptureHostStamp,
        diagnostics:BrowserSurfaceCaptureDiagnostics) -> BrowserAcquiredCaptureFence {
        BrowserAcquiredCaptureFence { readback:Some(readback),ready:None,stamp,probe:self.probe.clone(),diagnostics,done:false }
    }
}

/// Poll/drop/cancel outside every Host borrow. Mapping is guarded by a brief
/// weak-Host probe before it, and complete pixels are checked again afterward.
#[must_use="poll, cancel, or drop this move-only fence outside Host borrow"]
pub struct BrowserAcquiredCaptureFence {
    readback:Option<AcceptedSurfaceReadback>,
    ready:Option<AcquiredSurfaceFrame>,
    stamp:CaptureHostStamp,
    probe:Probe,
    diagnostics:BrowserSurfaceCaptureDiagnostics,
    done:bool,
}
pub struct BrowserAcquiredCaptureFrame {
    presentation:PresentedFrame,
    frame:AcquiredSurfaceFrame,
    diagnostics:BrowserSurfaceCaptureDiagnostics,
    stamp:CaptureHostStamp,
    probe:Probe,
}
impl BrowserAcquiredCaptureFrame {
    pub fn presentation(&self)->PresentedFrame { self.presentation }
    pub fn frame(&self)->&AcquiredSurfaceFrame { &self.frame }
    pub fn diagnostics(&self)->&BrowserSurfaceCaptureDiagnostics { &self.diagnostics }
    /// Optional live-state query: actual loss/deadline brackets the brief
    /// Host/source snapshot. Historical accepted CPU pixels remain exportable
    /// after disposal without claiming that later application state is current.
    pub fn check_current(&self)->Result<(),Failure> {
        self.frame.check_current()?;
        let host=(self.probe)(&self.stamp);
        // Actual loss/deadline becoming observable during the CPU probe wins.
        self.frame.check_current()?;
        match host {
            Ok(true)=>Ok(()),
            Ok(false)=>Err(Failure::Properties("Host borrowed during capture observation")),
            Err(reason)=>Err(Failure::Cancelled(reason)),
        }
    }
}
impl BrowserAcquiredCaptureFence {
    pub fn diagnostics(&self)->&BrowserSurfaceCaptureDiagnostics { &self.diagnostics }
    fn check_gpu(&self)->Result<(),Failure> {
        if let Some(frame)=&self.ready { frame.check_current() }
        else { self.readback.as_ref().ok_or(Failure::AlreadyCollected)?.check_current() }
    }
    fn finish_error(&mut self,failure:Failure)->Option<Result<BrowserAcquiredCaptureFrame,Failure>> {
        self.done=true;
        drop(self.ready.take());
        // All GPU/map/reservation teardown stays on this outside-borrow turn.
        drop(self.readback.take());
        Some(Err(failure))
    }
    /// A None result means map pending or a brief existing application borrow.
    /// Busy deferral occurs only while the original loss/deadline remains valid:
    /// the next poll checks it first, with no renewal or Core reacquisition.
    /// Callers must poll, cancel and drop outside every Host borrow.
    pub fn poll(&mut self)->Option<Result<BrowserAcquiredCaptureFrame,Failure>> {
        if self.done { return Some(Err(Failure::AlreadyCollected)); }
        // Poll/drop/cancel is contractually outside every Host borrow. A real
        // original loss/deadline is never deferred by a later busy Host probe.
        if let Err(error)=self.check_gpu() { return self.finish_error(error); }
        let host=(self.probe)(&self.stamp);
        if let Err(error)=self.check_gpu() { return self.finish_error(error); }
        match host {
            Ok(false)=>return None,
            Err(reason)=>return self.finish_error(Failure::Cancelled(reason)),
            Ok(true)=>{},
        }
        if self.ready.is_none() {
            // probe's Host Ref has ended. Genet poll may run callback dispatch;
            // no Host or source Ref/RefMut is held across this call.
            match self.readback.as_mut().expect("owned accepted readback").poll()? {
                Err(error)=>return self.finish_error(error),
                Ok(frame)=>{ self.ready=Some(frame); drop(self.readback.take()); },
            }
        }
        if let Err(error)=self.check_gpu() { return self.finish_error(error); }
        let host=(self.probe)(&self.stamp);
        if let Err(error)=self.check_gpu() { return self.finish_error(error); }
        match host {
            Ok(false)=>return None,
            Err(reason)=>return self.finish_error(Failure::Cancelled(reason)),
            Ok(true)=>{},
        }
        self.done=true;
        Some(Ok(BrowserAcquiredCaptureFrame { presentation:self.stamp.presentation,
            frame:self.ready.take().expect("checked complete pixels"),diagnostics:self.diagnostics.clone(),
            stamp:self.stamp.clone(),probe:self.probe.clone() }))
    }
    pub fn cancel(mut self,reason:Cancel)->Failure {
        // Prioritize an actual old-generation loss/deadline over a host reason.
        let error=self.check_gpu().err().unwrap_or(Failure::Cancelled(reason));
        self.done=true;drop(self.ready.take());drop(self.readback.take());error
    }
}

impl<State,Logic,V,T> Host<State,Logic,V,T>
where State:'static,Logic:FnMut(&State)->V+'static,V:RootView<State>,T:HostTree<State>+'static,
{
    /// Arm one diagnostic capture on this exact independent Host. The same
    /// session budget must account for references and retained exported pixels.
    /// The callback is delivered only by outside-borrow retirement draining.
    pub fn request_browser_surface_capture(host:&Rc<RefCell<Self>>,budget:Arc<SurfaceCaptureBudget>,
        callback:&mut Option<BrowserSurfaceCaptureFn>) -> Result<(),BrowserSurfaceCaptureRequestFailure> {
        if callback.is_none() { return Err(BrowserSurfaceCaptureRequestFailure::Empty) }
        let weak=Rc::downgrade(host);
        let probe:Probe=Rc::new(move |expected| {
            let host=weak.upgrade().ok_or(Cancel::Disposed)?;
            let Ok(host)=host.try_borrow() else { return Ok(false) };
            if host.s.hidden { return Err(Cancel::Hidden) }
            if host.s.close_requested || host.s.surface.is_none() || host.s.shared.render_core.is_none() { return Err(Cancel::Disposed) }
            if host.s.browser_frame_epoch!=Some(expected.epoch) || host.s.shared.sheet_generation!=expected.sheet_generation {
                return Err(Cancel::Source);
            }
            let current=host.s.shared.render_core.as_ref().ok_or(Cancel::Disposed)?;
            if current.generation()!=expected.generation || expected.core.upgrade().is_none_or(|core|!Arc::ptr_eq(current,&core)) {
                return Err(Cancel::Disposed);
            }
            if host.s.presentation_host!=expected.presentation.host || host.s.presentation_sequence!=expected.presentation.sequence {
                return Err(Cancel::Source);
            }
            if host.layout_scale() as f32!=expected.presentation.layout_scale
                || host.s.window.as_ref().is_none_or(|window|window.inner_size()!=(expected.presentation.width,expected.presentation.height)) {
                return Err(Cancel::Viewport);
            }
            let surface=host.s.surface.as_ref().ok_or(Cancel::Disposed)?;
            if surface.format()!=expected.format || surface.browser_frame_surface_stamp().as_ref()!=Some(&expected.surface)
                || !std::ptr::eq(surface.core(),current.as_ref()) { return Err(Cancel::Surface) }
            if !host.s.shared.producers.browser_capture_sources_current(&expected.sources) { return Err(Cancel::Source) }
            Ok(true)
        });
        // On refusal, return without dropping caller callback under Host RefMut.
        let mut held=host.try_borrow_mut().map_err(|_|BrowserSurfaceCaptureRequestFailure::HostBorrowed)?;
        let refusal=if held.s.pending_browser_surface_capture.is_some() { Some(BrowserSurfaceCaptureRequestFailure::AlreadyArmed) }
            else if held.s.surface.is_none() || held.s.shared.render_core.is_none() || held.s.close_requested { Some(BrowserSurfaceCaptureRequestFailure::Disposed) }
            else if held.s.hidden { Some(BrowserSurfaceCaptureRequestFailure::Hidden) } else { None };
        if let Some(refusal)=refusal { drop(held);return Err(refusal) }
        held.s.pending_browser_surface_capture=Some(BrowserSurfaceCaptureRequest { identity:Arc::new(()),budget,callback:callback.take(),probe });
        // A late arm cannot use an already prepared unconfigured frame.
        held.invalidate_browser_frame();
        if let Some(window)=&held.s.window { window.request_redraw(); }
        Ok(())
    }
}
impl<State,Logic,V,T> Host<State,Logic,V,T>
where State:'static,Logic:FnMut(&State)->V+'static,V:RootView<State>,T:HostTree<State>,
{
    pub(crate) fn cancel_browser_surface_request(&mut self,error:Failure) {
        if let Some(request)=self.s.pending_browser_surface_capture.take() {
            self.s.browser_surface_capture_deliveries.push(request.delivery(Err(error)));
        }
    }
    pub fn browser_surface_capture_observation(&self)->Option<BrowserSurfaceCaptureDiagnostics> {
        self.s.last_browser_surface_capture_observation.clone()
    }
}
