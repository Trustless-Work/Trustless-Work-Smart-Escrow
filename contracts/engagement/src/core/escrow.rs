use soroban_sdk::{Address, Env};
use soroban_sdk::token::Client as TokenClient;

use crate::storage::types::{Escrow, DataKey};
use crate::error::ContractError;
use crate::events::escrows_by_engagement_id;

pub struct EscrowManager;

impl EscrowManager{

    pub fn initialize_escrow(
        e: Env,
        escrow_properties: Escrow
    ) -> Result<Escrow, ContractError> {

        if e.storage().instance().has(&DataKey::Escrow) {
            return Err(ContractError::EscrowAlreadyInitialized);
        }

        if escrow_properties.amount == 0 {
            return Err(ContractError::AmountCannotBeZero);
        }
        
        e.storage().instance().set(&DataKey::Escrow, &escrow_properties);

        Ok(escrow_properties)
    }

    pub fn fund_escrow(
        e: Env, 
        signer: Address, 
        amount_to_deposit: i128
    ) -> Result<(), ContractError> {
        signer.require_auth();

        let escrow_result = Self::get_escrow(e.clone());
        let escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        if escrow.dispute_flag {
            return Err(ContractError::EscrowOpenedForDisputeResolution);
        }
    
        let usdc_client = TokenClient::new(&e, &escrow.trustline);

        let signer_balance = usdc_client.balance(&signer);

        let contract_address = e.current_contract_address();
        
        if usdc_client.balance(&contract_address) as i128 > escrow.amount {
            return Err(ContractError::EscrowFullyFunded);
        }

        if amount_to_deposit as i128 > escrow.amount {
            return Err(ContractError::AmountToDepositGreatherThanEscrowAmount);
        }

        if signer_balance < amount_to_deposit {
            return Err(ContractError::SignerInsufficientFunds);
        }

        usdc_client.transfer(&signer, &contract_address, &amount_to_deposit);
    
        e.storage().instance().set(&DataKey::Escrow, &escrow);
    
        Ok(())
    }

    pub fn distribute_escrow_earnings(
        e: Env, 
        release_signer: Address, 
        trustless_work_address: Address
    ) -> Result<(), ContractError> {
        release_signer.require_auth();
        
        let escrow_result = Self::get_escrow(e.clone());
        let escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };
        
        if release_signer != escrow.release_signer {
            return Err(ContractError::OnlyReleaseSignerCanClaimEarnings);
        }
    
        if escrow.milestones.is_empty() {
            return Err(ContractError::NoMileStoneDefined);
        }
    
        if !escrow.milestones.iter().all(|milestone| milestone.approved_flag) {
            return Err(ContractError::EscrowNotCompleted);
        }
    
        if escrow.dispute_flag {
            return Err(ContractError::InvalidState);
        }
    
        let usdc_client = TokenClient::new(&e, &escrow.trustline);
        let contract_address = e.current_contract_address();
    
        // Check the actual balance of the contract for this escrow
        let contract_balance = usdc_client.balance(&contract_address);
        if contract_balance < escrow.amount as i128 {
            return Err(ContractError::EscrowBalanceNotSufficienteToSendEarnings);
        }
    
        let platform_fee_percentage = escrow.platform_fee as i128;
        let platform_address = escrow.platform_address.clone();
    
        let total_amount = escrow.amount as i128;
        let trustless_work_commission = ((total_amount * 30) / 10000) as i128; 
        let platform_commission = (total_amount * platform_fee_percentage) / 10000 as i128;
            
        usdc_client.transfer(
            &contract_address, 
            &trustless_work_address, 
            &trustless_work_commission
        );
    
        usdc_client.transfer(
            &contract_address, 
            &platform_address, 
            &platform_commission
        );
    
        let service_provider_amount = total_amount - trustless_work_commission - platform_commission;
    
        usdc_client.transfer(
            &contract_address, 
            &escrow.service_provider, 
            &service_provider_amount
        );
    
        e.storage().instance().set(&DataKey::Escrow, &escrow);
    
        Ok(())
    }

    pub fn change_escrow_properties(
        e: Env,
        escrow_properties: Escrow
    ) -> Result<(), ContractError> {
        let escrow_result = Self::get_escrow(e.clone());
        let existing_escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        if escrow_properties.platform_address != existing_escrow.platform_address {
            return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
        }
        
        escrow_properties.platform_address.require_auth();

        e.storage().instance().set(
            &DataKey::Escrow,
            &escrow_properties
        );

        let engagement_id_copy = escrow_properties.engagement_id.clone();

        escrows_by_engagement_id(&e, engagement_id_copy, escrow_properties);

        Ok(())
    }

    pub fn get_escrow_balance(e: Env) -> Result<i128, ContractError> {
        let escrow_result = EscrowManager::get_escrow(e.clone());
        let escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };
        let usdc_client = TokenClient::new(&e, &escrow.trustline);
        let address = e.current_contract_address();
        let balance = usdc_client.balance(&address);
        Ok(balance)
    }

    pub fn get_escrow(e: Env) -> Result<Escrow, ContractError> {
        let escrow = e.storage()
            .instance()
            .get::<_, Escrow>(&DataKey::Escrow)
            .ok_or(ContractError::EscrowNotFound);
        Ok(escrow?)
    }
}