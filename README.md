# caret-highlight

[![CI](https://github.com/AECVerge/caret-highlight/actions/workflows/ci.yml/badge.svg)](https://github.com/AECVerge/caret-highlight/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/caret-highlight.svg)](https://crates.io/crates/caret-highlight)
[![docs.rs](https://img.shields.io/docsrs/caret-highlight)](https://docs.rs/caret-highlight)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue)](#msrv-and-license)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#msrv-and-license)

Build and render rustc-style highlighted snippets — the `10 |     let x = 1;`
plus `   |     ^^^^` shape that rustc diagnostics are made of. It keeps its
distance from color: render a snippet as plain text with `Display`, or ask for
its pieces and paint them yourself. Marker lines are measured in display
columns through `unicode-width`; see [Features](#features) to build without it.

```rust
use caret_highlight::{Line, Snippet};

let mut snippet = Snippet::new();
snippet
    .set_above("error[E0308]: mismatched types")
    .push_line(
        Line::numbered(10, "    let x: u8 = 1i32;")
            .with_highlight((16, 20))
            .unwrap(),
    )
    .set_below("note: expected `u8`, found `i32`");

println!("{snippet}");
```

<!-- rendered:start -->
```text
error[E0308]: mismatched types
10 |     let x: u8 = 1i32;
   |                 ^^^^
note: expected `u8`, found `i32`
```
<!-- rendered:end -->

## The model

A `Snippet` is three parts and a setting:

```text
error[E0308]: mismatched types   ← above
 9 | fn plus_one(n: u8) -> u8 {  ┐
10 |     let x: u8 = 1i32;       ┴─lines
   |                 ^^^^        ← marker
note: expected `u8`, found `i32` ← below
```

- `above` — optional text printed before the snippet, a plain `String` with no
  line number.
- `lines` — the `Line`s of the snippet, in display order.
- `below` — optional text printed after the snippet, again a plain `String`.
- `marker` — the character repeated to mark a highlighted range, `'^'` by
  default.

```text
10 |     let x: u8 = 1i32;
└─┬─┘└─────────┬─────────┘
  │            └─ content     `Line::content()`
  └─ gutter                   `Snippet::gutter(index)`

   |                 ^^^^
└─┬─┘└─────────┬────────┘
  │            └─ marks       `Snippet::marker_content(&line)`
  └─ blank gutter             `Snippet::marker_gutter()`
```

A `Line` is text plus two pieces of optional data:

- `number` — a 1-based line number, or nothing for filler lines such as `...`.
- `content` — the text of the line: one row. A trailing line break — `\n`, `\r`,
  `\r\n`, `\u{2028}` or `\u{2029}` — is dropped when the text is read, so a line
  never renders a blank row under itself and pushes its marker line away from
  the text it marks. Nothing else is trimmed: trailing spaces and tabs are
  content and are kept.
- `highlight` — a half-open `(start, end)` range of **characters** to mark,
  relative to the text of the line **as it was given**, so a span taken from the
  source fits as it is; reading it reports the range clamped to that row.

The fields are private, so every part is set through a method. Each part of a
`Snippet` has an owned builder that consumes and returns `Self` and an
in-place setter that mutates and returns `&mut Self`:

- `above` and `below` — `with_above`/`set_above`, `with_below`/`set_below`.
- `lines` — `with_line` and `with_lines` / `push_line` and `extend_lines`.
- `marker` — `with_marker` / `set_marker`, both returning `Error` for a
  character that cannot be drawn.

`Line` follows the same shape for the two fields that can change after it is
built: `with_highlight` / `set_highlight` for the range, and `with_number` /
`set_number` for the number, though `with_number` is a constructor rather
than a copy of the number already on a line. What a `Line` removes, or
replaces without returning anything new, is in-place only: `clear_highlight`,
`clear_number` and `set_content`. A `Snippet` clears the same way, with
`clear_above`, `clear_below`, `clear_lines` and `clear`.

The two styles can be mixed:

```rust
use caret_highlight::{Line, Snippet};

let owned = Snippet::new().with_above("above").with_line((7, "let x = 1;"));

let mut in_place = Snippet::new();
in_place.set_above("above").push_line((7, "let x = 1;"));

assert_eq!(owned, in_place);
```

`Line` converts from `&str`, `String`, `(usize, &str)` and `(usize, String)`, so
the line-taking methods accept any of those without a wrapper, and a line with
no number renders without a gutter:

```rust
use caret_highlight::Snippet;

let snippet = Snippet::new().with_lines(["let a = 1;", "...", "let b = 2;"]);
assert_eq!(snippet.to_string(), "let a = 1;\n...\nlet b = 2;");
```

## Rendering

`Display` writes one part per line, joined by `\n` and **without a trailing
newline**:

- `above` and `below` are written verbatim, and only when they are set: a
  missing text costs no line, while an empty one still takes its own line.
- A numbered line is written as `10 | ` — the number right-aligned in a column
  as wide as the largest number of the snippet — and then its content. A
  snippet without a numbered line has no gutter at all, and an unnumbered line
  inside one keeps the `|` column, so that all text starts at the same offset.
- A line that carries a highlight range is followed by a marker line: a blank
  gutter and one marker per marked **column**, so a wide character is marked by
  two of them. An **empty** range is a position rather than nothing and still
  gets a single marker, which is how a missing token is pointed at: `(5, 5)` on
  a line of five characters marks the sixth column.

## Coloring the parts

Everything `Display` writes can also be asked for on its own:

- `above()` and `below()` — the context texts.
- `lines()` — the lines, whose `number()`, `content()` and `highlight()` are
  readable in turn.
- `gutter(index)` — the `" 8 | "` in front of the line at `index`, or `None`
  when the snippet has no such line.
- `marker_gutter()` — the blank gutter that stands in front of a marker line.
- `marker_content(&line)` — the `"    ^^^^"` under a line, or `None` when that
  line carries no range.
- `gutter_width()` — the width of the number column; a line's text starts at
  `gutter_width() + 3`, the three columns taken by `" | "`.

```rust
use caret_highlight::{Line, Snippet};

/// Stand-in for whatever turns text into colored text.
fn paint(text: &str, code: &str) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}

let snippet = Snippet::new().with_line(
    Line::numbered(10, "    let x: u8 = 1i32;")
        .with_highlight((16, 20))
        .unwrap(),
);

let mut out = String::new();
for (index, line) in snippet.lines().iter().enumerate() {
    let gutter = snippet.gutter(index).expect("index comes from lines()");
    out.push_str(&paint(&gutter, "34"));
    out.push_str(line.content());
    out.push('\n');

    if let Some(marks) = snippet.marker_content(line) {
        out.push_str(&paint(&snippet.marker_gutter(), "34"));
        out.push_str(&paint(&marks, "31"));
        out.push('\n');
    }
}
```

## Validation

A range has to fit the text it marks, and a marker has to be drawable; nothing
else can fail. Both are checked where the value is set, so the error comes back
from the call that would have built something undrawable, and a rejected change
leaves the snippet untouched:

- `Line::with_highlight` and `Line::set_highlight` — the range against the text
  of the line as it was given, trailing line break included: `Inverted` when it
  ends before it starts, `PastEnd` when it reaches past the last character. A
  range that only reaches into the trailing line break is accepted, and reads
  back clamped to the row that renders.
- `Line::set_content` — the range already on the line against the new text, as
  given.
- `Snippet::with_marker` and `Snippet::set_marker` — a marker that would split
  the marker line in two, such as a control character: `InvalidMarker`.

```rust
use caret_highlight::{Error, Line};

let mut line = Line::new("let x = 1;").with_highlight((4, 5)).unwrap();
assert_eq!(line.highlight(), Some((4, 5)));

// The range no longer fits, so nothing changes.
assert_eq!(line.set_content("let").unwrap_err(), Error::PastEnd {
    start: 4,
    end: 5,
    len: 3,
});
assert_eq!(line.content(), "let x = 1;");
```

`Error` is `#[non_exhaustive]`, and everything else is infallible: the
constructors, the `From` conversions, `set_number`, `clear_highlight`,
`clear_number` and the context text setters.

## Limits

- Columns are measured with the `unicode-width` feature, which is on by default.
  Without it every character counts as one column, so a caret under double-width
  text (CJK, emoji) drifts by one column per wide character; see
  [Features](#features).
- A tab counts as one column: the crate does not expand tabs, so a line indented
  with them drifts when a terminal expands them to its own tab stops.
- A marker that is itself two columns wide draws two columns per mark, which
  makes the mark line wider than the range it marks. A one-column marker stays
  under the text.
- A line's text, its range and its comparison are all the text as it was given,
  so `Line::new("a\n")` and `Line::new("a")` render the same and still compare
  unequal.
- A line is one row at its end only: the trailing line break is dropped when the
  text is read, but a line break inside the text is kept verbatim and renders as
  an extra row, with that line's marker line below all of it.

## Features

- `unicode-width` *(default)* — marker lines are measured in display columns, so
  a wide character (CJK, an emoji) is two columns, a combining mark none, and a
  sequence such as `👨‍👩‍👧` the two columns its glyph covers. It is the
  crate's only dependency, and it pulls in nothing else.
- Without default features — `default-features = false` — the crate has no
  dependencies and counts every character as one column.

Enabling the feature anywhere in a build graph enables it for the whole graph,
so a build either measures columns or it does not. Nothing else changes with it:
the public API is the same either way.

## MSRV and license

Rust 1.85 (edition 2024).

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
