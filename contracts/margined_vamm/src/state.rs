use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Addr, StdResult, Storage, Timestamp};
use cosmwasm_storage::{bucket, bucket_read, singleton, singleton_read};

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_STATE: &[u8] = b"state";
pub static KEY_RESERVE_SNAPSHOT: &[u8] = b"reserve_snapshot";
pub static KEY_RESERVE_SNAPSHOT_COUNTER: &[u8] = b"reserve_snapshot_counter";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub quote_asset: String,
    pub base_asset: String,
    pub decimals: Decimal256,
    pub toll_ratio: Decimal256,
    pub spread_ratio: Decimal256,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub quote_asset_reserve: Decimal256,
    pub base_asset_reserve: Decimal256,
    pub funding_rate: Decimal256,
    pub funding_period: u64,
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    singleton(storage, KEY_STATE).save(state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    singleton_read(storage, KEY_STATE).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct ReserveSnapshot {
    pub quote_asset_reserve: Decimal256,
    pub base_asset_reserve: Decimal256,
    pub timestamp: Timestamp,
    pub block_height: u64,
}

pub fn read_reserve_snapshot(storage: &dyn Storage, height: u64) -> StdResult<ReserveSnapshot> {
    bucket_read(storage, KEY_RESERVE_SNAPSHOT).load(&height.to_be_bytes())
}

pub fn store_reserve_snapshot(
    storage: &mut dyn Storage,
    reserve_snapshot: &ReserveSnapshot,
) -> StdResult<()> {
    increment_reserve_snapshot_counter(storage)?;

    let height = read_reserve_snapshot_counter(storage)?;

    bucket(storage, KEY_RESERVE_SNAPSHOT).save(&height.to_be_bytes(), reserve_snapshot)?;

    Ok(())
}

pub fn read_reserve_snapshot_counter(storage: &dyn Storage) -> StdResult<u64> {
    Ok(singleton_read(storage, KEY_RESERVE_SNAPSHOT_COUNTER)
        .may_load()?
        .unwrap_or_default())
}

pub fn increment_reserve_snapshot_counter(storage: &mut dyn Storage) -> StdResult<()> {
    let val = read_reserve_snapshot_counter(storage)? + 1;

    singleton(storage, KEY_RESERVE_SNAPSHOT_COUNTER).save(&val)
}
