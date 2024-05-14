#![no_std]
#![no_main]

extern crate alloc;
use alloc::{collections::BTreeMap, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{Key, URef};
mod market;
use market::{LimitOrderBuy, LimitOrderSell};

#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key("contract_purse").is_some() {
        todo!("Handle this error!");
    }

    let new_contract_purse: URef = system::create_purse();
    runtime::put_key("contract_purse", new_contract_purse.into());
}

#[no_mangle]
pub extern "C" fn limit_buy(){
    todo!("Implement the buy entry point");
}

#[no_mangle]
pub extern "C" fn limit_sell(){
    todo!("Implement the sell entry point");
}

// todo: market order, other order types?
// todo: perpetuals

#[no_mangle]
pub extern "C" fn call(){
    let sell_limit_order_map: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let sell_limit_order_map_uref: URef = storage::new_uref(sell_limit_order_map);

    let buy_limit_order_map: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let buy_limit_order_map_uref: URef = storage::new_uref(buy_limit_order_map);

    let maybe_u64: Option<u64> = None;

    let highest_buy_price: URef = storage::new_uref(maybe_u64);
    let lowest_sell_price: URef = storage::new_uref(maybe_u64);

}