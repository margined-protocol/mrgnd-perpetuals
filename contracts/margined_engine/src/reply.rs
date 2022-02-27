use crate::{
    handle::{clear_position, get_position, internal_increase_position},
    state::{read_config, read_tmp_swap, remove_tmp_swap, store_position, store_tmp_swap},
    utils::side_to_direction,
};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, Env, ReplyOn, Response, StdError, StdResult, Storage,
    SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use std::str::FromStr;

// Increases position after successful execution of the swap
pub fn increase_position_reply(
    deps: DepsMut,
    env: Env,
    _input: Decimal256,
    output: Decimal256,
) -> StdResult<Response> {
    // let config = read_config(deps.storage)?;
    let tmp_swap = read_tmp_swap(deps.storage)?;
    if tmp_swap.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    let swap = tmp_swap.unwrap();
    let mut position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );

    // now update the position
    position.size = position.size + output;
    position.notional = position.notional + swap.open_notional;
    position.direction = side_to_direction(swap.side);

    // TODO make my own decimal math lib
    position.margin = position.notional / swap.leverage;
    // .checked_mul(config.decimals)?

    store_position(deps.storage, &position)?;

    // create transfer message
    let msg = execute_transfer_from(
        deps.storage,
        &swap.trader,
        &env.contract.address,
        position.margin,
    )
    .unwrap();

    remove_tmp_swap(deps.storage);

    Ok(Response::new().add_submessage(msg))
}

// Decreases position after successful execution of the swap
pub fn decrease_position_reply(
    deps: DepsMut,
    env: Env,
    _input: Decimal256,
    output: Decimal256,
) -> StdResult<Response> {
    let tmp_swap = read_tmp_swap(deps.storage)?;
    if tmp_swap.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    let swap = tmp_swap.unwrap();
    let mut position = get_position(
        env,
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );

    // now update the position
    position.size = position.size - output;
    position.notional = position.notional - swap.open_notional;

    store_position(deps.storage, &position)?;

    // remove the tmp position
    remove_tmp_swap(deps.storage);

    Ok(Response::new())
}

// Decreases position after successful execution of the swap
pub fn reverse_position_reply(
    deps: DepsMut,
    env: Env,
    _input: Decimal256,
    output: Decimal256,
) -> StdResult<Response> {
    let response: Response = Response::new();
    let tmp_swap = read_tmp_swap(deps.storage)?;
    if tmp_swap.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    let mut swap = tmp_swap.unwrap();
    let mut position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );
    let margin_amount = position.margin;

    position = clear_position(env, position)?;

    let msg: SubMsg;
    // now increase the position again if there is additional position
    let open_notional;
    if swap.open_notional > output {
        open_notional = swap.open_notional - output;
        swap.open_notional = swap.open_notional - output;
    } else {
        open_notional = output - swap.open_notional;
        swap.open_notional = output - swap.open_notional;
    }
    if open_notional / swap.leverage == Decimal256::zero() {
        // create transfer message
        msg = execute_transfer(deps.storage, &swap.trader, margin_amount).unwrap();
        remove_tmp_swap(deps.storage);
    } else {
        store_tmp_swap(deps.storage, &swap)?;

        msg = internal_increase_position(swap.vamm, swap.side, open_notional)?
    }

    store_position(deps.storage, &position)?;

    Ok(response.add_submessage(msg))
}

// Closes position after successful execution of the swap
pub fn close_position_reply(
    deps: DepsMut,
    env: Env,
    _input: Decimal256,
    output: Decimal256,
) -> StdResult<Response> {
    let tmp_swap = read_tmp_swap(deps.storage)?;
    if tmp_swap.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    let swap = tmp_swap.unwrap();
    let mut position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );

    let margin_returned: Decimal256;
    if output > swap.open_notional {
        margin_returned = position.margin - (output - swap.open_notional);
    } else {
        margin_returned = position.margin - (swap.open_notional - output);
    }

    position = clear_position(env, position)?;

    store_position(deps.storage, &position)?;

    // create transfer message
    let msg = execute_transfer(deps.storage, &swap.trader, margin_returned).unwrap();

    remove_tmp_swap(deps.storage);

    Ok(Response::new().add_submessage(msg))
}

fn execute_transfer_from(
    storage: &dyn Storage,
    owner: &Addr,
    receiver: &Addr,
    amount: Decimal256,
) -> StdResult<SubMsg> {
    let config = read_config(storage)?;
    let msg = WasmMsg::Execute {
        contract_addr: config.eligible_collateral.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: owner.to_string(),
            recipient: receiver.to_string(),
            amount: Uint128::from(u128::from_str(&amount.to_string()).unwrap()),
        })?,
    };

    let transfer_msg = SubMsg {
        msg: CosmosMsg::Wasm(msg),
        gas_limit: None, // probably should set a limit in the config
        id: 0u64,
        reply_on: ReplyOn::Never,
    };

    Ok(transfer_msg)
}

fn execute_transfer(
    storage: &dyn Storage,
    receiver: &Addr,
    amount: Decimal256,
) -> StdResult<SubMsg> {
    let config = read_config(storage)?;

    let msg = WasmMsg::Execute {
        contract_addr: config.eligible_collateral.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: receiver.to_string(),
            amount: Uint128::from(u128::from_str(&amount.to_string()).unwrap()),
        })?,
    };

    let transfer_msg = SubMsg {
        msg: CosmosMsg::Wasm(msg),
        gas_limit: None, // probably should set a limit in the config
        id: 0u64,
        reply_on: ReplyOn::Never,
    };

    Ok(transfer_msg)
}
