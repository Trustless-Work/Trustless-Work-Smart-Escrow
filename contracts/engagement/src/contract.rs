use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, BytesN, Env, String, Symbol, Val, Vec,
};

use crate::core::{DisputeManager, EscrowManager, MilestoneManager};
use crate::error::ContractError;
use crate::storage::types::{AddressBalance, Escrow};

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

    pub fn initialize_escrow(e: Env, escrow_properties: Escrow) -> Result<Escrow, ContractError> {
        let initialized_escrow = EscrowManager::initialize_escrow(
            e.clone(),
            escrow_properties.engagement_id,
            escrow_properties.title,
            escrow_properties.description,
            escrow_properties.approver,
            escrow_properties.service_provider,
            escrow_properties.platform_address,
            escrow_properties.amount,
            escrow_properties.platform_fee,
            escrow_properties.milestones,
            escrow_properties.release_signer,
            escrow_properties.dispute_resolver,
            escrow_properties.trustline,
            escrow_properties.trustline_decimals,
            escrow_properties.oracle_id,
            escrow_properties.party_a,
            escrow_properties.party_b,
        )?;
        e.events().publish((symbol_short!("init_esc"),), ());
        Ok(initialized_escrow)
    }

    pub fn fund_escrow(
        e: Env,
        signer: Address,
        amount_to_deposit: i128,
    ) -> Result<(), ContractError> {
        let updated_funded_escrow =
            EscrowManager::fund_escrow(e.clone(), signer.clone(), amount_to_deposit.clone())?;
        e.events()
            .publish((symbol_short!("fund_esc"),), (signer, amount_to_deposit));
        Ok(updated_funded_escrow)
    }

    pub fn distribute_escrow_earnings(
        e: Env,
        release_signer: Address,
        trustless_work_address: Address,
    ) -> Result<(), ContractError> {
        let updated_distributed_escrow_earnings = EscrowManager::distribute_escrow_earnings(
            e.clone(),
            release_signer.clone(),
            trustless_work_address.clone(),
        )?;
        e.events().publish(
            (symbol_short!("dis_esc"),),
            (release_signer, trustless_work_address),
        );
        Ok(updated_distributed_escrow_earnings)
    }

    /// This function calls EscrowManager::release_funds_based_on_oracle,
    /// which checks the off-chain oracle condition and releases funds accordingly.
    pub fn release_funds_based_on_oracle(e: Env, caller: Address) -> Result<(), ContractError> {
        let res = EscrowManager::release_funds_based_on_oracle(e.clone(), caller)?;
        e.events().publish((symbol_short!("rel_orcl"),), ());
        Ok(res)
    }

    pub fn change_escrow_properties(
        e: Env,
        plataform_address: Address,
        escrow_properties: Escrow,
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

    pub fn get_escrow_by_contract_id(
        e: Env,
        contract_id: Address,
    ) -> Result<Escrow, ContractError> {
        EscrowManager::get_escrow_by_contract_id(e, &contract_id)
    }

    pub fn get_multiple_escrow_balances(
        e: Env,
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
        service_provider: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_status(e, milestone_index, new_status, service_provider)
    }

    pub fn change_milestone_flag(
        e: Env,
        milestone_index: i128,
        new_flag: bool,
        approver: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_flag(e, milestone_index, new_flag, approver)
    }

    ////////////////////////
    // Disputes /////
    ////////////////////////

    pub fn resolving_disputes(
        e: Env,
        dispute_resolver: Address,
        approver_funds: i128,
        service_provider_funds: i128,
        trustless_work_address: Address,
    ) -> Result<(), ContractError> {
        DisputeManager::resolving_disputes(
            e,
            dispute_resolver,
            approver_funds,
            service_provider_funds,
            trustless_work_address,
        )
    }

    pub fn change_dispute_flag(e: Env) -> Result<(), ContractError> {
        DisputeManager::change_dispute_flag(e)
    }
}
