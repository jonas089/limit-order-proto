use alloc::{borrow::ToOwned, collections::BTreeMap, vec::Vec};
use casper_types::{account::{Account, AccountHash}, bytesrepr::ToBytes, runtime_args, CLType, CLTyped, ContractHash, Key, RuntimeArgs, URef, U512};
use serde::{Deserialize, Serialize};

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
