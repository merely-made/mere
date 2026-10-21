// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Opt-in monotonic entry observations; absent from ordinary builds.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrowserWorkCounts {
    pub relayout: u64,
    pub accessibility: u64,
    pub preparation: u64,
    pub saturated: bool,
}
pub(crate) enum WorkEntry { Relayout, Accessibility, Preparation }
impl BrowserWorkCounts {
    fn enter(&mut self, entry:WorkEntry) {
        let count = match entry {
            WorkEntry::Relayout => &mut self.relayout,
            WorkEntry::Accessibility => &mut self.accessibility,
            WorkEntry::Preparation => &mut self.preparation,
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
