use crate::error::ContractError;
use crate::{
    handle::{append_multiple_price, append_price, update_config},
    query::{query_config, query_get_previous_price, query_get_price, query_get_twap_price},
    state::{store_config, Config},
};

use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use margined_perp::margined_pricefeed::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config { owner: info.sender };

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AppendPrice {
            key,
            price,
            timestamp,
        } => append_price(deps, info, key, price, timestamp),
        ExecuteMsg::AppendMultiplePrice {
            key,
            prices,
            timestamps,
        } => append_multiple_price(deps, info, key, prices, timestamps),
        ExecuteMsg::UpdateConfig { owner } => update_config(deps, info, owner),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::GetPrice { key } => to_binary(&query_get_price(deps, key)?),
        QueryMsg::GetPreviousPrice {
            key,
            num_round_back,
        } => to_binary(&query_get_previous_price(deps, key, num_round_back)?),
        QueryMsg::GetTwapPrice { key, interval } => {
            to_binary(&query_get_twap_price(deps, env, key, interval)?)
        }
    }
}
