# revm - Rust Ethereum Virtual Machine

Fork of [REVM](https://github.com/bluealloy/revm) that uses 176 bit 22 byte ICAN addresses.

To run unit tests:
```bash
cargo test --all --all-features
```

To run the integration tests:
```bash
cargo run --package revm-test --bin ican_send_eth
```

```bash
cargo run --package revm-test --bin ican_deploy_contract
```

### Disclaimer
Tests are passing but will need more testing

