use soroban_sdk::{
    contract, contractimpl, Address, BytesN, Env, String, Symbol, Val, Vec
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
        EscrowManager::initialize_escrow(e, escrow_properties)
    }
    
    pub fn fund_escrow(
        e: Env, 
        signer: Address, 
        amount_to_deposit: i128
    ) -> Result<(), ContractError> {
        EscrowManager::fund_escrow(
            e, 
            signer, 
            amount_to_deposit
        )
    }

    pub fn release_milestone_payment(
        e: Env, 
        release_signer: Address, 
        trustless_work_address: Address,
        milestone_index: i128
    ) -> Result<(), ContractError> {
        EscrowManager::release_milestone_payment(
            e, 
            release_signer, 
            trustless_work_address,
            milestone_index
        )
    }

    pub fn change_escrow_properties(
        e: Env,
        escrow_properties: Escrow
    ) -> Result<(), ContractError> {
        EscrowManager::change_escrow_properties(e, escrow_properties)
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
        milestone_index: i128,
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
    
    pub fn change_milestone_flag(
        e: Env,
        milestone_index: i128,
        new_flag: bool,
        client: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_flag(
            e,
            milestone_index,
            new_flag,
            client
        )
    }

    ////////////////////////
    // Disputes /////
    ////////////////////////

    pub fn resolving_milestone_disputes(
        e: Env,
        dispute_resolver: Address,
        milestone_index: u32,
        client_funds: i128,
        service_provider_funds: i128,
        trustless_work_address: Address
    ) -> Result<(), ContractError> {
        DisputeManager::resolving_milestone_disputes(
            e,
            dispute_resolver,
            milestone_index,
            client_funds,
            service_provider_funds,
            trustless_work_address
        )
    }
    
    pub fn change_dispute_flag(
        e: Env, 
    ) -> Result<(), ContractError> {
        DisputeManager::change_dispute_flag(e)
    }
}