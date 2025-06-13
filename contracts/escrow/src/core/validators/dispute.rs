use soroban_sdk::Address;

use crate::{
    error::ContractError,
    modules::fee::DisputeFeeResult,
    storage::types::{Escrow, Milestone, Roles},
};

pub fn validate_dispute_resolution_conditions(
    escrow: &Escrow,
    milestone: &Milestone,
    dispute_resolver: &Address,
    approver_funds: i128,
    receiver_funds: i128,
    fee_result: &DisputeFeeResult,
    total_funds: i128,
) -> Result<(), ContractError> {
    if dispute_resolver != &escrow.roles.dispute_resolver {
        return Err(ContractError::OnlyDisputeResolverCanExecuteThisFunction);
    }

    if milestone.flags.resolved {
        return Err(ContractError::MilestoneAlreadyResolved);
    }

    if total_funds > milestone.amount {
        return Err(ContractError::InsufficientFundsForResolution);
    }

    if !milestone.flags.disputed {
        return Err(ContractError::MilestoneNotInDispute);
    }

    if approver_funds < fee_result.net_approver_funds {
        return Err(ContractError::InsufficientApproverFundsForCommissions);
    }

    if receiver_funds < fee_result.net_provider_funds {
        return Err(ContractError::InsufficientServiceProviderFundsForCommissions);
    }

    Ok(())
}

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

    let milestone = escrow.milestones.get(milestone_index as u32)
        .ok_or(ContractError::InvalidMileStoneIndex)?;
    
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