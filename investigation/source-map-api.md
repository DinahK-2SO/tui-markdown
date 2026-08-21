# Source-map API contribution

Base: `origin/main@25282fc6b289201233d3a8e87709e8bed43f5076`.

Product commit: `d6fa461a398c746209ec56208117f92d1307bc9b`.

## Design

- Add an opt-in `from_str_with_options_and_source_map` API.
- Return private-field, getter-based `SourceMappedText` and `SourceMappedBlock` types.
- Report byte ranges in the exact input and line ranges in the canonical rendered `Text`.
- Detect reference-style definitions because they can rewrite links outside their source position.
- Keep the legacy `Parser<Event> -> TextWriter::run()` path and every child renderer unchanged.
- Drive the same `TextWriter::handle_event` from a separate offset iterator only for the new API.

## TDD evidence

Baseline:

- `cargo test --package tui-markdown --all-features`: 153 passed, 1 ignored; 7 doc tests passed.
- `cargo test --package tui-markdown --no-default-features`: 138 passed, 2 ignored; 5 doc tests passed.
- `cargo fmt --all -- --check`: passed.

RED:

```text
cargo test --package tui-markdown source_mapped_render -- --nocapture
error[E0425]: cannot find function `from_str_with_options_and_source_map` in this scope
```

Initial GREEN: 3 passed. Expanded source-map suite: 9 passed.

Final validation:

- all features: 162 passed, 1 ignored; 8 doc tests passed.
- no default features: 147 passed, 2 ignored; 6 doc tests passed.
- workspace excluding the reproduced baseline snapshot: passed.
- `cargo doc` with `RUSTDOCFLAGS=-D warnings`: passed.
- workspace Clippy with `-D warnings`: passed.
- workspace `cargo check --all-features`: passed.
- rustfmt check: passed.

The configured MSRV is Rust 1.88. This machine only has Rust 1.97 installed, so the exact MSRV
execution is left to upstream CI. The implementation does not intentionally use post-1.88 syntax.

## Existing Windows snapshot baseline

`cargo test --workspace --all-features` fails `markdown-reader`'s `feature_showcase_text` snapshot
on this Windows checkout. A detached, untouched `origin/main` worktree reproduced the same failure
and generated a byte-identical `.snap.new` file (SHA-256
`647857B7860EC42CD546CBE693D481DC23E7B6737EB77EC6A2C56172C12E742E`). The snapshot was not
updated as part of this contribution.