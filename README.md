# How to run Tests

In order to run integration tests:

```bash
cd ./casper-contract-proto && ./compile.sh
cd ../cspr-session && ./compile.sh

cd ../casper-contract-tests && cargo test
```

End to end tests are yet to be implemented.
