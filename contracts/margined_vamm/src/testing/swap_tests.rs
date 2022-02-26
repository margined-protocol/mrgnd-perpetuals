use crate::contract::instantiate;
use crate::{
    handle::{get_input_price_with_reserves, get_output_price_with_reserves},
    testing::setup::to_decimals,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
// use cosmwasm_std::Uint128;
use cosmwasm_bignumber::Decimal256;
use margined_perp::margined_vamm::{Direction, InstantiateMsg};

/// Unit tests
#[test]
fn test_get_input_and_output_price() {
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

    // amount = 100(quote asset reserved) - (100 * 1000) / (1000 + 50) = 4.7619...
    // price = 50 / 4.7619 = 10.499
    let quote_asset_amount = to_decimals("50");
    let result =
        get_input_price_with_reserves(deps.as_ref(), &Direction::AddToAmm, quote_asset_amount)
            .unwrap();
    assert_eq!(result, to_decimals("4.761904761904761904"));

    // amount = (100 * 1000) / (1000 - 50) - 100(quote asset reserved) = 5.2631578947368
    // price = 50 / 5.263 = 9.5
    let quote_asset_amount = to_decimals("50");
    let result =
        get_input_price_with_reserves(deps.as_ref(), &Direction::RemoveFromAmm, quote_asset_amount)
            .unwrap();
    assert_eq!(result, to_decimals("5.263157894736842106"));

    // amount = 1000(base asset reversed) - (100 * 1000) / (100 + 5) = 47.619047619047619048
    // price = 47.619 / 5 = 9.52
    let quote_asset_amount = to_decimals("5");
    let result =
        get_output_price_with_reserves(deps.as_ref(), &Direction::AddToAmm, quote_asset_amount)
            .unwrap();
    assert_eq!(result, to_decimals("47.619047619047619047"));

    // a dividable number should not plus 1 at mantissa
    let quote_asset_amount = to_decimals("25");
    let result =
        get_output_price_with_reserves(deps.as_ref(), &Direction::AddToAmm, quote_asset_amount)
            .unwrap();
    assert_eq!(result, to_decimals("200"));

    // amount = (100 * 1000) / (100 - 5) - 1000(base asset reversed) = 52.631578947368
    // price = 52.631 / 5 = 10.52
    let quote_asset_amount = to_decimals("5");
    let result = get_output_price_with_reserves(
        deps.as_ref(),
        &Direction::RemoveFromAmm,
        quote_asset_amount,
    )
    .unwrap();
    assert_eq!(result, to_decimals("52.631578947368421053"));

    // divisable output
    let quote_asset_amount = to_decimals("37.5");
    let result = get_output_price_with_reserves(
        deps.as_ref(),
        &Direction::RemoveFromAmm,
        quote_asset_amount,
    )
    .unwrap();
    assert_eq!(result, to_decimals("600"));
}
