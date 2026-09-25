/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A read-only, source-borrowing projection for collapsed source regions.
//!
//! [`fold_projection`] borrows the caller's source and validates a byte-length
//! witness before it derives DOM children. It deliberately has no [`TextInput`]
//! dependency: the caller remains the authority for source storage and for any
//! later editable coordinate map.
//!
//! [`FoldProjection::rows`] lays the same projection out one row per visible
//! line, each with a gutter cell the host fills, such as fold controls beside
//! the lines they fold.

use std::cmp::Reverse;
use std::ops::Range;

use crate::{FieldChild, Keyed, StyleRange, el};

/// The CSS class carried by a collapsed-region marker.
///
/// Hosts theme this plain class as appropriate for their source view.
pub const FOLD_MARKER_CLASS: &str = "fold-marker";

/// The CSS class of one row of [`FoldProjection::rows`].
pub const FOLD_ROW_CLASS: &str = "fold-row";

/// The CSS class of a row's gutter cell, which the host fills.
pub const FOLD_GUTTER_CLASS: &str = "fold-gutter";

/// The CSS class of a row's visible source text.
pub const FOLD_LINE_CLASS: &str = "fold-line";

/// Structural CSS for [`FoldProjection::rows`]: the gutter beside each line,
/// and the line wrapping within its own column. Hosts set the gutter's width.
pub const FOLD_ROWS_CSS: &str = "\
    .fold-row { display: flex; align-items: flex-start; } \
    .fold-gutter { flex: none; width: 1.5em; } \
    .fold-line { flex: 1 1 0px; min-width: 0; white-space: pre-wrap; overflow-wrap: anywhere; }";

/// The accessible name on each collapsed-region marker.
///
/// The visible marker is `…`; this label makes its purpose available to
/// assistive technology without claiming that the hidden source is absent.
pub const FOLD_MARKER_ACCESSIBLE_LABEL: &str = "Folded content";

/// A source interval that is replaced by one collapsed-region marker.
pub type FoldRange = Range<usize>;

/// A validated part of a [`FoldProjection`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FoldProjectionSegment {
    /// Source bytes which remain visible. The range indexes [`FoldProjection::source`].
    Source(Range<usize>),
    /// Source bytes replaced by a marker. The range indexes [`FoldProjection::source`].
    FoldMarker(Range<usize>),
}

/// Why [`fold_projection`] could not derive a safe source projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FoldProjectionError {
    /// The caller's length witness did not describe the supplied source.
    SourceLengthMismatch { expected: usize, actual: usize },
    /// A fold was empty, out of bounds, or cut through a UTF-8 code point.
    InvalidFoldRange(FoldRange),
    /// Two folds overlap without either one containing the other.
    CrossingFoldRanges { first: FoldRange, second: FoldRange },
}

/// One visible line of a [`FoldProjection`].
///
/// A collapsed interval stays on the line it starts on, so the visible text
/// after it continues that line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoldProjectionLine {
    /// Where the line's first visible segment begins in the source.
    pub start: usize,
    /// The 1-based source line of `start`. Lines hidden inside a collapsed
    /// interval leave a gap in the numbering.
    pub number: usize,
    /// The line's visible source and markers, without its line break.
    pub segments: Vec<FoldProjectionSegment>,
    /// The source bytes of the break that ends the line, `\n` or `\r\n`;
    /// `None` on a last line with no break.
    pub line_break: Option<Range<usize>>,
}

impl FoldProjectionLine {
    fn starting(start: usize, number: usize) -> Self {
        Self {
            start,
            number,
            segments: Vec::new(),
            line_break: None,
        }
    }
}

/// A read-only rendering plan over source owned by the caller.
///
/// The plan only borrows `source`. Its [`field_children`](Self::field_children)
/// method turns the visible source intervals into text or styled spans and each
/// collapsed interval into a semantic marker. It never allocates, changes, or
/// synchronizes an editable source buffer.
#[derive(Clone, Debug)]
pub struct FoldProjection<'source> {
    source: &'source str,
    segments: Vec<FoldProjectionSegment>,
    styles: Vec<StyleRange>,
}

