use soroban_sdk::{Address, Env, Symbol, Val, Vec};
use soroban_sdk::token::Client as TokenClient;

use crate::storage::types::{Escrow, DataKey, AddressBalance, Milestone};
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
    
        let usdc_client = TokenClient::new(&e, &escrow.trustline);

        let signer_balance = usdc_client.balance(&signer);

        let contract_address = e.current_contract_address();

        if signer_balance < amount_to_deposit {
            return Err(ContractError::SignerInsufficientFunds);
        }

        usdc_client.transfer(&signer, &contract_address, &amount_to_deposit);
    
        e.storage().instance().set(&DataKey::Escrow, &escrow);
    
        Ok(())
    }

    pub fn release_milestone_payment(
        e: Env, 
        release_signer: Address, 
        trustless_work_address: Address,
        milestone_index: u32
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
    
        if milestone_index >= escrow.milestones.len() {
            return Err(ContractError::InvalidMileStoneIndex);
        }

        let milestone = escrow.milestones.get(milestone_index).unwrap();

        if !milestone.approved_flag {
            return Err(ContractError::EscrowNotCompleted);
        }

        if milestone.dispute_flag {
            return Err(ContractError::CantReleaseAMilestoneInDispute);
        }

        let usdc_client = TokenClient::new(&e, &escrow.trustline);
        let contract_address = e.current_contract_address();

        let contract_balance = usdc_client.balance(&contract_address);
        if contract_balance < milestone.amount as i128 {
            return Err(ContractError::EscrowBalanceNotSufficienteToSendEarnings);
        }

        let platform_fee_percentage = escrow.platform_fee as i128;
        let platform_address = escrow.platform_address.clone();

        let total_amount = milestone.amount as i128;
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

        let mut updated_milestones = Vec::<Milestone>::new(&e);
        for (index, milestone) in escrow.milestones.iter().enumerate() {
            let mut new_milestone = milestone.clone();
            if index as u32 == milestone_index {
                new_milestone.release_flag = true;
            }
            updated_milestones.push_back(new_milestone);
        }

        escrow.milestones = updated_milestones;

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
    
            let token_client = TokenClient::new(&e, &escrow.trustline);
            let balance = token_client.balance(&address);

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