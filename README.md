# Margined Protocol Perpetuals

This repo contains a the Margined Protocol a decentralized perpetual contract protocol for CosmWasm networks.

## Overview

Overview is at [doc](./docs/overview.md)

## Contracts

| Contract       | Reference                                  | Description                                                                                      |
| -------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| Margin Engine  | [doc](./contracts/margined_engine)         | Margin engine that manages users positions and the collateral management                         |
| vAMM           | [doc](./contracts/margined_vamm)           | Virtual AMM enabling users to take perpetual positions                                           |
| Insurance Fund | [doc](./contracts/margined_insurance_fund) | Contract that holds funds to cover shortfalls                                                    |
| Fee Pool       | [doc](./contracts/margined_fee_pool)       | Contract that accrues the fees generated by protocol to be redistributed to `$MRG` token holders |
| Price Feed     | [doc](./contracts/margined_price_feed)     | Integration contract for the data oracles and other data related logic                           |

We migrate contracts directly without using factory pattern, migration proposals are voted on Oraichain.

## Get started

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- https://github.com/oraichain/cosmwasm-tools

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

### Unit / Integration Tests

To run the tests after installing pre-requisites do the following:

```sh
cargo test
```
