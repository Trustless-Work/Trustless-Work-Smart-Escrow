use soroban_sdk::{Address, Env, Vec};

use crate::{
    error::ContractError, 
    storage::types::{DataKey, Escrow, Milestone}
};

pub fn validate_funding_conditions(
    milestones: &Vec<Milestone>,
    signer_balance: i128,
    contract_balance: i128,
    amount_to_deposit: i128,
) -> Result<(), ContractError> {
    let total_amount: i128 = milestones.iter().map(|m| m.amount).sum();
    let has_dispute = milestones.iter().any(|m| m.flags.disputed);

    if has_dispute{
        return Err(ContractError::MilestoneOpenedForDisputeResolution);
    }

    if contract_balance >= total_amount{
        return Err(ContractError::EscrowFullyFunded);
    }

    if signer_balance < amount_to_deposit {
        return Err(ContractError::SignerInsufficientFunds);
    }

    Ok(())
}

pub fn validate_release_conditions(
    escrow: &Escrow,
    milestone: &Milestone,
    release_signer: &Address,
    milestone_index: u32,
) -> Result<(), ContractError> {
    if milestone.flags.released {
        return Err(ContractError::MilestoneAlreadyReleased);
    }

    if release_signer != &escrow.roles.release_signer {
        return Err(ContractError::OnlyReleaseSignerCanReleaseEarnings);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMileStoneDefined);
    }

    if !milestone.flags.approved{
        return Err(ContractError::MilestoneNotCompleted);
    }

    if milestone.flags.disputed {
        return Err(ContractError::MilestoneOpenedForDisputeResolution);
    }

    if milestone_index >= escrow.milestones.len() {
        return Err(ContractError::InvalidMileStoneIndex);
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
            if milestone.flags.disputed {
                return Err(ContractError::MilestoneOpenedForDisputeResolution);
            }
            if milestone.flags.approved {
                return Err(ContractError::MilestoneApprovedCantChangeEscrowProperties);
            }
        }
    }

    if platform_address != &existing_escrow.roles.platform_address {
        return Err(ContractError::OnlyPlatformAddressExecuteThisFunction);
    }

    for milestone in existing_escrow.milestones.iter() {
        if milestone.flags.approved {
            return Err(ContractError::MilestoneApprovedCantChangeEscrowProperties);
        }
    }

    if contract_balance > 0 {
        return Err(ContractError::EscrowHasFunds);
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

    if !escrow_properties.milestones.is_empty() {
        for (_, milestone) in escrow_properties.milestones.iter().enumerate() {
            if milestone.amount == 0 {
                return Err(ContractError::AmountCannotBeZero);
            }
        }
    }

    if escrow_properties.milestones.len() > 10 {
        return Err(ContractError::TooManyMilestones);
    }

    Ok(())
}