use crate::core::reflector_oracle::{Asset, Client as PriceOracleClient, PriceData};
use crate::storage::types::oracle_key;
use soroban_sdk::{symbol_short, Address, Env};

pub struct PriceOracle;

impl PriceOracle {
    pub fn initialize_oracle(env: Env, oracle_address: Address) -> bool {
        env.storage().instance().set(&oracle_key, &oracle_address);
        true
    }

    pub fn fetch_price(env: &Env, asset: Asset) -> PriceData {
        let reflector_contract_id = match env.storage().instance().get(&oracle_key) {
            Some(x) => x,
            None => panic!("Please_initialize_oracle"),
        };

        let reflector_contract = PriceOracleClient::new(env, &reflector_contract_id);

        let asset_price: Option<PriceData> = reflector_contract.lastprice(&asset);

        match asset_price {
            Some(x) => x,
            None => panic!("failed_to_fetch_price"),
        }
    }

    pub fn checks_price_condition(
        env: Env,
        asset: Asset,
        target_price: i128,
        amount: i128,
    ) -> bool {
        let current_price = Self::fetch_price(&env, asset);
        current_price.price * amount >= target_price
    }
}
