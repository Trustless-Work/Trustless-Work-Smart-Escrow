use soroban_sdk::{Address, Env};
use soroban_sdk::token::Client as TokenClient;

use crate::core::escrow::EscrowManager;
use crate::error::ContractError;
use crate::modules::{
    fee::{FeeCalculator, FeeCalculatorTrait},
    math::{BasicArithmetic, BasicMath},
};
use crate::storage::types::{DataKey, Escrow};

use super::validators::dispute::{
    validate_dispute_flag_change_conditions, validate_dispute_resolution_conditions,
};

pub struct DisputeManager;

impl DisputeManager {
    pub fn resolve_dispute(
        e: &Env,
        dispute_resolver: Address,
        trustless_work_address: Address,
        approver_funds: i128,
        receiver_funds: i128,
    ) -> Result<Escrow, ContractError> {
        dispute_resolver.require_auth();
        let mut escrow = EscrowManager::get_escrow(e)?;
        let contract_address = e.current_contract_address();
        
        let token_client = TokenClient::new(&e, &escrow.trustline.address);
        let current_balance = token_client.balance(&contract_address);
        let total_funds = BasicMath::safe_add(approver_funds, receiver_funds)?;

        let fee_result = FeeCalculator::calculate_dispute_fees(
            approver_funds,
            receiver_funds,
            escrow.platform_fee,
            total_funds
        )?;

        validate_dispute_resolution_conditions(
            &escrow,
            &dispute_resolver,
            approver_funds,
            receiver_funds,
            total_funds,
            current_balance,
        )?;

        token_client.transfer(&contract_address, &trustless_work_address, &fee_result.trustless_work_fee);
        token_client.transfer(&contract_address, &escrow.roles.platform_address, &fee_result.platform_fee);

        if fee_result.net_approver_funds > 0 {
            token_client.transfer(&contract_address, &escrow.roles.approver, &fee_result.net_approver_funds);
        }

        if fee_result.net_receiver_funds > 0 {
            let receiver = EscrowManager::get_receiver(&escrow);
            token_client.transfer(&contract_address, &receiver, &fee_result.net_receiver_funds);
        }

        escrow.flags.resolved = true;
        escrow.flags.disputed = false;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        Ok(escrow)
    }

    pub fn dispute_escrow(e: &Env, signer: Address) -> Result<Escrow, ContractError> {
        signer.require_auth();
        let mut escrow = EscrowManager::get_escrow(e)?;
        validate_dispute_flag_change_conditions(&escrow, &signer)?;

        escrow.flags.disputed = true;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        Ok(escrow)
    }
}
