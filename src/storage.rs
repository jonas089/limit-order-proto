use std::collections::HashMap;
use crate::market::LimitOrder;

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
        println!("Enter brute force!");
        for i in lower_bound_motes..upper_bound_motes{
            if self.sell_limit_orders.contains_key(&i){
                self.lowest_sell_price = Some(i)
            }
        }
        self.lowest_sell_price = None
    }

    pub fn brute_force_highest_buy_in_range(mut self, lower_bount_motes: u64, upper_bound_motes: u64){
        println!("[WARNING] Enter brute force!");
        for i in (lower_bount_motes..upper_bound_motes).rev(){
            if self.buy_limit_orders.contains_key(&i){
                self.highest_buy_price = Some(i)
            }
        }
        self.highest_buy_price = None
    }
}

/* #[derive(Debug, Clone)]
pub struct MarketOrder{
    pub side: String, // "buy", "sell"
    pub price: u64,
    pub amount: u64
}*/

#[derive(Debug, Clone, Copy)]
pub struct Account{
    pub cspr_balance: u64,
    pub usdc_balance: u64
}