use casper_engine_test_support::{
    ExecuteRequestBuilder, WasmTestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{
    bytesrepr::ToBytes, account::AccountHash, crypto::{PublicKey, SecretKey}, runtime_args, system::{handle_payment::ARG_TARGET, mint::ARG_ID}, Key, RuntimeArgs, U256, contracts::NamedKeys
};
use std::path::Path;
use base64::{engine::general_purpose::STANDARD, Engine};
use casper_engine_test_support::{InMemoryWasmTestBuilder, PRODUCTION_RUN_GENESIS_REQUEST};
use casper_types::{ContractHash, URef};
use std::env;

pub const ADMIN_SECRET_KEY: [u8; 32] = [1u8; 32];
pub const USER_SECRET_KEY: [u8; 32] = [2u8; 32];

pub struct TestContext {
    builder: InMemoryWasmTestBuilder,
    pub admin: AccountHash,
    pub user: AccountHash,
    pub contract_hash: ContractHash,
    pub contract_package_key: Key,
    pub contract_purse: URef,
    pub cep18_contract_hash: ContractHash,
    pub cep18_contract_package_key: Key
}

impl TestContext {
    pub fn new() -> TestContext {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST);
        let admin: AccountHash = create_funded_account_for_secret_key_bytes(&mut builder, ADMIN_SECRET_KEY);
        let user: AccountHash = create_funded_account_for_secret_key_bytes(&mut builder, USER_SECRET_KEY);
        let market_maker_path: std::path::PathBuf = std::path::Path::new(env!("PATH_TO_WASM_BINARIES"))
            .join("casper-contract-proto-optimized.wasm");
        install_wasm_with_args(
            &mut builder,
            &market_maker_path,
            admin,
            runtime_args! {},
        );
        let cep18_contract_path: std::path::PathBuf = std::path::Path::new(env!("PATH_TO_WASM_BINARIES"))
            .join("cep18-optimized.wasm");
        let admin_list: Vec<Key> = vec![admin.into()];
        let minter_list: Vec<Key> = vec![admin.into()];
        install_wasm_with_args(
            &mut builder, 
            &cep18_contract_path, 
            admin, 
            runtime_args! {
                "name" => "usdc_contract".to_string(),
                "symbol" => "usdc",
                "decimals" => 9u8,
                "total_supply" => U256::from(1_000_000u64),
                "admin_list" => admin_list,
                "minter_list" => minter_list,
                "enable_mint_burn" => 1u8
        });

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

        let cep18_contract_hash = builder
            .get_expected_account(admin)
            .named_keys()
            .get("cep18_contract_hash_usdc_contract")
            .expect("must haveses contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");
    
        let contract_package = builder
            .get_expected_account(admin)
            .named_keys()
            .get("contract_package")
            .expect("must haveses contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");

        let cep18_contract_package = builder
            .get_expected_account(admin)
            .named_keys()
            .get("cep18_contract_package_usdc_contract")
            .expect("must haveses contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");

        let _cep18_contract = builder
            .get_contract(cep18_contract_hash)
            .expect("should have contract");

        TestContext {
            builder,
            admin,
            user,
            contract_hash,
            contract_package_key: contract_package.into(),
            contract_purse,
            cep18_contract_hash,
            cep18_contract_package_key: cep18_contract_package.into()
        }
    }
    
    pub fn mint(&mut self, amount: U256, recipient: AccountHash){
        let session_args = runtime_args!{
            "owner" => Key::from(recipient),
            "amount" => amount
        };

        let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            self.admin,
            self.cep18_contract_hash,
            "mint",
            session_args
        ).build();

        self.builder
            .exec(mint_request)
            .commit()
            .expect_success();
    }

    pub fn named_keys(&self) -> NamedKeys {
        self.builder
            .get_expected_account(*DEFAULT_ACCOUNT_ADDR)
            .named_keys()
            .clone()
    }

    pub fn contract_named_keys(&self, contract_hash: ContractHash, key_name: &str) -> Key {
        *self
            .builder
            .get_contract(contract_hash)
            .expect("should have contract")
            .named_keys()
            .get(key_name)
            .unwrap()
    }

    pub fn cep_balance(&self, account: Key, contract_hash: ContractHash) -> U256 {
        let seed_uref: URef = *self
            .contract_named_keys(contract_hash, "balances")
            .as_uref()
            .expect("Failed to find balances");
        let dictionary_key = make_dictionary_item_key(account);
        self.builder
            .query_dictionary_item(None, seed_uref, &dictionary_key)
            .unwrap()
            .as_cl_value()
            .unwrap()
            .clone()
            .into_t()
            .unwrap()
    }

    pub fn approve(&mut self, approver: AccountHash, spender: Key, amount: U256, cep18_contract_hash: ContractHash){
        let session_args = runtime_args! {
            "spender" => spender,
            "amount" => amount
        };
        let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
            approver,
            cep18_contract_hash,
            "approve",
            session_args
        ).build();

        self.builder
            .exec(approve_request)
            .commit()
            .expect_success();
    }

    // place an order to buy cspr for usdc
    pub fn limit_buy(&mut self, price: u64, amount: u64, sender: AccountHash){
        let session_args = runtime_args! {
            "price" => price,
            "amount" => amount,
            "token_hash" => self.cep18_contract_hash,
            "contract_hash" => Key::from(self.contract_hash)
        };

        let limit_buy_request = ExecuteRequestBuilder::contract_call_by_hash(
            sender,
            self.contract_hash,
            "limit_buy",
            session_args
        ).build();

        self.builder
            .exec(limit_buy_request)
            .commit()
            .expect_success();
    }
    
    pub fn limit_sell(&mut self, sender: AccountHash, price: u64, amount: u64, token_hash: ContractHash){
        let market_maker_path: std::path::PathBuf = std::path::Path::new(env!("PATH_TO_WASM_BINARIES")).join("cspr-session-optimized.wasm");
        install_wasm_with_args(
            &mut self.builder, 
            &market_maker_path,
            sender,
            runtime_args! {
                "price" => price,
                "amount" => amount,
                "token_hash" => token_hash,
                "contract_hash" => self.contract_hash
        });
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
    builder
        .exec(session_request)
        .expect_success()
        .commit();
}

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

fn make_dictionary_item_key(admin: Key) -> String {
    let preimage = admin.to_bytes().unwrap();
    STANDARD.encode(preimage)
}

// Creates a dummy account and transfer funds to it
pub fn create_funded_dummy_account(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    account_string: Option<[u8; 32]>,
) -> AccountHash {
    let (_, account_public_key) =
        create_dummy_key_pair(if let Some(account_string) = account_string {
            account_string
        } else {
            [7u8; 32]
        });
    let account = account_public_key.to_account_hash();
    fund_account(builder, account);
    account
}

pub fn create_dummy_key_pair(account_string: [u8; 32]) -> (SecretKey, PublicKey) {
    let secret_key =
        SecretKey::ed25519_from_bytes(account_string).expect("failed to create secret key");
    let public_key = PublicKey::from(&secret_key);
    (secret_key, public_key)
}

pub fn fund_account(builder: &mut WasmTestBuilder<InMemoryGlobalState>, account: AccountHash) {
    let transfer = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            ARG_AMOUNT => DEFAULT_ACCOUNT_INITIAL_BALANCE / 10_u64,
            ARG_TARGET => account,
            ARG_ID => Option::<u64>::None,
        },
    )
    .build();
    builder.exec(transfer).expect_success().commit();
}