extern crate alloc;
use alloc::{borrow::ToOwned, collections::BTreeMap, vec::{self, Vec}};
use casper_contract::contract_api::{runtime, storage, system};
use casper_types::{account::{Account, AccountHash}, bytesrepr::ToBytes, CLType, CLTyped, Key, URef, U512};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum LimitOrder {
    BuyOrder { account_hash: AccountHash, price: u64, amount: u64 },
    SellOrder { account_hash: AccountHash, price: u64, amount: u64 }
}

impl LimitOrder{
    pub fn execute_order(self){
        match self {
            Self::BuyOrder { account_hash, price, amount } => {
                let mut to_be_filled = amount;
                match state.lowest_sell_price{
                    Some(mut sell) => {
                        while sell <= price && to_be_filled > 0{
                            let mut best_bid_list: Vec<LimitOrder> = state.sell_limit_orders[&sell].clone();
                            for (id, order) in best_bid_list.clone().into_iter().enumerate(){
                                match order{
                                    Self::SellOrder { account_hash, price, amount } => {
                                        let seller_account: &mut Key = state.accounts.get_mut(&account_hash).unwrap();
                                        // check all possible prices in range
                                        if amount < to_be_filled{
                                            todo!("write function to execute transfer from Key to Key");
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price / 1_000_000_000;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price / 1_000_000_000;
                                            to_be_filled -= amount;
                                        }
                                        else if amount == to_be_filled{
                                            // remove from list and commit
                                            best_bid_list.remove(id);
                                            // calculate balances
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price / 1_000_000_000;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price / 1_000_000_000;
                                            to_be_filled = 0;
                                        }
                                        else{
                                            seller_account.cspr_balance -= to_be_filled;
                                            seller_account.usdc_balance += to_be_filled * price;
                                            buyer_account.cspr_balance += to_be_filled;
                                            buyer_account.usdc_balance -= to_be_filled * price;
                                            to_be_filled = 0
                                        }
                                    },
                                    Self::BuyOrder { account_hash: _, price: _, amount: _ } => {
                                        panic!("Invalid order in sell_limit_orders")
                                    }
                                }
                            }
                            match state.sell_limit_orders.first_entry() {
                                Some(entry) => {
                                    sell = entry.key().to_owned()
                                },
                                None => {
                                    break;
                                }
                            }
                        }
                    },
                    None => {

                    }
                }
                if to_be_filled > 0{
                    // must add this order to the order book, since it was not filled.
                    match state.lowest_sell_price{
                        Some(buy) => {
                            if price > buy{
                                state.lowest_sell_price = Some(price);
                            }
                        },
                        None => {
                            state.lowest_sell_price = Some(price);
                        }
                    }
                    // insert new order
                    if state.buy_limit_orders.contains_key(&price){
                        let mut price_bound_orderbook = state.buy_limit_orders[&price].clone();
                        price_bound_orderbook.push(self);
                        state.buy_limit_orders.insert(price, price_bound_orderbook);
                    }
                    else{
                        state.buy_limit_orders.insert(price, vec![self]);
                    }
                }
                state.accounts.insert(account_hash, buyer_account);
            },
            Self::SellOrder { account_hash, price, amount } => {
                let mut seller_account: Account = state.accounts.remove(&account_hash).unwrap();
                let mut to_be_filled = amount;
                match state.highest_buy_price{
                    Some(mut buy) => {
                        while buy >= price && to_be_filled > 0{
                            let mut best_ask_list: Vec<LimitOrder> = state.buy_limit_orders[&buy].clone();
                            for (id, order) in best_ask_list.clone().into_iter().enumerate(){
                                match order{
                                    Self::BuyOrder { account_hash, price, amount } => {
                                        let buyer_account: &mut Account = state.accounts.get_mut(&account_hash).unwrap();
                                        // check all possible prices in range
                                        if amount < to_be_filled{
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price / 1_000_000_000;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price / 1_000_000_000;
                                            to_be_filled -= amount;
                                        }
                                        else if amount == to_be_filled{
                                            // remove from list and commit
                                            best_ask_list.remove(id);
                                            // calculate balances
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price / 1_000_000_000;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price / 1_000_000_000;
                                            to_be_filled = 0;
                                        }
                                        else{
                                            seller_account.cspr_balance -= to_be_filled;
                                            seller_account.usdc_balance += to_be_filled * price / 1_000_000_000;
                                            buyer_account.cspr_balance += to_be_filled;
                                            buyer_account.usdc_balance -= to_be_filled * price / 1_000_000_000;
                                            to_be_filled = 0
                                        }
                                    },
                                    Self::SellOrder { account_hash: _, price: _, amount: _ } => {
                                        panic!("Invalid order in sell_limit_orders")
                                    }
                                }
                            }

                            // for testing the max price is set to 1 usdt
                            match state.buy_limit_orders.first_entry() {
                                Some(entry) => {
                                    buy = entry.key().to_owned()
                                },
                                None => {
                                    break;
                                }
                            }
                        }
                    },
                    None => {

                    }
                }
                if to_be_filled > 0{
                    // must add this order to the order book, since it was not filled.
                    match state.lowest_sell_price{
                        Some(buy) => {
                            if price < buy{
                                state.lowest_sell_price = Some(price);
                            }
                        },
                        None => {
                            state.lowest_sell_price = Some(price);
                        }
                    }
                    // insert new order
                    if state.sell_limit_orders.contains_key(&price){
                        let mut price_bound_orderbook = state.sell_limit_orders[&price].clone();
                        price_bound_orderbook.push(self);
                        state.sell_limit_orders.insert(price, price_bound_orderbook);
                    }
                    else{
                        state.sell_limit_orders.insert(price, vec![self]);
                    }
                }
                state.accounts.insert(account_hash, seller_account);
            }
        }
    }
}

pub fn native_transfer_to_contract(purse: URef, amount: u64){
    let contract_purse: URef = contract_purse();
    system::transfer_from_purse_to_purse(
        purse, 
        contract_purse, 
        U512::from(amount), 
        None
    );
}

pub fn native_transfer_from_contract(recipient: AccountHash, amount: u64){
    let contract_purse = contract_purse();
    system::transfer_from_purse_to_account(
        contract_purse, 
        recipient, 
        U512::from(amount), 
        None
    );
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

pub fn get_lowest_ask() -> u64{
    let sell_limit_order_map_uref: URef = runtime::get_key("sell_limit_order_map")
    .unwrap()
    .into_uref()
    .unwrap();
    let mut sell_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(sell_limit_order_map_uref).unwrap().unwrap();
    sell_limit_order_map.first_entry().unwrap().key().to_owned()
}

pub fn get_highest_bid() -> u64{
    let buy_limit_order_map_uref: URef = runtime::get_key("buy_limit_order_map")
        .unwrap()
        .into_uref()
        .unwrap();
    let mut buy_limit_order_map: BTreeMap<u64, Vec<u8>> = storage::read(buy_limit_order_map_uref).unwrap().unwrap();
    buy_limit_order_map.last_entry().unwrap().key().to_owned()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LimitOrderBuyList{
    pub limit_orders: Vec<LimitOrderBuy>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LimitOrderBuy{
    pub amount: u64,
    pub account: AccountHash
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LimitOrderSellList{
    pub limit_orders: Vec<LimitOrderSell>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LimitOrderSell{
    pub amount: u64,
    pub account: AccountHash
}