use std::ops::Deref;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{to_value, from_value};
use serde::{Serialize, Deserialize};
use core::{*};
use std::sync::{Arc, Mutex};
use console_error_panic_hook;
use ethabi::ethereum_types::U256;
use wasm_bindgen::__rt::IntoJsResult;
use core::contracts::vault_logic::VaultLogic;
use core::contracts::token::Token;
use rust_decimal::prelude::Decimal;
use std::panic;
use wasm_logger::*;

use core::contracts::vault_logic;
// use core::contracts::Va;
use ethaddr::Address;
use std::collections::HashMap;






#[wasm_bindgen]
pub struct WasmRouter {
    router: Router,
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

        router.initilize(chain_id).expect("load_config failed");
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        WasmRouter {
            router,
        }
    }

    #[wasm_bindgen]
    pub fn load_config(&mut self, chain_id: u64) -> Result<JsValue, JsValue> {
        log::info!("start load_config");

        match self.router.initilize(chain_id) {
            Ok(config) => Ok(to_value(config).unwrap()),
            Err(e) => Err(JsValue::from_str(e)),
        }
    }

    #[wasm_bindgen]
    pub fn load_tokens(&self) -> JsValue {
        log::info!("start load_tokens");

        let tokens = self.router.load_tokens();
        log::info!("done load_tokens");
        return to_value(&tokens).unwrap();

    }
    //
    // pub fn load_tokens_string(&self) -> String {
    //     let tokens = self.router.load_tokens();
    //
    //     return serde_json::to_string(&tokens).unwrap()
    //     // to_value(&tokens).unwrap()
    // }

    #[wasm_bindgen]
    pub fn set_account(&mut self, account: String) {
        log::info!("start set_account");

        self.router.set_account(account);
        log::info!("done set_account");

    }


    #[wasm_bindgen]
    pub async fn fetch_async(&mut self, account: String) {


        log::info!("check set_account {}", account.clone());



        if account.len() > 0 {
            log::info!("into set account {}", account.clone());
            self.router.set_account(account);
        }

        self.router.fetch_data().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("fetch data failure");
        self.router.vault.init_vault_state().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("init vault state failure");
        self.router.calculate_price_plp();

    }

    #[wasm_bindgen]
    pub async fn fetch_data(&mut self) -> Result<(), JsValue> {
        self.router.fetch_data().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("fetch data failure");
        Ok(())


    }

    #[wasm_bindgen]
    pub async fn init_vault_state(&mut self) -> Result<(), JsValue> {
        self.router.vault.init_vault_state().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("init vault state failure");
        self.router.calculate_price_plp();
        Ok(())
    }

    // Note: Need to call init_vault_state first
    #[wasm_bindgen]
    pub fn get_vault_state(&mut self) -> JsValue {
        let vault_state = self.router.vault.state;
        to_value(&vault_state).unwrap()
    }

    // Buy GLP to token ( exact token to token)
    #[wasm_bindgen]
    pub fn get_buy_glp_from_amount(&mut self, to_amount: &str, token_address: &str) -> JsValue {

        let mut buy_glp = GetAmountOut{
            amount_out : U256::from(0),
            fee_basis_point: 0,
            mapping_fee_token : HashMap::new(),

        };
        for token_element in self.router.config.tokens.iter_mut() {
            if token_element.is_tradeable.unwrap() {

                let amount = U256::from_dec_str(to_amount).unwrap();

                let (glp_amount, fee_basis_point) = self.router.vault.state.get_buy_glp_from_amount(
                    amount,
                    token_element
                );

                buy_glp.mapping_fee_token.insert(token_element.address.clone(), fee_basis_point);
                // token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
                if token_address == token_element.address {
                    buy_glp.amount_out = glp_amount;
                    buy_glp.fee_basis_point = fee_basis_point
                }
            }
        }
        return to_value(&buy_glp).unwrap();

    }


    // Buy GLP to token ( token to exact token)
    #[wasm_bindgen]
    pub fn get_buy_glp_to_amount(&mut self, to_amount: &str, token_address: &str) -> JsValue {
        let mut buy_glp = GetAmountOut{
            amount_out : U256::from(0),
            fee_basis_point: 0,
            mapping_fee_token : HashMap::new(),
        };
        for token_element in self.router.config.tokens.iter_mut() {

            if token_element.is_tradeable.unwrap() {
                let amount = U256::from_dec_str(to_amount).unwrap();
                let (glp_amount, fee_basis_point) = self.router.vault.state.get_buy_glp_to_amount(
                    &amount,
                    token_element
                );
                buy_glp.mapping_fee_token.insert(token_element.address.clone(), fee_basis_point);

                // token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
                if token_address == token_element.address {
                    buy_glp.amount_out = glp_amount;
                    buy_glp.fee_basis_point = fee_basis_point
                }
            }

        }
        return to_value(&buy_glp).unwrap()
    }


    // Sell GLP to token ( token to exact token)
    #[wasm_bindgen]
    pub fn get_sell_glp_to_amount(&mut self, to_amount: &str, token_address: &str) -> JsValue {

        let mut buy_glp = GetAmountOut{
            amount_out : U256::from(0),
            fee_basis_point: 0,
            mapping_fee_token : HashMap::new(),


        };
        for token_element in self.router.config.tokens.iter_mut() {
            if token_element.is_tradeable.unwrap() {

                let amount = U256::from_dec_str(to_amount).unwrap();
                let (glp_amount, fee_basis_point) = self.router.vault.state.get_sell_glp_to_amount(
                    amount,
                    token_element
                );
                buy_glp.mapping_fee_token.insert(token_element.address.clone(), fee_basis_point);

                // token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
                if token_address == token_element.address {
                    buy_glp.amount_out = glp_amount;
                    buy_glp.fee_basis_point = fee_basis_point
                }
            }
        }
        return to_value(&buy_glp).unwrap();

    }

    // Sell GLP from amount ( exact token to token)
    #[wasm_bindgen]
    pub fn get_sell_glp_from_amount(&mut self, to_amount: &str, token_address: &str) -> JsValue {
        let mut buy_glp = GetAmountOut{
            amount_out : U256::from(0),
            fee_basis_point: 0,
            mapping_fee_token : HashMap::new(),


        };
        for token_element in self.router.config.tokens.iter_mut() {
            if token_element.is_tradeable.unwrap() {

                let amount = U256::from_dec_str(to_amount).unwrap();

                let (glp_amount, fee_basis_point) = self.router.vault.state.get_sell_glp_from_amount(
                    amount,
                    token_element
                );
                buy_glp.mapping_fee_token.insert(token_element.address.clone(), fee_basis_point);

                // token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
                if token_address == token_element.address {
                    buy_glp.amount_out = glp_amount;
                    buy_glp.fee_basis_point = fee_basis_point
                }
            }
        }
        return to_value(&buy_glp).unwrap();
    }

    #[wasm_bindgen]
    pub fn get_fee_basis_points(&mut self,
                                token_weight: u64,
                                token_usdg_amount: u64,
                                usdp_delta: u64,
                                increment: bool, ) -> JsValue {
        let fee_basis_point = self.router.vault.state.get_fee_basis_points(
            token_weight,
            &U256::from(token_usdg_amount),
            &U256::from(usdp_delta),
            increment);
        to_value(&fee_basis_point).unwrap()
    }

    #[wasm_bindgen]
    pub fn get_plp_price(&mut self, is_buy: bool) -> JsValue {
        to_value(&self.router.vault.state.get_plp_price(is_buy)).unwrap()
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
