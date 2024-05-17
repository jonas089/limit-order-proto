#[cfg(test)]
mod tests {
    use casper_types::U256;
    use crate::test_fixture::TestContext;
    
    #[test]
    fn should_install_contracts() {
        let _fixture = TestContext::new();
    }

    #[test]
    fn mint_cep18(){
        let mut fixture = TestContext::new();
        fixture.mint(U256::from(1_000u64), fixture.user);
        assert_eq!(U256::from(1_000u64), fixture.cep_balance(fixture.user.into(), fixture.cep18_contract_hash))
    }

    #[test]
    fn approve_cep18(){
        let mut fixture = TestContext::new();
        fixture.approve(fixture.admin, fixture.user.into(), U256::from(1000_u64), fixture.cep18_contract_hash)
    }

    #[test]
    fn buy_cspr_with_fixed_point_arithmetic(){
        let mut fixture: TestContext = TestContext::new();
        fixture.approve(fixture.admin, fixture.contract_package_key, U256::from(1_000_000_000_000u64), fixture.cep18_contract_hash);
        // using fixed point arithmetic for this test (similar to production defi)
        // price represents the amount of CSPR for 1_000_000_000:= 1 USDC, 
        // precision is hard-coded to 9 decimals for this prototype.
        // to calculate usdc amount: amount / price
        // to calculate cspr amount: amount * price
        let orders_to_be_placed: u64 = 10;
        let order_amount: u64 = 1_000_000_000; // request to sell 1 CSPR per order
        let mut current_price: u64 = 1_000_000_000; // start with 1 USDC : 1 CSPR
        let price_interval: u64 = 1_000_000_000; // increase the value of USDC by 1 against CSPR for each round
        for _i_u64 in 0_u64..orders_to_be_placed{
            fixture.limit_sell(fixture.user, current_price, order_amount, fixture.cep18_contract_hash);
            current_price += price_interval;
        };
        fixture.limit_buy(2_000_000_000_u64, 2_000_000_000_u64, fixture.admin);
        // Buyer gets an offer for 1 CSPR at 1 USDC and for 1 CSPR at 2 USDC => The seller should now have 3 USDC in their account.
        assert_eq!(fixture.cep_balance(fixture.user.into(), fixture.cep18_contract_hash), U256::from(3_000_000_000_u64));
    }

    #[test]
    fn sell_cspr_with_fixed_point_arithmetic(){
        let mut fixture: TestContext = TestContext::new();
        fixture.approve(fixture.admin, fixture.contract_package_key, U256::from(1000_000_000_000u64), fixture.cep18_contract_hash);
        // using fixed point arithmetic for this test (similar to production defi)
        // price represents the amount of CSPR for 1_000_000_000:= 1 USDC, 
        // precision is hard-coded to 9 decimals for this prototype.
        // to calculate usdc amount: amount / price
        // to calculate cspr amount: amount * price
        let orders_to_be_placed: u64 = 10;
        let order_amount: u64 = 1_000_000_000; // request to buy 1 CSPR per order
        let mut current_price: u64 = 1_000_000_000; // start with 1 CSPR : 1 USDC
        let price_interval: u64 = 1_000_000_000; // increase the value of CSPR by 1 against USDC for each round
        for _i_u64 in 0_u64..orders_to_be_placed{
            fixture.limit_buy(current_price, order_amount, fixture.admin);
            current_price += price_interval;
        };

        // sell 2 CSPR at a min price of 2 CSPR per USDC
        fixture.limit_sell(fixture.user, 2_000_000_000_u64, 2_000_000_000_u64, fixture.cep18_contract_hash);
        assert_eq!(fixture.cep_balance(fixture.user.into(), fixture.cep18_contract_hash), U256::from(19_000_000_000_u64));
    }

