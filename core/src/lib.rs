pub mod config;
pub mod contracts;
mod log;
mod utils;
use std::{cell::RefCell, rc::Rc, sync::Arc};

use async_trait::async_trait;
use contracts::vault::Vault;
use tokio::{spawn, sync::Mutex};
use instant::Instant;
// use futures::try_join;
// use std::{sync::{Arc, Mutex}};
// use tokio::{task::futures};

use crate::contracts::token::Token;
use crate::contracts::types::{VaultArc, TokensArc};
use crate::contracts::vault_logic::VaultLogic;
use contracts::global_fetch::*;
use ethabi::ethereum_types::U256;
use log::*;

// use contracts::vault_logic;

#[derive(Debug)]
pub struct Router {
    pub config: config::Config,
    pub vault: Vault,
    pub price_plp_buy: U256,
    pub price_plp_sell: U256,
}

#[async_trait(?Send)]
pub trait RouterTrait {
    fn new() -> Self;
    fn initilize(&mut self, chain_id: u64) -> Result<&config::Config, &'static str>;
    fn load_tokens(&self) -> Vec<Token>;
    /// this function will init the account
    fn set_account(&mut self, account: String);
    fn calculate_price_plp(&mut self);
    async fn fetch_balance(&mut self) -> anyhow::Result<()>;
    async fn fetch_vault(&mut self) -> anyhow::Result<()>;
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
                contract_spender: vec![],
            },
            vault: Vault::default(),
            price_plp_buy: U256::zero(),
            price_plp_sell: U256::zero(),
        }
    }

    fn initilize(&mut self, chain_id: u64) -> Result<&config::Config, &'static str> {
        self.config = config::load_config(chain_id).unwrap();
        let contract_address = self.config.contract_address.clone();
        self.vault = Vault::new(
            &contract_address.vault.to_lowercase(),
            &contract_address.plp_manager.to_lowercase(),
            &contract_address.plp_token.to_lowercase(),
            &self.config.chain,
            Rc::new(RefCell::new(contract_address)),
        );
        for token in self.config.tokens.iter_mut() {
            token.address = token.address.to_lowercase();
        }
        Ok(&self.config)
    }

    fn load_tokens(&self) -> Vec<Token> {
        self.config.tokens.clone()
    }

    fn set_account(&mut self, account: String) {
        self.config.set_selected_account(account);
    }

    fn calculate_price_plp(&mut self) {
        self.price_plp_buy = self.vault.state.get_plp_price(true); // &Option::from(self.vault.state.get_plp_price(true));
        self.price_plp_sell = self.vault.state.get_plp_price(false); //&Option::from(self.vault.state.get_plp_price(false));
    }

    async fn fetch_balance(&mut self) -> anyhow::Result<()>{

        let tokens = self.load_tokens();
        let tokens = Arc::new(tokio::sync::RwLock::new(tokens));

        async fn fetch_user_info(
            tokens: TokensArc,
            config: Arc<tokio::sync::RwLock<config::Config>>,
        ) -> anyhow::Result<()> {
            tokio::join![
                async {
                    let _config = config.read().await;
                    p!("task 5 start fetch balances");
                    _config.fetch_balances(&tokens).await;
                },
                async {
                    let _config = config.read().await;
                    p!("task 5 start fetch allowance");
                    _config.fetch_allowance(&tokens).await;
                },
            ];
            Ok(())
        }
        let config = Arc::new(tokio::sync::RwLock::new(self.config.clone()));

        let _tasks = tokio::try_join![
            fetch_user_info(Arc::clone(&tokens), config),
        ];

        print("all done");

        let tokens = tokens.read().await;

        print(format!("tokens full: {:?}", tokens).as_str());
        // re assign new tokens
        self.config.tokens = tokens.to_vec();
        print("all done");
        Ok(())

    }

    async fn fetch_vault(&mut self) -> anyhow::Result<()> {
        let tokens = self.load_tokens();
        // p!("tokens: ", tokens);
        let tokens = Arc::new(tokio::sync::RwLock::new(tokens));
        let vault = Arc::new(tokio::sync::RwLock::new(self.vault.clone()));
        // let tokens1 = Arc::clone(&tokens);
        // let tokens2 = Arc::clone(&tokens);
        let startTime = Instant::now();

        print(
            format!(
                "RUST:: start fetch data, chain id: {:?}",
                self.config.chain.chain_id
            )
                .as_str(),
        );

        async fn fetch_token_configuration(
            tokens: TokensArc,
            vault: VaultArc,
        ) -> anyhow::Result<()> {
            vault.read().await.fetch_token_configuration(tokens)
                .await;

            Ok(())
        }
        async fn fetch_vault_info(
            tokens: TokensArc,
            vault: VaultArc,
        ) -> anyhow::Result<()> {
            vault.read().await.fetch_vault_info(tokens).await;
            print("task 2 done");
            Ok(())
        }
        async fn fetch_token_prices(
            tokens: TokensArc,
            vault: VaultArc
        ) -> anyhow::Result<()> {
            vault.read().await.fetch_token_prices(tokens).await;
            print("task 3 done");
            Ok(())
        }

        async fn fetch_multi_vault_token_variables(
            tokens: TokensArc,
            vault: VaultArc,
        ) -> anyhow::Result<()> {
            let vault = vault.read().await;
            vault
                .fetch_multi_vault_token_variables(tokens)
                .await;
            print("task 4 done");
                Ok(())
        }
        let config = Arc::new(tokio::sync::RwLock::new(self.config.clone()));

        let _tasks = tokio::try_join![
            fetch_token_configuration(Arc::clone(&tokens), Arc::clone(&vault)),
            fetch_vault_info(Arc::clone(&tokens), Arc::clone(&vault)),
            fetch_token_prices(Arc::clone(&tokens), Arc::clone(&vault)),
            fetch_multi_vault_token_variables(Arc::clone(&tokens), Arc::clone(&vault)),
        ];

        print("all done");
        p!("all done, time: {}", startTime.elapsed().as_millis());

        let tokens = tokens.read().await;
        // self.vault
        //     .fetch_multi_vault_token_variables(&mut tokens)
        //     .await;
        print(format!("tokens full: {:?}", tokens).as_str());
        // re assign new tokens
        self.config.tokens = tokens.to_vec();

        for token in self.config.tokens.iter_mut() {
            token.calculate_available_liquidity();
        }

        print("all done");

        Ok(())
    }


    async fn fetch_data(&mut self) -> anyhow::Result<()> {
        let tokens = self.load_tokens();
        // p!("tokens: ", tokens);
        let tokens = Arc::new(tokio::sync::RwLock::new(tokens));
        let vault = Arc::new(tokio::sync::RwLock::new(self.vault.clone()));
        // let tokens1 = Arc::clone(&tokens);
        // let tokens2 = Arc::clone(&tokens);
        let startTime = Instant::now();

        print(
            format!(
                "RUST:: start fetch data, chain id: {:?}",
                self.config.chain.chain_id
            )
            .as_str(),
        );

        async fn fetch_token_configuration(
            tokens: TokensArc,
            vault: VaultArc,
            startTime: Instant,
        ) -> anyhow::Result<()> {
            p!("task 1 start after lock, elapsed: {:?}", startTime.elapsed());
            vault.read().await.fetch_token_configuration(tokens)
                .await;
            // if res.is_err() {
            //     print("task 1 error");
            //     return;
            // }
            p!("task 1 done, time {}", startTime.elapsed().as_millis());
            Ok(())
        }
        async fn fetch_vault_info(
            tokens: TokensArc,
            vault: VaultArc,
            startTime: Instant,
        ) -> anyhow::Result<()> {
            p!("task 2 start after lock, elapsed: {:?}", startTime.elapsed());
            vault.read().await.fetch_vault_info(tokens).await;
            print("task 2 done");
            p!("task 2 done, time: {}", startTime.elapsed().as_millis());
            Ok(())
        }
        async fn fetch_token_prices(
            tokens: TokensArc,
            vault: VaultArc,
            startTime: Instant,
        ) -> anyhow::Result<()> {
            p!("task 3 start after lock, elapsed: {:?}", startTime.elapsed());
            vault.read().await.fetch_token_prices(tokens).await.expect("fetch token prices");
            print("task 3 done");
            p!("task 3 done, time: {}", startTime.elapsed().as_millis());
            Ok(())
        }

        async fn fetch_multi_vault_token_variables(
            tokens: TokensArc,
            vault: VaultArc,
            startTime: Instant,
        ) -> anyhow::Result<()> {
            let vault = vault.read().await;
            p!("task 4 start after lock, elapsed: {:?}", startTime.elapsed());
            vault
                .fetch_multi_vault_token_variables(tokens)
                .await;
            print("task 4 done");
            p!("task 4 done, time: {}", startTime.elapsed().as_millis());
            Ok(())
        }

        async fn fetch_user_info(
            tokens: TokensArc,
            config: Arc<tokio::sync::RwLock<config::Config>>,
            startTime: Instant,
        ) -> anyhow::Result<()> {
            p!("task 5 start after lock, elapsed: {:?}", startTime.elapsed());
            tokio::join![
                async {
                    
                    let _config = config.read().await;
                    p!("task 5 start fetch balances");
                    _config.fetch_balances(&tokens).await;
                },
                async {
                    let _config = config.read().await;
                    p!("task 5 start fetch allowance");
                    _config.fetch_allowance(&tokens).await;
                },
            ];
            // config.fetch_balances(&tokens).await;
            // config.fetch_allowance(&tokens).await;
            p!("task 5 done, time: {}", startTime.elapsed().as_millis());
            Ok(())
        }
        let config = Arc::new(tokio::sync::RwLock::new(self.config.clone()));

        let _tasks = tokio::try_join![
            fetch_token_configuration(Arc::clone(&tokens), Arc::clone(&vault), startTime),
            fetch_vault_info(Arc::clone(&tokens), Arc::clone(&vault), startTime),
            fetch_token_prices(Arc::clone(&tokens), Arc::clone(&vault), startTime),
            fetch_user_info(Arc::clone(&tokens), config, startTime),
            fetch_multi_vault_token_variables(Arc::clone(&tokens), Arc::clone(&vault), startTime),
        ];

        print("all done");
        p!("all done, time: {}", startTime.elapsed().as_millis());

        let tokens = tokens.read().await;
        // self.vault
        //     .fetch_multi_vault_token_variables(&mut tokens)
        //     .await;
        print(format!("tokens full: {:?}", tokens).as_str());
        // re assign new tokens
        self.config.tokens = tokens.to_vec();

        for token in self.config.tokens.iter_mut() {
            token.calculate_available_liquidity();
        }

        print("all done");

        // let fetch_token_task = tokio::spawn(vault.fetch_token_configuration(&mut tokens));
        // let fetch_token_price_task = tokio::spawn(vault.fetch_token_prices(&mut tokens));
        // tokio::try_join!(fetch_token_task, fetch_token_price_task, fetch_account_balance)?;
        Ok(())
    }
}

