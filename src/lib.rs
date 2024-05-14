pub mod storage;
use storage::{MemoryState, LimitOrder, LimitOrder::BuyOrder, LimitOrder::SellOrder, MarketOrder, Account};

struct MarketMaker{
    state: MemoryState
}
impl MarketMaker{
    fn submit_limit_order(self, order: LimitOrder){
        match order{
            BuyOrder{account_id, price, amount} => {

            },
            SellOrder{account_id, price, amount} => {

            }
        }
    }
}