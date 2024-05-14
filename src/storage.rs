use std::collections::BTreeMap;
use crate::market::LimitOrder;

#[derive(Debug, Clone)]
pub struct MemoryState{
    pub buy_limit_orders: BTreeMap<u64, Vec<LimitOrder>>,
    pub sell_limit_orders: BTreeMap<u64, Vec<LimitOrder>>,
    pub accounts: BTreeMap<u64, Account>,
    pub lowest_sell_price: Option<u64>,
    pub highest_buy_price: Option<u64>
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