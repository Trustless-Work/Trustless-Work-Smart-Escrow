use soroban_sdk::{ Address, Env, String };
use soroban_sdk::token::Client as TokenClient;

use crate::storage::types::{DataKey, Milestone};
use crate::error::ContractError;
use crate::events::escrows_by_engagement_id;
use crate::core::escrow::EscrowManager;

pub struct DisputeManager;

impl DisputeManager {

    pub fn resolving_milestone_disputes(
        e: Env,
        dispute_resolver: Address,
        milestone_index: u32,
        client_funds: i128,
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

        let total_funds = client_funds + service_provider_funds;
        if total_funds != milestone.amount {
            return Err(ContractError::InsufficientFundsForResolution);
        }

        let trustless_work_fee = (total_funds * 30) / 10000; // 0.3%
        let platform_fee = (total_funds * escrow.platform_fee) / 10000;
        let total_fees = trustless_work_fee + platform_fee;

        let net_client_funds = client_funds - (client_funds * total_fees) / total_funds;
        let net_provider_funds =
            service_provider_funds - (service_provider_funds * total_fees) / total_funds;

        let usdc_client = TokenClient::new(&e, &escrow.trustline);
        let contract_address = e.current_contract_address();

        if trustless_work_fee > 0 {
            usdc_client.transfer(&contract_address, &trustless_work_address, &trustless_work_fee);
        }
        if platform_fee > 0 {
            usdc_client.transfer(&contract_address, &escrow.platform_address, &platform_fee);
        }

        if net_client_funds > 0 {
            usdc_client.transfer(&contract_address, &escrow.client, &net_client_funds);
        }
        if net_provider_funds > 0 {
            usdc_client.transfer(&contract_address, &escrow.service_provider, &net_provider_funds);
        }

        let mut updated_milestones = escrow.milestones.clone();

        updated_milestones.set(
            milestone_index,
            Milestone {
                status: String::from_str(&e, "resolved"),
                ..milestone.clone()
            }
        );

        escrow.milestones = updated_milestones;

        e.storage().instance().set(&DataKey::Escrow, &escrow);
        escrows_by_engagement_id(&e, escrow.engagement_id.clone(), escrow);

        Ok(())
    }

    pub fn change_dispute_flag(e: Env) -> Result<(), ContractError> {
        let escrow_result = EscrowManager::get_escrow(e.clone());
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => {
                return Err(err);
            }
        };

        if escrow.dispute_flag {
            return Err(ContractError::EscrowAlreadyInDispute);
        }

        escrow.dispute_flag = true;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        escrows_by_engagement_id(&e, escrow.engagement_id.clone(), escrow);

        Ok(())
    }
}
