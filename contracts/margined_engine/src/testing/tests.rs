use crate::contract::{execute, instantiate, query};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr};
use margined_perp::margined_engine::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

const TOKEN: &str = "token";
const OWNER: &str = "owner";

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        eligible_collateral: TOKEN.to_string(),
        initial_margin_ratio: Decimal256::from_ratio(100u64, 1_000_000_000u64),
        maintenance_margin_ratio: Decimal256::from_ratio(100u64, 1_000_000_000u64),
        liquidation_fee: Decimal256::from_ratio(100u64, 1_000_000_000u64),
        vamm: vec!["test".to_string()],
    };
    let info = mock_info(OWNER, &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info(OWNER, &[]);
    assert_eq!(
        config,
        ConfigResponse {
            owner: info.sender.clone(),
            eligible_collateral: Addr::unchecked(TOKEN),
        }
    );
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        eligible_collateral: TOKEN.to_string(),
        initial_margin_ratio: Decimal256::from_ratio(100u64, 1_000_000_000u64),
        maintenance_margin_ratio: Decimal256::from_ratio(100u64, 1_000_000_000u64),
        liquidation_fee: Decimal256::from_ratio(100u64, 1_000_000_000u64),
        vamm: vec!["test".to_string()],
    };
    let info = mock_info(OWNER, &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: "addr0001".to_string(),
    };

    let info = mock_info(OWNER, &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            owner: Addr::unchecked("addr0001".to_string()),
            eligible_collateral: Addr::unchecked(TOKEN),
        }
    );

    // Update should fail
    let msg = ExecuteMsg::UpdateConfig {
        owner: OWNER.to_string(),
    };

    let info = mock_info(OWNER, &[]);
    let result = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(result.is_err());
}
