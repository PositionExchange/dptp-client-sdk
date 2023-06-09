use console_error_panic_hook;
use core::contracts::token::Token;
use core::contracts::vault_logic::VaultLogic;
use core::*;
use ethabi::ethereum_types::U256;
use rust_decimal::prelude::Decimal;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::cell::RefCell;
use std::ops::Deref;
use std::panic;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use wasm_bindgen::__rt::IntoJsResult;
use wasm_bindgen::prelude::*;
use wasm_logger::*;

use core::contracts::vault_logic;
// use core::contracts::Va;
use ethaddr::Address;
use std::collections::HashMap;

/** RETURN TYPE **/

#[derive(Serialize, Deserialize)]
pub struct SwapDetails {
    amount_out: String,
    fee_amount: String,
    fees_bps: String,
}

#[wasm_bindgen]
pub struct WasmRouter {
    router: Rc<RefCell<Router>>,
    lock: Arc<Mutex<u64>>,
}

// Note: this is the example to custom export struct to js
// #[wasm_bindgen]
// pub struct WasmChain {
//     pub chain_id: u64,
//     rpc_urls: js_sys::Array,
//     multicall_address: JsValue,
// }
//
// impl From<core::config::Chain> for WasmChain {
//     fn from(chain: core::config::Chain) -> Self {
//         let rpc_urls: js_sys::Array = chain
//             .rpc_urls
//             .into_iter()
//             .map(|x| JsValue::from_str(&x))
//             .collect();
//
//         WasmChain {
//             chain_id: chain.chain_id,
//             rpc_urls: rpc_urls.into(),
//             multicall_address: chain.multicall_address.into(),
//         }
//     }
//
// }
// #[wasm_bindgen]
// impl WasmChain {
//     #[wasm_bindgen(getter)]
//     pub fn rpc_urls(&self) -> js_sys::Array {
//         self.rpc_urls.clone()
//     }
// }

trait WasmRouterTrait {}

#[wasm_bindgen]
impl WasmRouter {
    #[wasm_bindgen(constructor)]
    pub fn new(chain_id: u64) -> Self {
        init(wasm_logger::Config::default());
        let mut router = Router::new();
        log::info!("start new, chainId {}", chain_id);

        router.initilize(chain_id).expect("load_config failed");
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        WasmRouter {
            router: Rc::new(RefCell::new(router)),
            lock: Arc::new(Mutex::new(0)),
        }
    }

    #[wasm_bindgen]
    pub fn load_config(&self, chain_id: u64) -> Result<JsValue, JsValue> {
        log::info!("start load_config {:?}", chain_id);

        let router = self.router.try_borrow_mut();
        match router {
            Ok(mut router) => match router.initilize(chain_id) {
                Ok(config) => Ok(to_value(config).unwrap()),
                Err(e) => Err(JsValue::from_str(e)),
            },
            Err(e) => Err(JsValue::from_str(e.to_string().as_str())),
        }
    }

    #[wasm_bindgen]
    pub fn load_tokens(&self) -> Result<JsValue, JsValue> {
        log::info!("start load_tokens");
        let router = self.router.try_borrow().map_err(|e| JsValue::from_str(&e.to_string()))?;

        let tokens = router.load_tokens();
        log::info!("done load_tokens");
        Ok(to_value(&tokens).unwrap())
    }
    //
    // pub fn load_tokens_string(&self) -> String {
    //     let tokens = self.router.load_tokens();
    //
    //     return serde_json::to_string(&tokens).unwrap()
    //     // to_value(&tokens).unwrap()
    // }

    #[wasm_bindgen]
    pub fn set_account(&self, account: String) {
        log::info!("start set_account");

        match self.router.try_borrow_mut() {
            Ok(mut router) => {
                router.set_account(account);
                log::info!("done set_account");
            }
            Err(e) => {
                log::info!("set_account error {}", e.to_string());
            }
        }
    }

    #[wasm_bindgen]
    pub async fn fetch_balance(&self) {
        match self.router.try_borrow_mut() {
            Ok(mut router) => {
                let mut lock = self.lock.lock().await;
                log::info!("lock done");
                let res1 = router
                    .fetch_balance()
                    .await
                    .map_err(|e| JsValue::from_str(&e.to_string()))
                    .expect("fetch data failure");
                *lock += 1;
                log::info!("fetch async done");
            }
            Err(e) => {
                log::error!("fetch async error: {}", e.to_string());
            }
        }

    }

