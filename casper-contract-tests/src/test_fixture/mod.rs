use casper_engine_test_support::{
    ExecuteRequestBuilder, WasmTestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{
    account::AccountHash,
    crypto::{PublicKey, SecretKey},
    runtime_args,
    system::{handle_payment::ARG_TARGET, mint::ARG_ID},
    RuntimeArgs,
};
use std::path::Path;

use casper_engine_test_support::{InMemoryWasmTestBuilder, PRODUCTION_RUN_GENESIS_REQUEST};
use casper_types::{ContractHash, URef};
use std::env;

pub const ADMIN_SECRET_KEY: [u8; 32] = [1u8; 32];

#[derive(Default)]
pub struct TestContext {
    builder: InMemoryWasmTestBuilder,
    pub admin: AccountHash,
    contract_hash: ContractHash,
    contract_purse: URef,
}

impl TestContext {
    pub fn new() -> TestContext {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST);

        let admin = create_funded_account_for_secret_key_bytes(&mut builder, ADMIN_SECRET_KEY);

        let deposit_contract_path: std::path::PathBuf = std::path::Path::new(env!("PATH_TO_WASM_BINARIES"))
            .join("casper-contract-proto-optimized.wasm");
        install_wasm_with_args(
            &mut builder,
            &deposit_contract_path,
            admin,
            runtime_args! {},
        );

        let cep18_contract_path: std::path::PathBuf = std::path::Path::new(env!("PATH_TO_WASM_BINARIES"))
            .join("cep18-optimized.wasm");
        install_wasm_with_args(
            &mut builder, 
            &cep18_contract_path, 
            admin, 
            runtime_args! {});

        let contract_hash = builder
            .get_expected_account(admin)
            .named_keys()
            .get("contract_hash")
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");

        let contract = builder
            .get_contract(contract_hash)
            .expect("should have contract");

        let contract_purse = *contract
            .named_keys()
            .get("contract_purse")
            .expect("Key not found")
            .as_uref()
            .unwrap();

        TestContext {
            builder,
            admin,
            contract_hash,
            contract_purse,
        }
    }
}

pub fn install_wasm_with_args(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    session_wasm_path: &Path,
    user: AccountHash,
    runtime_args: RuntimeArgs,
) {
    let session_request =
        ExecuteRequestBuilder::standard(user, session_wasm_path.to_str().unwrap(), runtime_args)
            .build();
    builder.exec(session_request).commit();
}

/// Creates a funded account for the given ed25519 secret key in bytes
/// It panics if the passed secret key bytes cannot be read
pub fn create_funded_account_for_secret_key_bytes(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    account_secret_key_bytes: [u8; 32],
) -> AccountHash {
    let account_secret_key = SecretKey::ed25519_from_bytes(account_secret_key_bytes).unwrap();
    let account_public_key = PublicKey::from(&account_secret_key);
    let account_hash = account_public_key.to_account_hash();
    let transfer = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            ARG_AMOUNT => DEFAULT_ACCOUNT_INITIAL_BALANCE / 10_u64,
            ARG_TARGET => account_hash,
            ARG_ID => Option::<u64>::None,
        },
    )
    .build();
    builder.exec(transfer).expect_success().commit();
    account_hash
}