#![no_std]
#![no_main]
use casper_contract::contract_api::{account, runtime, system};
use casper_types::{account::AccountHash, runtime_args, ContractHash, RuntimeArgs, URef, U512, Key};

#[no_mangle]
pub extern "C" fn call() {
    let amount: u64 = runtime::get_named_arg("amount");
    let price: u64 = runtime::get_named_arg("price");
    let token_hash: ContractHash = runtime::get_named_arg("token_hash");
    let contract_hash: ContractHash = runtime::get_named_arg("contract_hash");
    let source: URef  = account::get_main_purse();
    let temp_purse: URef = system::create_purse();
    // fund the temporary purse
    system::transfer_from_purse_to_purse(source, temp_purse, U512::from(amount), None).unwrap();
    // place a limit order through cross-contract call

    runtime::call_contract::<()>(
        contract_hash,
        "limit_sell",
        runtime_args! {
            "amount" => amount,
            "price" => price,
            "temp_purse" => temp_purse,
            "token_hash" => token_hash,
        }
    );
}