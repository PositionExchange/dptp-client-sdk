use core::contracts::{token::Token, vault_logic::VaultLogic, vault::VaultState};
use core::*;
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use serde_json::*;
use static_init::dynamic;
use ethabi::ethereum_types::U256;

#[derive(Serialize, Deserialize)]
pub struct SwapDetails {
    amount_out: String,
    fee_amount: String,
    fees_bps: String,
}

#[derive(Debug, Clone)]
pub struct FlutterRouter {
    router: Router,
}

impl FlutterRouter {
    pub fn new() -> Self {
        let router = Router::new();
        FlutterRouter { router }
    }

    pub fn initialize(&mut self, chain_id: u64) {
        self.router.initilize(chain_id).expect("load_config failed");
    }

    pub fn set_account(&mut self, account: String) {
        self.router.set_account(account.to_string());
    }

    pub fn load_tokens(&mut self) -> Vec<Token> {
        self.router.load_tokens()
    }

    pub fn calculate_price_plp(&mut self) {
        self.router.calculate_price_plp();
    }

    pub async fn fetch_data(&mut self) {
        self.router.fetch_data().await;
    }

    pub async fn init_vault_state(&mut self) {
        self.router.vault.init_vault_state().await;
    }

    pub fn get_swap_details(
        &self,
        token_in: String,
        token_out: String,
        amount_in: String,
    ) -> SwapDetails {
        let token_in = self
            .router
            .config
            .get_token_by_token_address(token_in)
            .expect("token_in not found");
        let token_out = self
            .router
            .config
            .get_token_by_token_address(token_out)
            .expect("token_out not found");
        let (amount_out, fee_amount, fees_bps) = self.router.vault.state.get_swap_details(
            &token_in,
            &token_out,
            U256::from_dec_str(&amount_in.to_string()).unwrap(),
        );
        SwapDetails {
            amount_out: amount_out.to_string(),
            fee_amount: fee_amount.to_string(),
            fees_bps: fees_bps.to_string(),
        }
    }

    pub fn get_price_plp(&mut self, is_buy: bool) -> U256 {
        U256::from(&self.router.vault.state.get_plp_price(is_buy))
    }

    pub fn get_vault_state(&mut self) -> VaultState {
        self.router.vault.state.clone()
    }
}

#[dynamic]
static mut FLUTTER_ROUTER: FlutterRouter = FlutterRouter::new();

pub fn initialize(chain_id: u64) {
    FLUTTER_ROUTER.write().initialize(chain_id);
}

pub fn load_tokens() -> String {
    let token: Vec<Token> = FLUTTER_ROUTER.write().load_tokens();
    let serialized = to_string(&token).unwrap();
    serialized
}

pub fn get_swap_details(token_in: String, token_out: String, amount_in: String) -> String {
    let swap_detail: SwapDetails = FLUTTER_ROUTER
        .write()
        .get_swap_details(token_in, token_out, amount_in);
    let serialized = to_string(&swap_detail).unwrap();
    serialized
}

pub fn get_price_plp(is_buy: bool) -> String {
    let price: U256 = FLUTTER_ROUTER.write().get_price_plp(is_buy);
    let serialized = to_string(&price).unwrap();
    serialized
}

pub fn get_vault_state() -> String {
    let state: VaultState = FLUTTER_ROUTER.write().get_vault_state();
    let serialized = to_string(&state).unwrap();
    serialized
}

//
// pub fn get_router() -> String {
//     let router = FLUTTER_ROUTER.write().get_router();
//     to_string(&router).unwrap()
// }
//
// pub fn set_account(account: String) {
//     FLUTTER_ROUTER.write().set_account(account);
// }
//
// pub fn calculate_price_plp() {
//     FLUTTER_ROUTER.write().calculate_price_plp();
// }
//
// pub fn fetch_data() {
//     block_on(async { FLUTTER_ROUTER.write().fetch_data().await })
// }
//
// pub fn init_vault_state() {
//     block_on(async { FLUTTER_ROUTER.write().init_vault_state().await });
// }

#[tokio::main]
pub async fn fetch_async(chain_id: u64, account: String) {
    block_on(async {
        FLUTTER_ROUTER.write().initialize(chain_id);
        if account.len() > 0 {
            FLUTTER_ROUTER.write().set_account(account);
        }
        FLUTTER_ROUTER.write().fetch_data().await;
        FLUTTER_ROUTER.write().init_vault_state().await;
        FLUTTER_ROUTER.write().calculate_price_plp();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // 1. load config
        fetch_async(
            42161,
            "0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string(),
        );
        // initialize(421613);
        // set_account("0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string());
        // fetch_data();
        // init_vault_state();
        // calculate_price_plp();
        // let tokens = load_tokens();
        // let detail = get_swap_details(
        //     "0x38193a1c61b2b44446289265580f73746f5bb5ae".to_owned(),
        //     "0xa8cc0c527a271c7d196f12c23a65dbfb58c033f5".to_owned(),
        //     "10000000000".to_owned(),
        // );
        let state = get_price_plp(true);
        println!("Loaded tokens: {:?}", state);
        // println!("token detail: {:?}", detail);
        // assert_eq!(tokens.len(), 3);
        // assert_eq!(tokens[0].symbol, "USDT");
        // assert_eq!(tokens[1].symbol, "BTC");
        // 2. set account
        // assert_eq!(get_router().config.selected_account, Some("0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string()));
    }
}
