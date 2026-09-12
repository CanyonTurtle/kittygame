![demo](./demo.gif)

Play official game here: <https://wasm4.org/play/kittygame>

Play latest development release here: <https://canyonturtle.github.io/kittygame>

Find and play older versions here: <https://github.com/CanyonTurtle/kittygame/releases>

# Kitty Game

A game written in Rust for the [WASM-4](https://wasm4.org) fantasy console.

## Building

Build the cart by running:

```shell
cargo build --release
```

Then run it with:

```shell
w4 run target/wasm32-unknown-unknown/release/cart.wasm
```

For more info about setting up WASM-4, see the [quickstart guide](https://wasm4.org/docs/getting-started/setup?code-lang=rust#quickstart).

## Assets

`kitty-ss.png` (192x64) and `kitty_title.png` (152x50) are the actual source
of the game's spritesheet data -- `src/kitty_ss.rs` and `src/title_ss.rs`
are just tiny stubs that `include!()` code generated at build time by
`build.rs`, which shells out to WASM-4's `png2src` tool. Editing either PNG
and running `cargo build` (or `cargo clippy`, `cargo test`, etc.) picks up
the change automatically -- there's no separate "regenerate the source"
step to remember or forget, and no way for the compiled game to drift from
the PNGs.

`build.rs` only re-runs `png2src` when the corresponding PNG actually
changes (via `cargo:rerun-if-changed`), and patches the generated `const`s
to `pub const` since `png2src` emits them private.

One naming quirk lives entirely inside `build.rs` now and needs no manual
handling: `png2src` derives its Rust constant names from the input
filename, and `title_ss.rs`'s constants (`OUTPUT_ONLINEPNGTOOLS_*`) were
first named after a PNG called `output_onlinepngtools.png` (a typical
export name from an online PNG editor), not `kitty_title.png`'s current
name. `build.rs` feeds `png2src` a renamed copy of `kitty_title.png` to
keep reproducing those same constant names, so nothing downstream
(`lib.rs`'s `use title_ss::{OUTPUT_ONLINEPNGTOOLS_WIDTH, ...}`) needs to
change.

## Links

- [Documentation](https://wasm4.org/docs): Learn more about WASM-4.
- [Snake Tutorial](https://wasm4.org/docs/tutorials/snake/goal): Learn how to build a complete game
  with a step-by-step tutorial.
- [GitHub](https://github.com/aduros/wasm4): Submit an issue or PR. Contributions are welcome!
