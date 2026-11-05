// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// A per-frame hook: return `true` to keep frames coming.
pub type FrameHook<State, Logic, V, T = Runner<State, Logic, V>> =
    Box<dyn FnMut(&mut AppCtx<'_, State, Logic, V, T>) -> bool>;

/// A plain application hook over the standard context.
pub type AppHook<State, Logic, V, T = Runner<State, Logic, V>> =
    Box<dyn FnMut(&mut AppCtx<'_, State, Logic, V, T>)>;

/// A request from the OS or an application command to close the root window.
pub type CloseRequestHook<State, Logic, V, T = Runner<State, Logic, V>> =
    Box<dyn FnMut(&mut AppCtx<'_, State, Logic, V, T>, CloseRequest) -> CloseDisposition>;

/// The text-seam query: which text field has focus, if any.
pub type FocusedTextHook<State, Logic, V> =
    Box<dyn Fn(&Runner<State, Logic, V>) -> Option<FocusedTextSlot<State>>>;

/// A pre-dispatch keyboard intercept: return `true` to consume the event.
pub type KeyInterceptHook<State, Logic, V> =
    Box<dyn FnMut(&mut Runner<State, Logic, V>, &KeyPress) -> bool>;

/// The application's hooks. Plain closures, owned state lives in their
/// captured environment (an `Rc<RefCell<...>>` for anything shared).
pub struct HostHooks<State: 'static, Logic, V, T = Runner<State, Logic, V>>
where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    /// Runs at the top of every frame, before layout and paint: drive
    /// animations, sync leaves, poll backends. Return `true` to keep frames
    /// coming (an animation is live).
    pub frame: FrameHook<State, Logic, V, T>,
    /// The tail of every input dispatch: persist, push state to backends,
    /// drain window-chrome requests.
    pub after_dispatch: AppHook<State, Logic, V, T>,
    /// Runs after a redraw attempt and accessibility synchronization. Inspect
    /// `AppCtx::presentation` before doing work that requires a new presented
    /// frame. Failed acquisition still gives asynchronous readback and bounded
    /// presentation waits a turn; it must not advance a presentation scenario.
    pub after_frame: AppHook<State, Logic, V, T>,
    /// Freeze the facts described by this prepared picture. The returned
    /// callback publishes only after that picture reaches the presentation
    /// queue; its guard can revoke a replaced source before presentation.
    pub prepare_presentation: Box<
        dyn FnMut(
            &AppCtx<'_, State, Logic, V, T>,
            PresentedFrame,
        ) -> Option<crate::PreparedPresentationObserver>,
    >,
    /// Runs after an application-owned worker wakes the host, before the
    /// redraw it requested. Drain the application's own channel here.
    pub after_wake: AppHook<State, Logic, V, T>,
    /// Decides what a native or application-requested close means. The host
    /// owns the resulting visibility or exit; the application owns policy.
    pub close_request: CloseRequestHook<State, Logic, V, T>,
    /// The text seam: which text field has focus, if any.
    pub focused_text: Box<dyn Fn(&T) -> Option<FocusedTextSlot<State>>>,
    /// Pre-dispatch keyboard intercept (Escape policy and friends). Return
    /// `true` to consume the event; `after_dispatch` runs either way when
    /// consumed.
    pub key_intercept: Box<dyn FnMut(&mut T, &KeyPress) -> bool>,
}