async fn task_handle(
    tokens: Arc<Mutex<Vec<Token>>>,
    vault: Arc<Mutex<Vault>>,
    cb: fn(tokens: Arc<Mutex<Vec<Token>>>, vault: Arc<Mutex<Vault>>),
) {
    cb(tokens, vault);
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::contracts::vault_logic;
    use rust_decimal::Decimal;

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
        assert_eq!(
            router.config.selected_account,
            Some("0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string())
        );

        println!(
            "reward_router {}",
            router.config.contract_address.reward_router
        );
    }

    #[tokio::test]
    async fn it_works_arb() {
        // 1. load config
        let mut router = Router::new();
        router.initilize(42161).unwrap();

        println!("spender {:?}", router.config.contract_spender);
        router.vault.init_vault_state().await.unwrap();

        // 2. set account
        let account_address = "0xF9939C389997B5B65CBa58d298772262ecAc3F8A".to_string();
        router.set_account("0xF9939C389997B5B65CBa58d298772262ecAc3F8A".to_string());


        router.fetch_data().await.expect("fetch data failed");
        // Assert balance and allowance
        let tokens = router.load_tokens();
        assert_ne!(tokens[0].balances, None);
        let balance = tokens[0]
            .get_balance(&account_address);
        assert_ne!(balance, "0");



        println!(
            "reward_router {}",
            router.config.contract_address.reward_router
        );
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

        println!(
            "available_liquidity {}",
            tokens[0].available_liquidity.unwrap().to_string()
        );

        // assert_eq!(tokens.len(), 4);
        // assert_eq!(tokens[0].token_weight, Some(100));
        // assert_eq!(tokens[1].token_weight, Some(100));
    }

    #[tokio::test]
    async fn should_fetch_data_with_account_success() {
        let mut router = Router::new();
        router.initilize(42161).unwrap();
        let account = "0x1e8b86cd1b420925030fe72a8fd16b47e81c7515".to_string();
        println!("****** set account *****");

        router.set_account(account.clone());
        println!("start fetching account");
        router
            .fetch_data()
            .await
            .expect("fetch data in should_fetch_data_with_account_success failure");
        println!("****** done fetch data *****");

        let tokens = router.load_tokens();

        println!("Loaded tokens: {:?}", tokens);

        assert_eq!(tokens.len() >= 1, true);
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
    async fn get_buy_glp_to_amount() {
        let mut router = Router::new();
        router.initilize(42161).unwrap();
        router.vault.init_vault_state().await.unwrap();

        router.calculate_price_plp();
        // println!("price plp buy {}", router.price_plp_buy.unwrap().as_u32());
        router.fetch_data().await.expect("fetch data failed");

        let tokens = router.load_tokens();
        println!(
            "&token[0]: {} {:?}",
            &tokens[0].min_price.unwrap().parsed,
            &tokens[0].symbol
        );
        let (amount, fee) = router
            .vault
            .state
            .get_buy_glp_to_amount(&U256::from_dec_str("10000000").unwrap(), &tokens[0]);
        println!("amount: {}", amount);
        println!("fee: {}", fee);

        println!("****************************************************************");

        println!("&token[0]: {} ", &tokens[2].symbol);
        let (amount, fee) = router
            .vault
            .state
            .get_buy_glp_to_amount(&U256::from_dec_str("100000000").unwrap(), &tokens[2]);
        println!("amount: {}", amount);
    }
    #[tokio::test]
    async fn get_buy_glp_from_amount() {
        let mut router = Router::new();
        router.initilize(42161).unwrap();
        router.vault.init_vault_state().await.unwrap();

        router.calculate_price_plp();
        // println!("price plp buy {}", router.price_plp_buy.unwrap().as_u32());
        router.fetch_data().await.expect("fetch data failed");

        let tokens = router.load_tokens();
        println!("&token[0]: {}", &tokens[0].min_price.unwrap().parsed);
        let (amount, fee) = router.vault.state.get_buy_glp_from_amount(
            U256::from_dec_str("34688316279096605298").unwrap(),
            &tokens[0],
        );
        println!("amount: {}", amount);
        println!("fee: {}", fee);

        println!("****************************************************************");

        println!("&token[0]: {} ", &tokens[2].symbol);
        let (amount, fee) = router.vault.state.get_buy_glp_from_amount(
            U256::from_dec_str("101045757422875282155013").unwrap(),
            &tokens[2],
        );
        println!("amount: {}", amount);
    }

    #[tokio::test]
    async fn get_sell_glp_to_amount() {
        let mut router = Router::new();
        router.initilize(421613).unwrap();
        router.vault.init_vault_state().await.unwrap();

        router.calculate_price_plp();
        // println!("price plp buy {}", router.price_plp_buy.unwrap().as_u32());
        router.fetch_data().await.expect("fetch data failed");

        //
        // let tokens  = router.load_tokens();
        // println!("&token[0]: {}",  &tokens[0].min_price.unwrap().parsed);
        // let (amount, fee) = router.vault.state.get_sell_glp_to_amount(
        //     U256::from_dec_str("34688316279096605298").unwrap(),
        //     &tokens[0] );
        // println!("amount: {}", amount);
        // println!("fee: {}", fee);
        //
        // println!("****************************************************************");
        //
        // println!("&token[0]: {} ", &tokens[2].symbol);
        // let (amount, fee) = router.vault.state.get_sell_glp_to_amount(
        //     U256::from_dec_str("101045757422875282155013").unwrap(),
        //     &tokens[2] );
        // println!("amount: {}", amount);

        println!("****************************************************************");

        let tokens = router.load_tokens();
        println!("&token[0]: {}", &tokens[0].min_price.unwrap().parsed);
        let (amount, fee) = router.vault.state.get_sell_glp_to_amount(
            U256::from_dec_str("1000000000000000000000000").unwrap(),
            &tokens[0],
        );
        println!("amount: {}", amount);
        println!("fee: {}", fee);
    }
    #[tokio::test]
    async fn get_sell_glp_from_amount() {
        let mut router = Router::new();
        router.initilize(421613).unwrap();
        router.vault.init_vault_state().await.unwrap();

        router.calculate_price_plp();
        // println!("price plp buy {}", router.price_plp_buy.unwrap().as_u32());
        router.fetch_data().await.expect("fetch data failed");

        //
        let tokens = router.load_tokens();
        // println!("&token[0]: {}",  &tokens[0].min_price.unwrap().parsed);
        // let (amount, fee) = router.vault.state.get_sell_glp_from_amount(
        //     U256::from_dec_str("10000000").unwrap(),
        //     &tokens[0] );
        // println!("amount: {}", amount);
        // println!("fee: {}", fee);
        //

        println!("****************************************************************");

        println!("&token[0] symbol: {} ", &tokens[1].symbol);
        let (amount, fee) = router
            .vault
            .state
            .get_sell_glp_from_amount(U256::from_dec_str("100000000000000").unwrap(), &tokens[1]);
        println!("amount: {}", amount);
        println!("fee: {}", fee);
    }

    #[tokio::test]
    async fn test_get_swap_details() {
        let mut router = Router::new();
        router.initilize(421613).unwrap();
        router.vault.init_vault_state().await.unwrap();

        router.calculate_price_plp();
        // println!("price plp buy {}", router.price_plp_buy.unwrap().as_u32());
        router.fetch_data().await.expect("fetch data failed");
        let tokens = router.load_tokens();

        let (amount_out, fee_amount, fee_bps) = router.vault.state.get_swap_details(
            &tokens[2],
            &tokens[3],
            U256::from_dec_str("5000000").unwrap(),
        );

        println!(
            "fee_amount {}, &tokens[2] {}, &tokens[4] {}",
            fee_amount, &tokens[2].symbol, &tokens[3].symbol
        );
    }

    #[tokio::test]
    async fn should_switch_chain_success() {
        let mut router = Router::new();
        router.initilize(421613).unwrap();
        router.vault.init_vault_state().await.unwrap();
        p!("config {:?}", router.config);

        router.initilize(97).unwrap();
        router.vault.init_vault_state().await.unwrap();
        println!("config {:?}", router.config);
    }
}
