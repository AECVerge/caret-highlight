# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
