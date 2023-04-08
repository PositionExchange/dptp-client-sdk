use async_trait::async_trait;
use ethabi::ethereum_types::U256;

use crate::config::Config;
use super::multicall::*;


#[async_trait]
trait GlobalFetch {
    async fn fetch_balances(&mut self);
}

#[async_trait]
impl GlobalFetch for Config {
    async fn fetch_balances(&mut self) {
        if(self.selected_account.is_none()) {
            return;
        }
        let account = self.selected_account.clone().unwrap();
        let calls: Vec<_> = self.tokens.iter().map(|token| {
            let (call_address, data) = token.build_balance_of_call(&account);
            (call_address, data)
        }).collect();
        let results = self.chain.execute_multicall(calls, include_str!("../../abi/erc20.json").to_string(), "balanceOf").await.unwrap();
        for (token, result) in self.tokens.iter_mut().zip(results) {
            let balance = result[0].clone().into_uint().expect("failed to parse balance");
            token.update_balance(&account, balance);
        }
    }
}

#[cfg(test)]

mod tests {
    use crate::config::load_config;

    const FAKE_ADDRESS: &str = "0x1e8b86cd1b420925030fe72a8fd16b47e81c7515";

    use super::*;
    #[tokio::test]
    async fn should_not_update_when_no_account_selected() {
        let mut config = load_config(97).unwrap();
        config.fetch_balances().await;
        assert_eq!(config.tokens[0].get_balance(&FAKE_ADDRESS.to_string()), "0");
    }

    #[tokio::test]
    async fn should_update_when_account_is_selected() {
        let mut config = load_config(97).unwrap();
        config.set_selected_account(FAKE_ADDRESS.to_string());
        config.fetch_balances().await;
        let token0Balance = config.tokens[0].get_balance(&FAKE_ADDRESS.to_string());
        assert!(token0Balance.parse::<U256>().unwrap().gt(&U256::from(0)), "token 0 balance ({}) should be greater than 0", token0Balance);
        assert_eq!(config.tokens[1].get_balance(&FAKE_ADDRESS.to_string()), "10000000000000000000");
    }

}
