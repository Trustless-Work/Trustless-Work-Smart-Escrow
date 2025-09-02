use core::fmt;
use soroban_sdk::contracterror;

#[derive(Debug, Copy, Clone, PartialEq)]
#[contracterror]
pub enum ContractError {
    EscrowNotFunded = 1,
    AmountCannotBeZero = 2,
    EscrowAlreadyInitialized = 3,
    EscrowNotFound = 4,
    OnlyReleaseSignerCanReleaseEarnings = 5,
    MilestoneNotCompleted = 6,
    EscrowBalanceNotEnoughToSendEarnings = 7,
    OnlyPlatformAddressExecuteThisFunction = 8,
    OnlyServiceProviderChangeMilstoneStatus = 9,
    NoMileStoneDefined = 10,
    InvalidMileStoneIndex = 11,
    OnlyApproverChangeMilstoneFlag = 12,
    OnlyDisputeResolverCanExecuteThisFunction = 13,
    MilestoneAlreadyInDispute = 14,
    MilestoneNotInDispute = 15,
    InsufficientFundsForResolution = 16,
    MilestoneOpenedForDisputeResolution = 17,
    Overflow = 18,
    Underflow = 19,
    DivisionError = 20,
    InsufficientApproverFundsForCommissions = 21,
    InsufficientServiceProviderFundsForCommissions = 22,
    MilestoneApprovedCantChangeEscrowProperties = 23,
    EscrowHasFunds = 24,
    MilestoneAlreadyResolved = 25,
    TooManyEscrowsRequested = 26,
    UnauthorizedToChangeDisputeFlag = 27,
    TooManyMilestones = 28,
    CantReleaseAMilestoneInDispute = 29,
    MilestoneAlreadyReleased = 30,
    MilestoneNotFound = 31,
    MilestoneHasAlreadyBeenApproved = 32,
    EmptyMilestoneStatus = 33,
    PlatformFeeTooHigh = 34,
    FlagsMustBeFalse = 35,
    EscrowPropertiesMismatch = 36,
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::EscrowNotFunded => write!(f, "Escrow not funded"),
            ContractError::AmountCannotBeZero => write!(f, "Amount cannot be zero"),
            ContractError::EscrowAlreadyInitialized => write!(f, "Escrow already initialized"),
            ContractError::EscrowNotFound => write!(f, "Escrow not found"),
            ContractError::OnlyReleaseSignerCanReleaseEarnings => write!(
                f,
                "Only the release signer can release the escrow funds"
            ),
            ContractError::MilestoneNotCompleted => {
                write!(f, "The milestone must be completed to release funds")
            }
            ContractError::EscrowBalanceNotEnoughToSendEarnings => write!(
                f,
                "The escrow balance must be equal to the amount of earnings defined for the escrow"
            ),
            ContractError::OnlyPlatformAddressExecuteThisFunction => write!(
                f,
                "Only the platform address should be able to execute this function"
            ),
            ContractError::OnlyServiceProviderChangeMilstoneStatus => {
                write!(f, "Only the service provider can change milestone status")
            }
            ContractError::NoMileStoneDefined => write!(f, "Escrow initialized without milestone"),
            ContractError::InvalidMileStoneIndex => write!(f, "Invalid milestone index"),
            ContractError::OnlyApproverChangeMilstoneFlag => {
                write!(f, "Only the approver can change milestone flag")
            }
            ContractError::OnlyDisputeResolverCanExecuteThisFunction => {
                write!(f, "Only the dispute resolver can execute this function")
            }
            ContractError::MilestoneAlreadyInDispute => write!(f, "Milestone already in dispute"),
            ContractError::MilestoneNotInDispute => write!(f, "Milestone not in dispute"),
            ContractError::InsufficientFundsForResolution => {
                write!(f, "Insufficient funds for resolution")
            }
            ContractError::MilestoneOpenedForDisputeResolution => {
                write!(f, "Milestone has been opened for dispute resolution")
            }
            ContractError::InsufficientApproverFundsForCommissions => {
                write!(f, "Insufficient approver funds for commissions")
            }
            ContractError::InsufficientServiceProviderFundsForCommissions => {
                write!(f, "Insufficient Service Provider funds for commissions")
            }
            ContractError::MilestoneApprovedCantChangeEscrowProperties => {
                write!(
                    f,
                    "You cannot change the properties of an escrow after one of the milestones has been marked as approved"
                )
            }
            ContractError::EscrowHasFunds => write!(f, "Escrow has funds"),
            ContractError::Overflow => write!(f, "This operation can cause an Overflow"),
            ContractError::Underflow => write!(f, "This operation can cause an Underflow"),
            ContractError::DivisionError => write!(f, "This operation can cause Division error"),
            ContractError::MilestoneAlreadyResolved => write!(f, "This milestone is already resolved"),
            ContractError::TooManyEscrowsRequested => {
                write!(f, "You have requested too many escrows")
            }
            ContractError::UnauthorizedToChangeDisputeFlag => {
                write!(f, "You are not authorized to change the dispute flag")
            }
            ContractError::TooManyMilestones => {
                write!(f, "Cannot define more than 10 milestones in an escrow")
            },
            ContractError::CantReleaseAMilestoneInDispute => write!(f, "You cannot launch a milestone in dispute"),
            ContractError::MilestoneAlreadyReleased => write!(f, "This milestone is already released"),
            ContractError::MilestoneNotFound => write!(f, "Milestone not found"),
            ContractError::MilestoneHasAlreadyBeenApproved => {
                write!(f, "You cannot approve a milestone that has already been approved previously")
            },
            ContractError::EmptyMilestoneStatus => {
                write!(f, "The milestone status cannot be empty")
            },
            ContractError::PlatformFeeTooHigh => {
                write!(f, "The platform fee cannot exceed 99%")
            },
            ContractError::FlagsMustBeFalse => {
                write!(f, "All flags (approved, disputed, released) must be false in order to execute this function.")
            },
            ContractError::EscrowPropertiesMismatch => {
                write!(f, "The provided escrow properties do not match the stored escrow.")
            }
        }
    }
}