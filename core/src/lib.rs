mod config;
mod contracts;
use async_trait::async_trait;

use crate::contracts::token::Token;
use wasm_bindgen::prelude::*;


#[derive(Debug)]

pub struct Router {
    pub config: config::Config,
}

#[async_trait]
pub trait RouterTrait {
    fn new() -> Self;
    fn load_config(&mut self, chain_id: u64) -> Result<&config::Config, &'static str>;
    fn load_tokens(&self) -> Vec<Token>;
    /// this function will init the account
    fn set_account(&mut self, account: String);
    async fn fetch_data(&self) -> anyhow::Result<()>;
}

#[async_trait]
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
            },
        }
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

    async fn fetch_data(&self) -> anyhow::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // 1. load config
        let mut router = Router::new();
        router.load_config(97).unwrap();
        let tokens = router.load_tokens();
        println!("Loaded tokens: {:?}", tokens);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].name, "Test");
        assert_eq!(tokens[1].name, "Test 2");

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
        assert_eq!(tokens[0].token_weight, Some(1000));
        assert_eq!(tokens[1].token_weight, Some(1000));
    }

    #[tokio::test]
    async fn should_fetch_data_with_account_success() {
        let mut router = Router::new();
        router.load_config(97).unwrap();
        let account = "".to_string();
        router.set_account(account.clone());
        router.fetch_data().await;
        let tokens = router.load_tokens();

        println!("Loaded tokens: {:?}", tokens);

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].get_balance(&account), "1000");
        assert_eq!(tokens[1].get_balance(&account), "0");
        assert_eq!(tokens[0].get_allowance(&account), "0");
        assert_eq!(tokens[1].get_allowance(&account), "0");
    }
}
