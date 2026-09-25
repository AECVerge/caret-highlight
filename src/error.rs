//! Errors of this crate.
//!
//! What is rejected: a [`Line`](crate::Line) given a highlight range that does
//! not fit its text, and a [`Snippet`](crate::Snippet) given a marker or a
//! gutter bar that cannot be drawn.

use std::fmt;

/// Something a snippet, or one of its lines, cannot be built with.
///
/// A range is half-open and counts characters, so `0..len` is the widest range
/// a line of `len` characters accepts. A marker and the gutter bar each have to
/// be a single printable column, so that the lines they draw stay on one line
/// and line up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The range ends before it starts.
    Inverted {
        /// First character of the range.
        start: usize,
        /// One past the last character of the range.
        end: usize,
    },
    /// The range reaches past the last character of the line.
    PastEnd {
        /// First character of the range.
        start: usize,
        /// One past the last character of the range.
        end: usize,
        /// Number of characters in the line.
        len: usize,
    },
    /// The marker cannot be drawn on a marker line: a control character, a
    /// line separator, or a character that is not one column wide.
    InvalidMarker {
        /// The rejected character.
        marker: char,
    },
    /// The bar cannot be drawn on a gutter line: a control character, a line
    /// separator, or a character that is not one column wide.
    InvalidBar {
        /// The rejected character.
        bar: char,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Inverted { start, end } => {
                write!(
                    f,
                    "highlight range {start}..{end} ends before it starts"
                )
            }
            Self::PastEnd { start, end, len } => write!(
                f,
                "highlight range {start}..{end} does not fit in {len} chars"
            ),
            Self::InvalidMarker { marker } => {
                write!(f, "marker {marker:?} cannot be drawn")
            }
            Self::InvalidBar { bar } => {
                write!(f, "gutter bar {bar:?} cannot be drawn")
            }
        }
    }
}

impl std::error::Error for Error {}
