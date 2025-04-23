use soroban_sdk::Env;
use soroban_token_sdk::{metadata::TokenMetadata, TokenUtils};

#[allow(dead_code)]
pub fn write_metadata(e: &Env, metadata: TokenMetadata) {
    let util = TokenUtils::new(e);
    util.metadata().set_metadata(&metadata);
}
