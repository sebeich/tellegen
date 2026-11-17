# Block diagrams from the code

The RMS engine (`crates/tellegen/src/rms`) documents its structure in the code
itself: the code is divided into **blocks**, and each block carries its own
explanation. `tellegen-blocks` reads them and draws an interactive block
diagram: the blocks of one level and what uses what, a summary on hover, the
sub-blocks on a click, and the full documentation and code beside it. Nobody
has to read the code to see how a run goes through the engine, and the
diagram cannot drift from the code, because the code is all it reads.

- **On a running server:** `/docs/blocks/rms` (drawn when `tellegen-server` was
  built, so it shows exactly the code that server runs).
- **Locally:** `just blocks` writes `target/blocks/rms.html`.

## Declaring a block

One tag in the documentation the code already has. There are three kinds,
nested by where they are:

```rust
//! Time integration of `M ẋ = f(x)` with the implicit trapezoidal rule, …
//!
//! @block Time integration                 <- a module: the whole file

/// Advance `(x, f)` by one step `h` …
///
/// @block One step                         <- an item: this fn, struct, enum,
pub(crate) fn step(…) {                     //    trait, impl, type, const, mod
    // @block Predict                       <- a section of code, up to the
    // Newton starts from the parabola …    //    next section at this depth,
    …                                       //    `// @end`, or the closing brace
    // @block Newton iterations
    // Up to 25 corrections …
    …
}
```

- **Module:** `@block <title>` in the file's `//!` docs. The documentation is
  the module docs.
- **Item:** `@block <title>` in an item's `///` docs. The documentation is the
  item's docs (rustdoc shows the tag as a last line).
- **Section:** an own-line `// @block <title>` comment, followed by the `//`
  lines that document it. A section can group anything: statements in a
  function, methods in an `impl`, items or `mod` declarations in a module
  (`rms/mod.rs` groups its modules into stages this way). `// @end` ends one
  early.

Tags close a doc comment, like JSDoc's. The first paragraph is the summary the
diagram shows in a box; the rest appears beside it.

## What comes from the code

| | From |
|---|---|
| Id | The Rust path: `rms::integrate::Stepper::step`, a section `…#newton-iterations` |
| Parent | The smallest block around it; a module's parent is the block around its `mod` declaration |
| Order | Where it is: sections in a function are numbered steps |
| Arrows | A name used in one block and defined in another: a type, function, method, constant or module. Code under `#[cfg(test)]` and `use` declarations do not count |
| Size, location | The block's lines in its file |

Nothing is written twice: there is no diagram to keep in step with the code,
no list of edges, no separate documentation.

## Checks

`cargo test -p tellegen-blocks` (part of `cargo test --workspace`) fails when
the RMS engine's annotations are incomplete, naming the file and line: a
module without a block, a block without documentation, an unknown tag, an
`@end` without a section, or a `///` block tag that documents no item. The
server's build script reports the same as build warnings.

## Command line

```sh
cargo run -p tellegen-blocks -- crates/tellegen/src/rms \
    --title "tellegen RMS engine" --out target/blocks/rms.html \
    [--json target/blocks/rms.json] [--source-url <prefix>] [--check]
```

`--source-url` links each block to its lines (for example
`https://github.com/<org>/<repo>/blob/<rev>/`), `--check` exits with an error
when there are annotation issues. The generator has no dependencies and works
on any module tree, so another part of the code can adopt it by adding its
tags and one command.
