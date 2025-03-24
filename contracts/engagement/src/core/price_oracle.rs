use crate::error::ContractError;
use core::fmt::Error;

use soroban_sdk::{symbol_short, Address, Env, TryIntoVal, Val, Vec , contracttype  ,Symbol};

pub struct PriceOracle;

#[contracttype]
struct PriceData {
  price: i128,
  timestamp: u64
}


impl PriceOracle {
    pub fn initialize_oracle(env: Env, oracle_address: Address) -> bool {
        env.storage()
            .instance()
            .set(&symbol_short!("oracle"), &oracle_address);
        true
    }

    pub fn fetch_price(env: &Env) -> Result<i128, ContractError> {
        let lastprice_symbol = symbol_short!("lastprice");
        let oracle_key = symbol_short!("oracle");

        let oracle_address = match env.storage().instance().get(&oracle_key) {
            Some(x) => x,
            None => panic!("Please_initialize_oracle"),
        };

        let args = Vec::<Val>::new(env);
        let price_val: Val = env.invoke_contract(&oracle_address, &lastprice_symbol, args);

        match price_val.try_into_val(env) {
            Ok(price) => price,
            Err(_) => Err(ContractError::FailedToFetchPrice),
        }
    }

    pub fn checks_price_condition(env: Env, target_price: i128) -> (bool, i128) {
        let current_xlm_price = Self::fetch_price(&env).expect("failed_to_fetch_price");
        if current_xlm_price >= target_price {
            (true, current_xlm_price)
        } else {
            (false, current_xlm_price)
        }
    }
}
