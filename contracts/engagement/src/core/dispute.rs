use soroban_sdk::{ Address, Env, String, Vec };
use soroban_sdk::token::Client as TokenClient;

use crate::storage::types::{DataKey, Milestone, Escrow};
use crate::error::ContractError;
use crate::events::escrows_by_engagement_id;
use crate::core::escrow::EscrowManager;

pub struct DisputeManager;

impl DisputeManager {

    pub fn resolving_milestone_disputes(
        e: Env,
        dispute_resolver: Address,
        milestone_index: u32,
        approver_funds: i128,
        service_provider_funds: i128,
        trustless_work_address: Address
    ) -> Result<(), ContractError> {
        dispute_resolver.require_auth();

        let escrow_result = EscrowManager::get_escrow(e.clone());
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => {
                return Err(err);
            }
        };

        if dispute_resolver != escrow.dispute_resolver {
            return Err(ContractError::OnlyDisputeResolverCanExecuteThisFunction);
        }

        let milestones = escrow.milestones.clone();
        let milestone = match milestones.get(milestone_index) {
            Some(m) => m,
            None => {
                return Err(ContractError::InvalidMileStoneIndex);
            }
        };

        if !milestone.dispute_flag {
            return Err(ContractError::MilestoneNotInDispute);
        }

        let total_funds = approver_funds
            .checked_add(service_provider_funds)
            .ok_or(ContractError::Overflow)?;
        if total_funds != milestone.amount {
            return Err(ContractError::InsufficientFundsForResolution);
        }

        let trustless_work_fee = total_funds
            .checked_mul(30)
            .ok_or(ContractError::Overflow)?
            .checked_div(10000)
            .ok_or(ContractError::DivisionError)?; // 0.3%
        let platform_fee = total_funds
            .checked_mul(escrow.platform_fee)
            .ok_or(ContractError::Overflow)?
            .checked_div(10000)
            .ok_or(ContractError::DivisionError)?;
        let total_fees = trustless_work_fee
            .checked_add(platform_fee)
            .ok_or(ContractError::Overflow)?;

        let approver_fee = approver_funds
            .checked_mul(total_fees)
            .ok_or(ContractError::Overflow)?
            .checked_div(total_funds)
            .ok_or(ContractError::DivisionError)?;
        let net_approver_funds = approver_funds
            .checked_sub(approver_fee)
            .ok_or(ContractError::Underflow)?;
        let fees_portion = service_provider_funds
            .checked_mul(total_fees)
            .ok_or(ContractError::Overflow)?
            .checked_div(total_funds)
            .ok_or(ContractError::DivisionError)?;
        let net_provider_funds = service_provider_funds
            .checked_sub(fees_portion)
            .ok_or(ContractError::Underflow)?;

        let usdc_approver = TokenClient::new(&e, &escrow.trustline);
        let contract_address = e.current_contract_address();

        if trustless_work_fee > 0 {
            usdc_approver.transfer(&contract_address, &trustless_work_address, &trustless_work_fee);
        }
        if platform_fee > 0 {
            usdc_approver.transfer(&contract_address, &escrow.platform_address, &platform_fee);
        }

        if net_approver_funds > 0 {
            usdc_approver.transfer(&contract_address, &escrow.approver, &net_approver_funds);
        }
        if net_provider_funds > 0 {
            usdc_approver.transfer(&contract_address, &escrow.service_provider, &net_provider_funds);
        }

        let mut updated_milestones = escrow.milestones.clone();

        updated_milestones.set(
            milestone_index,
            Milestone {
                status: String::from_str(&e, "resolved"),
                resolved_flag: true,
                ..milestone.clone()
            }
        );

        escrow.milestones = updated_milestones;

        e.storage().instance().set(&DataKey::Escrow, &escrow);
        escrows_by_engagement_id(&e, escrow.engagement_id.clone(), escrow);

        Ok(())
    }

    pub fn change_milestone_dispute_flag(
        e: Env,
        milestone_index: i128,
    ) -> Result<(), ContractError> {
        let escrow_result = EscrowManager::get_escrow(e.clone());
        let existing_escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        if existing_escrow.milestones.is_empty() {
            return Err(ContractError::NoMileStoneDefined);
        }

        if milestone_index < 0 || milestone_index >= existing_escrow.milestones.len() as i128 {
            return Err(ContractError::InvalidMileStoneIndex);
        }

        let milestone = existing_escrow.milestones.get(milestone_index as u32)
            .ok_or(ContractError::InvalidMileStoneIndex)?;
        
        if milestone.dispute_flag {
            return Err(ContractError::MilestoneAlreadyInDispute);
        }

        let mut updated_milestones = Vec::new(&e);
        for (index, milestone) in existing_escrow.milestones.iter().enumerate() {
            let mut new_milestone = milestone.clone();
            if index as i128 == milestone_index {
                new_milestone.dispute_flag = true;
            }
            updated_milestones.push_back(new_milestone);
        }

        let updated_escrow = Escrow {
            milestones: updated_milestones,
            ..existing_escrow
        };

        e.storage().instance().set(
            &DataKey::Escrow,
            &updated_escrow,
        );

        escrows_by_engagement_id(&e, updated_escrow.engagement_id.clone(), updated_escrow);

        Ok(())
    }
}
