#!/bin/bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ./target/wasm32-unknown-unknown/release/casper-contract-proto.wasm -o ../casper-contract-tests/binaries/casper-contract-proto-optimized.wasm