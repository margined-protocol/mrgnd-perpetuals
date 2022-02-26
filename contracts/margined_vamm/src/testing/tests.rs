use crate::contract::{execute, instantiate, query};
use crate::testing::setup::to_decimals;
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr};
use margined_perp::margined_vamm::{
    ConfigResponse, Direction, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
};

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            owner: info.sender.clone(),
            quote_asset: "ETH".to_string(),
            base_asset: "USD".to_string(),
            toll_ratio: Decimal256::zero(),
            spread_ratio: Decimal256::zero(),
            // decimals: DECIMAL_MULTIPLIER,
            decimals: Decimal256::one(),
        }
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1000"),
            base_asset_reserve: to_decimals("100"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("addr0001".to_string()),
        toll_ratio: None,
        spread_ratio: None,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            owner: Addr::unchecked("addr0001".to_string()),
            quote_asset: "ETH".to_string(),
            base_asset: "USD".to_string(),
            toll_ratio: Decimal256::zero(),
            spread_ratio: Decimal256::zero(),
            // decimals: DECIMAL_MULTIPLIER,
            decimals: Decimal256::one(),
        }
    );
}

#[test]
fn test_swap_input_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals("600"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1600"),
            base_asset_reserve: to_decimals("62.5"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_input_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals("600"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("400"),
            base_asset_reserve: to_decimals("250"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_output_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals("150"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("400"),
            base_asset_reserve: to_decimals("250"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_output_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals("50"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("2000"),
            base_asset_reserve: to_decimals("50"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_input_short_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals("480"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("520"),
            base_asset_reserve: to_decimals("192.307692307692307693"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals("960"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1480"),
            base_asset_reserve: to_decimals("67.567567567567567568"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_input_short_long_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals("200"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("800"),
            base_asset_reserve: to_decimals("125"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals("100"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("900"),
            base_asset_reserve: to_decimals("111.111111111111111112"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals("200"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1100"),
            base_asset_reserve: to_decimals("90.909090909090909092"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_input_short_long_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals("200"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("800"),
            base_asset_reserve: to_decimals("125"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals("450"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1250"),
            base_asset_reserve: to_decimals("80"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals("250"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1000"),
            base_asset_reserve: to_decimals("100"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_input_long_integration_example() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals("600"), // this is swapping 60 at 10x leverage
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1600"),
            base_asset_reserve: to_decimals("62.5"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_input_long_short_integration_example() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals("600"), // this is swapping 60 at 10x leverage
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1600"),
            base_asset_reserve: to_decimals("62.5"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );

    // Swap in ETH
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals("37.5"), // this is swapping 60 at 10x leverage
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1000"),
            base_asset_reserve: to_decimals("100"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_input_twice_short_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals("10"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals("10"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1000"),
            base_asset_reserve: to_decimals("100.000000000000000001"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_input_twice_long_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals("10"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals("10"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1000"),
            base_asset_reserve: to_decimals("100.000000000000000001"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_output_twice_short_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals("10"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals("10"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1000.000000000000000001"),
            base_asset_reserve: to_decimals("100"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}

#[test]
fn test_swap_output_twice_long_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals("1000"),
        base_asset_reserve: to_decimals("100"),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::zero(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals("10"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals("10"),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals("1000.000000000000000001"),
            base_asset_reserve: to_decimals("100"),
            funding_rate: Decimal256::zero(),
            funding_period: 3_600 as u64,
        }
    );
}
