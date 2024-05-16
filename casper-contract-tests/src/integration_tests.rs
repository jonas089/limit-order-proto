use crate::test_fixture;
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
    fn place_buy_limit_order(){
        let mut fixture = TestContext::new();
        fixture.approve(fixture.admin, fixture.contract_package_key, U256::from(1000_000_000u64), fixture.cep18_contract_hash);
        fixture.limit_buy(1_000_000_000u64, 1_000_000_000u64, fixture.admin);
    }

    #[test]
    fn place_sell_limit_order(){
        let mut fixture = TestContext::new();
        fixture.limit_sell(fixture.admin, 1_000_000_000_u64, 1_000_000_000_u64, fixture.cep18_contract_hash)
    }

    #[test]
    fn fill_sell_order_instant(){
        let mut fixture: TestContext = TestContext::new();
        fixture.approve(fixture.admin, fixture.contract_package_key, U256::from(1000u64), fixture.cep18_contract_hash);
        fixture.limit_buy(1_000_000_000_u64, 10u64, fixture.admin);
        fixture.limit_sell(fixture.user, 1_000_000_000_u64, 10_000_000_000_u64, fixture.cep18_contract_hash);
        assert_eq!(fixture.cep_balance(fixture.admin.into(), fixture.cep18_contract_hash),U256::from(999990));
        assert_eq!(fixture.cep_balance(fixture.user.into(), fixture.cep18_contract_hash), U256::from(10));
    }

    #[test]
    fn fill_buy_order_instant(){
        let mut fixture: TestContext = TestContext::new();
        fixture.approve(fixture.admin, fixture.contract_package_key, U256::from(1000u64), fixture.cep18_contract_hash);
        fixture.limit_sell(fixture.user, 1_000_000_000_u64, 10_000_000_000_u64, fixture.cep18_contract_hash);
        fixture.limit_buy(1_000_000_000_u64, 10u64, fixture.admin);
        assert_eq!(fixture.cep_balance(fixture.admin.into(), fixture.cep18_contract_hash),U256::from(999990));
        assert_eq!(fixture.cep_balance(fixture.user.into(), fixture.cep18_contract_hash), U256::from(10));
    }

    // todo:
    // 1. mint, approve, place a Buy order
    // 2. write session code to place a Sell order
    // 3. assert balances and check that orders are being filled as expected
}