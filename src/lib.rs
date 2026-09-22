//! Build the data behind a rustc-style highlighted snippet.
//!
//! This crate provides the construction layer only: a small, dependency-free
//! description of a highlighted snippet that a renderer can turn into text.
//! Nothing here commits to a concrete output format.
//!
//! # Model
//!
//! A [`Highlight`] is three parts:
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
//! use caret_highlight::{Highlight, Line};
//!
//! let mut highlight = Highlight::new();
//! highlight
//!     .set_above("error[E0308]: mismatched types")
//!     .push_line((1, "fn main() {"))
//!     .push_line("...")
//!     .push_line((9, "}"))
//!     .set_below("note: expected `u8`, found `i32`");
//!
//! assert_eq!(highlight.line_numbers(), vec![Some(1), None, Some(9)]);
//! assert_eq!(highlight.lines()[0], Line::numbered(1, "fn main() {"));
//! ```
//!
//! # Construction
//!
//! The fields are private, so two builder styles are provided and can be mixed:
//!
//! * owned methods ([`Highlight::with_line`], [`Highlight::with_above`]) that
//!   consume and return `Self`;
//! * in-place methods ([`Highlight::push_line`], [`Highlight::set_above`]) that
//!   return `&mut Self`.
//!
//! [`Line`] converts from `&str`, `String` and `(usize, ...)` tuples, so
//! [`Highlight::with_lines`] and [`Highlight::extend_lines`] accept all of them
//! without a wrapper.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod error;
mod highlight;
mod line;

pub use highlight::Highlight;
pub use line::Line;
