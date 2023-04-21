use static_init::{dynamic};
use core::{*};
use core::contracts::token::Token;
use serde_json::*;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct FlutterRouter {
    router: Router,
}

impl FlutterRouter {
    pub fn new(chain_id: u64) -> Self {
        let mut router = Router::new();
        router.initilize(chain_id).expect("load_config failed");

        FlutterRouter {
            router,
        }
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
        self.router.fetch_data();
    }

    pub fn get_router(&mut self) -> Router {
        self.router.clone()
    }
}


#[dynamic]
static mut FLUTTER_ROUTER: FlutterRouter = FlutterRouter::new(97);

pub fn initialize(chain_id: u64) {
    unsafe { FLUTTER_ROUTER.write().initialize(chain_id); }
}

pub fn load_tokens() -> String {
    unsafe {
        let token: Vec<Token> = FLUTTER_ROUTER.write().load_tokens();
        let serialized = to_string(&token).unwrap();
        serialized
    }
}

pub fn get_router() -> String {
    unsafe {
        let router = FLUTTER_ROUTER.write().get_router();
        to_string(&router).unwrap()
    }
}

pub fn set_account(account: String) {
    unsafe { FLUTTER_ROUTER.write().set_account(account); }
}

pub fn calculate_price_plp() {
    unsafe { FLUTTER_ROUTER.write().calculate_price_plp(); }
}

pub fn fetch_data() {
    unsafe { FLUTTER_ROUTER.write().fetch_data(); }
}

mod tests {
    use std::str::FromStr;
    use rust_decimal::Decimal;
    use super::*;

    #[test]
    fn it_works() {
        // 1. load config
        let tokens = load_tokens();
        println!("Loaded tokens: {:?}", tokens);
        // assert_eq!(tokens.len(), 3);
        // assert_eq!(tokens[0].symbol, "USDT");
        // assert_eq!(tokens[1].symbol, "BTC");
        // 2. set account
        set_account("0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string());
        assert_eq!(get_router().config.selected_account, Some("0xaC7c1a2fFb8b3f3bEa3e6aB4bC8b1A2Ff4Bb4Aa4".to_string()));
    }
}