/// Validate and derive a read-only source projection with collapsed intervals.
///
/// `expected_source_len` is a source-length witness held by the caller. A stale
/// witness returns [`FoldProjectionError::SourceLengthMismatch`] rather than
/// projecting ranges against a changed source. Every fold must be nonempty,
/// in bounds, and begin and end at UTF-8 character boundaries. Nested folds are
/// normalized to their outermost range; crossing or partial-overlap folds are
/// rejected because neither has an unambiguous visible result.
///
/// `styles` use Cambium's existing [`StyleRange`] vocabulary. Valid styles are
/// clipped to each visible source interval. Malformed style ranges are ignored,
/// preserving read-only projection even when a highlighter result is stale.
pub fn fold_projection<'source>(
    source: &'source str,
    expected_source_len: usize,
    folds: &[FoldRange],
    styles: &[StyleRange],
) -> Result<FoldProjection<'source>, FoldProjectionError> {
    if expected_source_len != source.len() {
        return Err(FoldProjectionError::SourceLengthMismatch {
            expected: expected_source_len,
            actual: source.len(),
        });
    }

    let folds = normalize_folds(source, folds)?;
    let mut segments = Vec::with_capacity(folds.len() * 2 + 1);
    let mut cursor = 0;
    for fold in folds {
        if cursor < fold.start {
            segments.push(FoldProjectionSegment::Source(cursor..fold.start));
        }
        segments.push(FoldProjectionSegment::FoldMarker(fold.clone()));
        cursor = fold.end;
    }
    if cursor < source.len() {
        segments.push(FoldProjectionSegment::Source(cursor..source.len()));
    }

    Ok(FoldProjection {
        source,
        segments,
        styles: styles.to_vec(),
    })
}

