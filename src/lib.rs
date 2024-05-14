mod storage;
mod market;
use std::collections::{BTreeMap, HashMap};

use storage::{MemoryState, Account};
use market::LimitOrder;

pub fn test_limit_order_simple(){
    let seller_account: Account = Account{
        // 1000 cspr
        cspr_balance: 1_000_000_000_000,
        usdc_balance: 0
    };

    let buyer_account: Account = Account{
        cspr_balance: 0,
        // 500 usdc
        usdc_balance: 500_000_000_000
    };

    let mut accounts: BTreeMap<u64, Account> = BTreeMap::new();
    accounts.insert(0u64, seller_account);
    accounts.insert(1u64, buyer_account);

    let mut state = MemoryState{
        buy_limit_orders: BTreeMap::new(),
        sell_limit_orders: BTreeMap::new(),
        accounts,
        lowest_sell_price: None,
        highest_buy_price: None
    };

    // sell 1 cspr at 0.5 USDC / cspr
    let sell_order: LimitOrder = LimitOrder::SellOrder { account_id: 0, price: (1_000_000_000 / 2), amount: 1_000_000_000};
    // buy 1 cspr at 0.5 USDC / cspr
    let buy_order: LimitOrder = LimitOrder::BuyOrder { account_id: 1, price: (1_000_000_000 / 2), amount: 1_000_000_000 };
    // execute both orders
    sell_order.execute_order(&mut state);
    buy_order.execute_order(&mut state);
    // check resulting balances cspr
    let seller_cspr_balance: u64 = state.accounts[&0].cspr_balance;
    let buyer_cspr_balance: u64 = state.accounts[&1].cspr_balance;

    assert_eq!(seller_cspr_balance, 999_000_000_000);
    assert_eq!(buyer_cspr_balance, 1_000_000_000);
    // check resulting balances usdc
}

#[test]
fn run_basic_test(){
    test_limit_order_simple();
}