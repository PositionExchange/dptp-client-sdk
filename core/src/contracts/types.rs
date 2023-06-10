use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

use super::{token::Token, vault::Vault};

pub type TokensArc = Arc<Mutex<Vec<Token>>>;
pub type VaultArc = Arc<RwLock<Vault>>;

pub fn to_tokens_arc(tokens: Vec<Token>) -> TokensArc {
    Arc::new(Mutex::new(tokens))
}
