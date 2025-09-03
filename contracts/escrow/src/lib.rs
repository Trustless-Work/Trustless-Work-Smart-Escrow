#![no_std]

mod core {
    pub mod dispute;
    pub mod escrow;
    pub mod milestone;
    pub use dispute::*;
    pub use escrow::*;
    pub use milestone::*;
    pub mod validators {
        pub mod dispute;
        pub mod escrow;
        pub mod milestone;
    }
}
mod storage {
    pub mod types;
}
mod events {
    pub mod handler;
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
}
mod error;
mod contract;
mod tests;

pub use crate::contract::EscrowContract;