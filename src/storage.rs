use std::{collections::HashMap, os::macos::raw::stat};

#[derive(Debug, Clone)]
pub struct MemoryState{
    pub buy_limit_orders: HashMap<u64, Vec<LimitOrder>>,
    pub sell_limit_orders: HashMap<u64, Vec<LimitOrder>>,
    pub accounts: HashMap<u64, Account>,
    pub lowest_sell_price: Option<u64>,
    pub highest_buy_price: Option<u64>
}

impl MemoryState{
    pub fn brute_force_lowest_sell_in_range(mut self, lower_bound_motes: u64, upper_bound_motes: u64){
        for i in lower_bound_motes..upper_bound_motes{
            if self.sell_limit_orders.contains_key(&i){
                self.lowest_sell_price = Some(i)
            }
        }
        self.lowest_sell_price = None
    }

    pub fn brute_force_highest_buy_in_range(mut self, lower_bount_motes: u64, upper_bound_motes: u64){
        for i in (lower_bount_motes..upper_bound_motes).rev(){
            if self.buy_limit_orders.contains_key(&i){
                self.highest_buy_price = Some(i)
            }
        }
        self.highest_buy_price = None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LimitOrder {
    BuyOrder { account_id: u64, price: u64, amount: u64 },
    SellOrder { account_id: u64, price: u64, amount: u64 },
}

impl LimitOrder{
    fn execute_order(self, mut state: MemoryState){
        match self {
            Self::BuyOrder { account_id, price, amount } => {
                let mut buyer_account: Account = state.accounts[&account_id];
                let mut to_be_filled = amount;
                match state.lowest_sell_price{
                    Some(sell) => {
                        while sell <= price && to_be_filled > 0{
                            let mut best_bid_list: Vec<LimitOrder> = state.sell_limit_orders[&sell].clone();
                            for (id, order) in best_bid_list.clone().into_iter().enumerate(){
                                match order{
                                    Self::SellOrder { account_id, price, amount } => {
                                        let mut seller_account: Account = state.accounts[&account_id];
                                        // check all possible prices in range
                                        if amount < to_be_filled{
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price;
                                            to_be_filled -= amount;
                                        }
                                        else if amount == to_be_filled{
                                            // remove from list and commit
                                            best_bid_list.remove(id);
                                            // calculate balances
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price;
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
                            // must find next best price, list is now empty
                            // todo!("Implement an efficient price discovery algorithm")

                            // this is probably a bad algorithm.
                            state.clone().brute_force_lowest_sell_in_range(0u64, 1000u64)
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
                }
            },
            Self::SellOrder { account_id, price, amount } => {
                let mut seller_account: Account = state.accounts[&account_id];
                let mut to_be_filled = amount;
                match state.highest_buy_price{
                    Some(buy) => {
                        while buy >= price && to_be_filled > 0{
                            let mut best_ask_list: Vec<LimitOrder> = state.buy_limit_orders[&buy].clone();
                            for (id, order) in best_ask_list.clone().into_iter().enumerate(){
                                match order{
                                    Self::BuyOrder { account_id, price, amount } => {
                                        let mut buyer_account: Account = state.accounts[&account_id];
                                        // check all possible prices in range
                                        if amount < to_be_filled{
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price;
                                            to_be_filled -= amount;
                                        }
                                        else if amount == to_be_filled{
                                            // remove from list and commit
                                            best_ask_list.remove(id);
                                            // calculate balances
                                            seller_account.cspr_balance -= amount;
                                            seller_account.usdc_balance += amount * price;
                                            buyer_account.cspr_balance += amount;
                                            buyer_account.usdc_balance -= amount * price;
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
                                    Self::SellOrder { account_id: _, price: _, amount: _ } => {
                                        panic!("Invalid order in sell_limit_orders")
                                    }
                                }
                            }
                            // must find next best price, list is now empty
                            state.clone().brute_force_highest_buy_in_range(0u64, 1000u64)
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
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarketOrder{
    pub side: String, // "buy", "sell"
    pub price: u64,
    pub amount: u64
}

#[derive(Debug, Clone, Copy)]
pub struct Account{
    pub cspr_balance: u64,
    pub usdc_balance: u64
}