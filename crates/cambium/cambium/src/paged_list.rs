// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A paged list for shell menus and popups: one page of caller-rendered rows,
//! a range line, and Previous / More controls named for what is listed.
//! The host owns the items, row content, row actions and the page offset;
//! this module owns the paging arithmetic and the wording around it.

use crate::shell::{ShellControl, ShellSlot, shell_control};
use crate::{OptionalAction, PointerClick, el};

/// The visible slice of a list: `start..end` of `total`, with `start`
/// clamped to a page boundary so a stale offset never shows an empty page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PagedWindow {
    pub start: usize,
    pub end: usize,
    pub total: usize,
    pub page_size: usize,
}

impl PagedWindow {
    pub fn new(start: usize, page_size: usize, total: usize) -> Self {
        let page_size = page_size.max(1);
        let start = start.min(total.saturating_sub(1)) / page_size * page_size;
        Self {
            start,
            end: (start + page_size).min(total),
            total,
            page_size,
        }
    }

    pub fn has_previous(&self) -> bool {
        self.start > 0
    }

    pub fn has_more(&self) -> bool {
        self.end < self.total
    }

    pub fn is_paged(&self) -> bool {
        self.total > self.page_size
    }
}

/// One page of `items` as shell slots: `empty` as a status line when there are
/// none; otherwise the rows, and, past one page, `a–b of n` with `Previous
/// {noun}` and `More {noun}`. `set_start` receives the new item offset.
pub fn shell_paged_list<State, Action, OA, T>(
    noun: &str,
    empty: &str,
    items: &[T],
    start: usize,
    page_size: usize,
    mut row: impl FnMut(&T) -> ShellSlot<State, Action>,
    set_start: impl Fn(&mut State, usize) -> OA + Clone + 'static,
) -> Vec<ShellSlot<State, Action>>
where
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action> + 'static,
{
    if items.is_empty() {
        return vec![Box::new(el("p", empty.to_string()).attr("role", "status"))];
    }
    let window = PagedWindow::new(start, page_size, items.len());
    let mut slots: Vec<ShellSlot<State, Action>> =
        items[window.start..window.end].iter().map(&mut row).collect();
    if window.is_paged() {
        slots.push(Box::new(
            el(
                "p",
                format!("{}–{} of {}", window.start + 1, window.end, window.total),
            )
            .attr("class", "shell-paged-range"),
        ));
        for (label, enabled, target) in [
            (
                format!("Previous {noun}"),
                window.has_previous(),
                window.start.saturating_sub(window.page_size),
            ),
            (
                format!("More {noun}"),
                window.has_more(),
                window.start + window.page_size,
            ),
        ] {
            let set_start = set_start.clone();
            slots.push(shell_control(
                label.clone(),
                label,
                ShellControl {
                    disabled: !enabled,
                    ..Default::default()
                },
                move |state: &mut State, _: PointerClick| set_start(state, target),
            ));
        }
    }
    slots
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::shell_navigation_item;
    use crate::{DomHandle, GenetAppRunner};
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::LayoutDom;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn window_clamps_a_stale_offset_to_the_last_page() {
        assert_eq!(PagedWindow::new(0, 6, 0).end, 0);
        let last = PagedWindow::new(60, 6, 14);
        assert_eq!((last.start, last.end), (12, 14));
        assert!(last.has_previous() && !last.has_more());
        let mid = PagedWindow::new(7, 6, 14);
        assert_eq!((mid.start, mid.end), (6, 12));
    }

    fn html(dom: &DomHandle, root: genet_scripted_dom::NodeId) -> Vec<String> {
        let dom = dom.borrow();
        dom.dom_children(root).map(|n| dom.outer_html(n)).collect()
    }

    fn contains_in_order(html: &[String], expected: &[&str]) {
        assert_eq!(html.len(), expected.len(), "{html:#?}");
        for (html, expected) in html.iter().zip(expected) {
            assert!(html.contains(expected), "{html} lacks {expected}");
        }
    }

    macro_rules! paged {
        ($items:expr) => {{
            let items: Vec<u32> = $items;
            let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
            let runner = GenetAppRunner::<_, _, _, ()>::new(
                dom.clone(),
                move |start: &usize| {
                    el(
                        "div",
                        shell_paged_list(
                            "items",
                            "Nothing here",
                            &items,
                            *start,
                            2,
                            |n| {
                                shell_navigation_item(
                                    format!("Item {n}"),
                                    "kind",
                                    Default::default(),
                                    |_: &mut usize, _| (),
                                )
                            },
                            |start: &mut usize, to| *start = to,
                        ),
                    )
                },
                0usize,
            );
            (dom, runner)
        }};
    }

    #[test]
    fn empty_list_is_one_status_line() {
        let (dom, runner) = paged!(Vec::new());
        contains_in_order(&html(&dom, runner.root()), &["Nothing here"]);
    }

    #[test]
    fn controls_page_through_the_items_and_disable_at_the_ends() {
        let (dom, mut runner) = paged!(vec![1, 2, 3]);
        let root = runner.root();
        let first = html(&dom, root);
        contains_in_order(
            &first,
            &["kind: Item 1", "kind: Item 2", "1–2 of 3", "Previous items", "More items"],
        );
        assert!(first[3].contains("aria-disabled=\"true\""));
        let more = dom.borrow().dom_children(root).last().unwrap();
        runner.dispatch_click(more, PointerClick::at((0.0, 0.0)));
        assert_eq!(*runner.state(), 2);
        let second = html(&dom, root);
        contains_in_order(&second, &["kind: Item 3", "3–3 of 3", "Previous items", "More items"]);
        assert!(second[3].contains("aria-disabled=\"true\""));
    }

    #[test]
    fn one_page_has_no_paging_controls() {
        let (dom, runner) = paged!(vec![1, 2]);
        contains_in_order(&html(&dom, runner.root()), &["kind: Item 1", "kind: Item 2"]);
    }
}
