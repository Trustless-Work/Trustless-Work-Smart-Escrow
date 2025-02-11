use soroban_sdk::{Address, Env, Symbol, Val, Vec};
use soroban_sdk::token::Client as TokenClient;

use crate::storage::types::{Escrow, DataKey, AddressBalance};
use crate::error::ContractError;

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
    
        let usdc_approver = TokenClient::new(&e, &escrow.trustline);

        let signer_balance = usdc_approver.balance(&signer);

        let contract_address = e.current_contract_address();
        
        if usdc_approver.balance(&contract_address) as i128 > escrow.amount {
            return Err(ContractError::EscrowFullyFunded);
        }

        if amount_to_deposit as i128 > escrow.amount {
            return Err(ContractError::AmountToDepositGreatherThanEscrowAmount);
        }

        if signer_balance < amount_to_deposit {
            return Err(ContractError::SignerInsufficientFunds);
        }

        usdc_approver.transfer(&signer, &contract_address, &amount_to_deposit);
    
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
        let mut escrow = match escrow_result {
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
    
        let usdc_approver = TokenClient::new(&e, &escrow.trustline);
        let contract_address = e.current_contract_address();
    
        // Check the actual balance of the contract for this escrow
        let contract_balance = usdc_approver.balance(&contract_address);
        if contract_balance < escrow.amount as i128 {
            return Err(ContractError::EscrowBalanceNotSufficienteToSendEarnings);
        }
    
        let platform_fee_percentage = escrow.platform_fee as i128;
        let platform_address = escrow.platform_address.clone();
    
        let total_amount = escrow.amount as i128;
        let trustless_work_commission = total_amount.checked_mul(30).ok_or(ContractError::Overflow)?.checked_div(10000)
        .ok_or(ContractError::DivisionError)?; 
        let platform_commission = total_amount.checked_mul(platform_fee_percentage).ok_or(ContractError::Overflow)?.checked_div(10000_i128)
        .ok_or(ContractError::DivisionError)?;

        usdc_approver.transfer(
            &contract_address, 
            &trustless_work_address, 
            &trustless_work_commission
        );
    
        usdc_approver.transfer(
            &contract_address, 
            &platform_address, 
            &platform_commission
        );
    
        let service_provider_amount = total_amount.checked_sub(trustless_work_commission).ok_or(ContractError::Underflow)?.checked_sub(platform_commission).ok_or(ContractError::Underflow)?;
    
        usdc_approver.transfer(
            &contract_address, 
            &escrow.service_provider, 
            &service_provider_amount
        );

        escrow.release_flag = true;
    
        e.storage().instance().set(&DataKey::Escrow, &escrow);
    
        Ok(())
    }

    pub fn change_escrow_properties(
        e: Env,
        plataform_address: Address,
        escrow_properties: Escrow
    ) -> Result<Escrow, ContractError> {
        let escrow_result = Self::get_escrow(e.clone());
        let existing_escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        if plataform_address != existing_escrow.platform_address {
            return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
        }
        
        plataform_address.require_auth();

        e.storage().instance().set(
            &DataKey::Escrow,
            &escrow_properties
        );

        Ok(escrow_properties)
    }

    pub fn get_multiple_escrow_balances(
        e: Env,
        addresses: Vec<Address>
    ) -> Result<Vec<AddressBalance>, ContractError> {
        let mut balances: Vec<AddressBalance> = Vec::new(&e);
        
        for address in addresses.iter() {
            let escrow_result = Self::get_escrow_by_contract_id(e.clone(), &address);
            let escrow = match escrow_result {
                Ok(esc) => esc,
                Err(err) => return Err(err),
            };
    
            let token_approver = TokenClient::new(&e, &escrow.trustline);
            let balance = token_approver.balance(&address);

            balances.push_back(AddressBalance {
                address: address.clone(),
                balance,
            })
        }
        
        Ok(balances)
    }
    
    pub fn get_escrow_by_contract_id(e: Env, contract_id: &Address) -> Result<Escrow, ContractError> {
        let args: Vec<Val> = Vec::new(&e);

        let result = e.invoke_contract::<Escrow>(
            contract_id,
            &Symbol::new(&e, "get_escrow"),
            args.try_into().unwrap()
        );

        Ok(result)
    }

    pub fn get_escrow(e: Env) -> Result<Escrow, ContractError> {
        let escrow = e.storage()
            .instance()
            .get::<_, Escrow>(&DataKey::Escrow)
            .ok_or(ContractError::EscrowNotFound);
        Ok(escrow?)
    }
}