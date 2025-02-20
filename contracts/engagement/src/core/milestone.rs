use soroban_sdk::{Address, Env, String,symbol_short, Vec};
use crate::storage::types::{DataKey, Escrow, Milestone};
use crate::error::ContractError;
use crate::events::escrows_by_engagement_id;
use crate::core::escrow::EscrowManager;

pub struct MilestoneManager;

impl MilestoneManager {
    pub fn change_milestone_status(
        e: Env,
        milestone_index: i128,
        new_status: String,
        service_provider: Address,
    ) -> Result<(), ContractError> {
        let escrow_result = EscrowManager::get_escrow(e.clone());
        let existing_escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

        let engagement_id = existing_escrow.engagement_id.clone();
    
        if service_provider != existing_escrow.service_provider {
            return Err(ContractError::OnlyServiceProviderChangeMilstoneStatus);
        }
        service_provider.require_auth();
    
        if existing_escrow.milestones.is_empty() {
            return Err(ContractError::NoMileStoneDefined);
        }
    
        if milestone_index < 0 || milestone_index >= existing_escrow.milestones.len() as i128 {
            return Err(ContractError::InvalidMileStoneIndex);
        }

        let mut updated_milestones = Vec::<Milestone>::new(&e);
        for (index, milestone) in existing_escrow.milestones.iter().enumerate() {
            let mut new_milestone = milestone.clone();
            if index as i128 == milestone_index {
                new_milestone.status = new_status.clone();
            }
            updated_milestones.push_back(new_milestone);
        }
    
        let updated_escrow = Escrow {
            milestones: updated_milestones,
            ..existing_escrow
        };
    
        e.storage().instance().set(
            &DataKey::Escrow,
            &updated_escrow,
        );
    
        escrows_by_engagement_id(&e, updated_escrow.engagement_id.clone(), updated_escrow);

        // Emit `milestone_status_changed` event after milestone status update
        let old_status = existing_escrow.milestones.get(milestone_index as u32).unwrap().status;
        e.events().publish(
            (symbol_short!("milestone"), symbol_short!("status")), // Topics
            (engagement_id, old_status, new_status) // Payload: ID, Index, Old Status, New Status
        );
    
        Ok(())
    }
    
    pub fn change_milestone_flag(
        e: Env,
        milestone_index: i128,
        new_flag: bool,
        approver: Address,
    ) -> Result<(), ContractError> {
        let escrow_result = EscrowManager::get_escrow(e.clone());
        let existing_escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };
    
        if approver != existing_escrow.approver {
            return Err(ContractError::OnlyApproverChangeMilstoneFlag);
        }

        approver.require_auth();
    
        if existing_escrow.milestones.is_empty() {
            return Err(ContractError::NoMileStoneDefined);
        }

        if milestone_index < 0 || milestone_index >= existing_escrow.milestones.len() as i128 {
            return Err(ContractError::InvalidMileStoneIndex);
        }
    
        let mut updated_milestones = Vec::<Milestone>::new(&e);
        for (index, milestone) in existing_escrow.milestones.iter().enumerate() {
            let mut new_milestone = milestone.clone();
            if index as i128 == milestone_index {
                new_milestone.approved_flag = new_flag;
            }
            updated_milestones.push_back(new_milestone);
        }
    
        let updated_escrow = Escrow {
            milestones: updated_milestones,
            ..existing_escrow
        };
    
        e.storage().instance().set(
            &DataKey::Escrow,
            &updated_escrow,
        );
    
        escrows_by_engagement_id(&e, updated_escrow.engagement_id.clone(), updated_escrow);
        Ok(())
    }

}