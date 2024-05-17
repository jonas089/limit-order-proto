use alloc::vec::Vec;
use casper_types::account::AccountHash;
use serde::{Deserialize, Serialize};

// suggestion: Enum for Orders
// I chose not to use an enum to reduce complexity in singular functions
// for prototyping.
// Once the code works and tests pass, this can be reconsidered and restructured.

#[derive(Serialize, Deserialize, Clone)]
pub struct LimitOrderBuyList{
    pub limit_orders: Vec<LimitOrderBuy>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LimitOrderBuy{
    pub amount: u64,
    pub price: u64,
    pub account: AccountHash
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LimitOrderSellList{
    pub limit_orders: Vec<LimitOrderSell>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LimitOrderSell{
    pub amount: u64,
    pub price: u64,
    pub account: AccountHash
}
