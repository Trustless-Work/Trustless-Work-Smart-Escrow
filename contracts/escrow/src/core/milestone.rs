use soroban_sdk::{Address, Env, String};
use crate::storage::types::DataKey;
use crate::error::ContractError;
use crate::events::escrows_by_contract_id;
use crate::core::escrow::EscrowManager;

use super::validators::milestone::{validate_milestone_flag_change_conditions, validate_milestone_status_change_conditions};

pub struct MilestoneManager;

impl MilestoneManager {
    pub fn change_milestone_status(
        e: Env,
        milestone_index: i128,
        new_status: String,
        new_evidence: Option<String>,
        service_provider: Address,
    ) -> Result<(), ContractError> {
        service_provider.require_auth();

        let mut escrow = EscrowManager::get_escrow(e.clone())?;

        validate_milestone_status_change_conditions(
            &escrow,
            milestone_index,
            &service_provider,
        )?;

        let mut milestone_to_update = escrow
        .milestones
        .get(milestone_index as u32)
        .ok_or(ContractError::InvalidMileStoneIndex)?;

        if let Some(evidence) = new_evidence {
            milestone_to_update.evidence = evidence;
        }
    
        milestone_to_update.status = new_status;

        escrow
            .milestones
            .set(milestone_index as u32, milestone_to_update);
    
        e.storage()
            .instance()
            .set(&DataKey::Escrow, &escrow);
            escrows_by_contract_id(&e, escrow.engagement_id.clone(), escrow);
    
    
        Ok(())
    }
    
    pub fn change_milestone_approved_flag(
        e: Env,
        milestone_index: i128,
        new_flag: bool,
        approver: Address,
    ) -> Result<(), ContractError> {
        approver.require_auth();
        
        let mut escrow = EscrowManager::get_escrow(e.clone())?;
    
        
        let mut milestone_to_update = escrow
        .milestones
        .get(milestone_index as u32)
        .ok_or(ContractError::InvalidMileStoneIndex)?;
    
        validate_milestone_flag_change_conditions(&escrow, &milestone_to_update, milestone_index, &approver)?;
        milestone_to_update.flags.approved = new_flag;

        escrow
            .milestones
            .set(milestone_index as u32, milestone_to_update);
        e.storage()
            .instance()
            .set(&DataKey::Escrow, &escrow);
        escrows_by_contract_id(&e, escrow.engagement_id.clone(), escrow);

        Ok(())
    }

}