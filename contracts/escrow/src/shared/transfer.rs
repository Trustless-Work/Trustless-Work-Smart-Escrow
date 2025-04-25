use soroban_sdk::{token::Client as TokenClient, Address, Env};
pub struct TokenTransferHandler<'a> {
    token_client: TokenClient<'a>,
    source_address: Address,
}
impl<'a> TokenTransferHandler<'a> {
    pub fn new(env: &Env, token_address: &Address, source_address: &Address) -> Self {
        Self {
            token_client: TokenClient::new(env, token_address),
            source_address: source_address.clone(),

        }
    }

    pub fn transfer(&self, to: &Address, amount: &i128) {
        self.token_client.transfer(
            &self.source_address,
            to,
            amount,
        );
    }

    pub fn balance(&self, address: &Address) -> i128 {
        self.token_client.balance(address)
    }
}