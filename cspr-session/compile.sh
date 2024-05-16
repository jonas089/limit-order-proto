rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ./target/wasm32-unknown-unknown/release/cspr-session.wasm -o ../casper-contract-tests/binaries/cspr-session-optimized.wasm