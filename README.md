# revm - Rust Ethereum Virtual Machine

Fork of [REVM](https://github.com/bluealloy/revm) that uses 176 bit 22 byte ICAN addresses.

To run tests:
```bash
cargo test --all --all-features
```

### Disclaimer
Precompile part of the REVM isn't yet working with H176

Tests are passing but will need more testing


## TODO
 - [] Modify the precompile part of the crate
 - [] Add benchmark tests from - [REVM](https://github.com/bluealloy/revm/tree/main/bins/revm-test/src/bin) 
 - [] Modify [official Ethereum tests](https://github.com/ethereum/tests/tree/develop/GeneralStateTests) for 22 byte addresses and add it to this crate


