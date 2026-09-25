# Working in this repository

`caret-highlight` builds and renders rustc-style highlighted snippets: a
`Snippet` holds a leading text, a block of `Line`s and a trailing text, lines
may carry a number and a range to mark, and `Display` writes the result as text.

- `src/lib.rs` — crate docs: a summary, one example, a pointer to the README.
- `src/snippet.rs` — `Snippet`: the three parts, the marker, and rendering.
- `src/line.rs` — `Line`: number, text, range, and their validation.
- `src/error.rs` — `Error`.
- `README.md` — the user-facing guide, and the primary documentation.
- `tests/readme.rs` — checks the rendered sample in the README.
- `CHANGELOG.md` — user-visible changes, one `[Unreleased]` section at a time.
- `.github/workflows/` — CI: fmt and clippy, the tests (also on the MSRV,
  1.85.0), the docs build, and the release that publishes from a tag.

## Commands

```
cargo test                                 # unit tests, README guard, doctests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo doc --no-deps                        # intra-doc links must resolve
```

All four must be clean before a change is done, and no line may exceed 80
columns — see Conventions.

## Invariants

1. **A line's range always fits its text.** Every way in is checked, whichever
   of the two it touches: `with_highlight` and `set_highlight` check the range
   against the current text, `set_content` checks the range already on the line
   against the new text. A rejected change leaves the value untouched, so
   validate first and assign afterwards. There is no unvalidated way in: that is
   why the fields are private and why `Line` has no `content_mut`.
2. **A marker, and the gutter bar, stay on one line, one column wide.**
   `set_marker` and `set_bar` reject a control character, a Unicode line or
   paragraph separator, and — where the crate measures display columns — any
   character that is not one column wide.
3. **Rendering has one source of truth.** `Display`, `Snippet::gutter` and
   `Snippet::marker_gutter` all write through `write_gutter`; `Display` and
   `Snippet::marker_content` lay a marker line out from `marks_of` and write it
   through `write_marks`. The `String`-returning `gutter_of` and
   `marker_content` are wrappers over those same functions, which is what lets
   `Display` write straight into a formatter without allocating. Keep it that
   way, so that the parts a caller prints on their own reassemble into exactly
   what `Display` writes.
4. **`Snippet::gutter` takes a line index.** That keeps a stray line — one
   numbered wider than the snippet's gutter — from widening that gutter and
   drifting the marker line under it.
5. **The library does not panic.** No `unwrap`, `expect`, `panic!` or slice
   indexing outside tests and doc examples; failures are returned as `Error`.
6. `gutter_width` uses `checked_ilog10`, because a plain `ilog10` panics on `0`.

## Conventions

- English everywhere — code, docs, commit messages — except test data that is
  deliberately non-ASCII.
- 80 columns. rustfmt enforces it for code but does not rewrap comments, so
  wrap doc comments by hand.
- LF line endings, pinned by `.gitattributes`; `rustfmt.toml` asks for the same.
- Actions in `.github/workflows/` are pinned to a commit SHA, with the version
  in a trailing comment for Dependabot to read: `@<sha> # vX.Y.Z`. The exception
  is `dtolnay/rust-toolchain`, whose ref *is* the toolchain name (`@stable`,
  `@1.85.0`): `.github/dependabot.yml` ignores it, because a version bump there
  would ask rustup for a release that may not exist, so the MSRV moves by hand,
  in step with `rust-version`.
- Lines that cannot wrap are allowed past 80 columns: a pinned `uses:` line, a
  shell one-liner in a `run:` block, and a badge line in `README.md`.
- Documentation: `README.md` is the guide. Keep `src/lib.rs` to a summary, one
  example and a pointer to the README instead of duplicating it; type-level docs
  stay with their items. Describe what the code does today: no notes about what
  changed, why an API is shaped differently than before, or what is planned.
- Tests are lean and target behaviour that could break silently — validation
  bounds, rendering seams, the README sample. Coverage is not a goal, and
  message formatting is not worth a test.
- The README sample sits between `<!-- rendered:start -->` and
  `<!-- rendered:end -->` and is compared against `Display` by
  `tests/readme.rs`: keep the markers, and keep the snippet in that test in step
  with the quick start example.
- Commits: conventional prefix (`feat`, `fix`, `docs`, `refactor`, `style`,
  `test`, `chore`), imperative English subject, `!` for breaking changes, and a
  body that describes behaviour rather than the journey.

## Changing the API

- A new mutation that touches a range or the text must validate, return
  `Result`, and leave the value unchanged when it fails. Prefer a new `Error`
  variant over panicking; the enum is `#[non_exhaustive]`, so adding one is not
  a breaking change.
- Anything a renderer might want to color separately should be reachable on its
  own and formatted through the same code as `Display` (invariant 3).
- Update `README.md` for user-visible behaviour and add a `CHANGELOG.md` entry
  under `[Unreleased]`.

