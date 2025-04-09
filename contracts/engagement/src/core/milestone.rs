use crate::core::escrow::EscrowManager;
use crate::error::ContractError;
use crate::events::escrows_by_contract_id;
use crate::storage::types::{DataKey, Escrow, Milestone};
use soroban_sdk::{Address, Env, String, Vec};

pub struct MilestoneManager;

impl MilestoneManager {
    pub fn change_milestone_status(
        e: Env,
        milestone_index: i128,
        new_status: String,
        new_evidence: Option<String>,
        service_provider: Address,
    ) -> Result<(), ContractError> {
        let escrow_result = EscrowManager::get_escrow(e.clone());
        let existing_escrow = match escrow_result {
            Ok(esc) => esc,
            Err(err) => return Err(err),
        };

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
                if let Some(evidence) = new_evidence.clone() {
                    new_milestone.evidence = evidence;
                }
                new_milestone.status = new_status.clone();
            }
            updated_milestones.push_back(new_milestone);
        }

        let updated_escrow = Escrow {
            milestones: updated_milestones,
            ..existing_escrow
        };

        e.storage()
            .instance()
            .set(&DataKey::Escrow, &updated_escrow);

        escrows_by_contract_id(&e, updated_escrow.engagement_id.clone(), updated_escrow);

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

        e.storage()
            .instance()
            .set(&DataKey::Escrow, &updated_escrow);

        escrows_by_contract_id(&e, updated_escrow.engagement_id.clone(), updated_escrow);

        Ok(())
    }
}
