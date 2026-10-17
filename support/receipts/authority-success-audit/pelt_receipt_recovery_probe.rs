
#![allow(dead_code,unused_variables)]
use std::cell::Cell;
use std::time::{Instant,Duration};
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum WorkspaceReceipt {
    /// A Gemtext link replaces only its own tile through capability routing.
    Mixed,
    /// An unavailable external engine visibly falls back to the owned Livery lane.
    Fallback,
    /// P6 host chrome controls a focused tile while the P5 mixed workspace remains live.
    Chrome,
    /// P6's document-lane loading and failed-navigation projection, without a
    /// native surface dependency.
    LoadingError,
    /// P6's Pelt-owned appearance drawer. This named receipt uses in-memory
    /// storage, while callers may supply a durable Pelt store.
    Appearance,
    /// P6's retained Chrome/Frisket AccessKit tree and typed action routing.
    Accessibility,
    /// P6's address TextInput SetValue route through the installed platform
    /// bridge, including retained loading and error documents.
    AccessibilityAddress,
    /// P7's one-tree Livery child semantics, namespaced and routed through the
    /// same Pelt input path as its content aperture.
    AccessibilityChildren,
    /// P7's native Livery text-control SetValue route, including retained
    /// value projection and ordinary form submission.
    AccessibilityEdit,
    /// P7's nested-scroll Livery ScrollIntoView route through Pelt's
    /// composite accessibility namespace.
    AccessibilityScroll,
    /// P7's clip-aware nested Livery Click route after an explicit reveal.
    AccessibilityClick,
    /// P7's nested Livery editor input and physical text-selection route.
    AccessibilityInput,
    /// P6's compact Chrome layout at a small logical viewport.
    NarrowChrome,
    /// P6's actual high-DPI Chrome input and capture alignment.
    ChromeDpi,
    /// Reader extracts a focused tile's already-held source response in the
    /// shared workspace, then releases it back to its original Livery route.
    Reader,
    /// Reader's already-presented partial link tree is namespaced under each
    /// Pelt content aperture. Focus is virtual until a later host source
    /// handoff can supply destination bodies for Reader navigation.
    ReaderAccessibility,
    /// A developer-only Tabard palette preview over the retained Pelt Chrome.
    /// This does not persist a Pelt appearance preference or recolor documents.
    TabardPreview,
    /// A developer-only Tabard palette preview over Reader's existing
    /// host-theme seam. This does not alter Fleece or persist a Pelt theme.
    TabardReaderPreview,
}

impl WorkspaceReceipt {
    pub fn id(self) -> &'static str {
        match self {
            Self::Mixed => "mixed",
            Self::Fallback => "fallback",
            Self::Chrome => "chrome",
            Self::LoadingError => "loading-error",
            Self::Appearance => "appearance",
            Self::Accessibility => "accessibility",
            Self::AccessibilityAddress => "accessibility-address",
            Self::AccessibilityChildren => "accessibility-children",
            Self::AccessibilityEdit => "accessibility-edit",
            Self::AccessibilityScroll => "accessibility-scroll",
            Self::AccessibilityClick => "accessibility-click",
            Self::AccessibilityInput => "accessibility-input",
            Self::NarrowChrome => "narrow-chrome",
            Self::ChromeDpi => "chrome-dpi",
            Self::Reader => "reader",
            Self::ReaderAccessibility => "reader-accessibility",
            Self::TabardPreview => "tabard-preview",
            Self::TabardReaderPreview => "tabard-reader-preview",
        }
    }

}
fn primary_startup_retry(presented: u32, revealed: Instant, now: Instant) -> bool {
    presented == 0 && now.saturating_duration_since(revealed) < Duration::from_secs(10)
}

