use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, BytesN, Env, String, Symbol, Val, Vec, IntoVal,
};

use crate::core::{DisputeManager, EscrowManager, MilestoneManager};
use crate::error::ContractError;
use crate::storage::types::{AddressBalance, DataKey, Escrow};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

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

    pub fn initialize_escrow(e: Env, escrow_properties: Escrow) -> Result<Escrow, ContractError> {
        let initialized_escrow =
            EscrowManager::initialize_escrow(e.clone(), escrow_properties)?;
        e.events().publish((symbol_short!("init_esc"),), ());
        Ok(initialized_escrow)
    }

    pub fn fund_escrow(
        e: Env,
        signer: Address,
        amount_to_deposit: i128,
    ) -> Result<(), ContractError> {
        EscrowManager::fund_escrow(e.clone(), signer.clone(), amount_to_deposit)?;
        e.events()
            .publish((symbol_short!("fund_esc"),), (signer, amount_to_deposit));
        Ok(())
    }

    pub fn release_funds(
        e: Env,
        release_signer: Address,
        trustless_work_address: Address,
    ) -> Result<(), ContractError> {
        EscrowManager::release_funds(
            e.clone(),
            release_signer.clone(),
            trustless_work_address.clone(),
        )?;
        e.events().publish(
            (symbol_short!("dis_esc"),),
            (release_signer, trustless_work_address),
        );
        Ok(())
    }

    pub fn update_escrow(
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
            (plataform_address, escrow_properties.engagement_id),
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
        signer: Address,
        addresses: Vec<Address>,
    ) -> Result<Vec<AddressBalance>, ContractError> {
        EscrowManager::get_multiple_escrow_balances(e, signer, addresses)
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
            e,
            milestone_index,
            new_status,
            new_evidence,
            service_provider,
        )
    }

    pub fn approve_milestone(
        e: Env,
        milestone_index: i128,
        new_flag: bool,
        approver: Address,
    ) -> Result<(), ContractError> {
        MilestoneManager::change_milestone_approved_flag(e, milestone_index, new_flag, approver)
    }

    ////////////////////////
    // Disputes /////
    ////////////////////////

    pub fn resolve_dispute(
        e: Env,
        dispute_resolver: Address,
        approver_funds: i128,
        receiver_funds: i128,
        trustless_work_address: Address,
    ) -> Result<(), ContractError> {
        DisputeManager::resolve_dispute(
            e,
            dispute_resolver,
            approver_funds,
            receiver_funds,
            trustless_work_address,
        )
    }

    pub fn dispute_escrow(e: Env, signer: Address) -> Result<(), ContractError> {
        DisputeManager::dispute_escrow(e, signer)
    }

    pub fn send_to_vault(e: Env) -> Result<(), ContractError> {
        EscrowManager::send_to_vault(e)
    }

    ////////////////////////
    // Vault Integration (Option 1) /////
    ////////////////////////

    // Operator-gated helper that lets a user deposit directly into a DeFindex vault.
    // - The `vault_operator` authorizes running this flow.
    // - The `user` must authorize because DeFindex's vault.deposit requires `from.require_auth()`.
    // - Funds are pulled by the vault from `user`; escrow only coordinates.
    // - We intentionally ignore the return to keep ABI simple; frontends can query the vault if needed.
    pub fn deposit_via_vault(
        e: Env,
        operator: Address,
        user: Address,
        vault_address: Address,
        amounts_desired: Vec<i128>,
        amounts_min: Vec<i128>,
        invest: bool,
    ) -> Result<(), ContractError> {
        // Auth: operator must be the configured vault_operator for this escrow instance
        let escrow = EscrowManager::get_escrow(e.clone())?;
        if operator != escrow.roles.vault_operator {
            return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
        }
        operator.require_auth();

        // Auth: user must authorize so the vault can pull funds from the user's balance
        user.require_auth();

        // Build args for DeFindex vault.deposit(amounts_desired, amounts_min, from, invest)
        let mut args: Vec<Val> = Vec::new(&e);
        args.push_back(amounts_desired.to_val());
        args.push_back(amounts_min.to_val());
        args.push_back(user.to_val());
        args.push_back(invest.into_val(&e));

        // Invoke vault. We don't rely on its concrete return type here.
        let _res: Val = e.invoke_contract(&vault_address, &Symbol::new(&e, "deposit"), args);

        e.events().publish((symbol_short!("dep_vlt"),), (operator, user, vault_address));
        Ok(())
    }

    // Set the persistent vault address. Only platform_address can set it.
    pub fn set_vault_address(e: Env, platform_address: Address, vault_address: Address) -> Result<(), ContractError> {
        let escrow = EscrowManager::get_escrow(e.clone())?;
        if platform_address != escrow.roles.platform_address {
            return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
        }
        platform_address.require_auth();
        e.storage().instance().set(&DataKey::VaultAddr, &vault_address);
        e.events().publish((symbol_short!("set_vlt"),), (platform_address, vault_address));
        Ok(())
    }

    // Get the stored vault address.
    pub fn get_vault_address(e: Env) -> Result<Address, ContractError> {
        let addr: Option<Address> = e.storage().instance().get(&DataKey::VaultAddr);
        match addr {
            Some(a) => Ok(a),
            None => Err(ContractError::AdminNotFound),
        }
    }

    // Returns the address of the vault share token. In DeFindex vault, the token
    // interface is implemented by the same contract, so this equals the vault address.
    pub fn get_vault_share_token_address(e: Env) -> Result<Address, ContractError> {
        Self::get_vault_address(e)
    }

    // Same as deposit_via_vault but reads the stored vault address.
    pub fn deposit_via_vault_stored(
        e: Env,
        operator: Address,
        user: Address,
        amounts_desired: Vec<i128>,
        amounts_min: Vec<i128>,
        invest: bool,
    ) -> Result<(), ContractError> {
        let vault_address = Self::get_vault_address(e.clone())?;
        Self::deposit_via_vault(
            e,
            operator,
            user,
            vault_address,
            amounts_desired,
            amounts_min,
            invest,
        )
    }
}
