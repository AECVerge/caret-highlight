use crate::{Error, Line};

/// A rustc-style highlighted snippet, built up from three parts.
///
/// A snippet is made of an optional leading text, a block of [`Line`]s, and an
/// optional trailing text. The two context texts are plain strings without line
/// numbers, so they can also carry prose such as an elision marker, while every
/// line in the snippet decides for itself whether it is numbered.
///
/// A line that carries a range of characters is marked: a line of
/// [`marker`](Snippet::marker) characters is drawn underneath it. Marker
/// lines are never stored, only built on demand — by
/// [`Display`](std::fmt::Display), or by [`Snippet::marker_content`] for a
/// caller that renders the parts itself.
///
/// The fields are private: use the constructors and builder methods. Owned
/// methods ([`Snippet::with_line`]) consume and return `Self` for chaining,
/// in-place methods ([`Snippet::push_line`]) mutate and return `&mut Self`.
/// Rendering to a string goes through [`Display`](std::fmt::Display).
///
/// # Examples
///
/// ```
/// use caret_highlight::{Snippet, Line};
///
/// let snippet = Snippet::new()
///     .with_above("error[E0308]: mismatched types")
///     .with_line(Line::numbered(1, "fn main() {"))
///     .with_lines(["...", "let x: u8 = 1i32;"])
///     .with_below("note: expected `u8`, found `i32`");
///
/// assert_eq!(snippet.line_numbers(), vec![Some(1), None, None]);
/// assert_eq!(snippet.lines().len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    /// Text printed before the snippet, without a line number.
    above: Option<String>,
    /// The lines of the snippet, in display order.
    lines: Vec<Line>,
    /// Text printed after the snippet, without a line number.
    below: Option<String>,
    /// Character repeated to mark a highlighted range.
    marker: char,
}

impl Default for Snippet {
    /// An empty snippet whose [`marker`](Snippet::marker) is `'^'`.
    fn default() -> Self {
        Self {
            above: None,
            lines: Vec::new(),
            below: None,
            marker: '^',
        }
    }
}

impl Snippet {
    /// Creates an empty snippet with no context text and no lines.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the leading context text, if any.
    pub fn above(&self) -> Option<&str> {
        self.above.as_deref()
    }

    /// Returns the trailing context text, if any.
    pub fn below(&self) -> Option<&str> {
        self.below.as_deref()
    }

    /// Returns the lines of the snippet.
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    /// Returns the lines of the snippet for in-place editing.
    pub fn lines_mut(&mut self) -> &mut Vec<Line> {
        &mut self.lines
    }

    /// Returns the line numbers of the snippet, in display order.
    pub fn line_numbers(&self) -> Vec<Option<usize>> {
        self.lines.iter().map(Line::number).collect()
    }

    /// Width of the line-number gutter in characters: the number of digits of
    /// the largest line number, or `0` when no line is numbered.
    ///
    /// The largest number is used rather than the last one, so that the numbers
    /// stay aligned even when a snippet lists them out of order.
    pub fn gutter_width(&self) -> usize {
        self.lines
            .iter()
            .filter_map(Line::number)
            .map(|number| number.checked_ilog10().unwrap_or(0) as usize + 1)
            .max()
            .unwrap_or(0)
    }

    /// The gutter in front of the line at `index`, such as `" 8 | "` or
    /// `"10 | "`, or [`None`] when the snippet has no such line.
    ///
    /// The number is read from the snippet, so a gutter is always as wide as
    /// [`Snippet::marker_gutter`], which keeps a marked line above its marks.
    /// It is empty when no line is numbered, and blank padding for a line
    /// without a number, so that the text of every line starts at the same
    /// offset.
    ///
    /// [`Display`](std::fmt::Display) writes this in front of every line; it is
    /// public so that a colored renderer can print the parts of a line
    /// separately.
    pub fn gutter(&self, index: usize) -> Option<String> {
        let line = self.lines.get(index)?;
        Some(gutter_of(self.gutter_width(), line.number()))
    }

    /// The character repeated to mark a highlighted range, `'^'` by default.
    ///
    /// A double-width marker is accepted, but it shifts the marks away from the
    /// text above them.
    pub fn marker(&self) -> char {
        self.marker
    }

