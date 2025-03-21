use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env};

use crate::core::escrow::EscrowManager;
use crate::error::ContractError;
use crate::events::{self, escrows_by_engagement_id};
use crate::storage::types::{DataKey, Escrow};

pub struct DisputeManager;

impl DisputeManager {
    pub fn resolving_disputes(
        e: Env,
        dispute_resolver: Address,
        approver_funds: i128,
        service_provider_funds: i128,
        trustless_work_address: Address,
    ) -> Result<(), ContractError> {
        dispute_resolver.require_auth();

        let escrow_result = EscrowManager::get_escrow(e.clone());
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        if dispute_resolver != escrow.dispute_resolver {
            return Err(ContractError::OnlyDisputeResolverCanExecuteThisFunction);
        }

        if !escrow.dispute_flag {
            return Err(ContractError::EscrowNotInDispute);
        }

        let usdc_approver = TokenClient::new(&e, &escrow.trustline);
        let escrow_balance = usdc_approver.balance(&e.current_contract_address());

        let total_funds = approver_funds
            .checked_add(service_provider_funds)
            .ok_or(ContractError::Overflow)?;
        if total_funds > escrow_balance {
            return Err(ContractError::InsufficientFundsForResolution);
        }

        let trustless_work_fee = total_funds
            .checked_mul(30)
            .ok_or(ContractError::Overflow)?
            .checked_div(10000)
            .ok_or(ContractError::DivisionError)?;
        let platform_fee = total_funds
            .checked_mul(escrow.platform_fee)
            .ok_or(ContractError::Overflow)?
            .checked_div(10000)
            .ok_or(ContractError::DivisionError)?;
        let total_fees = trustless_work_fee
            .checked_add(platform_fee)
            .ok_or(ContractError::Overflow)?;

        let approver_fee = approver_funds
            .checked_mul(total_fees)
            .ok_or(ContractError::Overflow)?
            .checked_div(total_funds)
            .ok_or(ContractError::DivisionError)?;
        let net_approver_funds = approver_funds
            .checked_sub(approver_fee)
            .ok_or(ContractError::Underflow)?;
        let fees_portion = service_provider_funds
            .checked_mul(total_fees)
            .ok_or(ContractError::Overflow)?
            .checked_div(total_funds)
            .ok_or(ContractError::DivisionError)?;
        let net_provider_funds = service_provider_funds
            .checked_sub(fees_portion)
            .ok_or(ContractError::Underflow)?;

        if approver_funds < net_approver_funds {
            return Err(ContractError::InsufficientApproverFundsForCommissions);
        }

        if service_provider_funds < net_provider_funds {
            return Err(ContractError::InsufficientServiceProviderFundsForCommissions);
        }

        usdc_approver.transfer(
            &e.current_contract_address(),
            &trustless_work_address,
            &trustless_work_fee,
        );

        usdc_approver.transfer(
            &e.current_contract_address(),
            &escrow.platform_address,
            &platform_fee,
        );

        if net_approver_funds > 0 {
            usdc_approver.transfer(
                &e.current_contract_address(),
                &escrow.approver,
                &net_approver_funds,
            );
        }

        if net_provider_funds > 0 {
            let receiver = if escrow.receiver == escrow.service_provider {
                escrow.service_provider.clone()
            } else {
                escrow.receiver.clone()
            };
            
            usdc_approver.transfer(
                &e.current_contract_address(),
                &receiver,
                &net_provider_funds,
            );
        }

        escrow.resolved_flag = true;
        escrow.dispute_flag = false;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        escrows_by_engagement_id(&e, escrow.engagement_id.clone(), escrow);

        Ok(())
    }

    pub fn change_dispute_flag(e: Env) -> Result<(), ContractError> {
        let escrow_result = EscrowManager::get_escrow(e.clone());
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        // Toggles the dispute flag (true -> false or false -> true)
        escrow.dispute_flag = !escrow.dispute_flag;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        escrows_by_engagement_id(&e, escrow.engagement_id.clone(), escrow);

        Ok(())
    }

    pub fn resolve_dispute(e: &Env, escrow: &mut Escrow) {
        // Ensure only the dispute resolver can resolve the dispute
        let caller = e.current_contract_address();
        if caller != escrow.dispute_resolver {
            panic!("Only the dispute resolver can resolve the dispute");
        }

        // Check if there is an active dispute
        if !escrow.dispute_flag {
            panic!("No active dispute to resolve");
        }

        // Resolver should set all relevant flags
        escrow.dispute_flag = false;
        escrow.resolved_flag = true;

        // Emit event for dispute resolution
        events::resolved(e, &escrow.engagement_id, escrow);
    }
}
