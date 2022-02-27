// Contains queries for external contracts,
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{to_binary, Deps, DepsMut, QueryRequest, StdResult, WasmQuery};
use margined_perp::margined_vamm::{Direction, QueryMsg, StateResponse};

// returns the state of the request vamm
// can be used to calculate the input and outputs
pub fn _query_vamm_state(deps: &DepsMut, address: String) -> StdResult<StateResponse> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: address,
        msg: to_binary(&QueryMsg::State {})?,
    }))
}

// returns the state of the request vamm
// can be used to calculate the input and outputs
pub fn query_vamm_output_price(
    deps: &Deps,
    address: String,
    direction: Direction,
    amount: Decimal256,
) -> StdResult<Decimal256> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: address,
        msg: to_binary(&QueryMsg::OutputPrice { direction, amount })?,
    }))
}
