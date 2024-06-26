use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Coin;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub ticket_price: Coin,
    pub round_duration: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    BuyTicket {},
    EndRound {},
    Pause {},
    Resume {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetTicketId { address: String },
    GetRoundWinners { round_id: u64 },
}
