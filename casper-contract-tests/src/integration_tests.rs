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
        assert_eq!(U256::from(1_000u64), fixture.cep_balance(fixture.user.into(), "cep18_contract_hash_usdc_contract"))
    }
}