/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

impl GraphLabelPlacement {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Above => "above",
            Self::Below => "below",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// The very same box is used for collision tests and absolute layout.
    pub(super) fn bounds(
        self,
        x: f32,
        y: f32,
        clearance: f32,
        canvas: Size,
        text_width: f32,
    ) -> LabelBox {
        let width = text_width.min(canvas.width).max(0.0);
        let height = 14.0_f32.min(canvas.height).max(0.0);
        let centred_left = (x - width * 0.5).clamp(0.0, (canvas.width - width).max(0.0));
        let centred_top = (y - height * 0.5).clamp(0.0, (canvas.height - height).max(0.0));
        match self {
            Self::Above => LabelBox {
                left: centred_left,
                top: (y - clearance - height).max(0.0),
                width,
                height,
            },
            Self::Below => LabelBox {
                left: centred_left,
                top: (y + clearance).min((canvas.height - height).max(0.0)),
                width,
                height,
            },
            Self::Left => {
                let room = (x - clearance).clamp(0.0, width);
                LabelBox {
                    left: (x - clearance - room).max(0.0),
                    top: centred_top,
                    width: room,
                    height,
                }
            },
            Self::Right => {
                let left = (x + clearance).min(canvas.width);
                LabelBox {
                    left,
                    top: centred_top,
                    width: (canvas.width - left).clamp(0.0, width),
                    height,
                }
            },
        }
    }
}

impl LabelBox {
    pub(super) fn overlaps(self, other: Self) -> bool {
        const GAP: f32 = 2.0;
        self.left < other.left + other.width + GAP
            && other.left < self.left + self.width + GAP
            && self.top < other.top + other.height + GAP
            && other.top < self.top + self.height + GAP
    }
}

impl ResolvedGraphLabel {
    pub(super) fn style(self) -> String {
        let LabelBox {
            left,
            top,
            width,
            height,
        } = self.bounds;
        let align = match self.placement {
            GraphLabelPlacement::Above | GraphLabelPlacement::Below => "center",
            GraphLabelPlacement::Left => "right",
            GraphLabelPlacement::Right => "left",
        };
        format!(
            "position:absolute;left:{left}px;top:{top}px;width:{width}px;height:{height}px;text-align:{align};overflow:hidden;text-overflow:ellipsis;"
        )
    }
}
