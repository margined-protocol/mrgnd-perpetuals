use crate::testing::setup::{self, to_decimals};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::Uint128;
use cw20::Cw20Contract;
use cw_multi_test::Executor;
use margined_perp::{
    margined_engine::{ConfigResponse, ExecuteMsg, PositionResponse, QueryMsg, Side},
    margined_vamm::{QueryMsg as VammQueryMsg, StateResponse},
};

#[test]
fn test_initialization() {
    let env = setup::setup();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    // verfiy the balances
    let owner_balance = usdc.balance(&env.router, env.owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::zero());
    let alice_balance = usdc.balance(&env.router, env.alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(5_000_000_000_000));
    let bob_balance = usdc.balance(&env.router, env.bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::new(5_000_000_000_000));
    let engine_balance = usdc.balance(&env.router, env.engine.addr.clone()).unwrap();
    assert_eq!(engine_balance, Uint128::zero());
}

#[test]
fn test_open_position_long() {
    let mut env = setup::setup();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(60u64),
        leverage: to_decimals(10u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    // expect to be 60
    let margin = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::TraderBalance {
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(to_decimals(60), margin);

    // personal position should be 37.5
    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(Decimal256::from_ratio(375, 10), position.size);
    assert_eq!(to_decimals(60u64), position.margin);

    // clearing house token balance should be 60
    let engine_balance = usdc.balance(&env.router, env.engine.addr.clone()).unwrap();
    assert_eq!(
        Decimal256::from_uint256(Uint256::from(engine_balance)),
        to_decimals(60)
    );
}

#[test]
fn test_open_position_two_longs() {
    let mut env = setup::setup();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(60u64),
        leverage: to_decimals(10u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(60u64),
        leverage: to_decimals(10u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    // expect to be 120
    let margin = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::TraderBalance {
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(to_decimals(120), margin);

    // retrieve the vamm state
    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        Decimal256::from_ratio(54_545_454_545u64, 1_000_000_000u64),
        position.size
    );
    assert_eq!(to_decimals(120), position.margin);
}

#[test]
fn test_open_position_two_shorts() {
    let mut env = setup::setup();

    // verify the engine owner
    let _config: ConfigResponse = env
        .router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::SELL,
        quote_asset_amount: to_decimals(40u64),
        leverage: to_decimals(5u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::SELL,
        quote_asset_amount: to_decimals(40u64),
        leverage: to_decimals(5u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    // personal balance with funding payment
    let margin = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::TraderBalance {
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(to_decimals(80), margin);

    // retrieve the vamm state
    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        Decimal256::from_ratio(66_666_666_667u64, 1_000_000_000u64),
        position.size
    );
    assert_eq!(to_decimals(80), position.margin);
}

#[test]
fn test_open_position_equal_size_opposite_side() {
    let mut env = setup::setup();

    // verify the engine owner
    let _config: ConfigResponse = env
        .router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(60),
        leverage: to_decimals(10),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::SELL,
        quote_asset_amount: to_decimals(300),
        leverage: to_decimals(2),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    // personal balance with funding payment
    let margin = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::TraderBalance {
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(Decimal256::zero(), margin);

    // retrieve the vamm state
    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(Decimal256::zero(), position.size);
    assert_eq!(Decimal256::zero(), position.margin);
}

#[test]
fn test_open_position_one_long_two_shorts() {
    let mut env = setup::setup();

    // verify the engine owner
    let _config: ConfigResponse = env
        .router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(60),
        leverage: to_decimals(10),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::SELL,
        quote_asset_amount: to_decimals(20),
        leverage: to_decimals(5),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    // retrieve the vamm state
    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        Decimal256::from_ratio(33_333_333_333u64, 1_000_000_000u64),
        position.size
    );
    assert_eq!(to_decimals(60), position.margin);

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::SELL,
        quote_asset_amount: to_decimals(50),
        leverage: to_decimals(10),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    // personal balance with funding payment
    let margin = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::TraderBalance {
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(Decimal256::zero(), margin);

    // retrieve the vamm state
    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(Decimal256::zero(), position.size);
    assert_eq!(Decimal256::zero(), position.margin);
}

#[test]
fn test_open_position_short_and_two_longs() {
    let mut env = setup::setup();

    // verify the engine owner
    let _config: ConfigResponse = env
        .router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::SELL,
        quote_asset_amount: to_decimals(40u64),
        leverage: to_decimals(5u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(to_decimals(25), position.size);
    assert_eq!(to_decimals(40), position.margin);

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(20u64),
        leverage: to_decimals(5u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        Decimal256::from_ratio(11_111_111_112u64, 1_000_000_000u64),
        position.size
    );
    assert_eq!(to_decimals(40), position.margin);

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(10u64),
        leverage: to_decimals(10u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    // retrieve the vamm state
    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        Decimal256::from_ratio(1u64, 1_000_000_000u64),
        position.size
    );
    assert_eq!(to_decimals(40u64), position.margin);
}

#[test]
fn test_open_position_short_long_short() {
    let mut env = setup::setup();

    // verify the engine owner
    let _config: ConfigResponse = env
        .router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::SELL,
        quote_asset_amount: to_decimals(20u64),
        leverage: to_decimals(10u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(150u64),
        leverage: to_decimals(3u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(to_decimals(20u64), position.size);
    assert_eq!(
        Decimal256::from_ratio(83_333_333_333u64, 1_000_000_000u64),
        position.margin
    );

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::SELL,
        quote_asset_amount: to_decimals(25u64),
        leverage: to_decimals(10u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    // retrieve the vamm state
    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(Decimal256::zero(), position.size);
    assert_eq!(Decimal256::zero(), position.margin);
}

#[test]
fn test_open_position_long_short_long() {
    let mut env = setup::setup();

    // verify the engine owner
    let _config: ConfigResponse = env
        .router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(25),
        leverage: to_decimals(10),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::SELL,
        quote_asset_amount: to_decimals(150),
        leverage: to_decimals(3),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(to_decimals(25), position.size);
    assert_eq!(
        Decimal256::from_ratio(66_666_666_666u64, 1_000_000_000u64),
        position.margin
    );

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(20),
        leverage: to_decimals(10),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    // retrieve the vamm state
    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(Decimal256::zero(), position.size);
    assert_eq!(Decimal256::zero(), position.margin);
}

#[test]
fn test_pnl_zero_no_others_trading() {
    let mut env = setup::setup();

    // verify the engine owner
    let _config: ConfigResponse = env
        .router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(250u64),
        leverage: to_decimals(1u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(750u64),
        leverage: to_decimals(1u64),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    // retrieve the vamm state
    let pnl: Uint128 = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::UnrealizedPnl {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(Uint128::zero(), pnl);
}

#[test]
fn test_close_safe_position() {
    let mut env = setup::setup();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    // verify the engine owner
    let _config: ConfigResponse = env
        .router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::SELL,
        quote_asset_amount: to_decimals(50),
        leverage: to_decimals(2),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    // retrieve the vamm state
    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        Decimal256::from_ratio(11_111_111_112u64, 1_000_000_000u64),
        position.size
    );

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(10),
        leverage: to_decimals(6),
    };

    let _res = env
        .router
        .execute_contract(env.bob.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let state: StateResponse = env
        .router
        .wrap()
        .query_wasm_smart(&env.vamm.addr, &VammQueryMsg::State {})
        .unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(960));
    assert_eq!(
        state.base_asset_reserve,
        Decimal256::from_ratio(104_166_666_668u64, 1_000_000_000u64)
    );

    let msg = ExecuteMsg::ClosePosition {
        vamm: env.vamm.addr.to_string(),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(position.size, Decimal256::zero());
    assert_eq!(position.margin, Decimal256::zero());
    assert_eq!(position.notional, Decimal256::zero());

    let state: StateResponse = env
        .router
        .wrap()
        .query_wasm_smart(&env.vamm.addr, &VammQueryMsg::State {})
        .unwrap();
    assert_eq!(
        state.quote_asset_reserve,
        Decimal256::from_ratio(1_074_626_865_681u64, 1_000_000_000u64),
        // Uint128::from(1_074_626_865_681u128)
    );
    assert_eq!(
        state.base_asset_reserve,
        Decimal256::from_ratio(93_055_555_556u64, 1_000_000_000u64)
    );

    // alice balance should be 4985.373134319
    let engine_balance = usdc.balance(&env.router, env.alice.clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(4_985_373_134_319u128));
}

#[test]
fn test_close_position_over_maintenance_margin_ration() {
    let mut env = setup::setup();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(25),
        leverage: to_decimals(10),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(position.size, to_decimals(20));

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::SELL,
        quote_asset_amount: Decimal256::from_ratio(3508u64, 100u64),
        leverage: to_decimals(1),
    };

    let _res = env
        .router
        .execute_contract(env.bob.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::ClosePosition {
        vamm: env.vamm.addr.to_string(),
    };

    let _res = env
        .router
        .execute_contract(env.alice.clone(), env.engine.addr.clone(), &msg, &[])
        .unwrap();

    let position: PositionResponse = env
        .router
        .wrap()
        .query_wasm_smart(
            &env.engine.addr,
            &QueryMsg::Position {
                vamm: env.vamm.addr.to_string(),
                trader: env.alice.to_string(),
            },
        )
        .unwrap();
    assert_eq!(position.size, Decimal256::zero());

    let state: StateResponse = env
        .router
        .wrap()
        .query_wasm_smart(&env.vamm.addr, &VammQueryMsg::State {})
        .unwrap();
    assert_eq!(
        state.quote_asset_reserve,
        Decimal256::from_ratio(977_422_074_621u64, 1_000_000_000u64),
    );
    assert_eq!(
        state.base_asset_reserve,
        Decimal256::from_ratio(102_309_946_334u64, 1_000_000_000u64)
    );
}
