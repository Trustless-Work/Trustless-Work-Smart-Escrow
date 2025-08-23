use soroban_sdk::Address;

use crate::{
    error::ContractError,
    storage::types::{Escrow, Roles},
};

#[inline]
pub fn validate_dispute_resolution_conditions(
    escrow: &Escrow,
    dispute_resolver: &Address,
    approver_funds: i128,
    receiver_funds: i128,
    total_funds: i128,
    current_balance: i128,
) -> Result<(), ContractError> {
    if dispute_resolver != &escrow.roles.dispute_resolver {
        return Err(ContractError::OnlyDisputeResolverCanExecuteThisFunction);
    }

    if !escrow.flags.disputed {
        return Err(ContractError::EscrowNotInDispute);
    }

    if approver_funds <= 0 || receiver_funds <= 0 {
        return Err(ContractError::ApproverOrReceiverFundsLessThanZero);
    }

    if current_balance < total_funds {
        return Err(ContractError::InsufficientFundsForResolution);
    }

    if total_funds != current_balance {
        return Err(ContractError::ReceiverAndApproverFundsNotEqual);
    }

    Ok(())
}

#[inline]
pub fn validate_dispute_flag_change_conditions(
    escrow: &Escrow,
    signer: &Address,
) -> Result<(), ContractError> {
    if escrow.flags.disputed {
        return Err(ContractError::EscrowAlreadyInDispute);
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