impl<'source> FoldProjection<'source> {
    /// The source borrowed from the caller. It remains the projection's authority.
    pub fn source(&self) -> &'source str {
        self.source
    }

    /// The validated visible-source and collapsed-marker sequence.
    pub fn segments(&self) -> &[FoldProjectionSegment] {
        &self.segments
    }

    /// The visible text, with each collapsed source interval represented by `…`.
    pub fn visible_text(&self) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                FoldProjectionSegment::Source(range) => &self.source[range.clone()],
                FoldProjectionSegment::FoldMarker(_) => "…",
            })
            .collect()
    }

    /// Render visible source as text or styled spans and folds as semantic markers.
    ///
    /// This takes no editable model. `State` and `Action` belong to the caller's
    /// containing view, so a projection remains useful in any read-only surface.
    pub fn field_children<State: 'static, Action: 'static>(
        &self,
    ) -> Vec<FieldChild<State, Action>> {
        let mut children = Vec::new();
        for segment in &self.segments {
            self.emit_segment(&mut children, segment);
        }
        children
    }

    /// The visible source split into lines, in order.
    pub fn lines(&self) -> Vec<FoldProjectionLine> {
        let mut lines = Vec::new();
        let mut open: Option<FoldProjectionLine> = None;
        // Line breaks in the source before the segment being read.
        let mut breaks = 0;
        for segment in &self.segments {
            match segment {
                FoldProjectionSegment::FoldMarker(range) => {
                    open.get_or_insert_with(|| {
                        FoldProjectionLine::starting(range.start, breaks + 1)
                    })
                    .segments
                    .push(segment.clone());
                    breaks += self.source[range.clone()].matches('\n').count();
                },
                FoldProjectionSegment::Source(range) => {
                    let mut cursor = range.start;
                    for (offset, _) in self.source[range.clone()].match_indices('\n') {
                        let at = range.start + offset;
                        let line = open.get_or_insert_with(|| {
                            FoldProjectionLine::starting(cursor, breaks + 1)
                        });
                        let text_end = if at > cursor && self.source.as_bytes()[at - 1] == b'\r' {
                            at - 1
                        } else {
                            at
                        };
                        if cursor < text_end {
                            line.segments
                                .push(FoldProjectionSegment::Source(cursor..text_end));
                        }
                        line.line_break = Some(text_end..at + 1);
                        lines.extend(open.take());
                        breaks += 1;
                        cursor = at + 1;
                    }
                    if cursor < range.end {
                        open.get_or_insert_with(|| {
                            FoldProjectionLine::starting(cursor, breaks + 1)
                        })
                        .segments
                        .push(FoldProjectionSegment::Source(cursor..range.end));
                    }
                },
            }
        }
        lines.extend(open);
        lines
    }

    /// Render the projection one row per visible line, keyed by where the
    /// line starts in the source. Each row holds a gutter cell with the
    /// children `gutter` gives its line, then the line's text. The text keeps
    /// its line break, so an empty line still takes a line's height and a
    /// copied row carries its break.
    pub fn rows<State: 'static, Action: 'static>(
        &self,
        mut gutter: impl FnMut(&FoldProjectionLine) -> Vec<FieldChild<State, Action>>,
    ) -> Keyed<usize, FieldChild<State, Action>> {
        self.lines()
            .into_iter()
            .map(|line| {
                let mut text = Vec::new();
                for segment in &line.segments {
                    self.emit_segment(&mut text, segment);
                }
                if line.line_break.is_some() {
                    text.push(Box::new("\n".to_owned()) as FieldChild<State, Action>);
                }
                let row: FieldChild<State, Action> = Box::new(
                    el::<_, State, Action>(
                        "div",
                        (
                            el::<_, State, Action>("span", gutter(&line))
                                .attr("class", FOLD_GUTTER_CLASS),
                            el::<_, State, Action>("span", text).attr("class", FOLD_LINE_CLASS),
                        ),
                    )
                    .attr("class", FOLD_ROW_CLASS)
                    .attr("data-line", line.number.to_string()),
                );
                (line.start, row)
            })
            .collect()
    }

    fn emit_segment<State: 'static, Action: 'static>(
        &self,
        children: &mut Vec<FieldChild<State, Action>>,
        segment: &FoldProjectionSegment,
    ) {
        match segment {
            FoldProjectionSegment::Source(range) => {
                emit_source(children, self.source, range.clone(), &self.styles);
            },
            FoldProjectionSegment::FoldMarker(_) => children.push(Box::new(
                el::<_, State, Action>("span", "…")
                    .attr("class", FOLD_MARKER_CLASS)
                    .attr("role", "note")
                    .attr("aria-label", FOLD_MARKER_ACCESSIBLE_LABEL),
            )),
        }
    }
}

fn normalize_folds(
    source: &str,
    folds: &[FoldRange],
) -> Result<Vec<FoldRange>, FoldProjectionError> {
    let mut ordered = folds.to_vec();
    for fold in &ordered {
        if fold.start >= fold.end
            || fold.end > source.len()
            || !source.is_char_boundary(fold.start)
            || !source.is_char_boundary(fold.end)
        {
            return Err(FoldProjectionError::InvalidFoldRange(fold.clone()));
        }
    }
    ordered.sort_by_key(|range| (range.start, Reverse(range.end)));

    let mut normalized: Vec<FoldRange> = Vec::with_capacity(ordered.len());
    for fold in ordered {
        if let Some(outer) = normalized.last() {
            if fold.start < outer.end {
                if fold.end <= outer.end {
                    continue;
                }
                return Err(FoldProjectionError::CrossingFoldRanges {
                    first: outer.clone(),
                    second: fold,
                });
            }
        }
        normalized.push(fold);
    }
    Ok(normalized)
}

