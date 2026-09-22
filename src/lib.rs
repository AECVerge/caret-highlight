//! Build the data behind a rustc-style highlighted snippet.
//!
//! This crate provides the construction layer only: a small, dependency-free
//! description of a highlighted snippet that a renderer can turn into text.
//! Nothing here commits to a concrete output format.
//!
//! # Model
//!
//! A [`Snippet`] is three parts:
//!
//! * a leading context text — optional, a plain [`String`] with no line number;
//! * the snippet — a `Vec` of [`Line`]s in display order, each holding an
//!   optional line number and its own text;
//! * a trailing context text — optional, again a plain [`String`].
//!
//! Keeping the line number optional lets one snippet mix numbered source lines
//! with unnumbered filler such as an elision marker:
//!
//! ```
//! use caret_highlight::{Line, Snippet};
//!
//! let mut snippet = Snippet::new();
//! snippet
//!     .set_above("error[E0308]: mismatched types")
//!     .push_line((1, "fn main() {"))
//!     .push_line("...")
//!     .push_line((9, "}"))
//!     .set_below("note: expected `u8`, found `i32`");
//!
//! assert_eq!(snippet.line_numbers(), vec![Some(1), None, Some(9)]);
//! assert_eq!(snippet.lines()[0], Line::numbered(1, "fn main() {"));
//! ```
//!
//! # Construction
//!
//! The fields are private, so two builder styles are provided and can be mixed:
//!
//! * owned methods ([`Snippet::with_line`], [`Snippet::with_above`]) that
//!   consume and return `Self`;
//! * in-place methods ([`Snippet::push_line`], [`Snippet::set_above`]) that
//!   return `&mut Self`.
//!
//! [`Line`] converts from `&str`, `String` and `(usize, ...)` tuples, so
//! [`Snippet::with_lines`] and [`Snippet::extend_lines`] accept all of them
//! without a wrapper.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod error;
mod line;
mod snippet;

pub use error::Error;
pub use line::Line;
pub use snippet::Snippet;