    #[wasm_bindgen]
    pub async fn fetch_vault(&self) {
        match self.router.try_borrow_mut() {
            Ok(mut router) => {
                let mut lock = self.lock.lock().await;
                log::info!("lock done");
                let res1 = router
                    .fetch_vault()
                    .await
                    .map_err(|e| JsValue::from_str(&e.to_string()))
                    .expect("fetch data failure");
                *lock += 1;
                log::info!("fetch async done");
            }
            Err(e) => {
                log::error!("fetch async error: {}", e.to_string());
            }
        }

    }

    #[wasm_bindgen]
    pub async fn fetch_async(&self, account: String) {
        log::info!("check set_account {}", account.clone());
        match self.router.try_borrow_mut() {
            Ok(mut router) => {
                if account.len() > 0 {
                    log::info!("into set account {}", account.clone());
                    router.set_account(account);
                }
                log::info!("after set_account, start fetch data");

                let mut lock = self.lock.lock().await;
                log::info!("lock done");
                let res1 = router
                    .fetch_data()
                    .await
                    .map_err(|e| JsValue::from_str(&e.to_string()))
                    .expect("fetch data failure");
                log::info!("fetch data done");
                router
                    .vault
                    .init_vault_state()
                    .await
                    .map_err(|e| JsValue::from_str(&e.to_string()))
                    .expect("init vault state failure");
                log::info!("init vault state done");
                router.calculate_price_plp();
                *lock += 1;
                log::info!("fetch async done");
            }
            Err(e) => {
                log::error!("fetch async error: {}", e.to_string());
            }
        }
    }

    // #[wasm_bindgen]
    // pub async fn fetch_data(&self) -> Result<(), JsValue> {
    //     self.router.borrow_mut().fetch_data().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("fetch data failure");
    //     Ok(())
    //
    //
    // }

    #[wasm_bindgen]
    pub async fn init_vault_state(&self) -> Result<(), JsValue> {
        match self.router.try_borrow_mut() {
            Ok(mut router) => {
                router
                    .vault
                    .init_vault_state()
                    .await
                    .map_err(|e| JsValue::from_str(&e.to_string()))
                    .expect("init vault state failure");
                router.calculate_price_plp();
                Ok(())
            }
            Err(e) => {
                log::error!("init_vault_state error: {}", e.to_string());
                Err(JsValue::from_str(&e.to_string()))
            }
        }
    }

    // Note: Need to call init_vault_state first
    #[wasm_bindgen]
    pub fn get_vault_state(&self) -> Result<JsValue, JsValue> {
        let router = self.router.try_borrow().map_err(|e| JsValue::from_str(&e.to_string()))?;
        let vault_state = router.vault.state.clone();
        Ok(to_value(&vault_state).unwrap())
    }

    #[wasm_bindgen]
    pub fn get_contract_address(&self) -> Result<JsValue, JsValue> {
        let router = self.router.try_borrow().map_err(|e| JsValue::from_str(&e.to_string()))?;
        let contract_address = &router.config.contract_address;
        Ok(to_value(&contract_address).unwrap())
    }

    // Buy GLP to token ( exact token to token)
    #[wasm_bindgen]
    pub fn get_buy_glp_from_amount(&self, to_amount: &str, token_address: &str) -> Result<JsValue, JsValue> {
        let mut buy_glp = GetAmountOut {
            amount_out: U256::from(0),
            fee_basis_point: 0,
            mapping_fee_token: HashMap::new(),
        };
        let router = self.router.try_borrow().map_err(|e| JsValue::from_str(&e.to_string()))?;
        for token_element in router.config.tokens.iter() {
            if token_element.is_tradeable.unwrap() {
                let amount = U256::from_dec_str(to_amount).unwrap();

                let (glp_amount, fee_basis_point) = self
                    .router
                    .borrow()
                    .vault
                    .state
                    .get_buy_glp_from_amount(amount, token_element);

                buy_glp
                    .mapping_fee_token
                    .insert(token_element.address.clone(), fee_basis_point);
                // token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
                if token_address == token_element.address {
                    buy_glp.amount_out = glp_amount;
                    buy_glp.fee_basis_point = fee_basis_point
                }
            }
        }
        Ok(to_value(&buy_glp).unwrap())
    }

