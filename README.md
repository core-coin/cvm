# revm - Rust Ethereum Virtual Machine

Fork of [REVM](https://github.com/bluealloy/revm) that uses 176 bit 22 byte ICAN addresses.

## Tests

To run tests:
```bash
cargo test --all --all-features
```

If you want to learn how to interact with REVM the best way is to write an integration test.
You can check out 2 well documented examples [here](https://github.com/Kuly14/ican-revm/tree/main/bins/revme/tests) and add some of your own tests.

## Benchmark Tests
Don't forget to run them with the `--release` flag.

```bash
cargo run --package revm-test --release --bin analysis
```
```bash
cargo run --package revm-test --release --bin snailtracer
```

## Disclaimer
Precompile part of the REVM isn't yet working with H176

## Contributing
Before opening a pr run:
```bash
cargo test --all --all-features
```
```bash
cargo +nightly clippy --all --all-features
```
```bash
cargo +nightly fmt --all
```

Make sure they all pass.
You will need to have nightly installed.

## TODO
 - [ ] Modify the EVM::new() methods so it takes what kind of network are we running it on: 1. Mainnet: "cb"...
 - [ ] Add benchmark tests from - [REVM](https://github.com/bluealloy/revm/tree/main/bins/revm-test/src/bin) 
 - [ ] Modify [official Ethereum tests](https://github.com/ethereum/tests/tree/develop/GeneralStateTests) for 22 byte addresses and add it to this crate
 - [x] Modify the precompile part of the crate
 - [ ] Implement Ed448


