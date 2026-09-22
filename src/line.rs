/// One line of a highlighted snippet.
///
/// A line is a piece of text plus an optional line number. Numbered lines are
/// rendered with a gutter (the `7 | ` of rustc diagnostics); unnumbered lines
/// are rendered bare, which suits filler such as an elision marker.
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
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Line {
    /// 1-based line number, or `None` for a line that carries no position.
    number: Option<usize>,
    /// Text of the line, without a trailing newline.
    content: String,
}

impl Line {
    /// Creates an unnumbered line from `content`.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            number: None,
            content: content.into(),
        }
    }

    /// Creates a line numbered `number` from `content`.
    pub fn numbered(number: usize, content: impl Into<String>) -> Self {
        Self {
            number: Some(number),
            content: content.into(),
        }
    }

    /// Creates a line from an already optional `number` and `content`.
    ///
    /// This is the lossless constructor; [`Line::new`] and [`Line::numbered`]
    /// cover the common cases.
    pub fn with_number(number: Option<usize>, content: impl Into<String>) -> Self {
        Self {
            number,
            content: content.into(),
        }
    }

    /// Returns the line number, if this line has one.
    pub fn number(&self) -> Option<usize> {
        self.number
    }

    /// Returns the text of the line.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the text of the line for in-place editing.
    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
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
    pub fn set_content(&mut self, content: impl Into<String>) -> &mut Self {
        self.content = content.into();
        self
    }

    /// Returns `true` if this line carries a line number.
    pub fn is_numbered(&self) -> bool {
        self.number.is_some()
    }

    /// Returns `true` if the text of the line is empty.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
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
    /// Writes the raw text of the line, ignoring its line number.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::Line;

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
        line.set_number(Some(3)).set_content("b");
        assert_eq!(line, Line::numbered(3, "b"));

        line.content_mut().push('!');
        assert_eq!(line.content(), "b!");

        line.clear_number();
        assert_eq!(line, Line::new("b!"));
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
    fn content_round_trips_non_ascii_text() {
        let text = "let 变量 = \"你好\"; // 🦀";
        let line = Line::numbered(3, text);
        assert_eq!(line.content(), text);
        assert_eq!(line.to_string(), text);
    }
}
