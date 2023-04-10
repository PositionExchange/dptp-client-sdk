pub mod config;
mod contracts;
use async_trait::async_trait;
use contracts::vault::Vault;
// use futures::try_join;
use std::{sync::{Arc, Mutex}, time::Instant};
// use tokio::{task::futures};

use crate::contracts::token::Token;
use contracts::global_fetch::*;

#[derive(Debug)]

pub struct Router {
    pub config: config::Config,
}

#[async_trait(?Send)]
pub trait RouterTrait {
    fn new() -> Self;
    fn load_config(&mut self, chain_id: u64) -> Result<&config::Config, &'static str>;
    fn load_tokens(&self) -> Vec<Token>;
    /// this function will init the account
    fn set_account(&mut self, account: String);
    async fn fetch_data(&mut self) -> anyhow::Result<()>;
}

#[async_trait(?Send)]
impl RouterTrait for Router {
    fn new() -> Self {
        Self {
            config: config::Config {
                selected_account: None,
                chain: config::Chain {
                    chain_id: 0,
                    rpc_urls: vec![],
                    multicall_address: "".to_string(),
                },
                tokens: vec![],
                contract_address: config::ContractAddress {
                    vault: "".to_string(),
                },
            },
        }
    }

    async fn fetch_data(&mut self) -> anyhow::Result<()> {
        let vault = Vault::new(
            &self.config.contract_address.vault,
            &self.config.chain
        );
        let tokens = self.load_tokens();
        let tokens = tokio::sync::Mutex::new(tokens);
        // let tokens1 = Arc::clone(&tokens);
        // let tokens2 = Arc::clone(&tokens);
        let startTime = Instant::now();

        // TODO move the lock to the function??
        let _tasks = tokio::join![
            async {
                println!("task 1 start");
                let mut tokens = tokens.lock().await;
                println!("task 1 start after lock");
                vault.fetch_token_configuration(&mut tokens).await;
                println!("task 1 done, time: {}", startTime.elapsed().as_millis());
            },
            async {
                println!("task 2 start");
                let mut tokens = tokens.lock().await;
                println!("task 2 start after lock");
                vault.fetch_token_prices(&mut tokens).await;
                println!("task 2 done, time {}", startTime.elapsed().as_millis());
            },
            async {
                println!("task 3 start");
                let mut tokens = tokens.lock().await;
                println!("task 3 start after lock");
                self.config.fetch_balances(&mut tokens).await;
                println!("task 3 done, time {}", startTime.elapsed().as_millis());
            },
        ];

        // re assign new tokens
        self.config.tokens = tokens.lock().await.to_vec();

        // let fetch_token_task = tokio::spawn(vault.fetch_token_configuration(&mut tokens));
        // let fetch_token_price_task = tokio::spawn(vault.fetch_token_prices(&mut tokens));
        // tokio::try_join!(fetch_token_task, fetch_token_price_task, fetch_account_balance)?;
        Ok(())
    }

    fn load_config(&mut self,chain_id:u64) -> Result<&config::Config, &'static str>  {
        self.config = config::load_config(chain_id).unwrap();
        Ok(&self.config)
    }

    fn load_tokens(&self) -> Vec<Token>  {
        self.config.tokens.clone()
    }


    fn set_account(&mut self, account:String) {
        self.config.set_selected_account(account);
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn it_works() {
        // 1. load config
        let mut router = Router::new();
        router.load_config(97).unwrap();
        let tokens = router.load_tokens();
        println!("Loaded tokens: {:?}", tokens);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].name, "USDT");
        assert_eq!(tokens[1].name, "BTC");

        // 2. set account
        router.set_account("0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string());
        assert_eq!(router.config.selected_account, Some("0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string()));
    }

    #[test]
    fn should_call_load_config_before_load_tokens() {
        let router = Router::new();
        let tokens = router.load_tokens();
        println!("Loaded tokens: {:?}", tokens);
        assert_eq!(tokens.len(), 0);
    }

    #[tokio::test]
    async fn should_fetch_data_without_account_success() {
        let mut router = Router::new();
        router.load_config(97).unwrap();
        router.fetch_data().await;
        let tokens = router.load_tokens();

        println!("Loaded tokens: {:?}", tokens);

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_weight, Some(100));
        assert_eq!(tokens[1].token_weight, Some(100));
    }

    #[tokio::test]
    async fn should_fetch_data_with_account_success() {
        let mut router = Router::new();
        router.load_config(97).unwrap();
        let account = "0x1e8b86cd1b420925030fe72a8fd16b47e81c7515".to_string();
        router.set_account(account.clone());
        println!("start fetching account");
        router.fetch_data().await;
        let tokens = router.load_tokens();

        println!("Loaded tokens: {:?}", tokens);

        assert_eq!(tokens.len(), 2);
        // epxect token data
        assert_eq!(tokens[0].token_weight, Some(100));
        assert_eq!(tokens[1].token_weight, Some(100));
        assert!(tokens[0].ask_price.clone().expect("No ask price").parsed >= Decimal::from_str(&"1").unwrap());
        assert!(tokens[1].bid_price.clone().expect("No ask price").parsed >= Decimal::from_str(&"1").unwrap());

        assert_eq!(tokens[0].get_balance(&account).parse::<f64>().unwrap(), 100.0);
        assert_eq!(tokens[1].get_balance(&account).parse::<f64>().unwrap(), 10.0);
        assert_eq!(tokens[0].get_allowance(&account), "0");
        assert_eq!(tokens[1].get_allowance(&account), "0");
    }
}