struct Config{workspace_receipt:Option<WorkspaceReceipt>,tearout_receipt:bool,tearout_cancellation_receipt:bool,workspace_receipt_stage_timeout:Duration}
#[derive(Default)]struct Stats{frames:u32,imports:u32,waits:u32,compositions:u32}
struct NativeSurfaces;impl NativeSurfaces{fn stats(&self)->Stats{Stats::default()}}
struct Acquirer;impl Acquirer{fn acquire(&self)->Option<()>{None}}
struct ActiveEventLoop{exits:Cell<u32>}impl ActiveEventLoop{fn exit(&self){self.exits.set(self.exits.get()+1)}}
struct WorkspaceApp{config:Config,receipt_complete:bool,workspace_receipt_stage_started:Instant,width:u32,height:u32,native_surfaces:NativeSurfaces,mixed_content_ready_frame:Option<u32>,mixed_verified_sizes:Vec<(u32,u32)>,receipt_error:Option<String>,requested_redraws:Cell<u32>}
impl WorkspaceApp {
fn request_redraw(&self){self.requested_redraws.set(self.requested_redraws.get()+1)}
fn tearout_receipt_timeout_error(&self)->Option<String>{None}
    pub fn workspace_receipt_timeout_error(&self) -> Option<String> {
        if matches!(
            self.config.workspace_receipt,
            Some(
                WorkspaceReceipt::LoadingError
                    | WorkspaceReceipt::Appearance
                    | WorkspaceReceipt::Accessibility
                    | WorkspaceReceipt::AccessibilityAddress
                    | WorkspaceReceipt::AccessibilityChildren
                    | WorkspaceReceipt::AccessibilityEdit
                    | WorkspaceReceipt::AccessibilityScroll
                    | WorkspaceReceipt::AccessibilityClick
                    | WorkspaceReceipt::AccessibilityInput
                    | WorkspaceReceipt::NarrowChrome
                    | WorkspaceReceipt::ChromeDpi
                    | WorkspaceReceipt::Reader
                    | WorkspaceReceipt::ReaderAccessibility
                    | WorkspaceReceipt::TabardPreview
                    | WorkspaceReceipt::TabardReaderPreview
            )
        ) && !self.receipt_complete
            && self.workspace_receipt_stage_started.elapsed()
                >= self.config.workspace_receipt_stage_timeout
        {
            let receipt = self
                .config
                .workspace_receipt
                .expect("the matching receipt was present above");
            return Some(format!(
                "{} workspace receipt timed out after {}s at {}x{}",
                receipt.id(),
                self.config.workspace_receipt_stage_timeout.as_secs_f32(),
                self.width,
                self.height
            ));
        }
        if !matches!(
            self.config.workspace_receipt,
            Some(WorkspaceReceipt::Mixed | WorkspaceReceipt::Chrome)
        ) || self.receipt_complete
            || self.workspace_receipt_stage_started.elapsed()
                < self.config.workspace_receipt_stage_timeout
        {
            return None;
        }
        #[cfg(target_os = "windows")]
        let progress = {
            let stats = self.native_surfaces.stats();
            format!(
                "frames={} imports={} waits={} compositions={} content_ready_frame={:?} verified_sizes={:?}",
                stats.frames,
                stats.imports,
                stats.waits,
                stats.compositions,
                self.mixed_content_ready_frame,
                self.mixed_verified_sizes
            )
        };
        #[cfg(not(target_os = "windows"))]
        let progress = "native surface unavailable on this platform".to_owned();
        Some(format!(
            "native workspace receipt timed out after {}s at {}x{} ({progress})",
            self.config.workspace_receipt_stage_timeout.as_secs_f32(),
            self.width,
            self.height
        ))
    }


fn attempt_capture_acquire(&mut self,event_loop:&ActiveEventLoop){let host=Acquirer;
        let Some(swap) = host.acquire() else {
            if let Some(error) = self.workspace_receipt_timeout_error() {
                self.receipt_error = Some(error);
                event_loop.exit();
                return;
            }
            if let Some(error) = self.tearout_receipt_timeout_error() {
                self.receipt_error = Some(error);
                event_loop.exit();
                return;
            }
            // An outdated or lost swapchain is deliberately skipped by the
            // shared host. A named receipt must drive one recovery frame,
            // otherwise a geometry change can leave its native surface
            // waiting without another redraw.
            if (self.config.workspace_receipt.is_some()
                || self.config.tearout_receipt
                || self.config.tearout_cancellation_receipt)
                && !self.receipt_complete
            {
                self.request_redraw();
            }
            return;
        };
drop(swap);
}}
fn app(receipt:WorkspaceReceipt,complete:bool,expired:bool)->WorkspaceApp{WorkspaceApp{config:Config{workspace_receipt:Some(receipt),tearout_receipt:false,tearout_cancellation_receipt:false,workspace_receipt_stage_timeout:Duration::from_secs(30)},receipt_complete:complete,workspace_receipt_stage_started:if expired {Instant::now()-Duration::from_secs(60)}else{Instant::now()},width:960,height:640,native_surfaces:NativeSurfaces,mixed_content_ready_frame:None,mixed_verified_sizes:vec![],receipt_error:None,requested_redraws:Cell::new(0)}}
#[test]fn semantic_completion_disables_error_and_recovery_while_capture_is_unavailable(){let mut a=app(WorkspaceReceipt::Appearance,true,true);assert!(a.workspace_receipt_timeout_error().is_none());let e=ActiveEventLoop{exits:Cell::new(0)};a.attempt_capture_acquire(&e);assert_eq!(a.requested_redraws.get(),0);assert!(a.receipt_error.is_none());assert_eq!(e.exits.get(),0);assert!(!primary_startup_retry(1,a.workspace_receipt_stage_started,Instant::now()));}
#[test]fn pending_semantics_get_a_recovery_redraw_control(){let mut a=app(WorkspaceReceipt::Appearance,false,false);let e=ActiveEventLoop{exits:Cell::new(0)};a.attempt_capture_acquire(&e);assert_eq!(a.requested_redraws.get(),1);assert!(a.receipt_error.is_none());}
#[test]fn pending_semantics_expiry_is_an_error_control(){let mut a=app(WorkspaceReceipt::Appearance,false,true);let e=ActiveEventLoop{exits:Cell::new(0)};a.attempt_capture_acquire(&e);assert!(a.receipt_error.as_deref().unwrap().contains("appearance workspace receipt timed out"));assert_eq!(e.exits.get(),1);assert_eq!(a.requested_redraws.get(),0);}
#[test]fn completed_mixed_semantics_also_disable_capture_recovery(){let mut a=app(WorkspaceReceipt::Mixed,true,true);let e=ActiveEventLoop{exits:Cell::new(0)};a.attempt_capture_acquire(&e);assert!(a.receipt_error.is_none());assert_eq!(a.requested_redraws.get(),0);assert_eq!(e.exits.get(),0);}
#[test]fn first_presentation_only_startup_retry_control(){let now=Instant::now();assert!(primary_startup_retry(0,now,now));assert!(!primary_startup_retry(1,now,now));}
