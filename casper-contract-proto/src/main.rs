#![no_std]
#![no_main]
extern crate alloc;
use alloc::{collections::BTreeMap, vec::Vec, vec, string::String, string::ToString};
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{account::AccountHash, contracts::NamedKeys, runtime_args, CLType, Contract, ContractHash, EntryPoint, EntryPoints, Key, NamedKey, RuntimeArgs, URef};
mod market;
use market::{execute_limit_buy, execute_limit_sell};
pub mod orders;

#[no_mangle]
pub extern "C" fn initialize() {
    if runtime::get_key("contract_purse").is_some() {
        todo!("Handle this error!");
    }

    let new_contract_purse: URef = system::create_purse();
    runtime::put_key("contract_purse", new_contract_purse.into());
}

#[no_mangle]
pub extern "C" fn limit_buy(){
    // price, amount, sender, temp_purse, token_hash, contract_key
    let price: u64 = runtime::get_named_arg("price");
    let amount: u64 = runtime::get_named_arg("price");
    let sender: AccountHash = runtime::get_caller();
    let token_hash: ContractHash = runtime::get_named_arg("token_hash");
    let contract_key: Key = runtime::get_key("contract_hash").unwrap();
    execute_limit_buy(price, amount, sender, token_hash, contract_key);
}

#[no_mangle]
pub extern "C" fn limit_sell(){
    // price, amount, sender, temp_purse, token_hash, contract_key
    let price: u64 = runtime::get_named_arg("price");
    let amount: u64 = runtime::get_named_arg("price");
    let sender: AccountHash = runtime::get_caller();
    let temp_purse: URef = runtime::get_named_arg("temp_purse");
    let token_hash: ContractHash = runtime::get_named_arg("token_hash");
    execute_limit_sell(price, amount, sender, temp_purse, token_hash);
}

// todo: market order, other order types?
// todo: perpetuals

#[no_mangle]
pub extern "C" fn call(){
    let mut named_keys: NamedKeys = NamedKeys::new();
    let mut entry_points: EntryPoints = EntryPoints::new();

    let sell_limit_order_map: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let sell_limit_order_map_uref: URef = storage::new_uref(sell_limit_order_map);

    let buy_limit_order_map: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let buy_limit_order_map_uref: URef = storage::new_uref(buy_limit_order_map);

    named_keys.insert("buy_limit_order_map".to_string(), buy_limit_order_map_uref.into());
    named_keys.insert("sell_limit_order_map".to_string(), sell_limit_order_map_uref.into());

    let limit_buy: EntryPoint = EntryPoint::new(
        "limit_buy",
        vec![],
        CLType::Unit,
        casper_types::EntryPointAccess::Public,
        casper_types::EntryPointType::Contract
    );

    let limit_sell: EntryPoint = EntryPoint::new(
        "limit_sell",
        vec![],
        CLType::Unit,
        casper_types::EntryPointAccess::Public,
        casper_types::EntryPointType::Contract
    );

    let initialize: EntryPoint = EntryPoint::new(
        "initialize",
        vec![],
        CLType::Unit,
        casper_types::EntryPointAccess::Public,
        casper_types::EntryPointType::Contract
    );

    entry_points.add_entry_point(limit_buy);
    entry_points.add_entry_point(limit_sell);
    entry_points.add_entry_point(initialize);

    let (contract_hash, _) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some("contract_package".to_string()),
        Some("contract_hash".to_string()),
    );
    runtime::put_key("contract_hash", Key::from(contract_hash));
    runtime::call_contract::<()>(
        contract_hash,
        "initialise",
        runtime_args! {},
    );
}