use crate::{
    error::ContractError,
    modules::{
        math::{BasicArithmetic, BasicMath},
        math::{SafeArithmetic, SafeMath},
    },
};

const TRUSTLESS_WORK_FEE_BPS: u32 = 30;
const BASIS_POINTS_DENOMINATOR: i128 = 10000;

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
        platform_fee_bps: u32,
    ) -> Result<StandardFeeResult, ContractError>;

    fn calculate_dispute_fees(
        approver_funds: i128,
        receiver_funds: i128,
        platform_fee_bps: u32,
        total_funds: i128,
    ) -> Result<DisputeFeeResult, ContractError>;
}

#[derive(Clone)]
pub struct FeeCalculator;

impl FeeCalculatorTrait for FeeCalculator {
    fn calculate_standard_fees(
        total_amount: i128,
        platform_fee_bps: u32,
    ) -> Result<StandardFeeResult, ContractError> {
        let trustless_work_fee = SafeMath::safe_mul_div(
            total_amount,
            TRUSTLESS_WORK_FEE_BPS,
            BASIS_POINTS_DENOMINATOR,
        )?;
        let platform_fee =
            SafeMath::safe_mul_div(total_amount, platform_fee_bps, BASIS_POINTS_DENOMINATOR)?;

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
        platform_fee_bps: u32,
        total_funds: i128,
    ) -> Result<DisputeFeeResult, ContractError> {
        let computed_total = BasicMath::safe_add(approver_funds, receiver_funds)?;
        if computed_total <= 0 {
            return Err(ContractError::DivisionError);
        }

        let trustless_work_fee = SafeMath::safe_mul_div(
            total_funds,
            TRUSTLESS_WORK_FEE_BPS,
            BASIS_POINTS_DENOMINATOR,
        )?;
        let platform_fee = SafeMath::safe_mul_div(
            total_funds,
            platform_fee_bps,
            BASIS_POINTS_DENOMINATOR,
        )?;
        let total_fees = BasicMath::safe_add(trustless_work_fee, platform_fee)?;

        let approver_fee_share =
            SafeMath::safe_mul_div(approver_funds, total_fees as u32, total_funds)?;
        let net_approver_funds = BasicMath::safe_sub(approver_funds, approver_fee_share)?;

        let receiver_fee_share =
            SafeMath::safe_mul_div(receiver_funds, total_fees as u32, total_funds)?;
        let net_receiver_funds = BasicMath::safe_sub(receiver_funds, receiver_fee_share)?;

        Ok(DisputeFeeResult {
            trustless_work_fee,
            platform_fee,
            net_approver_funds,
            net_receiver_funds,
        })
    }
}