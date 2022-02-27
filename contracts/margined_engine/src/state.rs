use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Addr, Api, DepsMut, StdResult, Storage, Timestamp};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, Singleton,
};
use cw_storage_plus::Item;

use margined_perp::margined_engine::Side;
use margined_perp::margined_vamm::Direction;

use sha3::{Digest, Sha3_256};

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_POSITION: &[u8] = b"position";
pub static KEY_TMP_SWAP: &[u8] = b"tmp-position";
pub const VAMM_LIST: Item<VammList> = Item::new("admin_list");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub eligible_collateral: Addr,
    pub decimals: Decimal256,
    pub initial_margin_ratio: Decimal256,
    pub maintenance_margin_ratio: Decimal256,
    pub liquidation_fee: Decimal256,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VammList {
    pub vamm: Vec<Addr>,
}

impl VammList {
    /// returns true if the address is a registered vamm
    pub fn is_vamm(&self, addr: &str) -> bool {
        self.vamm.iter().any(|a| a.as_ref() == addr)
    }
}

pub fn store_vamm(deps: DepsMut, input: &[String]) -> StdResult<()> {
    let cfg = VammList {
        vamm: map_validate(deps.api, input)?,
    };
    VAMM_LIST.save(deps.storage, &cfg)
}

pub fn read_vamm(storage: &dyn Storage) -> StdResult<VammList> {
    VAMM_LIST.load(storage)
}

pub fn map_validate(api: &dyn Api, input: &[String]) -> StdResult<Vec<Addr>> {
    input.iter().map(|addr| api.addr_validate(addr)).collect()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Position {
    pub vamm: Addr,
    pub trader: Addr,
    pub direction: Direction,
    pub size: Decimal256,
    pub margin: Decimal256,
    pub notional: Decimal256,
    pub premium_fraction: Decimal256,
    pub liquidity_history_index: Decimal256,
    pub timestamp: Timestamp,
}

impl Default for Position {
    fn default() -> Position {
        Position {
            vamm: Addr::unchecked(""),
            trader: Addr::unchecked(""),
            direction: Direction::AddToAmm,
            size: Decimal256::zero(),
            margin: Decimal256::zero(),
            notional: Decimal256::zero(),
            premium_fraction: Decimal256::zero(),
            liquidity_history_index: Decimal256::zero(),
            timestamp: Timestamp::from_seconds(0),
        }
    }
}

fn position_bucket(storage: &mut dyn Storage) -> Bucket<Position> {
    bucket(storage, KEY_POSITION)
}

fn position_bucket_read(storage: &dyn Storage) -> ReadonlyBucket<Position> {
    bucket_read(storage, KEY_POSITION)
}

pub fn store_position(storage: &mut dyn Storage, position: &Position) -> StdResult<()> {
    // hash the vAMM and trader together to get a unique position key
    let mut hasher = Sha3_256::new();

    // write input message
    hasher.update(position.vamm.as_bytes());
    hasher.update(position.trader.as_bytes());

    // read hash digest
    let hash = hasher.finalize();

    position_bucket(storage).save(&hash, position)
}

pub fn read_position(
    storage: &dyn Storage,
    vamm: &Addr,
    trader: &Addr,
) -> StdResult<Option<Position>> {
    // hash the vAMM and trader together to get a unique position key
    let mut hasher = Sha3_256::new();

    // write input message
    hasher.update(vamm.as_bytes());
    hasher.update(trader.as_bytes());

    // read hash digest
    let hash = hasher.finalize();
    position_bucket_read(storage).may_load(&hash)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Swap {
    pub vamm: Addr,
    pub trader: Addr,
    pub side: Side,
    pub quote_asset_amount: Decimal256,
    pub leverage: Decimal256,
    pub open_notional: Decimal256,
}

pub fn store_tmp_swap(storage: &mut dyn Storage, swap: &Swap) -> StdResult<()> {
    singleton(storage, KEY_TMP_SWAP).save(swap)
}

pub fn remove_tmp_swap(storage: &mut dyn Storage) {
    let mut store: Singleton<Swap> = singleton(storage, KEY_TMP_SWAP);
    store.remove()
}

pub fn read_tmp_swap(storage: &dyn Storage) -> StdResult<Option<Swap>> {
    singleton_read(storage, KEY_TMP_SWAP).load()
}