    // Buy GLP to token ( token to exact token)
    #[wasm_bindgen]
    pub fn get_buy_glp_to_amount(&self, to_amount: &str, token_address: &str) -> Result<JsValue, JsValue> {
        let mut buy_glp = GetAmountOut {
            amount_out: U256::from(0),
            fee_basis_point: 0,
            mapping_fee_token: HashMap::new(),
        };
        let router = self.router.try_borrow().map_err(|e| JsValue::from_str(&e.to_string()))?;

        for token_element in router.config.tokens.iter() {
            if token_element.is_tradeable.unwrap() {
                let amount = U256::from_dec_str(to_amount).unwrap();
                let (glp_amount, fee_basis_point) = self
                    .router
                    .borrow()
                    .vault
                    .state
                    .get_buy_glp_to_amount(&amount, token_element);
                buy_glp
                    .mapping_fee_token
                    .insert(token_element.address.clone(), fee_basis_point);

                // token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
                if token_address == token_element.address {
                    buy_glp.amount_out = glp_amount;
                    buy_glp.fee_basis_point = fee_basis_point
                }
            }
        }
        Ok(to_value(&buy_glp).unwrap())
    }

    // Sell GLP to token ( token to exact token)
    #[wasm_bindgen]
    pub fn get_sell_glp_to_amount(&mut self, to_amount: &str, token_address: &str) -> Result<JsValue, JsValue> {
        let mut buy_glp = GetAmountOut {
            amount_out: U256::from(0),
            fee_basis_point: 0,
            mapping_fee_token: HashMap::new(),
        };
        let router = self.router.try_borrow().map_err(|e| JsValue::from_str(&e.to_string()))?;
        for token_element in router.config.tokens.iter() {
            if token_element.is_tradeable.unwrap() {
                let amount = U256::from_dec_str(to_amount).unwrap();
                let (glp_amount, fee_basis_point) = self
                    .router
                    .borrow()
                    .vault
                    .state
                    .get_sell_glp_to_amount(amount, token_element);
                buy_glp
                    .mapping_fee_token
                    .insert(token_element.address.clone(), fee_basis_point);

                // token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
                if token_address == token_element.address {
                    buy_glp.amount_out = glp_amount;
                    buy_glp.fee_basis_point = fee_basis_point
                }
            }
        }
        Ok(to_value(&buy_glp).unwrap())
    }

    // Sell GLP from amount ( exact token to token)
    #[wasm_bindgen]
    pub fn get_sell_glp_from_amount(&mut self, to_amount: &str, token_address: &str) -> Result<JsValue, JsValue> {
        let mut buy_glp = GetAmountOut {
            amount_out: U256::from(0),
            fee_basis_point: 0,
            mapping_fee_token: HashMap::new(),
        };
        let router = self.router.try_borrow().map_err(|e| JsValue::from_str(&e.to_string()))?;
        for token_element in router.config.tokens.iter() {
            if token_element.is_tradeable.unwrap() {
                let amount = U256::from_dec_str(to_amount).unwrap();

                let (glp_amount, fee_basis_point) = self
                    .router
                    .borrow()
                    .vault
                    .state
                    .get_sell_glp_from_amount(amount, token_element);
                buy_glp
                    .mapping_fee_token
                    .insert(token_element.address.clone(), fee_basis_point);

                // token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
                if token_address == token_element.address {
                    buy_glp.amount_out = glp_amount;
                    buy_glp.fee_basis_point = fee_basis_point
                }
            }
        }
        Ok(to_value(&buy_glp).unwrap())
    }

    #[wasm_bindgen]
    pub fn get_fee_basis_points(
        &self,
        token_weight: u64,
        token_usdg_amount: u64,
        usdp_delta: u64,
        increment: bool,
    ) -> Result<JsValue, JsValue> {
        let router = self.router.try_borrow().map_err(|e| JsValue::from_str(&e.to_string()))?;
        let fee_basis_point = router.vault.state.get_fee_basis_points(
            token_weight,
            &U256::from(token_usdg_amount),
            &U256::from(usdp_delta),
            increment,
        );
        Ok(to_value(&fee_basis_point).unwrap())
    }

