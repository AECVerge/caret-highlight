# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- A marker must be one column wide: a wider one (`'好'`) would push the marks
  away from the text above them, and a narrower one (a zero-width space, U+200B)
  would draw nothing at all. Both are refused with `InvalidMarker`, the error a
  marker that breaks the marker line in two already returned. Where the crate
  cannot measure columns — without the `unicode-width` feature — only the
  line-breaking markers are refused.

## [0.2.0] - 2026-09-24

### Added

- `unicode-width` as an optional dependency, on by default: a marker line is
  measured in display columns, so a wide character (CJK, an emoji) is marked by
  two of them, a combining mark by none, and an emoji sequence by the glyph it
  renders rather than by the sum of its parts.

### Changed

- A marker line is indented and repeated by column instead of by character, so
  the marks sit under the text they mark in wide text as well. Without default
  features the dependency is dropped and every character counts as one column.

## [0.1.1] - 2026-09-23

### Changed

- A line keeps the text and the range as they were given and reads them back as
  one row: `Line::content` drops a trailing line break (`\n`, `\r`, `\r\n`,
  `\u{2028}`, `\u{2029}`) and `Line::highlight` reports the range clamped to
  what is left, so a line no longer renders a blank row that pushes its marker
  line away from the text it marks. A range is still checked against the text as
  given, so a span taken from the source fits as it is; trailing spaces and tabs
  are kept, and comparison stays on the text as given.

## [0.1.0] - 2026-09-22

### Added

- `Snippet`, a rustc-style highlighted snippet: an optional leading text, a
  block of `Line`s, an optional trailing text, and the marker character used to
  mark a range.
- `Line`, one line of such a snippet: an optional 1-based line number, its text,
  and an optional half-open range of characters to mark.
- Rendering through `Display`, one part per line, joined by `\n` and without a
  trailing newline: line numbers right-aligned in a gutter as wide as the
  largest number, and a marker line under every marked line.
- An empty range marks a position rather than nothing, so a token missing at the
  end of a line is pointed at just past its last character.
- The pieces of a rendered snippet on their own, for a renderer that colors
  them: `Snippet::gutter`, `Snippet::marker_gutter`, `Snippet::marker_content`
  and `Snippet::gutter_width`, next to `Snippet::above`, `Snippet::below`,
  `Snippet::lines` and `Snippet::line_numbers`.
- A custom marker character, `'^'` by default.
- An owned builder and an in-place setter for the context texts, the lines and
  the marker, over private fields; everything that clears a part is in-place
  only.
- `Line` conversions from `&str`, `String`, `(usize, &str)` and
  `(usize, String)`, plus `FromIterator` and `Extend` for `Snippet`.
- `Error`, returned when a range does not fit its line (`Inverted`, `PastEnd`)
  or a marker cannot be drawn on a marker line (`InvalidMarker`). Ranges and
  markers are checked where they are set, and a rejected change leaves the
  value untouched.