    /// The blank gutter that stands in front of a marker line, such as
    /// `"   | "`.
    ///
    /// This is the gutter half of a marker line; [`Snippet::marker_content`]
    /// is the other half.
    pub fn marker_gutter(&self) -> String {
        gutter_of(self.gutter_width(), None)
    }

    /// The marks under `line`, indented to the start of its range — the content
    /// half of a marker line.
    ///
    /// [`None`] when `line` carries no range. An empty range still draws a
    /// single marker at its position, which is how a missing token is pointed
    /// at: `(5, 5)` on a line of five characters marks the sixth column.
    /// Ranges always fit their line, so marks never run past the end. Offsets
    /// count characters, not bytes, and every character is assumed to be one
    /// column wide.
    ///
    /// ```
    /// # use caret_highlight::{Line, Snippet};
    /// let line = Line::new("let x = 1;").with_highlight((4, 5)).unwrap();
    /// let marks = Snippet::new().marker_content(&line);
    /// assert_eq!(marks.as_deref(), Some("    ^"));
    /// ```
    pub fn marker_content(&self, line: &Line) -> Option<String> {
        let (start, end) = line.highlight()?;
        let marks = (end - start).max(1);
        Some(" ".repeat(start) + &self.marker.to_string().repeat(marks))
    }

