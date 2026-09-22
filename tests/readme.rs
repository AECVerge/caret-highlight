//! Guards the sample output in `README.md`.
//!
//! The code blocks of the README are not compiled, so nothing else would notice
//! a rendering change that leaves the sample behind. This rebuilds the quick
//! start snippet and checks the block between the `rendered` markers against
//! what `Display` writes.
//!
//! The snippet below mirrors the README by hand; only its output is checked.

use caret_highlight::{Line, Snippet};

/// The sample from `README.md`: the text between the `rendered` markers,
/// without the code fences around it.
fn readme_sample() -> String {
    let readme = include_str!("../README.md");
    let (_, rest) = readme
        .split_once("<!-- rendered:start -->")
        .expect("a rendered:start marker in README.md");
    let (block, _) = rest
        .split_once("<!-- rendered:end -->")
        .expect("a rendered:end marker in README.md");

    let lines: Vec<&str> = block
        .lines()
        .filter(|line| !line.starts_with("```"))
        .collect();
    lines.join("\n").trim_matches('\n').to_owned()
}

#[test]
fn readme_sample_is_what_display_writes() {
    let mut snippet = Snippet::new();
    snippet
        .set_above("error[E0308]: mismatched types")
        .push_line(
            Line::numbered(10, "    let x: u8 = 1i32;")
                .with_highlight((16, 20))
                .unwrap(),
        )
        .set_below("note: expected `u8`, found `i32`");

    assert_eq!(snippet.to_string(), readme_sample());
}
