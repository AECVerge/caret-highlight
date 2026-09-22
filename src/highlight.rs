use crate::Line;

/// A rustc-style highlighted snippet, built up from three parts.
///
/// A highlight is made of an optional leading text, a block of [`Line`]s (the
/// snippet itself), and an optional trailing text. The two context texts are
/// plain strings without line numbers, so they can also carry prose such as an
/// elision marker, while every line in the snippet decides for itself whether
/// it is numbered.
///
/// The fields are private: use the constructors and builder methods. Owned
/// methods ([`Highlight::with_line`]) consume and return `Self` for chaining,
/// in-place methods ([`Highlight::push_line`]) mutate and return `&mut Self`.
/// Rendering the highlight to a string is not part of this type yet.
///
/// # Examples
///
/// ```
/// use caret_highlight::{Highlight, Line};
///
/// let highlight = Highlight::new()
///     .with_above("error[E0308]: mismatched types")
///     .with_line(Line::numbered(1, "fn main() {"))
///     .with_lines(["...", "let x: u8 = 1i32;"])
///     .with_below("note: expected `u8`, found `i32`");
///
/// assert_eq!(highlight.line_numbers(), vec![Some(1), None, None]);
/// assert_eq!(highlight.lines().len(), 3);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Highlight {
    /// Text printed before the snippet, without a line number.
    above: Option<String>,
    /// The lines of the snippet, in display order.
    snippet: Vec<Line>,
    /// Text printed after the snippet, without a line number.
    below: Option<String>,
}

impl Highlight {
    /// Creates an empty highlight with no context text and no lines.
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
        &self.snippet
    }

    /// Returns the lines of the snippet for in-place editing.
    pub fn lines_mut(&mut self) -> &mut Vec<Line> {
        &mut self.snippet
    }

    /// Returns the line numbers of the snippet, in display order.
    pub fn line_numbers(&self) -> Vec<Option<usize>> {
        self.snippet.iter().map(Line::number).collect()
    }

    /// Returns `true` if there is nothing to show: no leading text, no trailing
    /// text and no lines.
    ///
    /// A highlight whose snippet is empty but whose context is set is not
    /// empty, because it still contributes text. Ask about the snippet alone
    /// with `highlight.lines().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.above.is_none() && self.snippet.is_empty() && self.below.is_none()
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
        self.snippet.push(line.into());
        self
    }

    /// Appends lines to the snippet.
    pub fn extend_lines<I>(&mut self, lines: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<Line>,
    {
        self.snippet.extend(lines.into_iter().map(Into::into));
        self
    }

    /// Replaces every line of the snippet.
    pub fn set_lines<I>(&mut self, lines: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<Line>,
    {
        self.snippet.clear();
        self.extend_lines(lines)
    }

    /// Removes every line of the snippet, keeping the context text.
    pub fn clear_lines(&mut self) -> &mut Self {
        self.snippet.clear();
        self
    }

    /// Removes the context text and every line.
    pub fn clear(&mut self) -> &mut Self {
        *self = Self::new();
        self
    }
}

impl FromIterator<Line> for Highlight {
    /// Collects lines into a highlight that has no context text.
    ///
    /// ```
    /// # use caret_highlight::{Highlight, Line};
    /// let lines = [Line::numbered(1, "a"), Line::new("b")];
    /// let highlight: Highlight = lines.into_iter().collect();
    /// assert_eq!(highlight.line_numbers(), vec![Some(1), None]);
    /// ```
    fn from_iter<I: IntoIterator<Item = Line>>(lines: I) -> Self {
        Self {
            snippet: lines.into_iter().collect(),
            ..Self::default()
        }
    }
}

impl Extend<Line> for Highlight {
    /// Appends lines to the snippet.
    fn extend<I: IntoIterator<Item = Line>>(&mut self, lines: I) {
        self.snippet.extend(lines);
    }
}

#[cfg(test)]
mod tests {
    use super::Highlight;
    use crate::Line;

    #[test]
    fn owned_and_in_place_builders_agree() {
        let owned = Highlight::new()
            .with_above("above")
            .with_line(Line::numbered(1, "a"))
            .with_lines(["...", "b"])
            .with_below("below");

        let mut in_place = Highlight::new();
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
        assert!(Highlight::new().is_empty());
        assert!(!Highlight::new().with_above("").is_empty());
        assert!(!Highlight::new().with_line("").is_empty());

        let mut highlight = Highlight::new()
            .with_above("a")
            .with_line("b")
            .with_below("c");
        highlight.clear_lines();
        assert!(highlight.lines().is_empty());
        assert_eq!(highlight.above(), Some("a"));
        assert_eq!(highlight.below(), Some("c"));

        highlight.clear_above().clear_below();
        assert!(highlight.is_empty());

        highlight.set_above("a").push_line("b").clear();
        assert_eq!(highlight, Highlight::new());
    }

    #[test]
    fn set_lines_replaces_and_lines_mut_edits() {
        let mut highlight = Highlight::new().with_lines([(1, "a"), (2, "b")]);
        highlight.set_lines([(9, "z")]);
        assert_eq!(highlight.lines(), [Line::numbered(9, "z")]);

        highlight.lines_mut()[0].set_content("Z");
        assert_eq!(highlight.lines(), [Line::numbered(9, "Z")]);
    }

    #[test]
    fn collect_and_extend_feed_the_snippet() {
        let mut highlight: Highlight = [Line::new("a")].into_iter().collect();
        highlight.extend([Line::numbered(2, "b")]);
        assert_eq!(highlight.line_numbers(), vec![None, Some(2)]);
        assert_eq!(highlight.above(), None);
    }
}
