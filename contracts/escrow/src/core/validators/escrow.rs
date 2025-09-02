use soroban_sdk::{Address, Env, Vec};

use crate::{
    error::ContractError,
    storage::types::{DataKey, Escrow, Milestone},
};

#[inline]
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

    if !milestone.flags.approved {
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

#[inline]
pub fn validate_escrow_property_change_conditions(
    existing_escrow: &Escrow,
    platform_address: &Address,
    contract_balance: i128,
    milestones: Vec<Milestone>,
) -> Result<(), ContractError> {
    if !milestones.is_empty() {
        for (_, milestone) in milestones.iter().enumerate() {
            if milestone.flags.disputed
                || milestone.flags.released
                || milestone.flags.resolved
                || milestone.flags.approved
            {
                return Err(ContractError::FlagsMustBeFalse);
            }
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

    if contract_balance > 0 {
        return Err(ContractError::EscrowHasFunds);
    }

    Ok(())
}

#[inline]
pub fn validate_initialize_escrow_conditions(
    e: Env,
    escrow_properties: Escrow,
) -> Result<(), ContractError> {
    if e.storage().instance().has(&DataKey::Escrow) {
        return Err(ContractError::EscrowAlreadyInitialized);
    }

    let max_bps_percentage: u32 = 99*100;
    if escrow_properties.platform_fee > max_bps_percentage {
        return Err(ContractError::PlatformFeeTooHigh);
    }

    if !escrow_properties.milestones.is_empty() {
        for (_, milestone) in escrow_properties.milestones.iter().enumerate() {
            if milestone.flags.disputed
                || milestone.flags.released
                || milestone.flags.resolved
                || milestone.flags.approved
            {
                return Err(ContractError::FlagsMustBeFalse);
            }
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

#[inline]
pub fn validate_fund_escrow_conditions(
    amount: i128,
    stored_escrow: &Escrow,
    expected_escrow: &Escrow,
) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::AmountCannotBeZero);
    }

    if !stored_escrow.eq(&expected_escrow) {
        return Err(ContractError::EscrowPropertiesMismatch);
    }

    Ok(())
}