use soroban_sdk::{token::Client as TokenClient, Address, Env};

use crate::error::ContractError;
pub trait TokenTransferHandlerTrait<'a> {
    fn transfer(&self, to: &Address, amount: &i128);
    fn has_sufficient_balance(&self, amount: i128) -> Result<(), ContractError>;
    fn balance(&self, address: &Address) -> i128;
}
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
}

impl<'a> TokenTransferHandlerTrait<'a> for TokenTransferHandler<'a> {
    fn transfer(&self, to: &Address, amount: &i128) {
        self.token_client.transfer(&self.source_address, to, amount)
    }

    fn has_sufficient_balance(&self, amount: i128) -> Result<(), ContractError> {
        let balance = self.balance(&self.source_address);
        if balance < amount {
            return Err(ContractError::EscrowBalanceNotEnoughToSendEarnings);
        }
        Ok(())
    }

    fn balance(&self, address: &Address) -> i128 {
        self.token_client.balance(address)
    }
}