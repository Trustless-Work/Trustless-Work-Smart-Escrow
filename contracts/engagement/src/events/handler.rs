use soroban_sdk::{Env, vec, IntoVal, Val, String, symbol_short};
use crate::storage::types::Escrow;

// ------ Escrows
pub fn escrows_by_engagement_id(e: &Env, engagement_id: String, escrow: Escrow) {
    let topics = (symbol_short!("p_by_spdr"),);
    
    let engagement_id_val: Val = engagement_id.into_val(e);
    let escrow_val: Val = escrow.into_val(e);

    let event_payload = vec![e, engagement_id_val, escrow_val];
    e.events().publish(topics, event_payload);
}