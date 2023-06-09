use std::collections::HashMap;
use async_trait::async_trait;

use crate::config::Config;
use super::{multicall::*, token::Token, types::TokensArc};
use ethers::{types::Bytes};
use ethabi::{ethereum_types::Address, ethereum_types::U256};


#[async_trait(?Send)]
pub trait GlobalFetch {
    async fn fetch_balances(&self, update_tokens: &TokensArc) -> anyhow::Result<()>;
    async fn fetch_allowance(&self, update_tokens: &TokensArc ) -> anyhow::Result<()>;
}

#[async_trait(?Send)]
impl GlobalFetch for Config {
    async fn fetch_balances(&self, update_tokens: &TokensArc ) -> anyhow::Result<()> {
        // let mut tokens = update_tokens.lock().await;
        
        eprintln!("DEBUGPRINT[1]: global_fetch.rs:20 (after )");

        if self.selected_account.is_none() {
            println!("no account selected");
            return Ok(());
        }
        let account = self.selected_account.clone().unwrap();
        let calls: Vec<_> = self.tokens.iter().map(|token| {
            let (call_address, data) = token.build_balance_of_call(&account);
            (call_address, data)
        }).collect();
        let results = self.chain.execute_multicall(calls, include_str!("../../abi/erc20.json").to_string(), "balanceOf").await.unwrap();
        let mut update_tokens = update_tokens.write().await;

        let balance_eth = self.chain.get_balance(&self.selected_account.clone().unwrap()).await.unwrap();

        for (token, result) in update_tokens.iter_mut().zip(results) {
            let balance = result[0].clone().into_uint().expect("failed to parse balance");

            if token.is_native_token.is_some() {
                token.update_balance(&account, balance_eth);
            }else {
                token.update_balance(&account, balance);
            }
        }
        println!("[DEBUGPRINT] fetch_balances: {:?}", update_tokens);
        // we update here ensure the old value get updated
        Ok(())
    }

    async fn fetch_allowance(&self, update_tokens: &TokensArc ) -> anyhow::Result<()> {
        // let mut tokens = update_tokens.lock().await;
        if self.selected_account.is_none() {
            return Ok(());
        }
        let account = self.selected_account.clone().unwrap();
        let mut calls = Vec::new();

        self.tokens.iter().for_each(|token| {
            self.contract_spender.iter().for_each(|spender| {
                calls.push(token.build_allowance_call(&account, &spender.address))
            })
        });
        let results = self.chain.execute_multicall(calls, include_str!("../../abi/erc20.json").to_string(), "allowance").await.unwrap();
        let mut index = 0;
        let mut update_tokens = update_tokens.write().await;
        update_tokens.iter_mut().for_each(|token| {
            self.contract_spender.iter().for_each(|spender| {
                let allowance_amount = results[index].clone()[0].clone().into_uint().expect("failed to parse allowance");
                token.update_allowance(&account,allowance_amount, &spender.address);
                index = index + 1;
            })
        });


        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use std::sync::Arc;

    use crate::config::load_config;

    const FAKE_ADDRESS: &str = "0x1e8b86cd1b420925030fe72a8fd16b47e81c7515";

    use super::*;
    #[tokio::test]
    async fn should_not_update_when_no_account_selected() {
        let mut config = load_config(97).unwrap();
        // let tokens = Mutex::new(config.tokens.clone());
        let mut tokens = (config.tokens.clone());
        // config.fetch_balances(&tokens).await;
        // assert_eq!(config.tokens[0].get_balance(&FAKE_ADDRESS.to_string()), "0");
    }

    #[tokio::test]
    async fn should_update_when_account_is_selected() {
        let mut config = load_config(97).unwrap();
        config.set_selected_account("0xDfbE56f4e2177a498B5C49C7042171795434e7D1".to_string());
        // let tokens = Mutex::new(config.tokens.clone());
        let mut tokens = config.tokens.clone();
        let tokens = Arc::new(tokio::sync::RwLock::new(tokens));
        config.fetch_balances(&tokens.clone()).await.expect("fetch_balances failed");
        let token0Balance = config.tokens[0].get_balance(&FAKE_ADDRESS.to_string());

        config.fetch_allowance(&tokens.clone()).await.expect("fetch_balances failed");

        // assert!(token0Balance.parse::<f64>().unwrap() > 0.0, "token 0 balance ({}) should be greater than 0", token0Balance);
        // assert_eq!(config.tokens[1].get_balance(&FAKE_ADDRESS.to_string()).parse::<f64>().unwrap(), 10.0);
    }

}
