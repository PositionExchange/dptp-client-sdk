use std::ops::Deref;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{to_value, from_value};
use serde::{Serialize, Deserialize};
use core::{*};
use std::sync::{Arc, Mutex};
use console_error_panic_hook;
use ethabi::ethereum_types::U256;
use serde::de::Unexpected::Option;
use wasm_bindgen::__rt::IntoJsResult;
use core::contracts::vault_logic::VaultLogic;
use core::contracts::token::Token;
use rust_decimal::prelude::Decimal;
use std::panic;
use web_sys::console;
use wasm_logger::*;

use core::contracts::vault_logic;
// use core::contracts::Va;
use ethaddr::Address;





#[wasm_bindgen]
pub struct WasmRouter {
    router: Mutex<Router>,
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
        let mut router = Mutex::new(Router::new());

        router.lock().unwrap().initilize(chain_id).expect("load_config failed");
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        WasmRouter {
            router,
        }
    }

    #[wasm_bindgen]
    pub fn load_config(&mut self, chain_id: u64) -> Result<JsValue, JsValue> {
        log::info!("start load_config");

        match self.router.lock().unwrap().initilize(chain_id) {
            Ok(config) => Ok(to_value(config).unwrap()),
            Err(e) => Err(JsValue::from_str(e)),
        }
    }

    #[wasm_bindgen]
    pub fn load_tokens(&self) -> JsValue {
        log::info!("start load_tokens");

        let tokens = self.router.lock().unwrap().load_tokens();
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

        self.router.lock().unwrap().set_account(account);
        log::info!("done set_account");

    }


    #[wasm_bindgen]
    pub async fn fetch_async(&mut self, account: String) {
        log::info!("start fetch_async");

        if Address::from_str_checksum(&account).is_ok() {
            self.router.lock().unwrap().set_account(account);
        }
        self.router.lock().unwrap().fetch_data().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("fetch data failure");
        self.router.lock().unwrap().vault.init_vault_state().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("init vault state failure");
        self.router.lock().unwrap().calculate_price_plp();
        log::info!("done fetch_async");

    }

    #[wasm_bindgen]
    pub async fn fetch_data(&mut self) -> Result<(), JsValue> {
        log::info!("start fetch_data");

        self.router.lock().unwrap().fetch_data().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("fetch data failure");
        log::info!("done fetch_data");
        Ok(())


    }

    #[wasm_bindgen]
    pub async fn init_vault_state(&mut self) -> Result<(), JsValue> {
        log::info!("start init_vault_state");
        self.router.lock().unwrap().vault.init_vault_state().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("init vault state failure");
        self.router.lock().unwrap().calculate_price_plp();
        log::info!("done init_vault_state");
        Ok(())
    }

    // Note: Need to call init_vault_state first
    #[wasm_bindgen]
    pub fn get_vault_state(&mut self) -> JsValue {
        log::info!("start get_vault_state");
        let vault_state = self.router.lock().unwrap().vault.state;
        log::info!("done get_vault_state");
        to_value(&vault_state).unwrap()
    }

    // Buy GLP to token ( exact token to token)
    #[wasm_bindgen]
    pub fn get_buy_glp_from_amount(&mut self, to_amount: &str, token_address: &str) -> JsValue {

        let mut buy_glp = GetAmountOut{
            amount_out : U256::from(0),
            fee_basis_point: 0

        };
        // for token_element in self.router.lock().unwrap().config.tokens.iter_mut() {
        //     let (glp_amount, fee_basis_point) = self.router.lock().unwrap().vault.state.get_buy_glp_from_amount(U256::from_str(to_amount).unwrap(), token_element);
        //     token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
        //     if token_address == token_element.address {
        //         buy_glp.amount_out = glp_amount;
        //         buy_glp.fee_basis_point = fee_basis_point
        //     }
        // }
        return to_value(&buy_glp).unwrap();

        // let token = WasmRouter::find_token_by_address(self, token_address);
        // if token.is_some() {
        //     let (amount_out, fee_basis_point) = self.router.vault.state.get_buy_glp_from_amount(U256::from_str(to_amount).unwrap(), token.unwrap());
        //     let buy_glp = GetAmountOut {
        //         amount_out,
        //         fee_basis_point,
        //     };
        //     to_value(&buy_glp).unwrap()
        // } else {
        //     to_value(&{}).unwrap()
        // }

    }


    // Buy GLP to token ( token to exact token)
    #[wasm_bindgen]
    pub fn get_buy_glp_to_amount(&mut self, to_amount: &str, token_address: &str) -> JsValue {
        log::info!("start get_buy_glp_to_amount  to_amount: {}, token_address: {}", to_amount, token_address);
        let mut buy_glp = GetAmountOut{
            amount_out : U256::from(0),
            fee_basis_point: 0

        };

        let mut _router = self.router.lock().unwrap();

        let  routerClone: &Router =  _router.deref();
        let mut count =0;
        for token_element in routerClone.config.tokens.iter() {
            // console::log!("token_element {}", count++ );

            count = count +1;
            if token_element.is_tradeable.unwrap() {
                // log::info!("into is_tradeable ");

                let token = token_element.clone();
                // log::info!("token clone ");

                let amount = U256::from_str(to_amount).unwrap();
                // log::info!("amount clone ");

                let price =  routerClone.price_plp_buy.clone();
                // log::info!("price clone ");

                let usdp_supply = routerClone.vault.state.usdp_supply.clone();
                // log::info!("usdp_supply clone ");

                let total_token_weights =  routerClone.vault.state.total_token_weights.clone();
                // log::info!("total_token_weights clone ");

                let has_dynamic_fees = routerClone.vault.state.has_dynamic_fees.clone();
                // log::info!("has_dynamic_fees clone ");

                let (glp_amount, fee_basis_point) = vault_logic::get_buy_glp_to_amount(
                    &amount,
                    &token,
                    &price,
                    &usdp_supply,
                    &total_token_weights,
                    has_dynamic_fees
                );
                log::info!("glp_amount fee_basis_point ");


                // let (glp_amount, fee_basis_point) = self.router.lock().unwrap().vault.state.get_buy_glp_to_amount(&U256::from_str(to_amount).unwrap(), token_element);
                // token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
                if token_address == token_element.address {
                    buy_glp.amount_out = glp_amount;
                    buy_glp.fee_basis_point = fee_basis_point
                }
            }else {
                log::info!("not is_tradeable ");

            }

        }
        return to_value(&buy_glp).unwrap()

        //
        // let token = WasmRouter::find_token_by_address(self, token_address);
        // if token.is_some() {
        //     let (glp_amount, fee_basis_point) = self.router.vault.state.get_buy_glp_to_amount(&U256::from_str(to_amount).unwrap(), token.unwrap());
        //     let buy_glp = GetAmountOut {
        //         amount_out: glp_amount,
        //         fee_basis_point,
        //     };
        //     to_value(&buy_glp).unwrap()
        // } else {
        //     to_value(&{}).unwrap()
        // }
    }


    // Sell GLP to token ( token to exact token)
    #[wasm_bindgen]
    pub fn get_sell_glp_to_amount(&mut self, to_amount: &str, token_address: &str) -> JsValue {

        let mut buy_glp = GetAmountOut{
            amount_out : U256::from(0),
            fee_basis_point: 0

        };
        // for token_element in self.router.lock().unwrap().config.tokens.iter_mut() {
        //     let (glp_amount, fee_basis_point) = self.router.lock().unwrap().vault.state.get_sell_glp_to_amount(U256::from_str(to_amount).unwrap(), token_element);
        //     token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
        //     if token_address == token_element.address {
        //         buy_glp.amount_out = glp_amount;
        //         buy_glp.fee_basis_point = fee_basis_point
        //     }
        // }
        return to_value(&buy_glp).unwrap();
        // let token = WasmRouter::find_token_by_address(self, token_address);
        // if token.is_some() {
        //     let (amount_out, fee_basis_point) = self.router.vault.state.get_sell_glp_to_amount(U256::from_str(to_amount).unwrap(), token.unwrap());
        //     let sell_glp = GetAmountOut {
        //         amount_out,
        //         fee_basis_point,
        //     };
        //     to_value(&sell_glp).unwrap()
        // } else {
        //     to_value(&{}).unwrap()
        // }
    }

    // Sell GLP from amount ( exact token to token)
    #[wasm_bindgen]
    pub fn get_sell_glp_from_amount(&mut self, to_amount: &str, token_address: &str) -> JsValue {
        let mut buy_glp = GetAmountOut{
            amount_out : U256::from(0),
            fee_basis_point: 0

        };
        // for token_element in self.router.lock().unwrap().config.tokens.iter_mut() {
        //     let (glp_amount, fee_basis_point) = self.router.lock().unwrap().vault.state.get_sell_glp_from_amount(U256::from_str(to_amount).unwrap(), token_element);
        //     token_element.buy_plp_fees = Some( Decimal::from(fee_basis_point));
        //     if token_address == token_element.address {
        //         buy_glp.amount_out = glp_amount;
        //         buy_glp.fee_basis_point = fee_basis_point
        //     }
        // }
        return to_value(&buy_glp).unwrap();
        // let token = WasmRouter::find_token_by_address(self, token_address);
        // if token.is_some() {
        //     let (amount_out, fee_basis_point) = self.router.vault.state.get_sell_glp_from_amount(U256::from_str(to_amount).unwrap(), token.unwrap());
        //     let sell_glp = GetAmountOut {
        //         amount_out,
        //         fee_basis_point,
        //     };
        //     to_value(&sell_glp).unwrap()
        // } else {
        //     to_value(&{}).unwrap()
        // }

    }

    #[wasm_bindgen]
    pub fn get_fee_basis_points(&mut self,
                                token_weight: u64,
                                token_usdg_amount: u64,
                                usdp_delta: u64,
                                increment: bool, ) -> JsValue {
        let fee_basis_point = self.router.lock().unwrap().vault.state.get_fee_basis_points(
            token_weight,
            &U256::from(token_usdg_amount),
            &U256::from(usdp_delta),
            increment);
        to_value(&fee_basis_point).unwrap()
    }

    #[wasm_bindgen]
    pub fn get_plp_price(&mut self, is_buy: bool) -> JsValue {
        to_value(&self.router.lock().unwrap().vault.state.get_plp_price(is_buy)).unwrap()
    }

    //
    // fn find_token_by_address(&self, token_address: &str) -> Option<&Token> {
    //     let token: Option<&Token> = None;
    //     for token_element in self.router.config.tokens.iter() {
    //         if token_address == token_element.address {
    //             return Some(&token_element);
    //         }
    //     }
    //     return token;
    // }
}

#[derive(Serialize, Deserialize)]
pub struct GetAmountOut {
    pub amount_out: U256,
    pub fee_basis_point: u64,
}

// #[derive(Serialize, Deserialize)]
// pub struct FetchInOne {
//     pub tokens: Vec<Token>,
//     pub vault_state: contracts::vault::VaultState,
// }

// #[derive(Serialize, Deserialize)]
// pub struct GetAmountOut {
//     pub glp_amount: U256,
//     pub fee_basis_point: u32,
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
