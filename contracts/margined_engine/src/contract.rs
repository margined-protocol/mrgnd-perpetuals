use crate::error::ContractError;
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Attribute, Binary, ContractResult, Deps, DepsMut, Env, Event,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsgExecutionResponse,
};
use cw20::Cw20ReceiveMsg;
use margined_perp::margined_engine::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
#[cfg(not(feature = "library"))]
use std::str::FromStr;

use crate::{
    handle::{close_position, open_position, update_config},
    query::{
        query_config, query_position, query_trader_balance_with_funding_payment,
        query_unrealized_pnl,
    },
    reply::{
        close_position_reply, decrease_position_reply, increase_position_reply,
        reverse_position_reply,
    },
    state::{read_config, store_config, store_vamm, Config},
};

pub const SWAP_INCREASE_REPLY_ID: u64 = 1;
pub const SWAP_DECREASE_REPLY_ID: u64 = 2;
pub const SWAP_REVERSE_REPLY_ID: u64 = 3;
pub const SWAP_CLOSE_REPLY_ID: u64 = 4;

// #[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // let decimals = Uint128::from(10u128.pow(msg.decimals as u32));
    let eligible_collateral = deps.api.addr_validate(&msg.eligible_collateral)?;

    // config parameters
    let config = Config {
        owner: info.sender,
        eligible_collateral,
        decimals: Decimal256::one(),
        initial_margin_ratio: msg.initial_margin_ratio,
        maintenance_margin_ratio: msg.maintenance_margin_ratio,
        liquidation_fee: msg.liquidation_fee,
    };

    store_config(deps.storage, &config)?;

    // store default vamms
    store_vamm(deps, &msg.vamm)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateConfig { owner } => update_config(deps, info, owner),
        ExecuteMsg::OpenPosition {
            vamm,
            side,
            quote_asset_amount,
            leverage,
        } => {
            let trader = info.sender.clone();
            open_position(
                deps,
                env,
                info,
                vamm,
                trader.to_string(),
                side,
                quote_asset_amount,
                leverage,
            )
        }
        ExecuteMsg::ClosePosition { vamm } => {
            let trader = info.sender.clone();
            close_position(deps, env, info, vamm, trader.to_string())
        }
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    // only asset contract can execute this message
    let config: Config = read_config(deps.storage)?;
    if config.eligible_collateral != deps.api.addr_validate(info.sender.as_str())? {
        return Err(StdError::generic_err("unauthorized"));
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::OpenPosition {
            vamm,
            side,
            leverage,
        }) => open_position(
            deps,
            env,
            info,
            vamm,
            cw20_msg.sender,
            side,
            // Decimal256::from_uint256(Decimal256::FromStr(cw20_msg.amount.to_string())), // not needed, we should take from deposited amount or validate
            Decimal256::from_str(&cw20_msg.amount.to_string()).unwrap(), // not needed, we should take from deposited amount or validate
            leverage,
        ),
        Err(_) => Err(StdError::generic_err("invalid cw20 hook message")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Position { vamm, trader } => to_binary(&query_position(deps, vamm, trader)?),
        QueryMsg::TraderBalance { trader } => {
            to_binary(&query_trader_balance_with_funding_payment(deps, trader)?)
        }
        QueryMsg::UnrealizedPnl { vamm, trader } => {
            to_binary(&query_unrealized_pnl(deps, vamm, trader)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.result {
        ContractResult::Ok(response) => match msg.id {
            SWAP_INCREASE_REPLY_ID => {
                let (input, output) = parse_swap(response);
                let response = increase_position_reply(deps, env, input, output)?;
                Ok(response)
            }
            SWAP_DECREASE_REPLY_ID => {
                let (input, output) = parse_swap(response);
                let response = decrease_position_reply(deps, env, input, output)?;
                Ok(response)
            }
            SWAP_REVERSE_REPLY_ID => {
                let (input, output) = parse_swap(response);
                let response = reverse_position_reply(deps, env, input, output)?;
                Ok(response)
            }
            SWAP_CLOSE_REPLY_ID => {
                let (input, output) = parse_swap(response);
                let response = close_position_reply(deps, env, input, output)?;
                Ok(response)
            }
            _ => Err(StdError::generic_err(format!(
                "reply (id {:?}) invalid",
                msg.id
            ))),
        },
        ContractResult::Err(e) => Err(StdError::generic_err(format!(
            "reply (id {:?}) error {:?}",
            msg.id, e
        ))),
    }
}

fn parse_swap(response: SubMsgExecutionResponse) -> (Decimal256, Decimal256) {
    // Find swap inputs and output events
    let wasm = response.events.iter().find(|&e| e.ty == "wasm");
    let wasm = wasm.unwrap();

    let input_str = read_event("input".to_string(), wasm).value;
    let input: Decimal256 = Decimal256::from_str(&input_str).unwrap();

    let output_str = read_event("output".to_string(), wasm).value;
    let output: Decimal256 = Decimal256::from_str(&output_str).unwrap();

    (input, output)
}

fn read_event(key: String, event: &Event) -> Attribute {
    let result = event
        .attributes
        .iter()
        .find(|&attr| attr.key == key)
        .unwrap();
    result.clone()
}
