// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! A real geometric indicator, independent of label text and font coverage.
use crate::{El,el};
pub type ChoiceIndicator<State,Action> = El<El<(),State,Action>,State,Action>;
pub fn choice_indicator<State:'static,Action:'static>(checked:bool,radio:bool)->ChoiceIndicator<State,Action> {
    el("span",el("span",()).attr("class","choice-mark"))
        .attr("class",if radio {"choice-indicator choice-radio"} else {"choice-indicator"})
        .attr("data-checked",if checked {"true"} else {"false"})
        .attr("aria-hidden","true")
}
