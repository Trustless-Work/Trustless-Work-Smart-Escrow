use soroban_sdk::{Address, Env, Symbol, Vec};
use soroban_sdk::token::Client as TokenClient;

use crate::modules::fee::{FeeCalculator, FeeCalculatorTrait};
use crate::storage::types::{Escrow, DataKey, AddressBalance, Milestone};
use crate::error::ContractError;

use super::validators::escrow::{validate_escrow_property_change_conditions, validate_initialize_escrow_conditions, validate_release_conditions};

pub struct EscrowManager;

impl EscrowManager{
    pub fn get_receiver(escrow: &Escrow) -> Address {
        if escrow.roles.receiver == escrow.roles.service_provider {
            escrow.roles.service_provider.clone()
        } else {
            escrow.roles.receiver.clone()
        }
    }

    pub fn initialize_escrow(
        e: Env,
        escrow_properties: Escrow
    ) -> Result<Escrow, ContractError> {

        validate_initialize_escrow_conditions(e.clone(), escrow_properties.clone())?;
        
        e.storage().instance().set(&DataKey::Escrow, &escrow_properties);

        Ok(escrow_properties)
    }

    pub fn fund_escrow(
        e: Env, 
        signer: Address, 
        amount_to_deposit: i128
    ) -> Result<(), ContractError> {
        signer.require_auth();

        let escrow = EscrowManager::get_escrow(e.clone())?;
        let token_client = TokenClient::new(&e, &escrow.trustline.address);

        token_client.transfer(&signer, &e.current_contract_address(), &amount_to_deposit);
    
        e.storage().instance().set(&DataKey::Escrow, &escrow);
    
        Ok(())
    }

    pub fn release_milestone_funds(
        e: Env, 
        release_signer: Address, 
        trustless_work_address: Address,
        milestone_index: u32
    ) -> Result<(), ContractError> {      
        release_signer.require_auth();
          
        let mut escrow = EscrowManager::get_escrow(e.clone())?;
        
        if let Some(milestone) = escrow.milestones.get(milestone_index) {
            validate_release_conditions(&escrow, &milestone, &release_signer, milestone_index)?;
    
            let mut updated_milestones = Vec::<Milestone>::new(&e);
            for (index, milestone) in escrow.milestones.iter().enumerate() {
                let mut new_milestone = milestone.clone();
                if index as u32 == milestone_index {
                    new_milestone.flags.released = true;
                }
                updated_milestones.push_back(new_milestone);
            }
    
            escrow.milestones = updated_milestones;
    
            e.storage().instance().set(&DataKey::Escrow, &escrow);

            let contract_address = e.current_contract_address();
            let token_client = TokenClient::new(&e, &escrow.trustline.address);
            if token_client.balance(&contract_address) < milestone.amount {
                return Err(ContractError::EscrowBalanceNotEnoughToSendEarnings);
            }
    
            let fee_result = FeeCalculator::calculate_standard_fees(
                milestone.amount as i128, 
                escrow.platform_fee as i128
            )?;
            let platform_address = escrow.roles.platform_address.clone();
    
            token_client.transfer(
                &contract_address,
                &trustless_work_address, 
                &fee_result.trustless_work_fee
            );
    
            token_client.transfer(
                &contract_address,
                &platform_address, 
                &fee_result.platform_fee
            );
    
            let receiver = Self::get_receiver(&escrow);
    
            token_client.transfer(
                &contract_address,
                &receiver, 
                &fee_result.receiver_amount
            );
        } else {
            return Err(ContractError::MilestoneNotFound);
        }

        Ok(())
    }

    pub fn change_escrow_properties(
        e: Env,
        platform_address: Address,
        escrow_properties: Escrow
    ) -> Result<Escrow, ContractError> {
        platform_address.require_auth();

        let escrow = EscrowManager::get_escrow(e.clone())?;

        let token_client = TokenClient::new(&e, &escrow.trustline.address);
        let contract_balance = token_client.balance(&e.current_contract_address());

        validate_escrow_property_change_conditions(
            &escrow,
            &platform_address,
            contract_balance,
            escrow.milestones.clone(),
        )?;

        e.storage()
            .instance()
            .set(&DataKey::Escrow, &escrow_properties);

        Ok(escrow_properties)
    }

    pub fn get_multiple_escrow_balances(
        e: Env,
        signer: Address,
        addresses: Vec<Address>
    ) -> Result<Vec<AddressBalance>, ContractError> {
        signer.require_auth();
        
        const MAX_ESCROWS: u32 = 20;
        if addresses.len() > MAX_ESCROWS {
            return Err(ContractError::TooManyEscrowsRequested);
        }

        let mut balances: Vec<AddressBalance> = Vec::new(&e);
        for address in addresses.iter() {
            let escrow = Self::get_escrow_by_contract_id(e.clone(), &address)?;

            let token_client = TokenClient::new(&e, &escrow.trustline.address);
            let balance = token_client.balance(&address);

            balances.push_back(AddressBalance {
                address: address.clone(),
                balance,
                trustline_decimals: escrow.trustline.decimals,
            })
        }

        Ok(balances)
    }
    
    pub fn get_escrow_by_contract_id(e: Env, contract_id: &Address) -> Result<Escrow, ContractError> {
        Ok(e.invoke_contract::<Escrow>(contract_id, &Symbol::new(&e, "get_escrow"), Vec::new(&e)))
    }

    pub fn get_escrow(e: Env) -> Result<Escrow, ContractError> {
        e.storage()
        .instance()
        .get(&DataKey::Escrow)
        .ok_or(ContractError::EscrowNotFound)?
    }
}