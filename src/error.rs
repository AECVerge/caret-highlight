//! Errors of this crate.
//!
//! Two things are rejected: a [`Line`](crate::Line) given a highlight range
//! that does not fit its text, and a [`Snippet`](crate::Snippet) given a marker
//! that cannot be drawn.

use std::fmt;

/// Something a snippet, or one of its lines, cannot be built with.
///
/// A range is half-open and counts characters, so `0..len` is the widest range
/// a line of `len` characters accepts. A marker has to be printable, so that a
/// marker line stays on one line.
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
    /// The marker would break a marker line in two.
    InvalidMarker {
        /// The rejected character.
        marker: char,
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
        }
    }
}

impl std::error::Error for Error {}
