use soroban_sdk::{Address, Env};

use crate::error::ContractError;
use crate::storage::types::DataKey;

pub fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().instance().has(&key)
}

pub fn read_administrator(e: &Env) -> Result<Address, ContractError> {
    let key = DataKey::Admin;
    e.storage()
        .instance()
        .get(&key)
        .ok_or(ContractError::AdminNotFound) // Added Result type to avoid panic if the return type is none
}

pub fn write_administrator(e: &Env, id: &Address) -> Result<(), ContractError> {
    // Added result type to return error if code fails
    let key = DataKey::Admin;
    e.storage().instance().set(&key, id);

    let stored_admin = read_administrator(&e);
    if stored_admin != Ok(id.clone()) {
        return Err(ContractError::AdminNotFound);
    } else {
        Ok(())
    }
}