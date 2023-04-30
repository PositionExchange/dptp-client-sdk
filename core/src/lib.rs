pub mod config;
pub mod contracts;
mod utils;
mod log;
use std::{sync::Arc, rc::Rc, cell::RefCell};

use async_trait::async_trait;
use contracts::vault::Vault;
// use futures::try_join;
// use std::{sync::{Arc, Mutex}};
// use tokio::{task::futures};

use crate::contracts::token::Token;
use contracts::global_fetch::*;
use crate::contracts::vault_logic::VaultLogic;
use ethabi::{ethereum_types::U256};
use log::*;




// use contracts::vault_logic;

#[derive(Debug)]
pub struct Router {
    pub config: config::Config,
    pub vault: Vault,
    pub price_plp_buy : U256,
    pub price_plp_sell : U256
}

#[async_trait(?Send)]
pub trait RouterTrait {
    fn new() -> Self;
    fn initilize(&mut self, chain_id: u64) -> Result<&config::Config, &'static str>;
    fn load_tokens(&self) -> Vec<Token>;
    /// this function will init the account
    fn set_account(&mut self, account: String);
    fn calculate_price_plp(&mut self);
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
                contract_address: config::ContractAddress::default(),
                contract_spender :vec![],
            },
            vault: Vault::default(),
            price_plp_buy : U256::zero(),
            price_plp_sell :U256::zero(),
        }
    }

    fn initilize(&mut self, chain_id:u64) -> Result<&config::Config, &'static str>  {
        self.config = config::load_config(chain_id).unwrap();
        let contract_address = self.config.contract_address.clone();
        self.vault = Vault::new(
            &contract_address.vault.to_lowercase(),
            &contract_address.plp_manager.to_lowercase(),
            &contract_address.plp_token.to_lowercase(),
            &self.config.chain,
            Rc::new(RefCell::new(contract_address))

        );
        for token in self.config.tokens.iter_mut() {
            token.address = token.address.to_lowercase();
        }
        Ok(&self.config)
    }

    fn load_tokens(&self) -> Vec<Token>  {
        self.config.tokens.clone()
    }

    fn set_account(&mut self, account:String) {
        self.config.set_selected_account(account);
    }

    fn calculate_price_plp(&mut self) {
        self.price_plp_buy= self.vault.state.get_plp_price(true);// &Option::from(self.vault.state.get_plp_price(true));
        self.price_plp_sell = self.vault.state.get_plp_price(false); //&Option::from(self.vault.state.get_plp_price(false));
    }


    async fn fetch_data(&mut self) -> anyhow::Result<()> {

        let tokens = self.load_tokens();
        // println!("tokens: ", tokens);
        let tokens = tokio::sync::Mutex::new(tokens);
        // let tokens1 = Arc::clone(&tokens);
        // let tokens2 = Arc::clone(&tokens);
        // let startTime = Instant::now();

        print(format!("RUST:: start fetch data, chain id: {:?}", self.config.chain.chain_id).as_str());
        // TODO move the lock to the function??
        let _tasks = tokio::join![
            async {
                print("task 1 start");
                let mut tokens = tokens.lock().await;
                print("task 1 start after lock");
                let res = self.vault.fetch_token_configuration(&mut tokens).await.expect("task 1 error");
                // if res.is_err() {
                //     print("task 1 error");
                //     return;
                // }
                print("task 1 done");
            },
            async {
                print("task 2 start");
                let mut tokens = tokens.lock().await;
                print("task 2 start after lock");
                self.vault.fetch_vault_info(&mut tokens).await;
                print("task 2 done");
                // print("task 1 done, time: {}", startTime.elapsed().as_millis());
            },
            async {
                print("task 3 start");
                let mut tokens = tokens.lock().await;
                print("task 3 start after lock");
                self.vault.fetch_token_prices(&mut tokens).await;
                print("task 3 done");
                // print("task 2 done, time {}", startTime.elapsed().as_millis());
            },
            async {
                print("task 4 start");
                let mut tokens = tokens.lock().await;
                print("task 4 start after lock");
                self.config.fetch_balances(&mut tokens).await;
                self.config.fetch_allowance(&mut tokens).await;
                print("task 4 done");
                // print("task 3 done, time {}", startTime.elapsed().as_millis());
            },
        ];

        let mut tokens = tokens.lock().await;
        self.vault.fetch_multi_vault_token_variables(&mut tokens).await;
        print(format!("tokens full: {:?}", tokens).as_str());
        // re assign new tokens
        self.config.tokens = tokens.to_vec();

        for token in self.config.tokens.iter_mut() {
            token.calculate_available_liquidity();
        };

        print("all done");

        // let fetch_token_task = tokio::spawn(vault.fetch_token_configuration(&mut tokens));
        // let fetch_token_price_task = tokio::spawn(vault.fetch_token_prices(&mut tokens));
        // tokio::try_join!(fetch_token_task, fetch_token_price_task, fetch_account_balance)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rust_decimal::Decimal;
    use crate::contracts::vault_logic;

    use super::*;

    #[test]
            fn it_works() {
        // 1. load config
        let mut router = Router::new();
        router.initilize(97).unwrap();

        println!("spender {:?}", router.config.contract_spender);
        // let tokens = router.load_tokens();
        // println!("Loaded tokens: {:?}", tokens);
        // assert_eq!(tokens.len(), 3);
        // assert_eq!(tokens[0].symbol, "USDT");
        // assert_eq!(tokens[1].symbol, "BTC");

        // 2. set account
        router.set_account("0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string());
        assert_eq!(router.config.selected_account, Some("0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string()));

        println!("reward_router {}", router.config.contract_address.reward_router);
    }

    #[tokio::test]
    async fn it_works_arb() {
        // 1. load config
        let mut router = Router::new();
        router.initilize(421613).unwrap();

        println!("spender {:?}", router.config.contract_spender);
        router.vault.init_vault_state().await.unwrap();


        // 2. set account
        router.set_account("0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string());
        assert_eq!(router.config.selected_account, Some("0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string()));

        router.fetch_data().await.expect("fetch data failed");


        println!("reward_router {}", router.config.contract_address.reward_router);
    }

    #[tokio::test]
    async fn should_fetch_token_prices() {
        let mut router = Router::new();
        router.initilize(97).unwrap();
        println!("start init_vault_state");
        router.vault.init_vault_state().await.unwrap();

        router.set_account("0xDfbE56f4e2177a498B5C49C7042171795434e7D1".to_string());
        println!("done init_vault_state");



        println!("vault state {:?}", router.vault.state);

        router.calculate_price_plp();
        // println!("price plp buy {}", router.price_plp_buy.unwrap().as_u32());
        router.fetch_data().await.expect("fetch data failed");


        let tokens = router.load_tokens();

        // println!("Loaded tokens: {:?}", tokens);
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
        router.initilize(97).unwrap();
        println!("start init_vault_state");
        router.vault.init_vault_state().await.unwrap();

        router.set_account("0xDfbE56f4e2177a498B5C49C7042171795434e7D1".to_string());
        println!("done init_vault_state");

        router.calculate_price_plp();
        // println!("price plp buy {}", router.price_plp_buy.unwrap().as_u32());
        router.fetch_data().await.expect("fetch data failed");


        // println!(router.price_plp_sell.unwrap());
        let tokens = router.load_tokens();

        // println!("Loaded tokens: {:?}", tokens);

        println!( "available_liquidity {}", tokens[0].available_liquidity.unwrap().to_string());


        // assert_eq!(tokens.len(), 4);
        // assert_eq!(tokens[0].token_weight, Some(100));
        // assert_eq!(tokens[1].token_weight, Some(100));
    }

    #[tokio::test]
    async fn should_fetch_data_with_account_success() {
        let mut router = Router::new();
        router.initilize(97).unwrap();
        let account = "0x1e8b86cd1b420925030fe72a8fd16b47e81c7515".to_string();
        println!("****** set account *****");

        router.set_account(account.clone());
        println!("start fetching account");
        router.fetch_data().await.expect("fetch data in should_fetch_data_with_account_success failure");
        println!("****** done fetch data *****");

        let tokens = router.load_tokens();


        println!("Loaded tokens: {:?}", tokens);



        assert_eq!(tokens.len() >=1, true );
        // epxect token data
        // assert_eq!(tokens[0].token_weight, Some(100));
        // assert_eq!(tokens[1].token_weight, Some(100));
        // assert!(tokens[0].ask_price.clone().expect("No ask price").parsed >= Decimal::from_str(&"1").unwrap());
        // assert!(tokens[1].bid_price.clone().expect("No ask price").parsed >= Decimal::from_str(&"1").unwrap());

        // assert_eq!(tokens[0].get_balance(&account).parse::<f64>().unwrap(), 100.0);
        // assert_eq!(tokens[1].get_balance(&account).parse::<f64>().unwrap(), 10.0);
        // assert_eq!(tokens[0].get_allowance(&account), "0");
        // assert_eq!(tokens[1].get_allowance(&account), "0");
    }

    #[tokio::test]
    async fn get_buy_glp_to_amount (){
        let mut router = Router::new();
        router.initilize(421613).unwrap();
        router.vault.init_vault_state().await.unwrap();

        router.calculate_price_plp();
        // println!("price plp buy {}", router.price_plp_buy.unwrap().as_u32());
        router.fetch_data().await.expect("fetch data failed");


        let tokens  = router.load_tokens();
        println!("&token[0]: {} {:?}",  &tokens[0].min_price.unwrap().parsed, &tokens[0].symbol);
        let (amount, fee) = router.vault.state.get_buy_glp_to_amount(
            &U256::from_dec_str("10000000").unwrap(),
            &tokens[0] );
        println!("amount: {}", amount);
        println!("fee: {}", fee);

        println!("****************************************************************");

        println!("&token[0]: {} ", &tokens[2].symbol);
        let (amount, fee) = router.vault.state.get_buy_glp_to_amount(
            &U256::from_dec_str("100000000").unwrap(),
            &tokens[2] );
        println!("amount: {}", amount);



    }
    #[tokio::test]
    async fn get_buy_glp_from_amount (){
        let mut router = Router::new();
        router.initilize(421613).unwrap();
        router.vault.init_vault_state().await.unwrap();

        router.calculate_price_plp();
        // println!("price plp buy {}", router.price_plp_buy.unwrap().as_u32());
        router.fetch_data().await.expect("fetch data failed");


        let tokens  = router.load_tokens();
        println!("&token[0]: {}",  &tokens[0].min_price.unwrap().parsed);
        let (amount, fee) = router.vault.state.get_buy_glp_from_amount(
            U256::from_dec_str("34688316279096605298").unwrap(),
            &tokens[0] );
        println!("amount: {}", amount);
        println!("fee: {}", fee);



        println!("****************************************************************");

        println!("&token[0]: {} ", &tokens[2].symbol);
        let (amount, fee) = router.vault.state.get_buy_glp_from_amount(
            U256::from_dec_str("101045757422875282155013").unwrap(),
            &tokens[2] );
        println!("amount: {}", amount);


    }


    #[tokio::test]
    async fn get_sell_glp_to_amount (){
        let mut router = Router::new();
        router.initilize(421613).unwrap();
        router.vault.init_vault_state().await.unwrap();

        router.calculate_price_plp();
        // println!("price plp buy {}", router.price_plp_buy.unwrap().as_u32());
        router.fetch_data().await.expect("fetch data failed");


        let tokens  = router.load_tokens();
        println!("&token[0]: {}",  &tokens[0].min_price.unwrap().parsed);
        let (amount, fee) = router.vault.state.get_sell_glp_to_amount(
            U256::from_dec_str("34688316279096605298").unwrap(),
            &tokens[0] );
        println!("amount: {}", amount);
        println!("fee: {}", fee);

        println!("****************************************************************");

        println!("&token[0]: {} ", &tokens[2].symbol);
        let (amount, fee) = router.vault.state.get_sell_glp_to_amount(
            U256::from_dec_str("101045757422875282155013").unwrap(),
            &tokens[2] );
        println!("amount: {}", amount);



    }
    #[tokio::test]
    async fn get_sell_glp_from_amount (){
        let mut router = Router::new();
        router.initilize(421613).unwrap();
        router.vault.init_vault_state().await.unwrap();

        router.calculate_price_plp();
        // println!("price plp buy {}", router.price_plp_buy.unwrap().as_u32());
        router.fetch_data().await.expect("fetch data failed");


        let tokens  = router.load_tokens();
        println!("&token[0]: {}",  &tokens[0].min_price.unwrap().parsed);
        let (amount, fee) = router.vault.state.get_sell_glp_from_amount(
            U256::from_dec_str("10000000").unwrap(),
            &tokens[0] );
        println!("amount: {}", amount);
        println!("fee: {}", fee);


        println!("****************************************************************");

        println!("&token[0]: {} ", &tokens[2].symbol);
        let (amount, fee) = router.vault.state.get_sell_glp_from_amount(
            U256::from_dec_str("100000000").unwrap(),
            &tokens[2] );
        println!("amount: {}", amount);


    }

    #[tokio::test]
    async fn should_switch_chain_success() {
        let mut router = Router::new();
        router.initilize(421613).unwrap();
        router.vault.init_vault_state().await.unwrap();
        println!("config {:?}", router.config);

        router.initilize(97).unwrap();
        router.vault.init_vault_state().await.unwrap();
        println!("config {:?}", router.config);
    }

}
