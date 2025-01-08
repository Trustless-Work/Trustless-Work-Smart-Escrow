use soroban_sdk::{
    contract, contractimpl, Address, BytesN, Env, String, Symbol, Val, Vec
};
use soroban_sdk::token::Client as TokenClient;

use crate::storage::types::Escrow;
use crate::error::ContractError;
use crate::events::balance_retrieved_event;
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
        usdc_contract: Address, 
        amount_to_deposit: i128
    ) -> Result<(), ContractError> {
        EscrowManager::fund_escrow(
            e, 
            signer, 
            usdc_contract, 
            amount_to_deposit
        )
    }

    pub fn distribute_escrow_earnings(
        e: Env, 
        release_signer: Address, 
        usdc_contract: Address,
        trustless_work_address: Address
    ) -> Result<(), ContractError> {
        EscrowManager::distribute_escrow_earnings(
            e, 
            release_signer, 
            usdc_contract,
            trustless_work_address
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

    pub fn get_escrow_balance(e: Env, usdc_token_address: Address) -> i128 {
        EscrowManager::get_escrow_balance(e, usdc_token_address)
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

    pub fn resolving_disputes(
        e: Env,
        dispute_resolver: Address,
        usdc_contract: Address,
        client_funds: i128,
        service_provider_funds: i128
    ) -> Result<(), ContractError> {
        DisputeManager::resolving_disputes(
            e,
            dispute_resolver,
            usdc_contract,
            client_funds,
            service_provider_funds
        )
    }
    
    pub fn change_dispute_flag(
        e: Env, 
    ) -> Result<(), ContractError> {
        DisputeManager::change_dispute_flag(e)
    }

    ////////////////////////
    // User /////
    ////////////////////////

    pub fn get_balance(e: Env, address: Address, usdc_token_address: Address) {
        let usdc_token = TokenClient::new(&e, &usdc_token_address);
        let balance = usdc_token.balance(&address);
        balance_retrieved_event(&e, address, usdc_token_address, balance);
    }
}