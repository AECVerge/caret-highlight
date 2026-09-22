//! Errors of this crate.
//!
//! A [`Line`](crate::Line) is the only thing that can be built wrong: give it a
//! highlight range that does not fit its text and the range is rejected.

use std::fmt;

/// A highlight range that does not fit the line it marks.
///
/// A range is half-open and counts characters, so `0..len` is the widest
/// range a line of `len` characters accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        }
    }
}

impl std::error::Error for Error {}