    /// Returns `true` if there is nothing to show: no leading text, no trailing
    /// text and no lines.
    ///
    /// A snippet whose lines are empty but whose context is set is not empty,
    /// because it still contributes text. Ask about the lines alone with
    /// `snippet.lines().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.above.is_none() && self.lines.is_empty() && self.below.is_none()
    }

    // --- builder: owned, chainable -------------------------------------

    /// Sets the leading context text.
    pub fn with_above(mut self, above: impl Into<String>) -> Self {
        self.set_above(above);
        self
    }

    /// Sets the trailing context text.
    pub fn with_below(mut self, below: impl Into<String>) -> Self {
        self.set_below(below);
        self
    }

    /// Appends one line to the snippet.
    pub fn with_line(mut self, line: impl Into<Line>) -> Self {
        self.push_line(line);
        self
    }

    /// Appends lines to the snippet.
    pub fn with_lines<I>(mut self, lines: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Line>,
    {
        self.extend_lines(lines);
        self
    }

    /// Sets the character repeated to mark a highlighted range.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when `marker` cannot be drawn on a marker line.
    pub fn with_marker(mut self, marker: char) -> Result<Self, Error> {
        self.set_marker(marker)?;
        Ok(self)
    }

    // --- builder: in place ---------------------------------------------

    /// Sets the leading context text, returning `self` for chaining.
    pub fn set_above(&mut self, above: impl Into<String>) -> &mut Self {
        self.above = Some(above.into());
        self
    }

    /// Sets the trailing context text, returning `self` for chaining.
    pub fn set_below(&mut self, below: impl Into<String>) -> &mut Self {
        self.below = Some(below.into());
        self
    }

    /// Sets the character repeated to mark a highlighted range.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when `marker` cannot be drawn on a marker line — a
    /// control character or a line separator: the marker already set is kept in
    /// that case.
    pub fn set_marker(&mut self, marker: char) -> Result<&mut Self, Error> {
        check_marker(marker)?;
        self.marker = marker;
        Ok(self)
    }

    /// Removes the leading context text.
    pub fn clear_above(&mut self) -> &mut Self {
        self.above = None;
        self
    }

    /// Removes the trailing context text.
    pub fn clear_below(&mut self) -> &mut Self {
        self.below = None;
        self
    }

    /// Appends one line to the snippet.
    pub fn push_line(&mut self, line: impl Into<Line>) -> &mut Self {
        self.lines.push(line.into());
        self
    }

    /// Appends lines to the snippet.
    pub fn extend_lines<I>(&mut self, lines: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<Line>,
    {
        self.lines.extend(lines.into_iter().map(Into::into));
        self
    }

    /// Replaces every line of the snippet.
    pub fn set_lines<I>(&mut self, lines: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<Line>,
    {
        self.lines.clear();
        self.extend_lines(lines)
    }

    /// Removes every line of the snippet, keeping the context text.
    pub fn clear_lines(&mut self) -> &mut Self {
        self.lines.clear();
        self
    }

    /// Removes the context text and every line, keeping the marker.
    pub fn clear(&mut self) -> &mut Self {
        self.above = None;
        self.lines.clear();
        self.below = None;
        self
    }
}

impl FromIterator<Line> for Snippet {
    /// Collects lines into a snippet that has no context text.
    ///
    /// ```
    /// # use caret_highlight::{Snippet, Line};
    /// let lines = [Line::numbered(1, "a"), Line::new("b")];
    /// let snippet: Snippet = lines.into_iter().collect();
    /// assert_eq!(snippet.line_numbers(), vec![Some(1), None]);
    /// ```
    fn from_iter<I: IntoIterator<Item = Line>>(lines: I) -> Self {
        Self {
            lines: lines.into_iter().collect(),
            ..Self::default()
        }
    }
}

impl Extend<Line> for Snippet {
    /// Appends lines to the snippet.
    fn extend<I: IntoIterator<Item = Line>>(&mut self, lines: I) {
        self.lines.extend(lines);
    }
}

impl std::fmt::Display for Snippet {
    /// Writes the leading text, the snippet and the trailing text, one per
    /// line, joined by `\n` and without a trailing newline. A marked line is
    /// followed by its marker line: [`Snippet::marker_gutter`] and then
    /// [`Snippet::marker_content`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = self.gutter_width();
        let mut first = true;

        if let Some(above) = &self.above {
            write_separator(f, &mut first)?;
            f.write_str(above)?;
        }

        for line in &self.lines {
            write_separator(f, &mut first)?;
            f.write_str(&gutter_of(width, line.number()))?;
            f.write_str(line.content())?;

            if let Some(marks) = self.marker_content(line) {
                write_separator(f, &mut first)?;
                f.write_str(&gutter_of(width, None))?;
                f.write_str(&marks)?;
            }
        }

        if let Some(below) = &self.below {
            write_separator(f, &mut first)?;
            f.write_str(below)?;
        }

        Ok(())
    }
}

/// The gutter of a line numbered `number` when the snippet has a gutter `width`
/// characters wide, or the blank gutter of a marker line for [`None`].
///
/// Empty when `width` is `0`, blank padding for an unnumbered line otherwise.
/// [`Snippet::gutter`], [`Snippet::marker_gutter`] and
/// [`Display`](std::fmt::Display) all go through this, so the parts a caller
/// prints on their own are the whole.
fn gutter_of(width: usize, number: Option<usize>) -> String {
    match (width, number) {
        (0, _) => String::new(),
        (width, Some(number)) => format!("{number:>width$} | "),
        (width, None) => format!("{:>width$} | ", ""),
    }
}

/// Writes the `\n` between two rendered lines, except before the first one.
fn write_separator(
    f: &mut std::fmt::Formatter<'_>,
    first: &mut bool,
) -> std::fmt::Result {
    if *first {
        *first = false;
        return Ok(());
    }
    f.write_str("\n")
}

/// Checks that a `marker` stays on a single line when it is repeated.
fn check_marker(marker: char) -> Result<(), Error> {
    if marker.is_control() || matches!(marker, '\u{2028}' | '\u{2029}') {
        return Err(Error::InvalidMarker { marker });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Snippet;
    use crate::{Error, Line};

    #[test]
    fn owned_and_in_place_builders_agree() {
        let owned = Snippet::new()
            .with_above("above")
            .with_line(Line::numbered(1, "a"))
            .with_lines(["...", "b"])
            .with_below("below");

        let mut in_place = Snippet::new();
        in_place
            .set_above("above")
            .push_line(Line::numbered(1, "a"))
            .extend_lines(["...", "b"])
            .set_below("below");

        assert_eq!(owned, in_place);
        assert_eq!(owned.above(), Some("above"));
        assert_eq!(owned.below(), Some("below"));
        assert_eq!(owned.lines().len(), 3);
        assert_eq!(owned.line_numbers(), vec![Some(1), None, None]);
        assert_eq!(owned.lines()[0], Line::numbered(1, "a"));
    }

    #[test]
    fn is_empty_ignores_nothing_and_clears_are_scoped() {
        assert!(Snippet::new().is_empty());
        assert!(!Snippet::new().with_above("").is_empty());
        assert!(!Snippet::new().with_line("").is_empty());

        let mut snippet = Snippet::new()
            .with_above("a")
            .with_line("b")
            .with_below("c");
        snippet.clear_lines();
        assert!(snippet.lines().is_empty());
        assert_eq!(snippet.above(), Some("a"));
        assert_eq!(snippet.below(), Some("c"));

        snippet.clear_above().clear_below();
        assert!(snippet.is_empty());

        snippet.set_above("a").push_line("b").clear();
        assert_eq!(snippet, Snippet::new());
    }

    #[test]
    fn set_lines_replaces_and_lines_mut_edits() {
        let mut snippet = Snippet::new().with_lines([(1, "a"), (2, "b")]);
        snippet.set_lines([(9, "z")]);
        assert_eq!(snippet.lines(), [Line::numbered(9, "z")]);

        snippet.lines_mut()[0].set_content("Z").unwrap();
        assert_eq!(snippet.lines(), [Line::numbered(9, "Z")]);
    }

    #[test]
    fn collect_and_extend_feed_the_snippet() {
        let mut snippet: Snippet = [Line::new("a")].into_iter().collect();
        snippet.extend([Line::numbered(2, "b")]);
        assert_eq!(snippet.line_numbers(), vec![None, Some(2)]);
        assert_eq!(snippet.above(), None);
    }

    #[test]
    fn gutter_width_counts_only_numbered_lines() {
        assert_eq!(Snippet::new().gutter_width(), 0);
        assert_eq!(Snippet::new().with_line("a").gutter_width(), 0);
        assert_eq!(
            Snippet::new()
                .with_lines([(7, "a"), (8, "b")])
                .gutter_width(),
            1
        );
        assert_eq!(
            Snippet::new()
                .with_lines([(9, "a"), (100, "b")])
                .gutter_width(),
            3
        );
        // The largest number wins, not the last one.
        assert_eq!(
            Snippet::new()
                .with_lines([(100, "a"), (9, "b")])
                .gutter_width(),
            3
        );
    }

    #[test]
    fn gutter_is_blank_padding_or_empty() {
        let snippet = Snippet::new()
            .with_line((8, "a"))
            .with_line("...")
            .with_line((10, "b"));
        assert_eq!(snippet.gutter(0).as_deref(), Some(" 8 | "));
        assert_eq!(snippet.gutter(1).as_deref(), Some("   | "));
        assert_eq!(snippet.gutter(2).as_deref(), Some("10 | "));
        assert_eq!(snippet.gutter(3), None);

        let unnumbered = Snippet::new().with_line("a");
        assert_eq!(unnumbered.gutter(0).as_deref(), Some(""));
        assert_eq!(unnumbered.gutter(1), None);
    }

    #[test]
    fn display_renders_context_around_the_snippet() {
        let snippet = Snippet::new()
            .with_above("error[E0308]: mismatched types")
            .with_lines([(1, "fn main() {"), (9, "}")])
            .with_below("note: expected `u8`, found `i32`");

        let expected = [
            "error[E0308]: mismatched types",
            "1 | fn main() {",
            "9 | }",
            "note: expected `u8`, found `i32`",
        ]
        .join("\n");
        assert_eq!(snippet.to_string(), expected);
    }

    #[test]
    fn display_sizes_the_gutter_and_keeps_the_bar_for_unnumbered_lines() {
        let snippet = Snippet::new()
            .with_line((8, "a"))
            .with_line("b")
            .with_line((10, "c"));
        assert_eq!(snippet.to_string(), " 8 | a\n   | b\n10 | c");
    }

    #[test]
    fn display_omits_the_gutter_without_numbers_and_context_that_is_none() {
        assert_eq!(Snippet::new().to_string(), "");
        assert_eq!(Snippet::new().with_lines(["a", "b"]).to_string(), "a\nb");
        assert_eq!(Snippet::new().with_above("only").to_string(), "only");

        // `Some("")` is set, so it still takes its own empty line.
        assert_eq!(
            Snippet::new().with_above("").with_below("").to_string(),
            "\n"
        );
    }

    #[test]
    fn marker_is_configurable_and_clear_keeps_it() {
        assert_eq!(Snippet::new().marker(), '^');

        let mut snippet = Snippet::new().with_marker('~').unwrap();
        assert_eq!(snippet.marker(), '~');

        snippet
            .set_marker('-')
            .unwrap()
            .set_above("a")
            .push_line("b");
        snippet.clear();
        assert_eq!(snippet.marker(), '-');
        assert!(snippet.is_empty());
    }

    #[test]
    fn markers_must_be_printable() {
        let mut snippet = Snippet::new().with_marker('~').unwrap();

        for marker in ['\n', '\r', '\t', '\0', '\u{2028}'] {
            assert_eq!(
                snippet.set_marker(marker).unwrap_err(),
                Error::InvalidMarker { marker }
            );
            assert_eq!(snippet.marker(), '~');
        }

        // Anything printable goes, even when it is only one column wide.
        assert_eq!(snippet.set_marker(' ').unwrap().marker(), ' ');
    }

    #[test]
    fn marker_content_marks_the_range() {
        let marked = |range| Line::new("abcdef").with_highlight(range).unwrap();
        let snippet = Snippet::new();

        assert_eq!(snippet.marker_content(&Line::new("abc")), None);
        assert_eq!(
            snippet.marker_content(&marked((0, 3))).as_deref(),
            Some("^^^")
        );
        assert_eq!(
            snippet.marker_content(&marked((2, 4))).as_deref(),
            Some("  ^^")
        );
        assert_eq!(
            snippet.marker_content(&marked((4, 6))).as_deref(),
            Some("    ^^")
        );

        // An empty range marks the position itself, so a token missing at the
        // end of the line is pointed at just past it.
        assert_eq!(
            snippet.marker_content(&marked((3, 3))).as_deref(),
            Some("   ^")
        );
        assert_eq!(
            snippet.marker_content(&marked((6, 6))).as_deref(),
            Some("      ^")
        );

        // Offsets count characters, not bytes.
        let wide = Line::new("变量").with_highlight((1, 2)).unwrap();
        assert_eq!(snippet.marker_content(&wide).as_deref(), Some(" ^"));

        let tilde = Snippet::new().with_marker('~').unwrap();
        assert_eq!(
            tilde.marker_content(&marked((0, 2))).as_deref(),
            Some("~~")
        );
    }

    #[test]
    fn display_places_marker_lines_between_the_context_texts() {
        let snippet = Snippet::new()
            .with_above("above")
            .with_line(
                Line::numbered(1, "let a = 1;")
                    .with_highlight((4, 5))
                    .unwrap(),
            )
            .with_line(
                Line::numbered(2, "let b").with_highlight((5, 5)).unwrap(),
            )
            .with_below("below");

        let expected = [
            "above",
            "1 | let a = 1;",
            "  |     ^",
            "2 | let b",
            "  |      ^",
            "below",
        ]
        .join("\n");
        assert_eq!(snippet.to_string(), expected);
    }

    #[test]
    fn display_writes_a_marker_line_under_a_marked_line() {
        let snippet = Snippet::new().with_lines([
            Line::numbered(8, "let x = 1;"),
            Line::numbered(10, "let y = 2;")
                .with_highlight((4, 5))
                .unwrap(),
        ]);
        let expected =
            [" 8 | let x = 1;", "10 | let y = 2;", "   |     ^"].join("\n");
        assert_eq!(snippet.to_string(), expected);

        assert_eq!(snippet.marker_gutter(), "   | ");
        assert_eq!(
            snippet.marker_content(snippet.lines().last().unwrap()),
            Some(String::from("    ^"))
        );

        // Without a gutter, the marker line has no gutter half either.
        let plain = Snippet::new()
            .with_line(Line::new("let z = 3;").with_highlight((0, 3)).unwrap());
        assert_eq!(plain.marker_gutter(), "");
        assert_eq!(plain.to_string(), "let z = 3;\n^^^");
    }
}
