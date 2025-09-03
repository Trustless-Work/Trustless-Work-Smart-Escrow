use soroban_sdk::Address;

use crate::{
    error::ContractError,
    storage::types::{Escrow, Milestone},
};

#[inline]
pub fn validate_milestone_status_change_conditions(
    escrow: &Escrow,
    service_provider: &Address,
) -> Result<(), ContractError> {
    if service_provider != &escrow.roles.service_provider {
        return Err(ContractError::OnlyServiceProviderChangeMilstoneStatus);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMilestoneDefined);
    }

    Ok(())
}

#[inline]
pub fn validate_milestone_flag_change_conditions(
    escrow: &Escrow,
    milestone: &Milestone,
    approver: &Address,
) -> Result<(), ContractError> {
    if approver != &escrow.roles.approver {
        return Err(ContractError::OnlyApproverChangeMilstoneFlag);
    }

    if milestone.approved {
        return Err(ContractError::MilestoneHasAlreadyBeenApproved);
    }

    if milestone.status.is_empty() {
        return Err(ContractError::EmptyMilestoneStatus);
    }

    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMilestoneDefined);
    }

    Ok(())
}
