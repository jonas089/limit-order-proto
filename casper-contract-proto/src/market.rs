extern crate alloc;
use alloc::{borrow::ToOwned, collections::BTreeMap, vec::Vec};
use casper_contract::contract_api::{runtime, storage, system};
use casper_types::{account::AccountHash, runtime_args, ContractHash, Key, RuntimeArgs, URef, U512};
use crate::orders::{LimitOrderSell, LimitOrderBuy};

// Buying CSPR for Cep18
pub fn execute_limit_buy(){
    todo!("
        1. Transfer Cep18 to contract, or revert with usererror
        2. Match against lowest_sell_price
        if there is a lowest_sell_price:
            start filling this order
            if there is no lowest_sell_price left
                store this buy order and update highest_buy_price
        if there is no lowest_sell_price:
            store this buy order in buy_limit_order_map and update highest_buy_price (if < or None)
    ");
}

// Selling CSPR for Cep18
pub fn execute_limit_sell(){
    todo!("
        1. Transfer CSPR to contract, or revert with usererror
        2. Match against highest_buy_price
        if there is a highest_buy_price:
            start filling this order
            if there is no highest_buy_price left
                store this sell order and update lowest_sell_price
        if there is no lowest_sell_price:
            store this sell order in sell_limit_order_map and update lowest_sell_price  (if > or None)
    ");
}

struct Cep18{
    token_hash: ContractHash
}

impl Cep18{
    // requires approval
    // lock cep18 in contract
    pub fn cep18_transfer_to_contract(self, contract: Key, sender: Key, amount: u64 ){
        runtime::call_contract::<()>(
            self.token_hash,
            "transfer_from",
            runtime_args! {
                "recipient" => contract,
                "owner" => sender,
                "amount" => amount
            },
        );
    }
    // withdraw cep18 from contract
    pub fn cep18_transfer_from_contract(self, recipient: Key, amount: u64){
        runtime::call_contract::<()>(
            self.token_hash,
            "transfer",
            runtime_args! {
                "recipient" => recipient,
                "amount" => amount
            },
        );
    }
}

struct NativeCspr{
    contract_purse: URef
}

impl NativeCspr{
    // lock cspr in contract
    pub fn native_transfer_to_contract(self, temp_purse: URef, amount: u64){
        system::transfer_from_purse_to_purse(
            temp_purse, 
            self.contract_purse, 
            U512::from(amount), 
            None
        );
    }
    // withdraw cspr to user
    pub fn native_transfer_from_contract(self, recipient: AccountHash, amount: u64){
        system::transfer_from_purse_to_account(
            self.contract_purse, 
            recipient, 
            U512::from(amount), 
            None
        );
    }
    
}

pub fn contract_purse() -> URef{
    runtime::get_key("contract_purse")
        .unwrap()
        .into_uref()
        .unwrap()
}

pub fn highest_buy_price() -> Option<u64>{
    let highest_buy_price_uref: URef = runtime::get_key("highest_buy_price")
        .unwrap()
        .into_uref()
        .unwrap();
    storage::read(highest_buy_price_uref).unwrap()
}

pub fn lowest_sell_price() -> Option<u64>{
    let lowest_sell_price_uref: URef = runtime::get_key("lowest_sell_price")
        .unwrap()
        .into_uref()
        .unwrap();
    storage::read(lowest_sell_price_uref).unwrap()
}

pub fn update_higest_buy_price(price: u64){
    let highest_buy_price_uref: URef = runtime::get_key("highest_buy_price")
    .unwrap()
    .into_uref()
    .unwrap();
    storage::write(highest_buy_price_uref, price);
}

pub fn update_lowest_sell_price(price: u64){
    let lowest_sell_price_uref: URef = runtime::get_key("lowest_sell_price")
        .unwrap()
        .into_uref()
        .unwrap();
    storage::write(lowest_sell_price_uref, price);
}

pub fn get_active_buy_order(price: u64) -> Option<LimitOrderBuy>{
    let buy_limit_order_map_uref: URef = runtime::get_key("buy_limit_order_map")
        .unwrap()
        .into_uref()
        .unwrap();
    let buy_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(buy_limit_order_map_uref).unwrap().unwrap();
    let current_price_list: Vec<LimitOrderBuy> = bincode::deserialize(&buy_limit_order_map[&price]).unwrap();
    if current_price_list.len() == 0{
        None
    }
    else{
        Some(current_price_list[0].clone())
    }
}

pub fn get_active_sell_order(price: u64) -> Option<LimitOrderSell>{
    let sell_limit_order_map_uref: URef = runtime::get_key("sell_limit_order_map")
        .unwrap()
        .into_uref()
        .unwrap();
    let sell_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(sell_limit_order_map_uref).unwrap().unwrap();
    let current_price_list: Vec<LimitOrderSell> = bincode::deserialize(&sell_limit_order_map[&price]).unwrap();
    if current_price_list.len() == 0{
        None
    }
    else{
        Some(current_price_list[0].clone())
    }
}

pub fn remove_active_buy_order(price: u64){
    let buy_limit_order_map_uref: URef = runtime::get_key("buy_limit_order_map")
        .unwrap()
        .into_uref()
        .unwrap();
    let mut buy_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(buy_limit_order_map_uref).unwrap().unwrap();
    let mut current_price_list: Vec<LimitOrderBuy> = bincode::deserialize(&buy_limit_order_map[&price]).unwrap();
    current_price_list.remove(0);
    buy_limit_order_map.insert(price, bincode::serialize(&current_price_list).unwrap());
    storage::write(buy_limit_order_map_uref, buy_limit_order_map);
}

pub fn remove_active_sell_order(price: u64){
    let sell_limit_order_map_uref: URef = runtime::get_key("sell_limit_order_map")
        .unwrap()
        .into_uref()
        .unwrap();
    let mut sell_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(sell_limit_order_map_uref).unwrap().unwrap();
    let mut current_price_list: Vec<LimitOrderSell> = bincode::deserialize(&sell_limit_order_map[&price]).unwrap();
    current_price_list.remove(0);
    sell_limit_order_map.insert(price, bincode::serialize(&current_price_list).unwrap());
    storage::write(sell_limit_order_map_uref, sell_limit_order_map);
}

// must return an option, list can be empty
pub fn get_lowest_ask() -> Option<u64>{
    let sell_limit_order_map_uref: URef = runtime::get_key("sell_limit_order_map")
    .unwrap()
    .into_uref()
    .unwrap();
    let mut sell_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(sell_limit_order_map_uref).unwrap().unwrap();
    match sell_limit_order_map.first_entry(){
        Some(entry) => {
            Some(entry.key().to_owned())
        },
        None => None
    }
}

// must return an option, list can be empty
pub fn get_highest_bid() -> Option<u64>{
    let buy_limit_order_map_uref: URef = runtime::get_key("buy_limit_order_map")
        .unwrap()
        .into_uref()
        .unwrap();
    let mut buy_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(buy_limit_order_map_uref).unwrap().unwrap();
    match buy_limit_order_map.last_entry(){
        Some(entry) => {
            Some(entry.key().to_owned())
        },
        None => None
    }
}