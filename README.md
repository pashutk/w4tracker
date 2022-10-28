# w4tracker

![Screenshot](https://api.apiflash.com/v1/urltoimage?access_key=2a6e871601a54cc6ad20b5f1990cdf91&wait_until=page_loaded&url=https://pashutk.com/w4tracker/&delay=3)

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

## Links

- [Documentation](https://wasm4.org/docs): Learn more about WASM-4.
- [Snake Tutorial](https://wasm4.org/docs/tutorials/snake/goal): Learn how to build a complete game
  with a step-by-step tutorial.
- [GitHub](https://github.com/aduros/wasm4): Submit an issue or PR. Contributions are welcome!
