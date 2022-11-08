use crate::contract::{instantiate, query};
use crate::handle::{get_input_price_with_reserves, get_output_price_with_reserves};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Uint128};
use cw_multi_test::Executor;
use margined_perp::margined_vamm::{Direction, InstantiateMsg, QueryMsg, StateResponse};
use margined_utils::scenarios::{to_decimals, SimpleScenario};

/// Unit tests
#[test]
fn test_get_input_add_to_amm() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1_000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
        pricefeed: "oracle".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // getInputPrice, add to amm
    // amount = 0
    // price = 0
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputPrice {
            direction: Direction::AddToAmm,
            amount: Uint128::zero(),
        },
    )
    .unwrap();
    let result: Uint128 = from_binary(&res).unwrap();
    assert_eq!(result, Uint128::zero());

    // getInputPrice, add to amm
    // amount = 100(quote asset reserved) - (100 * 1000) / (1000 + 50) = 4.7619...
    // price = 50 / 4.7619 = 10.499
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::InputPrice {
            direction: Direction::AddToAmm,
            amount: to_decimals(50),
        },
    )
    .unwrap();
    let result: Uint128 = from_binary(&res).unwrap();
    assert_eq!(result, Uint128::from(10_500_000_001u128));

    // amount = (100 * 1000) / (1000 - 50) - 100(quote asset reserved) = 5.2631578947368
    // price = 50 / 5.263 = 9.5
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::InputPrice {
            direction: Direction::RemoveFromAmm,
            amount: to_decimals(50),
        },
    )
    .unwrap();
    let result: Uint128 = from_binary(&res).unwrap();
    assert_eq!(result, Uint128::from(9_499_999_999u128));

    // getOutputPrice, add to amm
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputPrice {
            direction: Direction::AddToAmm,
            amount: to_decimals(5),
        },
    )
    .unwrap();
    let result: Uint128 = from_binary(&res).unwrap();
    assert_eq!(result, Uint128::from(105_000_000u128));

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputPrice {
            direction: Direction::AddToAmm,
            amount: to_decimals(25),
        },
    )
    .unwrap();
    let result: Uint128 = from_binary(&res).unwrap();
    assert_eq!(result, Uint128::from(125_000_000u128));

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputPrice {
            direction: Direction::RemoveFromAmm,
            amount: to_decimals(5),
        },
    )
    .unwrap();
    let result: Uint128 = from_binary(&res).unwrap();
    assert_eq!(result, Uint128::from(94_999_999u128));

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputPrice {
            direction: Direction::RemoveFromAmm,
            amount: Uint128::from(37_500_000_000u128),
        },
    )
    .unwrap();
    let result: Uint128 = from_binary(&res).unwrap();
    assert_eq!(result, Uint128::from(62_500_000u128));
}

/// Unit tests
#[test]
fn test_get_output_amount() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1_000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
        pricefeed: "oracle".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // getOutputPrice, add to amm
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputAmount {
            direction: Direction::AddToAmm,
            amount: to_decimals(5),
        },
    )
    .unwrap();
    let result: Uint128 = from_binary(&res).unwrap();
    assert_eq!(result, Uint128::from(47619047619u128));

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputAmount {
            direction: Direction::AddToAmm,
            amount: to_decimals(25),
        },
    )
    .unwrap();
    let result: Uint128 = from_binary(&res).unwrap();
    assert_eq!(result, Uint128::from(200_000_000_000u128));

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputAmount {
            direction: Direction::RemoveFromAmm,
            amount: to_decimals(5),
        },
    )
    .unwrap();
    let result: Uint128 = from_binary(&res).unwrap();
    assert_eq!(result, Uint128::from(52_631_578_948u128));

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputAmount {
            direction: Direction::RemoveFromAmm,
            amount: Uint128::from(37_500_000_000u128),
        },
    )
    .unwrap();
    let result: Uint128 = from_binary(&res).unwrap();
    assert_eq!(result, Uint128::from(600_000_000_000u128));
}

#[test]
fn test_get_input_and_output_price_with_reserves() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1_000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
        pricefeed: "oracle".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();

    // amount = 0
    // price = 0
    let result = get_input_price_with_reserves(
        deps.as_ref(),
        &Direction::AddToAmm,
        Uint128::zero(),
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )
    .unwrap();
    assert_eq!(result, Uint128::zero());

    // amount = 0
    // price = 0
    let result = get_output_price_with_reserves(
        deps.as_ref(),
        &Direction::AddToAmm,
        Uint128::zero(),
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )
    .unwrap();
    assert_eq!(result, Uint128::zero());

    // amount = 100(quote asset reserved) - (100 * 1000) / (1000 + 50) = 4.7619...
    // price = 50 / 4.7619 = 10.499
    let quote_asset_amount = to_decimals(50);
    let result = get_input_price_with_reserves(
        deps.as_ref(),
        &Direction::AddToAmm,
        quote_asset_amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )
    .unwrap();
    assert_eq!(result, Uint128::from(4761904761u128));

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();

    // amount = (100 * 1000) / (1000 - 50) - 100(quote asset reserved) = 5.2631578947368
    // price = 50 / 5.263 = 9.5
    let quote_asset_amount = to_decimals(50);
    let result = get_input_price_with_reserves(
        deps.as_ref(),
        &Direction::RemoveFromAmm,
        quote_asset_amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )
    .unwrap();
    assert_eq!(result, Uint128::from(5263157895u128));

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();

    // amount = 1000(base asset reversed) - (100 * 1000) / (100 + 5) = 47.619047619047619048
    // price = 47.619 / 5 = 9.52
    let quote_asset_amount = to_decimals(5);
    let result = get_output_price_with_reserves(
        deps.as_ref(),
        &Direction::AddToAmm,
        quote_asset_amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )
    .unwrap();
    assert_eq!(result, Uint128::from(47619047619u128));

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();

    // a dividable number should not plus 1 at mantissa
    let quote_asset_amount = to_decimals(25);
    let result = get_output_price_with_reserves(
        deps.as_ref(),
        &Direction::AddToAmm,
        quote_asset_amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )
    .unwrap();
    assert_eq!(result, to_decimals(200));

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();

    // amount = (100 * 1000) / (100 - 5) - 1000(base asset reversed) = 52.631578947368
    // price = 52.631 / 5 = 10.52
    let quote_asset_amount = to_decimals(5);
    let result = get_output_price_with_reserves(
        deps.as_ref(),
        &Direction::RemoveFromAmm,
        quote_asset_amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )
    .unwrap();
    assert_eq!(result, Uint128::from(52631578948u128));

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();

    // divisable output
    let quote_asset_amount = Uint128::from(37_500_000_000u128);
    let result = get_output_price_with_reserves(
        deps.as_ref(),
        &Direction::RemoveFromAmm,
        quote_asset_amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )
    .unwrap();
    assert_eq!(result, to_decimals(600));
}

#[test]
fn test_rebase_vamm() {
    let SimpleScenario {
        mut router,
        owner,
        vamm,
        pricefeed,
        ..
    } = SimpleScenario::new();

    let spot_price = vamm.spot_price(&router).unwrap();
    assert_eq!(spot_price, to_decimals(10u64));

    let price: Uint128 = Uint128::from(20u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.rebase_vamm().unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let new_spot_price = vamm.spot_price(&router).unwrap();
    assert_eq!(new_spot_price, Uint128::from(20u64));
}
