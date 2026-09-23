use crate::Error;

/// One line of a highlighted snippet.
///
/// A line is a piece of text plus an optional line number. Numbered lines are
/// rendered with a gutter (the `7 | ` of rustc diagnostics); unnumbered lines
/// are rendered bare, which suits filler such as an elision marker.
///
/// A line may also carry a range of characters to mark, which is rendered as a
/// `^^^^` line underneath it. The range is half-open and counts **characters**,
/// not bytes; without one the line is simply not marked. A range always fits
/// the text it marks: setting one that does not, or shortening the text under
/// it, is an [`Error`].
///
/// The text is stored as it was given, trailing line break included, and the
/// getters report one row: [`Line::content`] drops the trailing line break and
/// [`Line::highlight`] reads back clamped to what is left, so a line renders as
/// one row with its marker line under the text it marks. A range taken from the
/// source text, break included, therefore fits as it is.
///
/// Nothing else is trimmed: trailing spaces and tabs are content a caller may
/// well want to mark. Comparison is on the text as stored, so two lines that
/// render identically still differ when one arrived with a trailing line break.
///
/// # Examples
///
/// ```
/// use caret_highlight::Line;
///
/// let plain = Line::new("let x = 1;");
/// assert_eq!(plain.number(), None);
///
/// let numbered = Line::numbered(7, "let x = 1;");
/// assert_eq!(numbered.number(), Some(7));
/// assert_eq!(numbered.content(), "let x = 1;");
///
/// let marked = Line::new("let x = 1;").with_highlight((4, 5)).unwrap();
/// assert_eq!(marked.highlight(), Some((4, 5)));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Line {
    /// 1-based line number, or `None` for a line that carries no position.
    number: Option<usize>,
    /// Text of the line as it was given; [`Line::content`] drops its trailing
    /// line break.
    content: String,
    /// Half-open range of characters to mark, if any; [`Line::highlight`] reads
    /// it back clamped to the row.
    highlight: Option<(usize, usize)>,
}

/// The line breaks a line's text drops when it is read.
///
/// A rendered line is one row: a trailing break would print a blank row under
/// the line and push its marker line away from the text it marks. The set is
/// the one a marker may not be either (see
/// [`Snippet::set_marker`](crate::Snippet::set_marker)), so the two agree on
/// what breaks a line.
const NEWLINES: [char; 4] = ['\r', '\n', '\u{2028}', '\u{2029}'];

/// Checks that a half-open `range` fits in `len` characters.
fn check_range(range: (usize, usize), len: usize) -> Result<(), Error> {
    let (start, end) = range;
    if start > end {
        return Err(Error::Inverted { start, end });
    }
    if end > len {
        return Err(Error::PastEnd { start, end, len });
    }
    Ok(())
}

impl Line {
    /// Creates an unnumbered, unmarked line from `content`.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            number: None,
            content: content.into(),
            highlight: None,
        }
    }

    /// Creates a line numbered `number` from `content`.
    pub fn numbered(number: usize, content: impl Into<String>) -> Self {
        Self {
            number: Some(number),
            content: content.into(),
            highlight: None,
        }
    }

    /// Creates a line from an already optional `number` and `content`.
    ///
    /// This is the lossless constructor; [`Line::new`] and [`Line::numbered`]
    /// cover the common cases. The line starts unmarked.
    pub fn with_number(
        number: Option<usize>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            number,
            content: content.into(),
            highlight: None,
        }
    }

    /// Returns the line number, if this line has one.
    pub fn number(&self) -> Option<usize> {
        self.number
    }

    /// Returns the text of the line: one row, without the trailing line break
    /// it was given with.
    pub fn content(&self) -> &str {
        self.content.trim_end_matches(NEWLINES)
    }

    /// Returns the half-open range of characters to mark, if any, clamped to
    /// [`Line::content`].
    ///
    /// A range set against the text as given may reach into the trailing line
    /// break that text ends with; it then marks the end of the row instead. An
    /// empty range still points just past the last character, which is how a
    /// token missing at the end of a line is marked.
    pub fn highlight(&self) -> Option<(usize, usize)> {
        let (start, end) = self.highlight?;
        let len = self.content().chars().count();
        Some((start.min(len), end.min(len)))
    }

    /// Marks `highlight` on the line, returning `self` for chaining.
    ///
    /// ```
    /// # use caret_highlight::Line;
    /// let line = Line::new("let x = 1;").with_highlight((4, 5)).unwrap();
    /// assert_eq!(line.highlight(), Some((4, 5)));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when `highlight` does not fit the text of the line.
    pub fn with_highlight(
        mut self,
        highlight: (usize, usize),
    ) -> Result<Self, Error> {
        self.set_highlight(highlight)?;
        Ok(self)
    }

    /// Sets the half-open range of characters to mark.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when `highlight` does not fit the text of the line:
    /// the range already on the line, if any, is kept in that case.
    pub fn set_highlight(
        &mut self,
        highlight: (usize, usize),
    ) -> Result<&mut Self, Error> {
        check_range(highlight, self.content.chars().count())?;
        self.highlight = Some(highlight);
        Ok(self)
    }

    /// Removes the range to mark, so the line is no longer marked.
    pub fn clear_highlight(&mut self) -> &mut Self {
        self.highlight = None;
        self
    }

    /// Sets (or clears, with [`None`]) the line number.
    pub fn set_number(&mut self, number: Option<usize>) -> &mut Self {
        self.number = number;
        self
    }

    /// Removes the line number, turning this into an unnumbered line.
    pub fn clear_number(&mut self) -> &mut Self {
        self.number = None;
        self
    }

    /// Replaces the text of the line.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the range already marked on the line no longer
    /// fits `content`: the line is left untouched in that case.
    pub fn set_content(
        &mut self,
        content: impl Into<String>,
    ) -> Result<&mut Self, Error> {
        let content = content.into();
        if let Some(highlight) = self.highlight {
            check_range(highlight, content.chars().count())?;
        }
        self.content = content;
        Ok(self)
    }

    /// Returns `true` if this line carries a line number.
    pub fn is_numbered(&self) -> bool {
        self.number.is_some()
    }

    /// Returns `true` if the text of the row is empty, so a line holding
    /// nothing but a line break is empty.
    pub fn is_empty(&self) -> bool {
        self.content().is_empty()
    }
}

