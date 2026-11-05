// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>,
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// Hand a file request a view filed in the latest dispatch to the
    /// platform's chooser. With no chooser, answer at once with nothing chosen.
    pub(super) fn open_requested_file(&mut self) {
        let Some(request) = self
            .s
            .runner
            .as_mut()
            .and_then(|runner| runner.take_file_request())
        else {
            return;
        };
        let answer =
            crate::FileAnswer::new(request.node, self.s.file_answers.clone(), self.wake.clone());
        match self.s.files.as_mut() {
            Some(files) => files.open(&request, answer),
            None => answer.send(cambium::FileEvent::default()),
        }
    }

    /// Dispatch every file answer waiting, each to the element that asked, as
    /// an input dispatch. Returns whether there were any.
    pub fn deliver_files(&mut self) -> bool {
        let answers = match self.s.file_answers.lock() {
            Ok(mut answers) => std::mem::take(&mut *answers),
            Err(_) => return false,
        };
        if answers.is_empty() {
            return false;
        }
        if let Some(runner) = self.s.runner.as_mut() {
            for (node, event) in answers {
                runner.dispatch_file(node, event);
            }
        }
        self.after_dispatch();
        true
    }
}

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>,
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// The callback handed to the AccessKit adapter. It fires on the adapter's
    /// own thread when a screen reader acts on an otherwise idle application,
    /// and signals the event source with its own flag. The next idle policy
    /// requests an action-service turn without labelling it application work.
    pub fn a11y_waker(&self) -> impl Fn() + Send + Sync + 'static {
        let notify = self.wake.notification(self.s.a11y_wake.clone());
        move || notify()
    }

    /// Consume an accessibility-only notification without preparing a frame.
    /// Event sources service the last tree's actions, including hidden-window
    /// refusals, before considering a new picture or native completion.
    pub fn take_a11y_wake(&self) -> bool {
        self.s.a11y_wake.swap(false, Ordering::AcqRel)
    }

    /// Decide what the event loop does on an idle turn, consuming the
    /// accessibility wake flag if one is set. Factored out of `about_to_wait`
    /// so the wake path is assertable: `Wait` really means nothing is pending,
    /// and a raised wake really becomes a repaint rather than being swallowed.
    pub fn idle_policy(&mut self, now: crate::Instant) -> IdlePolicy {
        if self.take_a11y_wake() {
            return IdlePolicy::A11yWake;
        }
        // Overlay scrollbars mid-hold/mid-fade keep frames coming until
        // hidden; `Wait` never wakes without an event, so ask for a timed wake.
        if self.s.scrollbar_fade.any_visible(now) {
            return IdlePolicy::Animate(std::time::Duration::from_millis(33));
        }
        IdlePolicy::Wait
    }
}

impl<State, Logic, V, T> HostHooks<State, Logic, V, T>
where
    State: 'static,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    /// Hooks that do nothing: the starting point for a test that only cares
    /// about routing, and what a multi-window host's windows hold between
    /// turns while the application's own hooks are lent to another window.
    pub fn inert() -> Self {
        Self {
            frame: Box::new(|_| false),
            after_dispatch: Box::new(|_| {}),
            after_frame: Box::new(|_| {}),
            prepare_presentation: Box::new(|_, _| None),
            after_wake: Box::new(|_| {}),
            close_request: Box::new(|_, _| CloseDisposition::Exit),
            focused_text: Box::new(|_| None),
            key_intercept: Box::new(|_, _| false),
        }
    }
}
