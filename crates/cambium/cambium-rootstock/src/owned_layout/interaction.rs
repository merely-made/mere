// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use genet_livery::StatePseudoClass;

/// A conservative absence proof, not a selector parser. In comment-free,
/// unescaped selector source a state pseudo must contain its literal name.
/// Anything ambiguous keeps the full cascade path, including escapes inside
/// identifiers or comments between the colon and the name. Attribute strings
/// may produce false positives; those only cost an unnecessary style pass.
pub(super) struct Dependencies {
    hover: bool,
    focus: bool,
}

impl Dependencies {
    pub(super) fn new(styles: &StyleSet) -> Self {
        let mut result = Self {
            hover: false,
            focus: false,
        };
        for rule in styles.rules() {
            let selector = rule.selector_text().to_ascii_lowercase();
            let ambiguous = selector.contains('\\') || selector.contains("/*");
            result.hover |= ambiguous || selector.contains(":hover");
            result.focus |= ambiguous || selector.contains(":focus");
        }
        result
    }
}

impl OwnedLayout {
    /// Report whether the interaction changed the rendered result. The state
    /// still advances when the cascade is equal, so a later move or rebuild
    /// starts from the correct hovered/focused node.
    pub(crate) fn set_interaction<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        hovered: Option<NodeId>,
        focused: Option<NodeId>,
    ) -> bool {
        let mut changed = false;
        let mut needs_styles = false;
        for (previous, next, pseudo, dependent) in [
            (
                self.hovered,
                hovered,
                StatePseudoClass::Hover,
                self.interaction_dependencies.hover,
            ),
            (
                self.focused,
                focused,
                StatePseudoClass::Focus,
                self.interaction_dependencies.focus,
            ),
        ] {
            if previous != next {
                needs_styles |= dependent;
                if let Some(old) = previous {
                    changed |= self.interactions.set(old, pseudo, false);
                }
                if let Some(next) = next {
                    changed |= self.interactions.set(next, pseudo, true);
                }
            }
        }
        self.hovered = hovered;
        self.focused = focused;
        self.style_resolve_us = 0;
        self.layout_with_text_us = 0;
        self.content_extent_us = 0;
        if !changed || !needs_styles {
            return false;
        }

        let phase = crate::Instant::now();
        let styles = resolve_styles(dom, &self.style_set, &self.device, &self.interactions);
        self.style_resolve_us = elapsed_us(phase.elapsed());
        if styles == self.resolved_styles {
            // Keep fragments, shaped text, scroll offsets and their generation.
            return false;
        }
        self.layout_resolved(dom, styles);
        true
    }
}

#[cfg(test)]
mod tests;