impl From<&str> for Line {
    fn from(content: &str) -> Self {
        Self::new(content)
    }
}

impl From<String> for Line {
    fn from(content: String) -> Self {
        Self::new(content)
    }
}

impl From<&String> for Line {
    fn from(content: &String) -> Self {
        Self::new(content)
    }
}

impl From<(usize, &str)> for Line {
    fn from((number, content): (usize, &str)) -> Self {
        Self::numbered(number, content)
    }
}

impl From<(usize, String)> for Line {
    fn from((number, content): (usize, String)) -> Self {
        Self::numbered(number, content)
    }
}

impl std::fmt::Display for Line {
    /// Writes the text of the line, without its line number or any highlight
    /// range: gutters and marker lines belong to [`Snippet`](crate::Snippet).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.content())
    }
}

#[cfg(test)]
mod tests {
    use super::Line;
    use crate::{Error, Snippet};

    #[test]
    fn constructors_set_the_optional_number() {
        assert_eq!(Line::new("a").number(), None);
        assert!(!Line::new("a").is_numbered());

        assert_eq!(Line::numbered(42, "a").number(), Some(42));
        assert_eq!(Line::with_number(None, "a").number(), None);
        assert_eq!(Line::with_number(Some(0), "a").number(), Some(0));
    }

    #[test]
    fn setters_edit_in_place() {
        let mut line = Line::new("a");
        line.set_number(Some(3)).set_content("b").unwrap();
        assert_eq!(line, Line::numbered(3, "b"));

        line.clear_number();
        assert_eq!(line, Line::new("b"));
    }

    #[test]
    fn is_empty_looks_at_the_content_only() {
        assert!(Line::numbered(1, "").is_empty());
        assert!(!Line::new(" ").is_empty());
    }

    #[test]
    fn conversions_default_to_unnumbered_but_tuples_carry_a_number() {
        assert_eq!(Line::from("a"), Line::new("a"));
        assert_eq!(Line::from(String::from("a")), Line::new("a"));
        assert_eq!(Line::from((9, "a")), Line::numbered(9, "a"));
        assert_eq!(Line::from((9, String::from("a"))), Line::numbered(9, "a"));
        assert_eq!(Line::numbered(9, "a").to_string(), "a");
    }

    #[test]
    fn highlight_range_is_optional_and_settable() {
        assert_eq!(Line::new("abc").highlight(), None);
        assert_eq!(Line::numbered(1, "abc").highlight(), None);

        let mut line = Line::new("abc").with_highlight((1, 3)).unwrap();
        assert_eq!(line.highlight(), Some((1, 3)));
        assert_eq!(
            Line::numbered(5, "abc")
                .with_highlight((0, 1))
                .unwrap()
                .number(),
            Some(5)
        );

        line.set_highlight((0, 1)).unwrap();
        assert_eq!(line.highlight(), Some((0, 1)));
        line.clear_highlight();
        assert_eq!(line.highlight(), None);
        assert_eq!(line, Line::new("abc"));
    }

