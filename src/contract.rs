#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, Round, RoundWinners, CONFIG, CURRENT_ROUND, ROUND_HISTORY}; 

const CONTRACT_NAME: &str = "crates.io:lottery";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = msg.admin.unwrap_or(info.sender.to_string());
    let validated_admin = deps.api.addr_validate(&admin)?;
    let config = Config {
        admin: validated_admin.clone(),
        ticket_price: msg.ticket_price.clone(),
        round_duration: msg.round_duration,
        paused: false,
    };
    CONFIG.save(deps.storage, &config)?;
    let init_round = Round {
        id: 1,
        total_funds: Coin::new(0, msg.ticket_price.denom.clone()),
        participants: Vec::new(),
        start_time: env.block.time.seconds(),
    };

    CURRENT_ROUND.save(deps.storage, &init_round)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

fn current_time(env: &Env) -> u64 {
    env.block.time.seconds()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BuyTicket {} => try_buy_ticket(deps, env, info),
        ExecuteMsg::EndRound {} => try_end_round(deps, env, info),
        ExecuteMsg::Pause {} => try_pause(deps, info),
        ExecuteMsg::Resume {} => try_resume(deps, info),
    }
}

fn try_buy_ticket(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut round = CURRENT_ROUND.load(deps.storage)?;
    if config.paused {
        return Err(ContractError::Paused {});
    }

    let sent_funds = info.funds.iter().find(|c| c.denom == config.ticket_price.denom);
    match sent_funds {
        Some(coin) if coin.amount == config.ticket_price.amount => {
            round.participants.push(info.sender.clone());
            round.total_funds.amount += coin.amount;
            CURRENT_ROUND.save(deps.storage, &round)?;
            Ok(Response::new().add_attribute("action", "buy_ticket").add_attribute("ticket_id", round.participants.len().to_string()))
        }
        _ => Err(ContractError::InvalidFunds {}),
    }
}

fn pseudo_random(env: &Env, word: &str, id: u64, max_value: usize) -> usize {
    use sha2::{Sha256, Digest};
    let input = format!("{}{}{}", env.block.time.seconds(), word, id);
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    let hash_value = u64::from_be_bytes(result[0..8].try_into().unwrap());
    (hash_value % max_value as u64) as usize
}

fn try_end_round(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let round = CURRENT_ROUND.load(deps.storage)?;

    if current_time(&env) < round.start_time + config.round_duration {
        return Err(ContractError::RoundNotEnded {});
    }

    if round.participants.is_empty() {
        return Err(ContractError::NoParticipants {});
    }

    let winner_count = 3;
    let prize_amount = round.total_funds.amount.u128() * 90 / 100 / winner_count;
    let admin_fee = round.total_funds.amount.u128() * 10 / 100;

    let mut messages: Vec<CosmosMsg> = Vec::new();
    let word = "totally random";
    let mut selected_indices = Vec::new();

    for i in 0..winner_count {
        let mut index = pseudo_random(&env, word, i as u64 + 1, round.participants.len());

        while selected_indices.contains(&index) {
            index = pseudo_random(&env, word, index as u64 + 1, round.participants.len());
        }
        selected_indices.push(index);

        let winner = round.participants[index].clone();
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: winner.clone().into(),
            amount: vec![Coin::new(prize_amount, &config.ticket_price.denom)],
        }));
    }

    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: config.admin.into(),
        amount: vec![Coin::new(admin_fee, &config.ticket_price.denom)],
    }));

    // Save the winners to ROUND_HISTORY
    let round_winners = RoundWinners {
        winners: selected_indices.iter().map(|&i| round.participants[i].clone()).collect(),
    };
    ROUND_HISTORY.save(deps.storage, round.id, &round_winners)?;

    let new_round = Round {
        id: round.id + 1,
        total_funds: Coin::new(0, config.ticket_price.denom.clone()),
        participants: Vec::new(),
        start_time: env.block.time.seconds(),
    };
    CURRENT_ROUND.save(deps.storage, &new_round)?;

    Ok(Response::new().add_messages(messages).add_attribute("action", "end_round"))
}

fn try_resume(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    config.paused = false;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "resume"))
}

fn try_pause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    config.paused = true;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "pause"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTicketId { address } => to_binary(&query_ticket_id(deps, address)?),
        QueryMsg::GetRoundWinners { round_id } => to_binary(&query_round_winners(deps, round_id)?),
    }
}

