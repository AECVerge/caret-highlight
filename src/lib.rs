//! Build and render rustc-style highlighted snippets.
//!
//! A [`Snippet`] is a leading text, a block of [`Line`]s and a trailing text.
//! A line may carry a number and a half-open range of characters to mark;
//! [`Display`](std::fmt::Display) writes the whole thing as text, and the same
//! pieces are available one at a time — [`Snippet::gutter`],
//! [`Snippet::marker_content`], [`Snippet::lines`] — for a renderer that colors
//! them itself.
//!
//! Ranges and markers are checked where they are set, so an [`Error`] comes
//! back from the call that would have built something undrawable.
//!
//! ```
//! use caret_highlight::{Line, Snippet};
//!
//! let snippet = Snippet::new().with_line(
//!     Line::numbered(7, "let x = 1;").with_highlight((4, 5)).unwrap(),
//! );
//! assert_eq!(snippet.to_string(), "7 | let x = 1;\n  |     ^");
//! ```
//!
//! The [README](https://github.com/AECVerge/caret-highlight#readme) covers the
//! rendering rules, coloring the parts, validation and the known limits.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod error;
mod line;
mod snippet;

pub use error::Error;
pub use line::Line;
pub use snippet::Snippet;
