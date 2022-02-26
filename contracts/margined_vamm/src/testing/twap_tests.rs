use crate::contract::{execute, instantiate, query};
use crate::testing::setup::to_decimals;
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{from_binary, Env, OwnedDeps};
use margined_perp::margined_vamm::{Direction, ExecuteMsg, InstantiateMsg, QueryMsg};

pub struct TestingEnv {
    pub deps: OwnedDeps<MockStorage, MockApi, MockQuerier>,
    pub env: Env,
}

fn setup() -> TestingEnv {
    let mut env = mock_env();
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: to_decimals("0.01"),
        spread_ratio: to_decimals("0.01"),
    };

    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    env.block.time = env.block.time.plus_seconds(14);

    for i in 0..30 {
        if i % 3 == 0 {
            let swap_msg = ExecuteMsg::SwapInput {
                direction: Direction::RemoveFromAmm,
                quote_asset_amount: to_decimals("100"),
            };

            let info = mock_info("addr0000", &[]);
            execute(deps.as_mut(), env.clone(), info, swap_msg).unwrap();
        } else {
            let swap_msg = ExecuteMsg::SwapInput {
                direction: Direction::AddToAmm,
                quote_asset_amount: to_decimals("50"),
            };

            let info = mock_info("addr0000", &[]);
            execute(deps.as_mut(), env.clone(), info, swap_msg).unwrap();
        }
        env.block.time = env.block.time.plus_seconds(14);
    }

    TestingEnv { deps, env }
}

#[test]
fn test_get_twap_price() {
    let app = setup();

    let res = query(
        app.deps.as_ref(),
        app.env,
        QueryMsg::TwapPrice { interval: 210 },
    )
    .unwrap();
    let twap: Decimal256 = from_binary(&res).unwrap();
    assert_eq!(twap, to_decimals("9.041666666666666665"));
}

#[test]
fn test_no_change_in_snapshot() {
    let mut app = setup();

    // the timestamp of latest snapshot is now, the latest snapshot wont have any effect
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals("100"),
    };

    let info = mock_info("addr0000", &[]);
    execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    let res = query(
        app.deps.as_ref(),
        app.env,
        QueryMsg::TwapPrice { interval: 210 },
    )
    .unwrap();
    let twap: Decimal256 = from_binary(&res).unwrap();
    assert_eq!(twap, to_decimals("9.041666666666666665"));
}

#[test]
fn test_interval_greater_than_snapshots_have() {
    let app = setup();

    let res = query(
        app.deps.as_ref(),
        app.env,
        QueryMsg::TwapPrice { interval: 900 },
    )
    .unwrap();
    let twap: Decimal256 = from_binary(&res).unwrap();
    assert_eq!(twap, to_decimals("9.072580645161290321"));
}

#[test]
fn test_interval_less_than_latest_snapshots() {
    let mut app = setup();

    // the timestamp of latest snapshot is now, the latest snapshot wont have any effect
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals("100"),
    };

    let info = mock_info("addr0000", &[]);
    execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
    app.env.block.time = app.env.block.time.plus_seconds(300);

    let res = query(
        app.deps.as_ref(),
        app.env,
        QueryMsg::TwapPrice { interval: 210 },
    )
    .unwrap();
    let twap: Decimal256 = from_binary(&res).unwrap();
    assert_eq!(twap, to_decimals("8.099999999999999998"));
}

#[test]
fn test_zero_interval() {
    let app = setup();

    let res = query(
        app.deps.as_ref(),
        app.env.clone(),
        QueryMsg::TwapPrice { interval: 0 },
    )
    .unwrap();
    let twap: Decimal256 = from_binary(&res).unwrap();

    let res = query(app.deps.as_ref(), app.env, QueryMsg::SpotPrice {}).unwrap();
    let spot: Decimal256 = from_binary(&res).unwrap();
    assert_eq!(twap, spot);
}
