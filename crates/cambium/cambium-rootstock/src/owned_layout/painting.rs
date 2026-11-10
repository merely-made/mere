// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl OwnedLayout {
    pub(crate) fn emit_paint_list_with_leaves<D, F, G>(
        &mut self,
        dom: &D,
        viewport: DeviceIntSize,
        focused: Option<FocusedTextPaint>,
        mut commands: F,
        mut fragment: G,
    ) -> LiveryPaintList
    where
        D: LayoutDom<NodeId = NodeId>,
        F: FnMut(u64) -> Option<Vec<paint_list_api::PaintCmd>>,
        G: FnMut(u64) -> Option<u64>,
    {
        let text_paint = focused.map(|(node, caret, selection)| {
            let selection_color = self
                .selection_style(dom, node)
                .map(|(bg, _)| ColorF {
                    r: bg[0],
                    g: bg[1],
                    b: bg[2],
                    a: bg[3],
                })
                .unwrap_or(ColorF {
                    r: 0.20,
                    g: 0.45,
                    b: 0.90,
                    a: 0.35,
                });
            let selection = selection
                .map(|(start, end)| self.selection_rects_at(dom, node, start, end))
                .unwrap_or_default()
                .into_iter()
                .map(|rect| text_rect_command(rect, selection_color))
                .collect::<Vec<_>>();
            let caret_color = self
                .caret_color(dom, node)
                .map(|rgba| ColorF {
                    r: rgba[0],
                    g: rgba[1],
                    b: rgba[2],
                    a: rgba[3],
                })
                .unwrap_or(ColorF {
                    r: 0.92,
                    g: 0.94,
                    b: 0.98,
                    a: 1.0,
                });
            let caret = self.caret_rect_at(dom, node, caret.byte).map(|mut rect| {
                rect.width = 2.0;
                text_rect_command(rect, caret_color)
            });
            (dom.opaque_id(node), selection, caret)
        });
        let mut list = emit_paint_list_with_text_system_scrolled_with_images(
            dom,
            &self.styles,
            &self.fragments,
            viewport,
            self.generation,
            &mut self.text,
            &self.element_scroll,
            &HostImageLookup {
                bytes: &self.images,
                vectors: self.vectors.as_ref(),
            },
        );
        // Slots were recorded against the un-translated list. Fill them before
        // the document viewport transform is added so their indices and their
        // CSS paint context stay aligned.
        list.splice_host_leaf_slots(&mut commands, &mut fragment);
        // These commands belong to the field, not to a window-wide overlay.
        // The recorded slots retain its CSS clipping, opacity and transforms;
        // raw coordinates avoid subtracting any scroll a second time.
        list.splice_text_paint_slots(|owner, phase| {
            let (focused_owner, selection, caret) = text_paint.as_ref()?;
            if owner != *focused_owner {
                return None;
            }
            Some(match phase {
                // Keep the existing translucent selection over the field's
                // glyphs and syntax backgrounds, followed by its caret. Both
                // stay below DOM surfaces painted after the field.
                TextPaintPhase::BeforeContent => Vec::new(),
                TextPaintPhase::AfterContent => selection
                    .iter()
                    .cloned()
                    .chain(caret.iter().cloned())
                    .collect(),
            })
        });
        list.scrolled(-self.viewport_scroll.0, -self.viewport_scroll.1)
    }

    pub(crate) fn append_scrollbars<D: LayoutDom<NodeId = NodeId>>(
        &self,
        _dom: &D,
        _list: &mut LiveryPaintList,
        _alpha: &impl Fn(ScrollTarget) -> f32,
    ) {
    }
}
