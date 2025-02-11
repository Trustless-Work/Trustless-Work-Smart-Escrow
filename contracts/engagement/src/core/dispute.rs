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
        client_funds: i128,
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
 
        let usdc_client = TokenClient::new(&e, &escrow.trustline);
        let escrow_balance = usdc_client.balance(&e.current_contract_address());

        let total_funds = client_funds.checked_add(service_provider_funds).ok_or(ContractError::Overflow)?;
        if total_funds > escrow_balance {
            return Err(ContractError::InsufficientFundsForResolution);
        }

        let trustless_work_commission = total_funds * 0.003 as i128;
        let platform_fee = escrow.platform_fee;
        let client_deductions: i128 = client_funds - platform_fee - trustless_work_commission;
        let service_provider_deductions: i128 = service_provider_funds - platform_fee - trustless_work_commission;
        
        if client_funds < client_deductions {
            return Err(ContractError::InsufficientClientFundsForCommissions);
        }

        if service_provider_funds < service_provider_deductions {
            return Err(ContractError::InsufficientServiceProviderFundsForCommissions);
        }
        
        let adjusted_client_funds = client_funds - client_deductions;
        let adjusted_service_provider_funds = service_provider_funds - service_provider_deductions;

        usdc_client.transfer(
            &e.current_contract_address(),
            &trustless_work_address,
            &trustless_work_commission
        );

        usdc_client.transfer(
            &e.current_contract_address(),
            &escrow.platform_address,
            &platform_fee
        );
    
        if adjusted_client_funds > 0 {
            usdc_client.transfer(
                &e.current_contract_address(),
                &escrow.client,
                &adjusted_client_funds
            );
        }

        if adjusted_service_provider_funds > 0 {
            usdc_client.transfer(
                &e.current_contract_address(),
                &escrow.service_provider,
                &adjusted_service_provider_funds
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