    #[test]
    fn ranges_must_fit_the_text() {
        // `0..len` is the widest legal range, and an empty one is legal too.
        assert_eq!(
            Line::new("abc").with_highlight((0, 3)).unwrap().highlight(),
            Some((0, 3))
        );
        assert_eq!(
            Line::new("abc").with_highlight((2, 2)).unwrap().highlight(),
            Some((2, 2))
        );
        assert_eq!(
            Line::new("").with_highlight((0, 0)).unwrap().highlight(),
            Some((0, 0))
        );

        assert_eq!(
            Line::new("abc").with_highlight((2, 1)).unwrap_err(),
            Error::Inverted { start: 2, end: 1 }
        );
        assert_eq!(
            Line::new("abc").with_highlight((0, 4)).unwrap_err(),
            Error::PastEnd {
                start: 0,
                end: 4,
                len: 3
            }
        );

        // A rejected range leaves an unmarked line unmarked...
        let mut line = Line::new("abc");
        assert!(line.set_highlight((0, 9)).is_err());
        assert_eq!(line.highlight(), None);

        // ...and keeps the range already on a marked one.
        let mut marked = Line::new("abc").with_highlight((0, 1)).unwrap();
        assert!(marked.set_highlight((9, 9)).is_err());
        assert_eq!(marked.highlight(), Some((0, 1)));

        // Offsets are characters, not bytes.
        assert_eq!(
            Line::new("变量")
                .with_highlight((1, 2))
                .unwrap()
                .highlight(),
            Some((1, 2))
        );
        assert_eq!(
            Line::new("变量").with_highlight((0, 3)).unwrap_err(),
            Error::PastEnd {
                start: 0,
                end: 3,
                len: 2
            }
        );
    }

    #[test]
    fn shrinking_content_past_the_range_is_rejected() {
        let mut line = Line::new("abcdef").with_highlight((2, 5)).unwrap();

        line.set_content("abcde").unwrap();
        assert_eq!(line.content(), "abcde");

        // The line is left untouched when the new text no longer fits.
        assert_eq!(
            line.set_content("ab").unwrap_err(),
            Error::PastEnd {
                start: 2,
                end: 5,
                len: 2
            }
        );
        assert_eq!(line.content(), "abcde");
        assert_eq!(line.highlight(), Some((2, 5)));
    }

    #[test]
    fn content_round_trips_non_ascii_text() {
        let text = "let 变量 = \"你好\"; // 🦀";
        let line = Line::numbered(3, text);
        assert_eq!(line.content(), text);
        assert_eq!(line.to_string(), text);
    }

    #[test]
    fn the_getters_report_one_row() {
        // The text and the range are kept as given...
        let marked = Line::numbered(7, "let x = 1i32;\n")
            .with_highlight((4, 14))
            .unwrap();

        // ...and read back as the row that renders, through every window.
        assert_eq!(marked.content(), "let x = 1i32;");
        assert_eq!(marked.highlight(), Some((4, 13)));
        assert_eq!(marked.to_string(), "let x = 1i32;");
        assert_eq!(
            Snippet::new().with_line(marked).to_string(),
            "7 | let x = 1i32;\n  |     ^^^^^^^^^"
        );

        // Every line break, and nothing else.
        for text in ["a\n", "a\r\n", "a\r", "a\u{2028}", "a\u{2029}\n"] {
            assert_eq!(Line::new(text).content(), "a", "{text:?}");
        }
        assert_eq!(Line::new("a \t").content(), "a \t");
        assert!(Line::new("\n").is_empty());
        assert_eq!(Snippet::new().with_line(Line::new("\n")).to_string(), "");
    }

    #[test]
    fn a_range_is_clamped_into_the_row_and_never_inverted() {
        // The whole text as given, break included, marks the whole row.
        assert_eq!(
            Line::new("abc\n")
                .with_highlight((0, 4))
                .unwrap()
                .highlight(),
            Some((0, 3))
        );

        // A range that covered only the break, and one entirely past it, both
        // point just past the row: clamping both ends keeps `end - start` from
        // underflowing, which is what the marker line is built from.
        for range in [(3, 4), (4, 4)] {
            let line = Line::new("abc\n").with_highlight(range).unwrap();
            assert_eq!(line.highlight(), Some((3, 3)), "{range:?}");
            assert_eq!(
                Snippet::new().marker_content(&line).as_deref(),
                Some("   ^"),
                "{range:?}"
            );
        }

        // Past the text as given is still rejected.
        assert_eq!(
            Line::new("abc\n").with_highlight((0, 5)).unwrap_err(),
            Error::PastEnd {
                start: 0,
                end: 5,
                len: 4
            }
        );
    }
}
