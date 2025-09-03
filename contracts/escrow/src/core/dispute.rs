use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, String, Vec};

use crate::core::escrow::EscrowManager;
use crate::error::ContractError;
use crate::modules::{
    fee::{FeeCalculator, FeeCalculatorTrait},
    math::{BasicArithmetic, BasicMath},
};
use crate::storage::types::{DataKey, Escrow, Milestone};

use super::validators::dispute::{
    validate_dispute_flag_change_conditions, validate_dispute_resolution_conditions,
};

pub struct DisputeManager;

impl DisputeManager {
    pub fn resolve_milestone_dispute(
        e: &Env,
        dispute_resolver: Address,
        milestone_index: u32,
        approver_funds: i128,
        receiver_funds: i128,
        trustless_work_address: Address,
    ) -> Result<Escrow, ContractError> {
        dispute_resolver.require_auth();

        let mut escrow = EscrowManager::get_escrow(e)?;
        let contract_address = e.current_contract_address();

        let token_client = TokenClient::new(&e, &escrow.trustline.address);

        let milestones = escrow.milestones.clone();
        let milestone = match milestones.get(milestone_index) {
            Some(m) => m,
            None => {
                return Err(ContractError::InvalidMileStoneIndex);
            }
        };

        let current_balance = token_client.balance(&contract_address);
        let total_funds = BasicMath::safe_add(approver_funds, receiver_funds)?;
        if current_balance < total_funds {
            return Err(ContractError::InsufficientFundsForResolution);
        }

        let fee_result = FeeCalculator::calculate_dispute_fees(
            approver_funds,
            receiver_funds,
            escrow.platform_fee,
            total_funds,
        )?;

        validate_dispute_resolution_conditions(
            &escrow,
            &milestone,
            &dispute_resolver,
            approver_funds,
            receiver_funds,
            total_funds,
            current_balance,
        )?;

        token_client.transfer(
            &contract_address,
            &trustless_work_address,
            &fee_result.trustless_work_fee,
        );

        token_client.transfer(
            &contract_address,
            &escrow.roles.platform_address,
            &fee_result.platform_fee,
        );

        if fee_result.net_approver_funds > 0 {
            token_client.transfer(
                &contract_address,
                &escrow.roles.approver,
                &fee_result.net_approver_funds,
            );
        }

        if fee_result.net_receiver_funds > 0 {
            token_client.transfer(
                &contract_address,
                &escrow.roles.receiver,
                &fee_result.net_receiver_funds,
            );
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
            },
        );

        escrow.milestones = updated_milestones;

        e.storage().instance().set(&DataKey::Escrow, &escrow);

        Ok(escrow)
    }

    pub fn dispute_milestone(
        e: &Env,
        milestone_index: i128,
        signer: Address,
    ) -> Result<Escrow, ContractError> {
        signer.require_auth();

        let escrow = EscrowManager::get_escrow(e)?;

        validate_dispute_flag_change_conditions(&escrow, milestone_index, &signer)?;

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

        e.storage()
            .instance()
            .set(&DataKey::Escrow, &updated_escrow);

        Ok(updated_escrow)
    }
}
