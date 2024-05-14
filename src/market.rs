use std::borrow::BorrowMut;

use crate::storage::{Account, MemoryState};
#[derive(Debug, Clone, Copy)]
pub enum LimitOrder {
    BuyOrder { account_id: u64, price: u64, amount: u64 },
    SellOrder { account_id: u64, price: u64, amount: u64 },
}

impl LimitOrder{
    pub fn execute_order(self, state: &mut MemoryState){
        match self {
            Self::BuyOrder { account_id, price, amount } => {
                let mut buyer_account: Account = state.accounts.remove(&account_id).unwrap();
                let mut to_be_filled = amount;
                match state.lowest_sell_price{
                    Some(mut sell) => {
                        while sell <= price && to_be_filled > 0{
                            let mut best_bid_list: Vec<LimitOrder> = state.sell_limit_orders[&sell].clone();
                            for (id, order) in best_bid_list.clone().into_iter().enumerate(){
                                match order{
                                    Self::SellOrder { account_id, price, amount } => {
                                        let seller_account: &mut Account = state.accounts.get_mut(&account_id).unwrap();
                                        // check all possible prices in range
                                        if amount < to_be_filled{
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price / 1000_000_000;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price / 1000_000_000;
                                            to_be_filled -= amount;
                                        }
                                        else if amount == to_be_filled{
                                            // remove from list and commit
                                            best_bid_list.remove(id);
                                            // calculate balances
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price / 1000_000_000;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price / 1000_000_000;
                                            to_be_filled = 0;
                                        }
                                        // this could be an "else", but being more explicit improves readabiliy for prototyping
                                        else if amount >= to_be_filled{
                                            seller_account.cspr_balance -= to_be_filled;
                                            seller_account.usdc_balance += to_be_filled * price;
                                            buyer_account.cspr_balance += to_be_filled;
                                            buyer_account.usdc_balance -= to_be_filled * price;
                                            to_be_filled = 0
                                        }
                                    },
                                    Self::BuyOrder { account_id: _, price: _, amount: _ } => {
                                        panic!("Invalid order in sell_limit_orders")
                                    }
                                }
                            }
                            // for testing the max price is set to 1 usdt
                            state.lowest_sell_price = None;
                            
                            // todo!("Implement an efficient price discovery algorithm")
                            state.clone().brute_force_lowest_sell_in_range(499_999_999u64, 500_000_000u64);
                            match state.lowest_sell_price{
                                Some(s) => {
                                    sell = s;
                                },
                                None => {
                                    println!("No sell found, breaking!");
                                    break;
                                }
                            }
                        }
                    },
                    None => {
                        println!("[WARNING] There are no Asks for this Asset, your Bid will be placed.");
                    }
                }
                if to_be_filled > 0{
                    // must add this order to the order book, since it was not filled.
                    match state.highest_buy_price{
                        Some(buy) => {
                            if price > buy{
                                state.highest_buy_price = Some(price);
                            }
                        },
                        None => {
                            state.highest_buy_price = Some(price);
                        }
                    }
                    if state.buy_limit_orders.contains_key(&price){
                        let mut price_bound_orderbook = state.buy_limit_orders[&price].clone();
                        price_bound_orderbook.push(self);
                        state.buy_limit_orders.insert(price, price_bound_orderbook);
                    }
                    else{
                        state.buy_limit_orders.insert(price, vec![self]);
                    }
                }
                state.accounts.insert(account_id, buyer_account);
            },
            Self::SellOrder { account_id, price, amount } => {
                let mut seller_account: Account = state.accounts.remove(&account_id).unwrap();
                let mut to_be_filled = amount;
                match state.highest_buy_price{
                    Some(mut buy) => {
                        while buy >= price && to_be_filled > 0{
                            let mut best_ask_list: Vec<LimitOrder> = state.buy_limit_orders[&buy].clone();
                            for (id, order) in best_ask_list.clone().into_iter().enumerate(){
                                match order{
                                    Self::BuyOrder { account_id, price, amount } => {
                                        let buyer_account: &mut Account = state.accounts.get_mut(&account_id).unwrap();
                                        // check all possible prices in range
                                        if amount < to_be_filled{
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price / 1000_000_000;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price / 1000_000_000;
                                            to_be_filled -= amount;
                                        }
                                        else if amount == to_be_filled{
                                            // remove from list and commit
                                            best_ask_list.remove(id);
                                            // calculate balances
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price / 1000_000_000;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price / 1000_000_000;
                                            to_be_filled = 0;
                                        }
                                        // this could be an "else", but being more explicit improves readabiliy for prototyping
                                        else if amount >= to_be_filled{
                                            seller_account.cspr_balance -= to_be_filled;
                                            seller_account.usdc_balance += to_be_filled * price / 1000_000_000;
                                            buyer_account.cspr_balance += to_be_filled;
                                            buyer_account.usdc_balance -= to_be_filled * price / 1000_000_000;
                                            to_be_filled = 0
                                        }
                                    },
                                    Self::SellOrder { account_id: _, price: _, amount: _ } => {
                                        panic!("Invalid order in sell_limit_orders")
                                    }
                                }
                            }

                            // for testing the max price is set to 1 usdt
                            state.highest_buy_price = None;
                            state.clone().brute_force_lowest_sell_in_range(499_999_999u64, 500_000_000u64);
                            match state.highest_buy_price{
                                Some(b) => {
                                    buy = b;
                                },
                                None => {
                                    break;
                                }
                            }
                        }
                    },
                    None => {
                        println!("[WARNING] There are no Bids for this Asset, your Ask will be placed.");
                    }
                }
                if to_be_filled > 0{
                    // must add this order to the order book, since it was not filled.
                    match state.lowest_sell_price{
                        Some(sell) => {
                            if price < sell{
                                state.lowest_sell_price = Some(price);
                            }
                        },
                        None => {
                            state.lowest_sell_price = Some(price);
                        }
                    }
                    if state.sell_limit_orders.contains_key(&price){
                        let mut price_bound_orderbook = state.sell_limit_orders[&price].clone();
                        price_bound_orderbook.push(self);
                        state.sell_limit_orders.insert(price, price_bound_orderbook);
                    }
                    else{
                        state.sell_limit_orders.insert(price, vec![self]);
                    }
                }
                state.accounts.insert(account_id, seller_account);
            }
        }
    }
}
