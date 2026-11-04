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
