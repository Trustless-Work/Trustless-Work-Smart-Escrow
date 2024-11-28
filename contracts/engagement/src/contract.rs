use soroban_sdk::{
    contract, contractimpl, Address, Env, String, BytesN, Val, Vec, Symbol
};
use soroban_sdk::token::Client as TokenClient;

use crate::storage_types::{DataKey, Escrow, Milestone};
use crate::error::ContractError;
use crate::events::{
    escrows_by_engagement_id, balance_retrieved_event, allowance_retrieved_event
};

#[contract]
pub struct EngagementContract;

#[contractimpl]
impl EngagementContract {

    pub fn deploy(
        env: Env,
        deployer: Address,
        wasm_hash: BytesN<32>,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
    ) -> (Address, Val) {
        if deployer != env.current_contract_address() {
            deployer.require_auth();
        }

        let deployed_address = env
            .deployer()
            .with_address(deployer, salt)
            .deploy(wasm_hash);

        let res: Val = env.invoke_contract(&deployed_address, &init_fn, init_args);
        (deployed_address, res)
    }


    pub fn initialize_escrow(
        e: Env,
        engagement_id: String,
        client: Address,
        service_provider: Address,
        platform_address: Address,
        amount: u128,
        platform_fee: u128,
        milestones: Vec<Milestone>,
        release_signer: Address,
        dispute_resolver: Address,
    ) -> Result<String, ContractError> {

        if e.storage().instance().has(&DataKey::Admin) {
            panic!("An escrow has already been initialized for this contract");
        }


        if amount == 0 {
            return Err(ContractError::AmountCannotBeZero);
        }

        let engagement_id_copy = engagement_id.clone();
        let escrow = Escrow {
            engagement_id: engagement_id.clone(),
            client: client.clone(),
            platform_address,
            release_signer: release_signer.clone(),
            service_provider: service_provider.clone(),
            amount,
            balance: 0,
            tw_fee: (0.3 * 10u128.pow(18) as f64) as u128,
            platform_fee: platform_fee,
            milestones: milestones,
            dispute_resolver: dispute_resolver.clone(),
            dispute_flag: false,
        };
        
        e.storage().instance().set(&DataKey::Escrow(engagement_id.clone().into()), &escrow);
        e.storage().instance().set(&DataKey::Admin, &true);

        Ok(engagement_id_copy)
    }
    
    pub fn fund_escrow(e: Env, engagement_id: String, signer: Address, usdc_contract: Address, contract_address: Address) -> Result<(), ContractError> {
        signer.require_auth();

        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = Self::get_escrow_by_id(e.clone(), engagement_id);
    
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };
    
        let half_price_in_micro_usdc = (escrow.amount as i128) / 2;
        let usdc_client = TokenClient::new(&e, &usdc_contract);
    
        let signer_balance = usdc_client.balance(&signer);
        if signer_balance < half_price_in_micro_usdc {
            return Err(ContractError::SignerInsufficientFunds);
        }
    
        usdc_client.transfer(&signer, &contract_address, &half_price_in_micro_usdc);
    
        escrow.balance = half_price_in_micro_usdc as u128;
        e.storage().instance().set(&escrow_key, &escrow);
    
