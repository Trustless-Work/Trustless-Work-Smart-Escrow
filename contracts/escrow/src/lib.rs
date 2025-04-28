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
mod modules {
    pub mod math {
        pub mod basic;
        pub mod safe;

        pub use basic::*;
        pub use safe::*;
    }

    pub mod fee {
        pub mod calculator;

        pub use calculator::*;
    }

    pub mod token {
        pub mod transfer_handler;

        pub use transfer_handler::*;
    }
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

pub use crate::contract::EscrowContract;
