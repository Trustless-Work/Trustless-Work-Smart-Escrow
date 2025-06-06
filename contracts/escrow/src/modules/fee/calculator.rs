use crate::{
    error::ContractError,
    modules::{
        math::{BasicArithmetic, BasicMath},
        math::{SafeArithmetic, SafeMath},
    },
};

const TRUSTLESS_WORK_FEE: i128 = 500_000_000; // 500 USDC in base units

#[derive(Debug, Clone)]
pub struct StandardFeeResult {
    pub trustless_work_fee: i128,
    pub platform_fee: i128,
    pub receiver_amount: i128,
}

#[derive(Debug, Clone)]
pub struct DisputeFeeResult {
    pub trustless_work_fee: i128,
    pub platform_fee: i128,
    pub net_approver_funds: i128,
    pub net_receiver_funds: i128,
}

pub trait FeeCalculatorTrait {
    fn calculate_standard_fees(
        total_amount: i128,
        platform_fee: i128,
    ) -> Result<StandardFeeResult, ContractError>;

    fn calculate_dispute_fees(
        approver_funds: i128,
        receiver_funds: i128,
        platform_fee: i128,
        total_resolved_funds: i128,
    ) -> Result<DisputeFeeResult, ContractError>;
}

#[derive(Clone)]
pub struct FeeCalculator;

impl FeeCalculatorTrait for FeeCalculator {
    fn calculate_standard_fees(
        total_amount: i128,
        platform_fee: i128,
    ) -> Result<StandardFeeResult, ContractError> {
        let trustless_work_fee = TRUSTLESS_WORK_FEE;
        let platform_fee = platform_fee;

        let after_tw = BasicMath::safe_sub(total_amount, trustless_work_fee)?;
        let receiver_amount = BasicMath::safe_sub(after_tw, platform_fee)?;

        Ok(StandardFeeResult {
            trustless_work_fee,
            platform_fee,
            receiver_amount,
        })
    }

    fn calculate_dispute_fees(
        approver_funds: i128,
        receiver_funds: i128,
        platform_fee: i128,
        total_resolved_funds: i128,
    ) -> Result<DisputeFeeResult, ContractError> {
        let trustless_work_fee = TRUSTLESS_WORK_FEE;
        let platform_fee = platform_fee;
        let total_fees = BasicMath::safe_add(trustless_work_fee, platform_fee)?;

        let net_approver_funds = if total_resolved_funds > 0 {
            let approver_fee_share =
                SafeMath::safe_mul_div(approver_funds, total_fees, total_resolved_funds)?;
            BasicMath::safe_sub(approver_funds, approver_fee_share)?
        } else {
            0
        };

        let net_receiver_funds = if total_resolved_funds > 0 {
            let receiver_fee_share =
                SafeMath::safe_mul_div(receiver_funds, total_fees, total_resolved_funds)?;
            BasicMath::safe_sub(receiver_funds, receiver_fee_share)?
        } else {
            0
        };

        Ok(DisputeFeeResult {
            trustless_work_fee,
            platform_fee,
            net_approver_funds,
            net_receiver_funds,
        })
    }
}
