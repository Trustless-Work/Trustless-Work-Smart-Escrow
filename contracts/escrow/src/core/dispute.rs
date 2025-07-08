use soroban_sdk::{ Address, Env, String, Vec };
use soroban_sdk::token::Client as TokenClient;

use crate::modules::{
    math::{BasicArithmetic, BasicMath}, 
    fee::{FeeCalculator, FeeCalculatorTrait}
};
use crate::storage::types::{DataKey, Milestone, Escrow};
use crate::error::ContractError;
use crate::core::escrow::EscrowManager;
use crate::events::escrows_by_contract_id;

use super::validators::dispute::{validate_dispute_flag_change_conditions, validate_dispute_resolution_conditions};

pub struct DisputeManager;

impl DisputeManager {

    pub fn resolve_milestone_dispute(
        e: Env,
        dispute_resolver: Address,
        milestone_index: u32,
        approver_funds: i128,
        receiver_funds: i128,
        trustless_work_address: Address
    ) -> Result<(), ContractError> {
        dispute_resolver.require_auth();

        let mut escrow = EscrowManager::get_escrow(e.clone())?;
        let contract_address = e.current_contract_address();

        let token_client = TokenClient::new(&e, &escrow.trustline.address);

        let milestones = escrow.milestones.clone();
        let milestone = match milestones.get(milestone_index) {
            Some(m) => m,
            None => {
                return Err(ContractError::InvalidMileStoneIndex);
            }
        };

        let total_funds = BasicMath::safe_add(approver_funds, receiver_funds)?;
        if token_client.balance(&contract_address) < total_funds {
            return Err(ContractError::InsufficientFundsForResolution);
        }

        let fee_result = FeeCalculator::calculate_dispute_fees(
            approver_funds,
            receiver_funds,
            escrow.platform_fee as i128,
            total_funds,
        )?;

        validate_dispute_resolution_conditions(
            &escrow,
            &milestone,
            &dispute_resolver,
            approver_funds,
            receiver_funds,
            &fee_result,
            total_funds,
        )?;

        token_client.transfer(&contract_address, &trustless_work_address, &fee_result.trustless_work_fee);

        token_client.transfer(&contract_address, &escrow.roles.platform_address, &fee_result.platform_fee);

        if fee_result.net_approver_funds > 0 {
            token_client.transfer(&contract_address, &escrow.roles.approver, &fee_result.net_approver_funds);
        }
        if fee_result.net_provider_funds > 0 {
            token_client.transfer(&contract_address, &escrow.roles.receiver, &fee_result.net_provider_funds);
        }

        let mut updated_milestones = escrow.milestones.clone();

        let mut new_flags = milestone.flags.clone();
        new_flags.resolved = true;
        new_flags.disputed = false;
        
        updated_milestones.set(
            milestone_index,
            Milestone {
                status: String::from_str(&e, "resolved"),
                flags: new_flags,
                ..milestone.clone()
            }
        );

        escrow.milestones = updated_milestones;

        e.storage().instance().set(&DataKey::Escrow, &escrow);
        escrows_by_contract_id(&e, escrow.engagement_id.clone(), escrow);

        Ok(())
    }

    pub fn dispute_milestone(
        e: Env,
        milestone_index: i128,
        signer: Address,
    ) -> Result<(), ContractError> {
        signer.require_auth();
        
        let escrow = EscrowManager::get_escrow(e.clone())?;

        validate_dispute_flag_change_conditions(
            &escrow,
            milestone_index,
            &signer,
        )?;

        let mut updated_milestones = Vec::new(&e);
        for (index, milestone) in escrow.milestones.iter().enumerate() {
            let mut new_milestone = milestone.clone();
            if index as i128 == milestone_index {
                new_milestone.flags.disputed = true;
            }
            updated_milestones.push_back(new_milestone);
        }

        let updated_escrow = Escrow {
            milestones: updated_milestones,
            ..escrow
        };

        e.storage().instance().set(
            &DataKey::Escrow,
            &updated_escrow,
        );

        escrows_by_contract_id(&e, updated_escrow.engagement_id.clone(), updated_escrow);

        Ok(())
    }
}
