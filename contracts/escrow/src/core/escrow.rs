use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Symbol, Val, Vec};

use crate::error::ContractError;
use crate::modules::{
    fee::{FeeCalculator, FeeCalculatorTrait},
    token::{TokenTransferHandler, TokenTransferHandlerTrait},
};
use crate::storage::types::{AddressBalance, DataKey, Escrow};

pub struct EscrowManager;

impl EscrowManager {
    /// Returns the final receiver address for escrow fund distribution.
    /// 
    /// If the receiver is the same as the service provider, it returns the service provider's address.
    /// Otherwise, it returns the receiver's address.
    pub fn get_receiver(escrow: &Escrow) -> Address {
        if escrow.receiver == escrow.service_provider {
            escrow.service_provider.clone()
        } else {
            escrow.receiver.clone()
        }
    }

    pub fn initialize_escrow(e: Env, escrow_properties: Escrow) -> Result<Escrow, ContractError> {
        if e.storage().instance().has(&DataKey::Escrow) {
            return Err(ContractError::EscrowAlreadyInitialized);
        }

        if escrow_properties.amount == 0 {
            return Err(ContractError::AmountCannotBeZero);
        }

        e.storage()
            .instance()
            .set(&DataKey::Escrow, &escrow_properties);

        Ok(escrow_properties)
    }

    pub fn fund_escrow(
        e: Env,
        signer: Address,
        amount_to_deposit: i128,
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

        let token_client = TokenClient::new(&e, &escrow.trustline);

        let signer_balance = token_client.balance(&signer);
        let contract_address = e.current_contract_address();
        let contract_balance = token_client.balance(&contract_address);

        if contract_balance >= escrow.amount {
            return Err(ContractError::EscrowFullyFunded);
        }

        if signer_balance < amount_to_deposit {
            return Err(ContractError::SignerInsufficientFunds);
        }

        token_client.transfer(&signer, &contract_address, &amount_to_deposit);

        e.storage().instance().set(&DataKey::Escrow, &escrow);

        Ok(())
    }

    pub fn distribute_escrow_earnings(
        e: Env,
        release_signer: Address,
        trustless_work_address: Address,
    ) -> Result<(), ContractError> {
        release_signer.require_auth();

        let escrow_result = Self::get_escrow(e.clone());
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        if release_signer != escrow.release_signer {
            return Err(ContractError::OnlyReleaseSignerCanDistributeEarnings);
        }

        if escrow.milestones.is_empty() {
            return Err(ContractError::NoMileStoneDefined);
        }

        if !escrow
            .milestones
            .iter()
            .all(|milestone| milestone.approved_flag)
        {
            return Err(ContractError::EscrowNotCompleted);
        }

        if escrow.dispute_flag {
            return Err(ContractError::EscrowOpenedForDisputeResolution);
        }

        let contract_address = e.current_contract_address();
        let transfer_handler = TokenTransferHandler::new(&e, &escrow.trustline, &contract_address);
        
        transfer_handler.has_sufficient_balance(escrow.amount)?;

        let transfer_handler = TokenTransferHandler::new(&e, &escrow.trustline, &contract_address);
        let total_amount = escrow.amount as i128;
        let platform_fee_percentage = escrow.platform_fee as i128;
        let fee_result = FeeCalculator::calculate_standard_fees(
            total_amount,
            platform_fee_percentage,
        )?;

        let platform_address = escrow.platform_address.clone();

        transfer_handler.transfer(&trustless_work_address, &fee_result.trustless_work_fee);
        transfer_handler.transfer(&platform_address, &fee_result.platform_fee);

        let receiver = Self::get_receiver(&escrow);
        transfer_handler.transfer(&receiver, &fee_result.receiver_amount);

        escrow.release_flag = true;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        Ok(())
    }

    pub fn change_escrow_properties(
        e: Env,
        plataform_address: Address,
        escrow_properties: Escrow,
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

        if !existing_escrow.milestones.is_empty() {
            for (_, milestone) in existing_escrow.milestones.iter().enumerate() {
                if milestone.approved_flag {
                    return Err(ContractError::MilestoneApprovedCantChangeEscrowProperties);
                }
            }
        }

        let current_address = e.current_contract_address();
        let token_client = TokenClient::new(&e, &existing_escrow.trustline);
        let contract_balance = token_client.balance(&current_address);

        if contract_balance > 0 {
            return Err(ContractError::EscrowHasFunds);
        }

        if existing_escrow.dispute_flag {
            return Err(ContractError::EscrowOpenedForDisputeResolution);
        }

        e.storage()
            .instance()
            .set(&DataKey::Escrow, &escrow_properties);

        Ok(escrow_properties)
    }

    pub fn get_multiple_escrow_balances(
        e: Env,
        addresses: Vec<Address>,
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
                trustline_decimals: escrow.trustline_decimals,
            })
        }

        Ok(balances)
    }

    pub fn get_escrow_by_contract_id(
        e: Env,
        contract_id: &Address,
    ) -> Result<Escrow, ContractError> {
        let args: Vec<Val> = Vec::new(&e);

        let result = e.invoke_contract::<Escrow>(
            contract_id,
            &Symbol::new(&e, "get_escrow"),
            args.try_into().unwrap(),
        );

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
