use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, BytesN, Env, String, Symbol, Val, Vec
};

use crate::storage::types::{AddressBalance, Escrow};
use crate::error::ContractError;
use crate::core::{DisputeManager, EscrowManager, MilestoneManager};

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

    ////////////////////////
    // Escrow /////
    ////////////////////////

    pub fn initialize_escrow(
        e: Env,
        escrow_properties: Escrow
    ) -> Result<Escrow, ContractError> {
        let initialized_escrow =
            EscrowManager::initialize_escrow(e.clone(), escrow_properties.clone())?;
        e.events().publish((symbol_short!("init_esc"),), ());

        Ok(initialized_escrow)
    }
    
    pub fn fund_escrow(
        e: Env, 
        signer: Address, 
        amount_to_deposit: i128
    ) -> Result<(), ContractError> {
        let updated_funded_escrow =
            EscrowManager::fund_escrow(e.clone(), signer.clone(), amount_to_deposit.clone())?;
        e.events()
            .publish((symbol_short!("fund_esc"),), (signer, amount_to_deposit));

        Ok(updated_funded_escrow)
    }

    pub fn release_milestone_payment(
        e: Env, 
        release_signer: Address, 
        trustless_work_address: Address,
        milestone_index: u32
    ) -> Result<(), ContractError> {
        let updated_release_escrow_earnings = EscrowManager::release_milestone_payment(
            e.clone(), 
            release_signer.clone(), 
            trustless_work_address.clone(),
            milestone_index
        )?;

        e.events().publish(
            (symbol_short!("dis_esc"),),
            (release_signer, trustless_work_address),
        );

        Ok(updated_release_escrow_earnings)
    }

    pub fn change_escrow_properties(
        e: Env,
        plataform_address: Address,
        escrow_properties: Escrow
    ) -> Result<Escrow, ContractError> {
        let updated_escrow = EscrowManager::change_escrow_properties(
            e.clone(),
            plataform_address.clone(),
            escrow_properties.clone(),
        )?;
        e.events().publish(
            (symbol_short!("chg_esc"),),
            (plataform_address, escrow_properties),
        );

        Ok(updated_escrow)
    }

    pub fn get_escrow(e: Env) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow(e)
    }

    pub fn get_multiple_escrow_balances(e: Env, addresses: Vec<Address>) -> Result<Vec<AddressBalance>, ContractError> {
        EscrowManager::get_multiple_escrow_balances(e, addresses)
    }

    ////////////////////////
    // Milestones /////
    ////////////////////////

    pub fn change_milestone_status(
        e: Env,
        milestone_index: u32,
        new_status: String,
        service_provider: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_status(
            e,
            milestone_index,
            new_status,
            service_provider
        )
    }
    
    pub fn change_milestone_approved_flag(
        e: Env,
        milestone_index: u32,
        new_flag: bool,
        approver: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_approved_flag(
            e,
            milestone_index,
            new_flag,
            approver
        )
    }

    ////////////////////////
    // Disputes /////
    ////////////////////////

    pub fn resolving_milestone_disputes(
        e: Env,
        dispute_resolver: Address,
        milestone_index: u32,
        approver_funds: i128,
        service_provider_funds: i128,
        trustless_work_address: Address
    ) -> Result<(), ContractError> {
        DisputeManager::resolving_milestone_disputes(
            e,
            dispute_resolver,
            milestone_index,
            approver_funds,
            service_provider_funds,
            trustless_work_address
        )
    }
    
    pub fn change_milestone_dispute_flag(
        e: Env,
        milestone_index: i128,
    ) -> Result<(), ContractError> {
        DisputeManager::change_milestone_dispute_flag(
            e,
            milestone_index,
        )
    }
}