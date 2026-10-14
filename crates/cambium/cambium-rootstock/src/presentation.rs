// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Presentation selection and the lifetime of context-bound resources.
//! Application state and content revisions are independent of this lifetime.

/// Browser presentation policy. Explicit choices never attempt another route.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PresentationChoice {
    #[default]
    Auto,
    WebGpu,
    WebGl2,
}

/// The graphics route. A wgpu adapter reports its actual platform separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationBackend {
    Wgpu,
    WebGl2,
}

/// Identity of a graphics lifetime within its owning host/context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContextGeneration(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationState {
    Ready,
    Lost,
    Restoring,
    Disposed,
}

/// Loss invalidates completions immediately; generation advances only after
/// rebuilt resources are ready. The owner also scopes every resource to itself.
#[derive(Debug)]
pub struct PresentationLifetime {
    generation: ContextGeneration,
    state: PresentationState,
}

impl PresentationLifetime {
    pub fn new(generation: ContextGeneration) -> Self {
        Self {
            generation,
            state: PresentationState::Ready,
        }
    }

    pub fn generation(&self) -> ContextGeneration {
        self.generation
    }

    pub fn state(&self) -> PresentationState {
        self.state
    }

    /// Idempotent for a lost context; a new loss cancels an unfinished rebuild.
    pub fn lose(&mut self) {
        if self.state != PresentationState::Disposed {
            self.state = PresentationState::Lost;
        }
    }

    pub fn begin_restore(&mut self) -> Result<(), String> {
        if self.state != PresentationState::Lost {
            return Err(format!("cannot begin restoration while {:?}", self.state));
        }
        self.state = PresentationState::Restoring;
        Ok(())
    }

    pub fn finish_restore(&mut self) -> Result<ContextGeneration, String> {
        if self.state != PresentationState::Restoring {
            return Err(format!("cannot finish restoration while {:?}", self.state));
        }
        let Some(next) = self.generation.0.checked_add(1) else {
            self.state = PresentationState::Lost;
            return Err("graphics context generation exhausted".into());
        };
        self.generation = ContextGeneration(next);
        self.state = PresentationState::Ready;
        Ok(self.generation)
    }

    pub fn accepts(&self, generation: ContextGeneration) -> bool {
        self.state == PresentationState::Ready && generation == self.generation
    }

    pub fn dispose(&mut self) {
        self.state = PresentationState::Disposed;
    }
}

/// Ordered boot attempts, rather than a claim that an available API can boot.
pub const fn backend_attempts(choice: PresentationChoice) -> &'static [PresentationBackend] {
    match choice {
        PresentationChoice::Auto => &[PresentationBackend::Wgpu, PresentationBackend::WebGl2],
        PresentationChoice::WebGpu => &[PresentationBackend::Wgpu],
        PresentationChoice::WebGl2 => &[PresentationBackend::WebGl2],
    }
}

mod frame;
pub use frame::*;
mod prepare;
pub(crate) mod wgpu;
