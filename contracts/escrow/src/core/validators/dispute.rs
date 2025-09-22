use soroban_sdk::{Address, Map};

use crate::modules::math::{BasicArithmetic, BasicMath};
use crate::{
    error::ContractError,
    storage::types::{Escrow, Roles},
};

#[inline]
pub fn validate_dispute_resolution_conditions(
    escrow: &Escrow,
    dispute_resolver: &Address,
    distributions: &Map<Address, i128>,
    current_balance: i128,
) -> Result<(), ContractError> {
    if dispute_resolver != &escrow.roles.dispute_resolver {
        return Err(ContractError::OnlyDisputeResolverCanExecuteThisFunction);
    }

    if !escrow.flags.disputed {
        return Err(ContractError::EscrowNotInDispute);
    }

    let mut total: i128 = 0;
    for (_addr, amount) in distributions.iter() {
        if amount < 0 {
            return Err(ContractError::AmountsToBeTransferredShouldBePositive);
        }
        total = BasicMath::safe_add(total, amount)?;
    }
    if total <= 0 {
        return Err(ContractError::AmountCannotBeZero);
    }
    if current_balance < total {
        return Err(ContractError::InsufficientFundsForResolution);
    }
    if total != current_balance {
        return Err(ContractError::DistributionsMustEqualEscrowBalance);
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
