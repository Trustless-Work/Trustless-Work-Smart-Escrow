use crate::storage::types::Escrow;
use soroban_sdk::{contractevent, String};

#[contractevent]
#[derive(Clone)]
pub struct InitEsc {
    pub escrow: Escrow,
}

#[contractevent(data_format = "vec")]
#[derive(Clone)]
pub struct FundEsc {
    pub signer: soroban_sdk::Address,
    pub amount: i128,
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct DisEsc {
    pub release_signer: soroban_sdk::Address,
}

#[contractevent(data_format = "vec")]
#[derive(Clone)]
pub struct ChgEsc {
    pub platform: soroban_sdk::Address,
    pub engagement_id: String,
}

#[contractevent(topics = ["p_by_spdr"], data_format = "vec")]
#[derive(Clone)]
pub struct EscrowsBySpdr {
    pub escrow: Escrow,
}
