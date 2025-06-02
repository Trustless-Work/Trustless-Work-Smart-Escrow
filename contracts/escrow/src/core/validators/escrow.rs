use soroban_sdk::{Address, Env, Vec};

use crate::{
    error::ContractError,
    storage::types::{DataKey, Escrow, Milestone},
};

pub fn validate_funding_conditions(
    escrow: &Escrow,
    signer_balance: i128,
    contract_balance: i128,
    amount_to_deposit: i128,
) -> Result<(), ContractError> {
    if escrow.flags.disputed {
        return Err(ContractError::EscrowOpenedForDisputeResolution);
    }

    if contract_balance >= escrow.amount {
        return Err(ContractError::EscrowFullyFunded);
    }

    if signer_balance < amount_to_deposit {
        return Err(ContractError::SignerInsufficientFunds);
    }

    Ok(())
}

pub fn validate_release_conditions(
    escrow: &Escrow,
    release_signer: &Address,
) -> Result<(), ContractError> {
    if escrow.flags.released {
        return Err(ContractError::EscrowAlreadyResolved);
    }

    if release_signer != &escrow.roles.release_signer {
        return Err(ContractError::OnlyReleaseSignerCanReleaseEarnings);
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

    if escrow.flags.disputed{
        return Err(ContractError::EscrowOpenedForDisputeResolution);
    }

    Ok(())
}

pub fn validate_escrow_property_change_conditions(
    existing_escrow: &Escrow,
    platform_address: &Address,
    contract_balance: i128,
    milestones: Vec<Milestone>,
) -> Result<(), ContractError> {
    if !milestones.is_empty() {
        for (_, milestone) in milestones.iter().enumerate() {
            if milestone.approved_flag {
                return Err(ContractError::MilestoneApprovedCantChangeEscrowProperties);
            }
        }
    }

    if platform_address != &existing_escrow.roles.platform_address {
        return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
    }

    for milestone in existing_escrow.milestones.iter() {
        if milestone.approved_flag {
            return Err(ContractError::MilestoneApprovedCantChangeEscrowProperties);
        }
    }

    if contract_balance > 0 {
        return Err(ContractError::EscrowHasFunds);
    }

    if existing_escrow.flags.disputed {
        return Err(ContractError::EscrowOpenedForDisputeResolution);
    }

    Ok(())
}

pub fn validate_initialize_escrow_conditions(
    e: Env,
    escrow_properties: Escrow,
) -> Result<(), ContractError> {
    if e.storage().instance().has(&DataKey::Escrow) {
        return Err(ContractError::EscrowAlreadyInitialized);
    }

    if escrow_properties.amount == 0 {
        return Err(ContractError::AmountCannotBeZero);
    }

    if escrow_properties.milestones.len() > 10 {
        return Err(ContractError::TooManyMilestones);
    }

    Ok(())
}
