use crate::storage::types::DataKey;
use soroban_sdk::{Address, Env};

#[allow(dead_code)]
pub fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().instance().has(&key)
}

#[allow(dead_code)]
pub fn read_administrator(e: &Env) -> Result<Address, &'static str> {
    let key = DataKey::Admin;
    e.storage().instance().get(&key).ok_or("Admin not found!")
}

#[allow(dead_code)]
pub fn write_administrator(e: &Env, id: &Address) -> Result<(), &'static str> {
    let key = DataKey::Admin;
    e.storage().instance().set(&key, id);

    let stored_admin = read_administrator(&e);
    if stored_admin != Ok(id.clone()) {
        return Err("Admin not found!");
    } else {
        Ok(())
    }
}