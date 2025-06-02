use soroban_sdk::{Address, Env, Symbol, Val, Vec};
use soroban_sdk::token::Client as TokenClient;

use crate::modules::{
    fee::{FeeCalculator, FeeCalculatorTrait},
    token::{TokenTransferHandler, TokenTransferHandlerTrait}
};
use crate::storage::types::{Escrow, DataKey, AddressBalance, Milestone};
use crate::error::ContractError;

use super::validators::escrow::{validate_escrow_property_change_conditions, validate_funding_conditions, validate_initialize_escrow_conditions, validate_release_conditions};

pub struct EscrowManager;

impl EscrowManager{
    /// Returns the final receiver address for escrow fund distribution.
    ///
    /// If the receiver is the same as the service provider, it returns the service provider's address.
    /// Otherwise, it returns the receiver's address.
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

        let escrow_result = Self::get_escrow(e.clone());
        let escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };
        let token_client = TokenClient::new(&e, &escrow.trustline.address);

        let signer_balance = token_client.balance(&signer);
        let contract_address = e.current_contract_address();
        let contract_balance = token_client.balance(&contract_address);

        validate_funding_conditions(&escrow.milestones, signer_balance, contract_balance, amount_to_deposit)?;

        token_client.transfer(&signer, &contract_address, &amount_to_deposit);
    
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
          
        let escrow_result = Self::get_escrow(e.clone());
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };
        
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
            let transfer_handler = TokenTransferHandler::new(&e, &escrow.trustline.address, &contract_address);
            transfer_handler.has_sufficient_balance(milestone.amount)?;
    
            let platform_fee_percentage = escrow.platform_fee as i128;
            let total_amount = milestone.amount as i128;
            let fee_result = FeeCalculator::calculate_standard_fees(total_amount, platform_fee_percentage)?;
            let platform_address = escrow.roles.platform_address.clone();
    
            transfer_handler.transfer(
                &trustless_work_address, 
                &fee_result.trustless_work_fee
            );
    
            transfer_handler.transfer(
                &platform_address, 
                &fee_result.platform_fee
            );
    
            let receiver = Self::get_receiver(&escrow);
    
            transfer_handler.transfer(
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

        let escrow_result = Self::get_escrow(e.clone());
        let existing_escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        let current_address = e.current_contract_address();
        let token_client = TokenClient::new(&e, &existing_escrow.trustline.address);
        let contract_balance = token_client.balance(&current_address);

        validate_escrow_property_change_conditions(
            &existing_escrow,
            &platform_address,
            contract_balance,
            existing_escrow.milestones.clone(),
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
            let escrow_result = Self::get_escrow_by_contract_id(e.clone(), &address);
            let escrow = match escrow_result {
                Ok(esc) => esc,
                Err(err) => return Err(err),
            };

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
        let args: soroban_sdk::Vec<Val> = soroban_sdk::Vec::new(&e);

        let result = e.invoke_contract::<Escrow>(contract_id, &Symbol::new(&e, "get_escrow"), args);

        Ok(result)
    }

    pub fn get_escrow(e: Env) -> Result<Escrow, ContractError> {
        let escrow = e
            .storage()
            .instance()
            .get::<_, Escrow>(&DataKey::Escrow)
            .ok_or(ContractError::EscrowNotFound);
        Ok(escrow?)
    }
}