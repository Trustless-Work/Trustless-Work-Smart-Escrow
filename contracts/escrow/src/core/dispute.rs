use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env};

use crate::core::escrow::EscrowManager;
use crate::error::ContractError;
use crate::events::escrows_by_contract_id;
use crate::shared::fee::FeeCalculator;
use crate::storage::types::DataKey;
use crate::traits::{BasicMath, BasicArithmetic};

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

        let token_client = TokenClient::new(&e, &escrow.trustline);
        let escrow_balance = token_client.balance(&e.current_contract_address());

        let total_funds = BasicMath::safe_add(approver_funds, service_provider_funds)?;
        if total_funds > escrow_balance {
            return Err(ContractError::InsufficientFundsForResolution);
        }

        let fee_calculator = FeeCalculator;

        let fee_result = fee_calculator.calculate_dispute_fees(
            approver_funds,
            service_provider_funds,
            escrow.platform_fee as i128,
            total_funds,
        )?;

        if approver_funds < fee_result.net_approver_funds {
            return Err(ContractError::InsufficientApproverFundsForCommissions);
        }

        if service_provider_funds < fee_result.net_provider_funds {
            return Err(ContractError::InsufficientServiceProviderFundsForCommissions);
        }

        token_client.transfer(
            &e.current_contract_address(),
            &trustless_work_address,
            &fee_result.trustless_work_fee,
        );

        token_client.transfer(
            &e.current_contract_address(),
            &escrow.platform_address,
            &fee_result.platform_fee,
        );

        if fee_result.net_approver_funds > 0 {
            token_client.transfer(
                &e.current_contract_address(),
                &escrow.approver,
                &fee_result.net_approver_funds,
            );
        }

        if fee_result.net_provider_funds > 0 {
            let receiver = if escrow.receiver == escrow.service_provider {
                escrow.service_provider.clone()
            } else {
                escrow.receiver.clone()
            };
            
            token_client.transfer(
                &e.current_contract_address(),
                &receiver,
                &fee_result.net_provider_funds,
            );
        }

        escrow.resolved_flag = true;
        escrow.dispute_flag = false;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        escrows_by_contract_id(&e, escrow.engagement_id.clone(), escrow);

        Ok(())
    }

    pub fn change_dispute_flag(e: Env) -> Result<(), ContractError> {
        let escrow_result = EscrowManager::get_escrow(e.clone());
        let mut escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        if escrow.dispute_flag {
            return Err(ContractError::EscrowAlreadyInDispute);
        }
        
        escrow.dispute_flag = true;
        e.storage().instance().set(&DataKey::Escrow, &escrow);

        escrows_by_contract_id(&e, escrow.engagement_id.clone(), escrow);

        Ok(())
    }
}
