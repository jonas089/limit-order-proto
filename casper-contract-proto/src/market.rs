extern crate alloc;
use alloc::{borrow::ToOwned, collections::BTreeMap, vec::Vec, vec};
use casper_contract::contract_api::{runtime, storage, system};
use casper_types::{account::AccountHash, runtime_args, ContractHash, Key, RuntimeArgs, URef, U512};
use crate::orders::{LimitOrderSell, LimitOrderBuy};

// Buying CSPR for Cep18
pub fn execute_limit_buy(price: u64, amount: u64, sender: AccountHash, token_hash: ContractHash, contract_key: Key){
    let mut cep_helper: CepEighteenHelper = CepEighteenHelper{
        token_hash
    };
    let mut native_helper: NativeTransferHelper = NativeTransferHelper{
        contract_purse: contract_purse()
    };
    cep_helper.cep18_transfer_to_contract(
        contract_key, 
        sender.into(), 
        amount
    );
    // start filling this order
    let mut unfilled: u64 = price;
    while unfilled > 0{
        match lowest_sell_price(){
            Some(ask) => {
                let best_offer: LimitOrderSell = get_active_sell_order(ask).unwrap();
                if best_offer.amount == unfilled{
                    native_helper.native_transfer_from_contract(sender, amount);
                    cep_helper.cep18_transfer_from_contract(best_offer.account.into(), amount / price);
                    remove_active_sell_order(ask);
                    unfilled = 0;
                }
                else if best_offer.amount > unfilled{
                    native_helper.native_transfer_from_contract(sender, unfilled);
                    cep_helper.cep18_transfer_from_contract(best_offer.account.into(), unfilled / price);
                    unfilled = 0;
                }
                else{
                    native_helper.native_transfer_from_contract(sender, best_offer.amount);
                    cep_helper.cep18_transfer_from_contract(best_offer.account.into(), best_offer.amount / price);
                    remove_active_sell_order(ask);
                    unfilled -= best_offer.amount;
                }
            },
            None => {
                insert_new_buy_order(
                    LimitOrderBuy{
                        amount: unfilled,
                        price,
                        account: sender
                    },
                    price
                );
                break;
            }
        }
    }
}

// Selling CSPR for Cep18
pub fn execute_limit_sell(price: u64, amount: u64, sender: AccountHash, temp_purse: URef, token_hash: ContractHash){
    let mut native_helper: NativeTransferHelper = NativeTransferHelper{
        contract_purse: contract_purse()
    };
    let mut cep_helper: CepEighteenHelper = CepEighteenHelper{
        token_hash
    };
    native_helper.native_transfer_to_contract(temp_purse, amount);
    // start filling this order
    let mut unfilled: u64 = price;
    while unfilled > 0{
        match highest_buy_price(){
            Some(bid) => {
                let best_offer: LimitOrderBuy = get_active_buy_order(bid).unwrap();
                if best_offer.amount == unfilled{
                    native_helper.native_transfer_from_contract(sender, amount / price);
                    cep_helper.cep18_transfer_from_contract(best_offer.account.into(), amount);
                    remove_active_sell_order(bid);
                    unfilled = 0;
                }
                else if best_offer.amount > unfilled{
                    native_helper.native_transfer_from_contract(sender, unfilled / price);
                    cep_helper.cep18_transfer_from_contract(best_offer.account.into(), unfilled);
                    unfilled = 0;
                }
                else{
                    native_helper.native_transfer_from_contract(sender, best_offer.amount / price);
                    cep_helper.cep18_transfer_from_contract(best_offer.account.into(), best_offer.amount);
                    remove_active_sell_order(bid);
                    unfilled -= best_offer.amount;
                }
            },
            None => {
                insert_new_buy_order(
                    LimitOrderBuy{
                        amount: unfilled,
                        price,
                        account: sender
                    },
                    price
                );
                break;
            }
        }
    }
}

struct CepEighteenHelper{
    token_hash: ContractHash
}

impl CepEighteenHelper{
    // requires approval
    // lock cep18 in contract
    pub fn cep18_transfer_to_contract(&mut self, contract: Key, sender: Key, amount: u64 ){
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
    pub fn cep18_transfer_from_contract(&mut self, recipient: Key, amount: u64){
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

struct NativeTransferHelper{
    contract_purse: URef
}

impl NativeTransferHelper{
    // lock cspr in contract
    pub fn native_transfer_to_contract(&mut self, temp_purse: URef, amount: u64){
        system::transfer_from_purse_to_purse(
            temp_purse, 
            self.contract_purse, 
            U512::from(amount), 
            None
        );
    }
    // withdraw cspr to user
    pub fn native_transfer_from_contract(&mut self, recipient: AccountHash, amount: u64){
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

pub fn insert_new_buy_order(order: LimitOrderBuy, price: u64){
    let buy_limit_order_map_uref: URef = runtime::get_key("buy_limit_order_map")
        .unwrap()
        .into_uref()
        .unwrap();
    let mut buy_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(buy_limit_order_map_uref).unwrap().unwrap();
    if buy_limit_order_map.contains_key(&price){
        let mut current_price_list: Vec<LimitOrderBuy> = bincode::deserialize(&buy_limit_order_map[&price]).unwrap();
        current_price_list.push(order);
    }
    else{
        buy_limit_order_map.insert(price, bincode::serialize(&order).unwrap());
    }
    storage::write(buy_limit_order_map_uref, buy_limit_order_map);
}

pub fn insert_new_sell_order(order: LimitOrderSell, price: u64){
    let sell_limit_order_map_uref: URef = runtime::get_key("sell_limit_order_map")
        .unwrap()
        .into_uref()
        .unwrap();
    let mut sell_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(sell_limit_order_map_uref).unwrap().unwrap();
    let mut current_price_list: Vec<LimitOrderSell> = bincode::deserialize(&sell_limit_order_map[&price]).unwrap();
    if sell_limit_order_map.contains_key(&price){
        let mut current_price_list: Vec<LimitOrderSell> = bincode::deserialize(&sell_limit_order_map[&price]).unwrap();
        current_price_list.push(order);
    }
    else{
        sell_limit_order_map.insert(price, bincode::serialize(&order).unwrap());
    }
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

pub fn update_active_buy_order(price: u64, updated_order: LimitOrderBuy){
    let buy_limit_order_map_uref: URef = runtime::get_key("buy_limit_order_map")
        .unwrap()
        .into_uref()
        .unwrap();
    let mut buy_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(buy_limit_order_map_uref).unwrap().unwrap();
    let mut current_price_list: Vec<LimitOrderBuy> = bincode::deserialize(&buy_limit_order_map[&price]).unwrap();
    current_price_list[0] = updated_order;
    storage::write(buy_limit_order_map_uref, buy_limit_order_map);
}

pub fn update_active_sell_order(price: u64, updated_order: LimitOrderSell){
    let sell_limit_order_map_uref: URef = runtime::get_key("sell_limit_order_map")
        .unwrap()
        .into_uref()
        .unwrap();
    let mut sell_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(sell_limit_order_map_uref).unwrap().unwrap();
    let mut current_price_list: Vec<LimitOrderSell> = bincode::deserialize(&sell_limit_order_map[&price]).unwrap();
    current_price_list[0] = updated_order;
    storage::write(sell_limit_order_map_uref, sell_limit_order_map);
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

pub fn validate_mote_price(price: u64) -> bool{
    price % 1_000_000_000 == 0
}