    #[test]
    fn buy_cspr_partial_fill(){
        let mut fixture: TestContext = TestContext::new();
        fixture.approve(fixture.admin, fixture.contract_package_key, U256::from(1000_000_000_000u64), fixture.cep18_contract_hash);
        let orders_to_be_placed: u64 = 10;
        let order_amount: u64 = 1_000_000_000; // request to sell 1 CSPR per order
        let mut current_price: u64 = 1_000_000_000; // start with 1 USDC : 1 CSPR
        let price_interval: u64 = 1_000_000_000; // increase the value of USDC by 1 against CSPR for each round
        for _i_u64 in 0_u64..orders_to_be_placed{
            fixture.limit_sell(fixture.user, current_price, order_amount, fixture.cep18_contract_hash);
            current_price += price_interval;
        };
        // highest price the CSPR buyer is willig to accept is 2 USDC per CSPR for a total of 0.5 CSPR
        fixture.limit_buy(2_000_000_000_u64, 500_000_000_u64, fixture.admin);
        assert_eq!(fixture.cep_balance(fixture.user.into(), fixture.cep18_contract_hash), U256::from(500_000_000_u64));
        fixture.limit_buy(2_000_000_000_u64, 500_000_000_u64, fixture.admin);
        assert_eq!(fixture.cep_balance(fixture.user.into(), fixture.cep18_contract_hash), U256::from(1_000_000_000u64));
        fixture.limit_buy(2_000_000_000_u64, 1_000_000_000_u64, fixture.admin);
        assert_eq!(fixture.cep_balance(fixture.user.into(), fixture.cep18_contract_hash), U256::from(3_000_000_000u64));
        // price: amount of USDC for 1 CSPR
    }

    #[test]
    fn sell_cspr_partial_fill(){
        let mut fixture: TestContext = TestContext::new();
        fixture.approve(fixture.admin, fixture.contract_package_key, U256::from(1000_000_000_000u64), fixture.cep18_contract_hash);
        let orders_to_be_placed: u64 = 10;
        let order_amount: u64 = 1_000_000_000; // request to buy 1 CSPR per order
        let mut current_price: u64 = 1_000_000_000; // start with 1 CSPR : 1 USDC
        let price_interval: u64 = 1_000_000_000; // increase the value of CSPR by 1 against USDC for each round
        for _i_u64 in 0_u64..orders_to_be_placed{
            fixture.limit_buy(current_price, order_amount, fixture.admin);
            current_price += price_interval;
        };
        // selling only 0.5 CSPR
        fixture.limit_sell(fixture.user, 2_000_000_000_u64, 500_000_000_u64, fixture.cep18_contract_hash);
        assert_eq!(fixture.cep_balance(fixture.user.into(), fixture.cep18_contract_hash), U256::from(5_000_000_000_u64));
        fixture.limit_sell(fixture.user, 2_000_000_000_u64, 500_000_000_u64, fixture.cep18_contract_hash);
        assert_eq!(fixture.cep_balance(fixture.user.into(), fixture.cep18_contract_hash), U256::from(10_000_000_000_u64));
    }

    #[test]
    fn stress_test_orderbook(){
        let mut fixture: TestContext = TestContext::new();
        fixture.approve(fixture.admin, fixture.contract_package_key, U256::from(1000_000_000_000u64), fixture.cep18_contract_hash);
        let order_count: u64 = 1000;
        let order_amount: u64 = 1_000_000_000; // request to buy 1 CSPR per order
        let mut current_price: u64 = 1_000_000_000; // start with 1 CSPR : 1 USDC
        let price_interval: u64 = 1_000_000_000; // increase the value of CSPR by 1 against USDC for each round
        for _i_u64 in 0_u64..order_count{
            fixture.limit_buy(current_price, order_amount, fixture.admin);
            current_price += price_interval;
        }
        println!("Done placing orders.");
        fixture.limit_sell(fixture.user, 2_000_000_000_u64, 2_000_000_000_u64, fixture.cep18_contract_hash);
    }
}

// 1000 orders:
// test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 141.86s

// 2000 orders:
// 