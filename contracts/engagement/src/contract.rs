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

    pub fn distribute_escrow_earnings(
        e: Env, 
        release_signer: Address, 
        trustless_work_address: Address
    ) -> Result<(), ContractError> {
        EscrowManager::distribute_escrow_earnings(
            e, 
            release_signer, 
            trustless_work_address
        )
    }

    pub fn change_escrow_properties(
        e: Env,
        plataform_address: Address,
        escrow_properties: Escrow
    ) -> Result<Escrow, ContractError> {
        EscrowManager::change_escrow_properties(e, plataform_address, escrow_properties)
    }

    pub fn get_escrow(e: Env) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow(e)
    }

    pub fn get_escrow_by_contract_id(e: Env, contract_id: Address) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow_by_contract_id(e, &contract_id)
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
        approver: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_flag(
            e,
            milestone_index,
            new_flag,
            approver
        )
    }

    ////////////////////////
    // Disputes /////
    ////////////////////////

    pub fn resolving_disputes(
        e: Env,
        dispute_resolver: Address,
        approver_funds: i128,
        service_provider_funds: i128,
        trustless_work_address: Address
    ) -> Result<(), ContractError> {
        DisputeManager::resolving_disputes(
            e,
            dispute_resolver,
            approver_funds,
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