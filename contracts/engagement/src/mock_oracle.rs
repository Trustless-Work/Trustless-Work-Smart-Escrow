use soroban_sdk::{contract, contractimpl, Env, Symbol};


#[contract]
pub struct MockOracle;

#[contractimpl]
impl MockOracle {
    pub fn initialize(e: Env, result: Option<bool>) {
        e.storage().instance().set(&Symbol::new(&e, "result"), &result);
    }

    pub fn get_result(e: Env) -> Option<bool> {
        e.storage().instance().get(&Symbol::new(&e, "result")).unwrap_or(None)
    }

    pub fn set_result(e: Env, result: Option<bool>) {
        e.storage().instance().set(&Symbol::new(&e, "result"), &result);
    }
}