    /// Get the amount of token out for a given amount of token in
    /// @param token_in The address of the token to swap from
    /// @param token_out The address of the token to swap to
    /// @param amount_in The amount of token to swap
    /// @return The amount of token out and fees following this struct
    /// {fees: String, fee_amount: String, amount_out: String}
    #[wasm_bindgen]
    pub fn get_swap_details(
        &self,
        token_in: String,
        token_out: String,
        amount_in: String,
    ) -> Result<JsValue, JsValue> {
        let token_in = self
            .router
            .borrow()
            .config
            .get_token_by_token_address(token_in)
            .expect("token_in not found");
        let token_out = self
            .router
            .borrow()
            .config
            .get_token_by_token_address(token_out)
            .expect("token_out not found");
        let router = self.router.try_borrow().map_err(|e| JsValue::from_str(&e.to_string()))?;
        let (amount_out, fee_amount, fees_bps) = router.vault.state.get_swap_details(
            &token_in,
            &token_out,
            U256::from_dec_str(&amount_in.to_string()).unwrap(),
        );
        let result = SwapDetails {
            amount_out: amount_out.to_string(),
            fee_amount: fee_amount.to_string(),
            fees_bps: fees_bps.to_string(),
        };
        Ok(to_value(&result).unwrap())
    }

    #[wasm_bindgen]
    pub fn get_plp_price(&self, is_buy: bool) -> Result<JsValue, JsValue> {
        let router = self.router.try_borrow().map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(to_value(&router.vault.state.get_plp_price(is_buy)).unwrap())
    }

    #[wasm_bindgen(getter)]
    pub fn connected_chain(&self) -> u64 {
        let router_ref = self.router.try_borrow();
        if router_ref.is_err() {
            return 0;
        }
        router_ref.unwrap().config.chain.chain_id
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetAmountOut {
    pub amount_out: U256,
    pub fee_basis_point: u64,
    pub mapping_fee_token: HashMap<String, u64>,
}

// #[derive(Serialize, Deserialize)]
// pub struct FetchInOne {
//     pub tokens: Vec<Token>,
//     pub vault_state: contracts::vault::VaultState,
// }

// #[cfg(test)]
// mod tests {
//     // use ethabi::ethereum_types::U256;
//     // use core::Token;
//     use crate::WasmRouter;
//
//     // fn create_tokens_wasm() -> Vec<Token> {
//     //     let mut tokens = vec![
//     //         Token::new(97, "0x542E4676238562b518B968a1d03626d544a7BCA2", "USDT", "USDT", 18, ""),
//     //         Token::new(97, "0xc4900937c3222CA28Cd4b300Eb2575ee0868540F", "BTC", "BTC", 18, ""),
//     //     ];
//     //     tokens
//     // }
//
//     #[wasm_bindgen_test(async)]
//     async fn test_buy_amount() {
//
//         let mut router = WasmRouter::new(97);
//         let mut tokens = create_tokens_wasm();
//         println!("len {} ", tokens.len());
//         router.initilize();
//         router.fetch_data().await.unwrap();
//         // router.load_tokens().unwrap();
//         // router.init_vault_state().unwrap();
//         // router.fetch_vault_info().unwrap();
//
//         // let result = router.get_buy_glp_to_amount(10000000000, "0x542E4676238562b518B968a1d03626d544a7BCA2" );
//
//         // println!("{}", result );
//
//     }
//
// }
//
// #[wasm_bindgen_test(async)]
// async fn test_buy_amount() {
//     use crate::WasmRouter;
//
//
//     let mut router = WasmRouter::new(97);
//     let mut tokens = create_tokens_wasm();
//     println!("len {} ", tokens.len());
//     router.initilize();
//     router.fetch_data().await.unwrap();
//     // router.load_tokens().unwrap();
//     // router.init_vault_state().unwrap();
//     // router.fetch_vault_info().unwrap();
//
//     // let result = router.get_buy_glp_to_amount(10000000000, "0x542E4676238562b518B968a1d03626d544a7BCA2" );
//     // println!("{}", result );
// }
//
// #[wasm_bindgen(start)]
// pub fn main() {
//     // Set the panic hook to forward messages to console.error
//     console_error_panic_hook::set_once();
//
//     // Your other code...
// }
