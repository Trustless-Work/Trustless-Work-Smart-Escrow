use soroban_sdk::{Address, Env};
use soroban_sdk::token::Client as TokenClient;

use crate::storage::types::DataKey;
use crate::error::ContractError;
use crate::events::escrows_by_engagement_id;
use crate::core::escrow::EscrowManager;

pub struct DisputeManager;

impl DisputeManager {
    
    pub fn resolving_disputes(
        e: Env,
        dispute_resolver: Address,
        approver_funds: i128,
        service_provider_funds: i128,
        trustless_work_address: Address
    ) -> Result<(), ContractError> {
        dispute_resolver.require_auth();
    
        let escrow_result = EscrowManager::get_escrow(e.clone());
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };
    
        if dispute_resolver != escrow.dispute_resolver {
            return Err(ContractError::OnlyDisputeResolverCanExecuteThisFunction);
        }
    
        if !escrow.dispute_flag {
            return Err(ContractError::EscrowNotInDispute);
        }
 
        let usdc_approver = TokenClient::new(&e, &escrow.trustline);
        let escrow_balance = usdc_approver.balance(&e.current_contract_address());

        let total_funds = approver_funds.checked_add(service_provider_funds).ok_or(ContractError::Overflow)?;
        if total_funds > escrow_balance {
            return Err(ContractError::InsufficientFundsForResolution);
        }

        let trustless_work_fee = (total_funds * 30) / 10000;
        let platform_fee = (total_funds * escrow.platform_fee) / 10000;
        let total_fees = trustless_work_fee + platform_fee;

        let net_approver_funds = approver_funds - (approver_funds * total_fees) / total_funds;
        let net_provider_funds = service_provider_funds - (service_provider_funds * total_fees) / total_funds;
        
        if approver_funds < net_approver_funds {
            return Err(ContractError::InsufficientApproverFundsForCommissions);
        }

        if service_provider_funds < net_provider_funds {
            return Err(ContractError::InsufficientServiceProviderFundsForCommissions);
        }

        usdc_approver.transfer(
            &e.current_contract_address(),
            &trustless_work_address,
            &trustless_work_fee
        );

        usdc_approver.transfer(
            &e.current_contract_address(),
            &escrow.platform_address,
            &platform_fee
        );
    
        if net_approver_funds > 0 {
            usdc_approver.transfer(
                &e.current_contract_address(),
                &escrow.approver,
                &net_approver_funds
            );
        }

        if net_provider_funds > 0 {
            usdc_approver.transfer(
                &e.current_contract_address(),
                &escrow.service_provider,
                &net_provider_funds
            );
        }
    
        escrow.resolved_flag = true;
        e.storage().instance().set(&DataKey::Escrow, &escrow);
    
        escrows_by_engagement_id(&e, escrow.engagement_id.clone(), escrow);
    
        Ok(())
    }

    pub fn change_dispute_flag(
        e: Env, 
    ) -> Result<(), ContractError> {
    
        let escrow_result = EscrowManager::get_escrow(e.clone());
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
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