fn valid_style(source: &str, style: &StyleRange) -> bool {
    style.range.start < style.range.end
        && style.range.end <= source.len()
        && source.is_char_boundary(style.range.start)
        && source.is_char_boundary(style.range.end)
}

fn emit_source<State: 'static, Action: 'static>(
    children: &mut Vec<FieldChild<State, Action>>,
    source: &str,
    visible: Range<usize>,
    styles: &[StyleRange],
) {
    let mut boundaries = vec![visible.start, visible.end];
    for style in styles.iter().filter(|style| valid_style(source, style)) {
        let start = style.range.start.max(visible.start);
        let end = style.range.end.min(visible.end);
        if start < end {
            boundaries.push(start);
            boundaries.push(end);
        }
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut previous: Option<(Range<usize>, Option<&str>)> = None;
    for pair in boundaries.windows(2) {
        let range = pair[0]..pair[1];
        let class = styles
            .iter()
            .filter(|style| {
                valid_style(source, style)
                    && style.range.start <= range.start
                    && range.end <= style.range.end
            })
            .min_by_key(|style| style.range.end - style.range.start)
            .map(|style| style.class.as_str());
        match &mut previous {
            Some((previous_range, previous_class)) if *previous_class == class => {
                previous_range.end = range.end;
            },
            Some(_) => {
                emit_piece(
                    children,
                    source,
                    previous.take().expect("previous run exists"),
                );
                previous = Some((range, class));
            },
            None => previous = Some((range, class)),
        }
    }
    if let Some(run) = previous {
        emit_piece(children, source, run);
    }
}

fn emit_piece<State: 'static, Action: 'static>(
    children: &mut Vec<FieldChild<State, Action>>,
    source: &str,
    (range, class): (Range<usize>, Option<&str>),
) {
    let text = source[range].to_owned();
    match class {
        Some(class) => children.push(Box::new(
            el::<_, State, Action>("span", text).attr("class", class.to_owned()),
        )),
        None => children.push(Box::new(text)),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::{LayoutDom, LocalName, Namespace};

    use super::*;
    use crate::{AnyView, DomHandle, GenetAppRunner, GenetCtx, GenetElement, el};

    fn node_text(dom: &ScriptedDom, node: genet_scripted_dom::NodeId) -> String {
        if let Some(text) = dom.text(node) {
            return text.to_owned();
        }
        dom.dom_children(node)
            .map(|child| node_text(dom, child))
            .collect()
    }

    /// A line's visible text, with `…` for a marker.
    fn visible(projection: &FoldProjection<'_>, line: &FoldProjectionLine) -> String {
        line.segments
            .iter()
            .map(|segment| match segment {
                FoldProjectionSegment::Source(range) => &projection.source()[range.clone()],
                FoldProjectionSegment::FoldMarker(_) => "…",
            })
            .collect()
    }

    #[test]
    fn a_marker_stays_on_the_line_it_starts_and_lines_keep_source_numbers() {
        let source = "# A\nbody one\nbody two\n## B\nsee also: x";
        let section = 4..source.find("\n## B").unwrap();
        let inline = source.find("also").unwrap()..source.find(": x").unwrap();
        let projection =
            fold_projection(source, source.len(), &[section.clone(), inline], &[]).unwrap();
        let lines = projection.lines();
        let seen = lines
            .iter()
            .map(|line| (line.start, line.number, visible(&projection, line)))
            .collect::<Vec<_>>();
        assert_eq!(
            seen,
            vec![
                (0, 1, "# A".to_owned()),
                (4, 2, "…".to_owned()),
                (22, 4, "## B".to_owned()),
                (27, 5, "see …: x".to_owned()),
            ]
        );
        assert_eq!(lines[1].line_break, Some(section.end..section.end + 1));
        assert_eq!(lines[3].line_break, None);
    }

    #[test]
    fn crlf_is_one_break_and_an_empty_line_is_a_line() {
        let source = "a\r\n\r\nb\n";
        let projection = fold_projection(source, source.len(), &[], &[]).unwrap();
        let lines = projection.lines();
        assert_eq!(lines.len(), 3, "a trailing break opens no empty line");
        assert_eq!(
            (
                lines[0].start,
                lines[0].number,
                visible(&projection, &lines[0])
            ),
            (0, 1, "a".to_owned())
        );
        assert_eq!(lines[0].line_break, Some(1..3));
        assert_eq!((lines[1].start, lines[1].number), (3, 2));
        assert!(lines[1].segments.is_empty());
        assert_eq!(lines[1].line_break, Some(3..5));
        assert_eq!(
            (
                lines[2].start,
                lines[2].number,
                visible(&projection, &lines[2])
            ),
            (5, 3, "b".to_owned())
        );
        assert_eq!(lines[2].line_break, Some(6..7));
    }

    #[test]
    fn rows_put_a_gutter_cell_beside_each_lines_text() {
        #[derive(Clone)]
        struct Model;
        fn view(_: &Model) -> Box<dyn AnyView<Model, (), GenetCtx, GenetElement>> {
            let source = "# A\nbody\n\n## B\n";
            let styles = [StyleRange {
                range: 0..source.len(),
                class: "syntax".into(),
            }];
            let body = source.find("body").unwrap()..source.find("\n\n").unwrap();
            let projection = fold_projection(source, source.len(), &[body], &styles).unwrap();
            Box::new(el(
                "div",
                projection.rows(|line: &FoldProjectionLine| {
                    if line.number == 1 {
                        vec![Box::new(el::<_, Model, ()>("span", "▾")) as FieldChild<Model, ()>]
                    } else {
                        Vec::new()
                    }
                }),
            ))
        }

        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = GenetAppRunner::new(dom.clone(), view, Model);
        let dom = runner.dom();
        let dom = dom.borrow();
        let attribute = |node, name: &str| {
            dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
                .map(str::to_owned)
        };
        let rows: Vec<_> = dom.dom_children(runner.root()).collect();
        let cells = |row| dom.dom_children(row).collect::<Vec<_>>();
        let seen = rows
            .iter()
            .map(|row| {
                let cells = cells(*row);
                (
                    attribute(*row, "class"),
                    attribute(*row, "data-line"),
                    node_text(&dom, cells[0]),
                    node_text(&dom, cells[1]),
                )
            })
            .collect::<Vec<_>>();
        let row = |line: &str, gutter: &str, text: &str| {
            (
                Some(FOLD_ROW_CLASS.to_owned()),
                Some(line.to_owned()),
                gutter.to_owned(),
                text.to_owned(),
            )
        };
        assert_eq!(
            seen,
            vec![
                row("1", "▾", "# A\n"),
                row("2", "", "…\n"),
                row("3", "", "\n"),
                row("4", "", "## B\n"),
            ]
        );
        let first = cells(rows[0]);
        assert_eq!(
            attribute(first[0], "class").as_deref(),
            Some(FOLD_GUTTER_CLASS)
        );
        assert_eq!(
            attribute(first[1], "class").as_deref(),
            Some(FOLD_LINE_CLASS)
        );
        let styled = dom.dom_children(first[1]).next().expect("styled text");
        assert_eq!(
            attribute(styled, "class").as_deref(),
            Some("syntax"),
            "a style spanning lines is clipped to each row"
        );
    }

    #[test]
    fn unicode_source_is_borrowed_and_folded_at_character_boundaries() {
        let source = "a\u{301} 👩\u{200d}🔬\nbody\n";
        let fold_start = source.find('👩').unwrap();
        let fold_end = source.find("\nbody").unwrap();
        let projection =
            fold_projection(source, source.len(), &[fold_start..fold_end], &[]).unwrap();

        assert_eq!(projection.source(), source);
        assert_eq!(projection.visible_text(), "a\u{301} …\nbody\n");
        assert_eq!(
            projection.segments(),
            &[
                FoldProjectionSegment::Source(0..fold_start),
                FoldProjectionSegment::FoldMarker(fold_start..fold_end),
                FoldProjectionSegment::Source(fold_end..source.len()),
            ]
        );
        assert_eq!(
            source, "a\u{301} 👩\u{200d}🔬\nbody\n",
            "projection never changes its source"
        );
    }

    #[test]
    fn rejects_stale_length_and_invalid_fold_ranges() {
        let source = "éx";
        assert!(matches!(
            fold_projection(source, source.len() + 1, &[], &[]),
            Err(FoldProjectionError::SourceLengthMismatch {
                expected: 4,
                actual: 3,
            })
        ));
        for range in [0..0, 0..4, 1..2] {
            assert!(matches!(
                fold_projection(source, source.len(), &[range.clone()], &[]),
                Err(FoldProjectionError::InvalidFoldRange(actual)) if actual == range
            ));
        }
    }

    #[test]
    fn outer_fold_supersedes_nested_ranges_and_crossing_ranges_fail() {
        let source = "0123456789";
        let projection = fold_projection(source, source.len(), &[2..5, 1..8, 3..4], &[]).unwrap();
        assert_eq!(
            projection.segments(),
            &[
                FoldProjectionSegment::Source(0..1),
                FoldProjectionSegment::FoldMarker(1..8),
                FoldProjectionSegment::Source(8..10),
            ]
        );
        assert!(matches!(
            fold_projection(source, source.len(), &[1..5, 4..8], &[]),
            Err(FoldProjectionError::CrossingFoldRanges {
                first,
                second,
            }) if first == (1..5) && second == (4..8)
        ));
    }

    #[test]
    fn styles_are_clipped_to_visible_source_and_markers_are_semantic() {
        #[derive(Clone)]
        struct Model {
            source: String,
        }
        fn view(model: &Model) -> Box<dyn AnyView<Model, (), GenetCtx, GenetElement>> {
            let source = model.source.as_str();
            let fold_start = source.find('👩').unwrap();
            let fold_end = source.find("\nbody").unwrap();
            let styles = [
                StyleRange {
                    range: 0..fold_end,
                    class: "syntax-a".into(),
                },
                StyleRange {
                    range: fold_start..fold_end,
                    class: "hidden".into(),
                },
                StyleRange {
                    range: 1..2,
                    class: "invalid-boundary".into(),
                },
            ];
            let projection =
                fold_projection(source, source.len(), &[fold_start..fold_end], &styles).unwrap();
            Box::new(el("div", projection.field_children::<Model, ()>()))
        }

        let model = Model {
            source: "a\u{301} 👩\u{200d}🔬\nbody".into(),
        };
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = GenetAppRunner::new(dom.clone(), view, model);
        let dom = runner.dom();
        let dom = dom.borrow();
        let children: Vec<_> = dom.dom_children(runner.root()).collect();
        assert_eq!(
            children.len(),
            3,
            "styled visible source, marker, visible source"
        );
        assert_eq!(node_text(&dom, children[0]), "a\u{301} ");
        assert_eq!(node_text(&dom, children[1]), "…");
        assert_eq!(node_text(&dom, children[2]), "\nbody");
        assert_eq!(
            dom.attribute(children[0], &Namespace::from(""), &LocalName::from("class")),
            Some("syntax-a")
        );
        assert_eq!(
            dom.attribute(children[1], &Namespace::from(""), &LocalName::from("class")),
            Some(FOLD_MARKER_CLASS)
        );
        assert_eq!(
            dom.attribute(
                children[1],
                &Namespace::from(""),
                &LocalName::from("aria-label")
            ),
            Some(FOLD_MARKER_ACCESSIBLE_LABEL)
        );
        assert_eq!(
            dom.attribute(children[1], &Namespace::from(""), &LocalName::from("role")),
            Some("note")
        );
    }
}
