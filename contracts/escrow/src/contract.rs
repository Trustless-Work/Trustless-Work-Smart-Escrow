use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, BytesN, Env, String, Symbol, Val, Vec,
};

use crate::core::{DisputeManager, EscrowManager, MilestoneManager};
use crate::error::ContractError;
use crate::storage::types::{AddressBalance, Escrow};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn __constructor() {}

    pub fn deploy(
        env: Env,
        deployer: Address,
        wasm_hash: BytesN<32>,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
        constructor_args: Vec<Val>,
    ) -> (Address, Val) {
        if deployer != env.current_contract_address() {
            deployer.require_auth();
        }

        let deployed_address = env
            .deployer()
            .with_address(deployer, salt)
            .deploy_v2(wasm_hash, constructor_args);

        let res: Val = env.invoke_contract(&deployed_address, &init_fn, init_args);
        (deployed_address, res)
    }

    ////////////////////////
    // Escrow /////
    ////////////////////////

    pub fn initialize_escrow(e: &Env, escrow_properties: Escrow) -> Result<Escrow, ContractError> {
        let initialized_escrow =
            EscrowManager::initialize_escrow(e, escrow_properties)?;
        e.events().publish((symbol_short!("init_esc"),), ());
        Ok(initialized_escrow)
    }

    pub fn fund_escrow(
        e: &Env,
        signer: Address,
        expected_escrow: Escrow,
        amount: i128,
    ) -> Result<(), ContractError> {
        EscrowManager::fund_escrow(e, &signer, &expected_escrow, amount)?;
        e.events()
            .publish((symbol_short!("fund_esc"),), (signer, amount));
        Ok(())
    }

    pub fn release_funds(
        e: &Env,
        release_signer: Address,
        trustless_work_address: Address,
    ) -> Result<(), ContractError> {
        EscrowManager::release_funds(
            e,
            &release_signer,
            &trustless_work_address,
        )?;
        e.events().publish(
            (symbol_short!("dis_esc"),),
            release_signer,
        );
        Ok(())
    }

    pub fn update_escrow(
        e: &Env,
        plataform_address: Address,
        escrow_properties: Escrow,
    ) -> Result<Escrow, ContractError> {
        let updated_escrow = EscrowManager::change_escrow_properties(
            e,
            &plataform_address,
            escrow_properties.clone(),
        )?;
        e.events().publish(
            (symbol_short!("chg_esc"),),
            (plataform_address, escrow_properties.engagement_id),
        );
        Ok(updated_escrow)
    }

    pub fn get_escrow(e: &Env) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow(e)
    }

    pub fn get_escrow_by_contract_id(
        e: &Env,
        contract_id: Address,
    ) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow_by_contract_id(e, &contract_id)
    }

    pub fn get_multiple_escrow_balances(
        e: &Env,
        addresses: Vec<Address>,
    ) -> Result<Vec<AddressBalance>, ContractError> {
        EscrowManager::get_multiple_escrow_balances(e, addresses)
    }

    ////////////////////////
    // Milestones /////
    ////////////////////////

    pub fn change_milestone_status(
        e: Env,
        milestone_index: i128,
        new_status: String,
        new_evidence: Option<String>,
        service_provider: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_status(
            &e,
            milestone_index,
            new_status,
            new_evidence,
            service_provider,
        )
    }

    pub fn approve_milestone(
        e: Env,
        milestone_index: i128,
        approver: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_approved_flag(&e, milestone_index, approver)
    }

    ////////////////////////
    // Disputes /////
    ////////////////////////

    pub fn resolve_dispute(
        e: Env,
        dispute_resolver: Address,
        trustless_work_address: Address,
        approver_funds: i128,
        receiver_funds: i128,
    ) -> Result<(), ContractError> {
        DisputeManager::resolve_dispute(
            &e,
            dispute_resolver,
            trustless_work_address,
            approver_funds,
            receiver_funds,
        )
    }

    pub fn dispute_escrow(e: Env, signer: Address) -> Result<(), ContractError> {
        DisputeManager::dispute_escrow(&e, signer)
    }
}
