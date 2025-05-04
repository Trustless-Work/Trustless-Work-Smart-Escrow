use crate::core::escrow::EscrowManager;
use crate::error::ContractError;
use crate::events::escrows_by_contract_id;
use crate::modules::{
    fee::{FeeCalculator, FeeCalculatorTrait},
    math::{BasicArithmetic, BasicMath},
    token::{TokenTransferHandler, TokenTransferHandlerTrait},
};
use crate::storage::types::DataKey;
use soroban_sdk::{Address, Env};

use super::validators::dispute::{
    validate_dispute_flag_change_conditions, validate_dispute_resolution_conditions,
};

pub struct DisputeManager;

impl DisputeManager {
    pub fn resolving_disputes(
        e: Env,
        dispute_resolver: Address,
        approver_funds: i128,
        service_provider_funds: i128,
        trustless_work_address: Address,
    ) -> Result<(), ContractError> {
        dispute_resolver.require_auth();

        let escrow_result = EscrowManager::get_escrow(e.clone());
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        let transfer_handler =
            TokenTransferHandler::new(&e, &escrow.trustline.address, &e.current_contract_address());

        let total_funds = BasicMath::safe_add(approver_funds, service_provider_funds)?;
        transfer_handler.has_sufficient_balance(total_funds)?;

        let fee_result = FeeCalculator::calculate_dispute_fees(
            approver_funds,
            service_provider_funds,
            escrow.platform_fee as i128,
            total_funds,
        )?;

        validate_dispute_resolution_conditions(
            &escrow,
            &dispute_resolver,
            approver_funds,
            service_provider_funds,
            &fee_result,
        )?;

        transfer_handler.transfer(&trustless_work_address, &fee_result.trustless_work_fee);

        transfer_handler.transfer(&escrow.roles.platform_address, &fee_result.platform_fee);

        if fee_result.net_approver_funds > 0 {
            transfer_handler.transfer(&escrow.roles.approver, &fee_result.net_approver_funds);
        }

        if fee_result.net_provider_funds > 0 {
            let receiver = EscrowManager::get_receiver(&escrow);
            transfer_handler.transfer(&receiver, &fee_result.net_provider_funds);
        }

        escrow.flags.resolved = true;
        escrow.flags.dispute = false;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        escrows_by_contract_id(&e, escrow.engagement_id.clone(), escrow);

        Ok(())
    }

    pub fn change_dispute_flag(e: Env, signer: Address) -> Result<(), ContractError> {
        let escrow_result = EscrowManager::get_escrow(e.clone());
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        validate_dispute_flag_change_conditions(&escrow, &signer)?;

        escrow.flags.dispute = true;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        escrows_by_contract_id(&e, escrow.engagement_id.clone(), escrow);

        Ok(())
    }
}
