use soroban_sdk::Address;

use crate::{
    error::ContractError,
    storage::types::{Escrow, Milestone, Roles},
};

#[inline]
pub fn validate_dispute_resolution_conditions(
    escrow: &Escrow,
    milestone: &Milestone,
    dispute_resolver: &Address,
    approver_funds: i128,
    receiver_funds: i128,
    total_funds: i128,
    current_balance: i128,
) -> Result<(), ContractError> {
    if dispute_resolver != &escrow.roles.dispute_resolver {
        return Err(ContractError::OnlyDisputeResolverCanExecuteThisFunction);
    }

    if milestone.flags.released {
        return Err(ContractError::MilestoneAlreadyReleased);
    }

    if milestone.flags.resolved {
        return Err(ContractError::MilestoneAlreadyResolved);
    }

    if total_funds > milestone.amount {
        return Err(ContractError::TotalDisputeFundsMustNotExceedTheMilestoneAmount);
    }

    if approver_funds <= 0 || receiver_funds <= 0 {
        return Err(ContractError::ApproverOrReceiverFundsLessThanZero);
    }

    if !milestone.flags.disputed {
        return Err(ContractError::MilestoneNotInDispute);
    }

    if current_balance < total_funds {
        return Err(ContractError::InsufficientFundsForResolution);
    }

    Ok(())
}

#[inline]
pub fn validate_dispute_flag_change_conditions(
    escrow: &Escrow,
    milestone_index: i128,
    signer: &Address,
) -> Result<(), ContractError> {
    if escrow.milestones.is_empty() {
        return Err(ContractError::NoMileStoneDefined);
    }

    if milestone_index < 0 || milestone_index >= escrow.milestones.len() as i128 {
        return Err(ContractError::InvalidMileStoneIndex);
    }

    let milestone = escrow
        .milestones
        .get(milestone_index as u32)
        .ok_or(ContractError::InvalidMileStoneIndex)?;

    // Guardrail: cannot open dispute on a released/resolved milestone
    if milestone.flags.released {
        return Err(ContractError::MilestoneAlreadyReleased);
    }
    if milestone.flags.resolved {
        return Err(ContractError::MilestoneAlreadyResolved);
    }

    if milestone.flags.disputed {
        return Err(ContractError::MilestoneAlreadyInDispute);
    }

    let Roles {
        approver,
        service_provider,
        platform_address,
        release_signer,
        dispute_resolver,
        receiver,
    } = &escrow.roles;

    let is_authorized = signer == approver
        || signer == service_provider
        || signer == platform_address
        || signer == release_signer
        || signer == dispute_resolver
        || signer == receiver;

    if !is_authorized {
        return Err(ContractError::UnauthorizedToChangeDisputeFlag);
    }

    Ok(())
}
