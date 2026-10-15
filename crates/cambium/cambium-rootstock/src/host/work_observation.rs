// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Opt-in monotonic entry observations; absent from ordinary builds.
use super::*;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserSurfaceControlPoint { BeforeAcquire, AfterCompose }
#[cfg(target_arch = "wasm32")]
pub type BrowserSurfaceControlFn = Box<dyn for<'a> FnMut(
    BrowserSurfaceControlPoint, &genet_render_host::BrowserFrameCommands<'a>,
) -> Result<(),genet_render_host::RasterizeFailure>>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrowserWorkCounts {
    pub relayout: u64,
    pub accessibility: u64,
    pub preparation: u64,
    pub surface_submission: u64,
    pub surface_acquired: u64,
    pub surface_present_call: u64,
    pub surface_skipped: u64,
    pub surface_completion_accepted: u64,
    pub surface_completion_refused: u64,
    pub publication: u64,
    pub saturated: bool,
}
pub(crate) enum WorkEntry {
    Relayout, Accessibility, Preparation, SurfaceSubmission, SurfaceAcquired,
    SurfacePresentCall, SurfaceSkipped, SurfaceCompletionAccepted,
    SurfaceCompletionRefused, Publication,
}
impl BrowserWorkCounts {
    fn enter(&mut self, entry:WorkEntry) {
        let count = match entry {
            WorkEntry::Relayout => &mut self.relayout,
            WorkEntry::Accessibility => &mut self.accessibility,
            WorkEntry::Preparation => &mut self.preparation,
            WorkEntry::SurfaceSubmission => &mut self.surface_submission,
            WorkEntry::SurfaceAcquired => &mut self.surface_acquired,
            WorkEntry::SurfacePresentCall => &mut self.surface_present_call,
            WorkEntry::SurfaceSkipped => &mut self.surface_skipped,
            WorkEntry::SurfaceCompletionAccepted => &mut self.surface_completion_accepted,
            WorkEntry::SurfaceCompletionRefused => &mut self.surface_completion_refused,
            WorkEntry::Publication => &mut self.publication,
        };
        if let Some(next) = count.checked_add(1) { *count = next; }
        else { self.saturated = true; }
    }
}
impl<State, Logic, V, T> Host<State, Logic, V, T>
where State:'static, Logic:FnMut(&State)->V, V:RootView<State>, T:crate::HostTree<State>,
{
    /// Enable once; repeated observation cannot reset a work baseline.
    pub fn enable_browser_work_observation(&mut self) {
        self.s.browser_work.get_or_insert_with(Default::default);
    }
    pub fn browser_work_counts(&self) -> Option<BrowserWorkCounts> { self.s.browser_work }
    pub(crate) fn record_browser_work(&mut self, entry:WorkEntry) {
        if let Some(counts) = &mut self.s.browser_work { counts.enter(entry); }
    }
}

#[cfg(target_arch = "wasm32")]
impl<State, Logic, V, T> Host<State, Logic, V, T>
where State:'static, Logic:FnMut(&State)->V + 'static, V:RootView<State>, T:crate::HostTree<State>,
{
    /// Opt-in synchronous final-surface diagnostic. It runs under the actual
    /// phase-2 scopes before actual acquisition and after composition, before copy/present. This callback
    /// must not retain commands or reenter a borrowed Host. Changing it while
    /// private work is pending would invalidate that work's control identity.
    /// Drop the swapped-out callback only after releasing the Host borrow.
    pub fn set_browser_surface_control(
        &mut self,
        control:&mut Option<BrowserSurfaceControlFn>,
    ) -> Result<(),genet_render_host::RasterizeFailure> {
        if self.has_pending_browser_frame() {
            return Err(crate::producer::staged_producer_refusal("surface diagnostic cannot change while a browser frame is pending"));
        }
        // Swap transfers the previous callback to the caller without dropping
        // either callback while Host RefMut is held. On refusal the caller
        // still owns its uninstalled callback.
        std::mem::swap(&mut self.s.browser_surface_control,control);
        self.invalidate_browser_frame();
        Ok(())
    }
}
