use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Coin};

#[cw_serde]
pub struct Profile {
    pub user_id: String,
    pub pubkey: String,
}

#[cw_serde]
pub struct Message {
    pub sender: Addr,
    pub content: Binary,
    pub funds: Vec<Coin>,
}
