use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub ticket_price: Coin,
    pub round_duration: u64,
    pub paused: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Round {
    pub id: u64,
    pub total_funds: Coin,
    pub participants: Vec<Addr>,
    pub start_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoundWinners {
    pub winners: Vec<Addr>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CURRENT_ROUND: Item<Round> = Item::new("current_round");
pub const ROUND_HISTORY: Map<u64, RoundWinners> = Map::new("round_history");