        Ok(())
    }

    pub fn claim_escrow_earnings(
        e: Env, 
        engagement_id: String, 
        service_provider: Address, 
        usdc_contract: Address
    ) -> Result<(), String> {
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let mut escrow = match Self::get_escrow_by_id(e.clone(), engagement_id.clone()) {
            Ok(esc) => esc,
            Err(_) => return Err(soroban_sdk::String::from_str(&e,"Escrow is not initialized for this commitment.")),
        };

        if service_provider != escrow.service_provider {
            return Err(soroban_sdk::String::from_str(&e,"Only the service provider can claim the profits").into());
        }
    
        if escrow.milestones.is_empty() {
            return Err(soroban_sdk::String::from_str(&e,"The escrow must have at least one milestone").into());
        }
    
        if !escrow.milestones.iter().all(|milestone| milestone.flag) {
            return Err(soroban_sdk::String::from_str(&e,"Not all milestones have been completed").into());
        }

        if escrow.dispute_flag {
            return Err(soroban_sdk::String::from_str(&e,"Escrow is currently in dispute").into());
        }
    
        if escrow.balance != escrow.amount {
            return Err(soroban_sdk::String::from_str(&e,"The escrow balance is not enough to send the profits").into());
        }
    
        let usdc_client = TokenClient::new(&e, &usdc_contract);
    
        let contract_address = e.current_contract_address();
    
        let platform_fee_percentage: u128 = e.storage().instance()
            .get(&DataKey::PlatformFee)
            .unwrap_or(0);
    
        let platform_address = e.storage().instance()
            .get(&DataKey::PlatformAddress)
            .expect("Platform address not configured");
    
        let total_amount = escrow.amount as f64;
        let trustless_work_commission = (total_amount * 0.003).floor() as i128; 
        let platform_commission = (total_amount * platform_fee_percentage as f64).floor() as i128;
        
        let trustless_work_address = Address::from_string(&soroban_sdk::String::from_str(&e, "GBPUACN7QETR4TCYTKINBDHTYTFXD3BQQQV7VSMZC5CX74E4MTUL2AMUB"));
    
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
    
        let service_provider_amount = (total_amount - trustless_work_commission as f64 - platform_commission as f64).floor() as i128;
    
        usdc_client.transfer(
            &contract_address, 
            &escrow.service_provider, 
            &service_provider_amount
        );
    
        escrow.balance = 0;
        e.storage().instance().set(&escrow_key, &escrow);
    
        Ok(())
    }

    pub fn refund_remaining_funds(e: Env, engagement_id: String, signer: Address, usdc_contract: Address, contract_address: Address) -> Result<(), ContractError> {
        signer.require_auth();

        let escrow_key = DataKey::Escrow(engagement_id.clone());
        let escrow_result = Self::get_escrow_by_id(e.clone(), engagement_id);
    
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };
        
        let invoker = signer.clone();
        if invoker != escrow.release_signer {
            return Err(ContractError::OnlySignerCanRequestRefund);
        }

        let usdc_client = TokenClient::new(&e, &usdc_contract);
        let contract_balance = usdc_client.balance(&contract_address);

        if  contract_balance == 0 {
            return Err(ContractError::ContractHasInsufficientBalance);
        }

        usdc_client.transfer(
            &e.current_contract_address(),
            &escrow.release_signer,
            &contract_balance
        );

        escrow.balance = 0;
        e.storage().instance().set(&escrow_key, &escrow);

        Ok(())
    }

    pub fn get_escrow_by_id(e: Env, engagement_id: String) -> Result<Escrow, ContractError> {
        let escrow_key = DataKey::Escrow(engagement_id.clone());
        if let Some(escrow) = e.storage().instance().get::<DataKey, Escrow>(&escrow_key) {
            escrows_by_engagement_id(&e, engagement_id.clone(), escrow.clone());
            Ok(escrow)
        } else {
            return Err(ContractError::EscrowNotFound)
        }
    }

    pub fn approve_amounts(e: Env, from: Address, spender: Address, amount: i128, usdc_token_address: Address ) {
        from.require_auth();
        let expiration_ledger = e.ledger().sequence() + 1000;
        let usdc_token = TokenClient::new(&e, &usdc_token_address);
        usdc_token.approve(&from, &spender, &amount, &expiration_ledger);
    }

    pub fn get_allowance(e: Env, from: Address, spender: Address, usdc_token_address: Address ) {
        let usdc_token = TokenClient::new(&e, &usdc_token_address);
        let allowance = usdc_token.allowance(&from, &spender);
        allowance_retrieved_event(&e, from, spender, allowance);
    }

    pub fn get_balance(e: Env, address: Address, usdc_token_address: Address) {
        let usdc_token = TokenClient::new(&e, &usdc_token_address);
        let balance = usdc_token.balance(&address);
        balance_retrieved_event(&e, address, usdc_token_address, balance);
    }

    pub fn change_escrow_properties(
        e: Env,
        engagement_id: String,
        client: Address,
        service_provider: Address,
        platform_address: Address,
        amount: u128,
        platform_fee: u128,
        milestones: Vec<Milestone>,
        release_signer: Address,
        dispute_resolver: Address,
    ) -> Result<(), ContractError> {
        let existing_escrow = Self::get_escrow_by_id(e.clone(), engagement_id.clone())?;

        if platform_address != existing_escrow.platform_address {
            return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
        }
        
        platform_address.require_auth();

        let updated_escrow = Escrow {
            engagement_id: engagement_id.clone(),
            client,
            platform_address,
            release_signer,
            service_provider,
            amount,
            balance: amount,
            tw_fee: (0.3 * 10u128.pow(18) as f64) as u128,
            platform_fee,
            milestones,
            dispute_resolver,
            dispute_flag: false,
        };

        e.storage().instance().set(
            &DataKey::Escrow(engagement_id.into()),
            &updated_escrow
        );

        escrows_by_engagement_id(&e, updated_escrow.engagement_id.clone(), updated_escrow);

        Ok(())
    }

}