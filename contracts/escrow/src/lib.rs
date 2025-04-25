#![no_std]

mod contract;
mod core {
    pub mod admin;
    pub mod dispute;
    pub mod escrow;
    pub mod milestone;
    pub use dispute::*;
    pub use escrow::*;
    pub use milestone::*;
}
mod error;
mod events {
    pub mod handler;
    pub(crate) use handler::escrows_by_contract_id;
}

mod traits {
    pub mod safe_math;
    pub mod basic_math;
    pub use basic_math::{BasicMath, BasicArithmetic};
}

/// This module is currently Work In Progress.
mod storage {
    pub mod types;
}
mod tests {
    #[cfg(test)]
    mod test;
}
mod token {
    pub mod allowance;
    pub mod balance;
    pub mod metadata;
    pub mod token;
}

pub mod shared;

pub use crate::contract::EscrowContract;