fn query_ticket_id(deps: Deps, address: String) -> StdResult<u64> {
    let round = CURRENT_ROUND.load(deps.storage)?;
    for (i, participant) in round.participants.iter().enumerate() {
        if participant == &deps.api.addr_validate(&address)? {
            return Ok(i as u64 + 1);
        }
    }
    Err(ContractError::ParticipantNotFound {}.into())
}

fn query_round_winners(deps: Deps, round_id: u64) -> StdResult<Vec<Addr>> {
    let round_winners = ROUND_HISTORY.load(deps.storage, round_id)?;
    Ok(round_winners.winners)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::CURRENT_ROUND;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, Addr, from_binary};

    #[test]
    fn test_buy_ticket() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("admin", &[]);

        let msg = InstantiateMsg {
            admin: Some("admin".to_string()),
            ticket_price: coin(1, "orai"),
            round_duration: 604800,
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("buyer", &[coin(1, "orai")]);
        let msg = ExecuteMsg::BuyTicket {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let round = CURRENT_ROUND.load(&deps.storage).unwrap();
        assert_eq!(round.participants.len(), 1);
        assert_eq!(round.total_funds.amount.u128(), 1);
    }

    #[test]
    fn test_buy_ticket_insufficient_funds() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("admin", &[]);

        let msg = InstantiateMsg {
            admin: Some("admin".to_string()),
            ticket_price: coin(1, "orai"),
            round_duration: 604800,
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("buyer", &[coin(1, "btc")]);
        let msg = ExecuteMsg::BuyTicket {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        assert!(res.is_err());
    }

    #[test]
    fn test_end_round_not_ended() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("admin", &[]);

        let msg = InstantiateMsg {
            admin: Some("admin".to_string()),
            ticket_price: coin(1, "orai"),
            round_duration: 604800,
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::EndRound {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        assert!(res.is_err());
    }

    #[test]
    fn test_end_round_authorized() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info("admin", &[]);

        let msg = InstantiateMsg {
            admin: Some("admin".to_string()),
            ticket_price: coin(1, "orai"),
            round_duration: 1000,
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("buyer1", &[coin(1, "orai")]);
        let msg = ExecuteMsg::BuyTicket {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("buyer2", &[coin(1, "orai")]);
        let msg = ExecuteMsg::BuyTicket {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("buyer3", &[coin(1, "orai")]);
        let msg = ExecuteMsg::BuyTicket {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("buyer4", &[coin(1, "orai")]);
        let msg = ExecuteMsg::BuyTicket {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        env.block.time = env.block.time.plus_seconds(1001);

        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::EndRound {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let new_round = CURRENT_ROUND.load(&deps.storage).unwrap();
        assert_eq!(new_round.id, 2);
        assert_eq!(new_round.total_funds.amount.u128(), 0);
        assert_eq!(new_round.participants.len(), 0);
    }

    #[test]
    fn test_query_ticket_id() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("admin", &[]);

        let msg = InstantiateMsg {
            admin: Some("admin".to_string()),
            ticket_price: coin(1, "orai"),
            round_duration: 604800,
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("buyer", &[coin(1, "orai")]);
        let msg = ExecuteMsg::BuyTicket {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let query_msg = QueryMsg::GetTicketId { address: "buyer".to_string() };
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let ticket_id: u64 = from_binary(&res).unwrap();
        assert_eq!(ticket_id, 1);
    }

    #[test]
    fn test_query_round_winners() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info("admin", &[]);

        let msg = InstantiateMsg {
            admin: Some("admin".to_string()),
            ticket_price: coin(1, "orai"),
            round_duration: 1000,
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("buyer1", &[coin(1, "orai")]);
        let msg = ExecuteMsg::BuyTicket {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("buyer2", &[coin(1, "orai")]);
        let msg = ExecuteMsg::BuyTicket {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("buyer3", &[coin(1, "orai")]);
        let msg = ExecuteMsg::BuyTicket {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("buyer4", &[coin(1, "orai")]);
        let msg = ExecuteMsg::BuyTicket {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        env.block.time = env.block.time.plus_seconds(1001);

        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::EndRound {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let query_msg = QueryMsg::GetRoundWinners { round_id: 1 };
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let winners: Vec<Addr> = from_binary(&res).unwrap();
        assert_eq!(winners.len(), 3);
    }
}
