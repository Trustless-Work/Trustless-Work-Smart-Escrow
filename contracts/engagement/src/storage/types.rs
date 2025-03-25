use soroban_sdk::{contracttype, Address, String, Vec};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Escrow {
    pub engagement_id: String,
    pub title: String,
    pub description: String,
    pub approver: Address,
    pub service_provider: Address,
    pub platform_address: Address,
    pub amount: i128,
    pub platform_fee: i128,
    pub milestones: Vec<Milestone>,
    pub release_signer: Address,
    pub dispute_resolver: Address,
    pub dispute_flag: bool,
    pub release_flag: bool,
    pub resolved_flag: bool,
    pub trustline: Address,
    pub trustline_decimals: i128,
     pub oracle_id: Address,             // Oracle contract address
    pub party_a: Address,               // First party (receives funds if condition is true)
    pub party_b: Address,               // Second party (receives funds if condition is false)
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    pub description: String,
    pub status: String,
    pub approved_flag: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AddressBalance {
    pub address: Address,
    pub balance: i128,
    pub trustline_decimals: i128,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Escrow,
    Balance(Address),
    Allowance(AllowanceDataKey),
    Admin,
}
