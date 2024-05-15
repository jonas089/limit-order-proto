# Casper Smart Contract with Limit Orderbook

This Smart Contract contains a simple BTreeMap order book for Limit orders of native Casper against some Cep18 Token.

# Account Model

Cep18 accounts are represented as `key`, but this Contract for now only supports `AccountHash`. 

Whenever interactions with Cep18 occur, `AccountHash.into()` is called to construct a `key`.

Cross-contract accounting is not supported for now, but can be added in the future.

