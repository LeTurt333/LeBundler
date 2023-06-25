#[cfg(not(feature = "library"))]
use crate::{
    error::{ContractError, ErrorToMsg}, 
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query::{BundleReturn},
    encoding::{token_key}
};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, WasmQuery, StdError, ContractResult, Empty
};
use cw2::set_contract_version;
use cw721::{NftInfoResponse, TokensResponse};
use cw721_metadata_onchain::{Extension};

const CONTRACT_NAME: &str = "crates.io:cw721-query-bundler";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type QueryResRaw = BundleReturn<String>;
pub type QueryResSmart = BundleReturn<NftInfoResponse<Extension>>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("Instantiate", "cw721-query-bundler"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::GenericError("Execute disabled".to_string()))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::BundleQuerySmart {
            token_ids,
            contract
        } => to_binary(&bundle_query_smart(deps, token_ids, contract)?),
        QueryMsg::BundleQueryRaw { 
            token_ids, 
            contract 
        } => to_binary(&bundle_query_raw(deps, token_ids, contract)?),
        QueryMsg::BundleQueryIds {
            loop_limit,
            contract,
            start_after
        } => to_binary(&bundle_query_ids(deps, loop_limit, contract, start_after)?)
    }
}

/// Smart Queries the provided contract with `Cw721QueryMsg::NftInfo`
/// - This function requires deserializing successful query responses into a provided type
/// - The type used here is [Extension](https://github.com/CosmWasm/cw-nfts/blob/f8600e6a760ce6ad340ce286262c55f471b2fb70/contracts/cw721-metadata-onchain/src/lib.rs#L31)
///   from the default `cw721-metadata-onchain` contract, but could be replaced with your own type
fn bundle_query_smart(
    deps: Deps,
    token_ids: Vec<String>,
    contract: String
) -> StdResult<Binary> {
    
    let _valid = deps.api.addr_validate(&contract)?;

    let mut res: Vec<(String, QueryResSmart)> = Vec::with_capacity(token_ids.len());

    for id in token_ids.iter() {

        // The entire call will fail if base64 serialization fails for -any- Query Msg
        let msg = to_binary(&cw721::Cw721QueryMsg::NftInfo {
            token_id: id.clone()
        })?;

        // I do not propogate Query Errors, instead handle them individually
        // so that successful Query Responses can still be returned
        let response: StdResult<NftInfoResponse<Extension>> = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.clone(),
            msg
        }));

        match response {
            Err(e) => res.push((id.to_owned(), QueryResSmart::error(e.to_msg()))),
            Ok(val) => res.push((id.to_owned(), QueryResSmart::success(val)))
        }
    }

    to_binary(&res)
}


/// Raw Queries the provided contract and token_id's
fn bundle_query_raw(
    deps: Deps,
    token_ids: Vec<String>,
    contract: String
) -> StdResult<Binary> {

    let _validate = deps.api.addr_validate(contract.as_str())?;

    let mut res: Vec<(String, QueryResRaw)> = Vec::with_capacity(token_ids.len());

    for id in token_ids.iter() {
        
        // If key encoding fails for any token_id, the entire call fails
        let key = token_key("tokens", id)?;

        let request: QueryRequest<Empty> = WasmQuery::Raw {
            contract_addr: contract.clone(),
            key
        }.into();
    
        // If query request serialization fails for any token_id, the entire call fails
        let raw = cosmwasm_std::to_vec(&request).map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
        })?;

        // Entire call only fails if raw_query results in aSystem Error,
        // If query results in Contract Error or Success, value is returned
        let response = deps.querier.raw_query(&raw)
            .into_result()
            .map_err(|e| StdError::generic_err(format!("System Err: {}", e)))
            .and_then(|val| match val {
                ContractResult::Err(err) => Ok(QueryResRaw::error(err)),
                ContractResult::Ok(val) => {
                    // Handle null byte, which means key did not exist
                    if val.len() == 0 {
                        return Ok(QueryResRaw::error("Nonexistent key"));
                    } else {
                        return Ok(QueryResRaw::success(val.to_base64()));
                    }
                }
            })?;

        res.push((id.to_owned(), response));
    }

    to_binary(&res)
}





/// Loops `loop_limit` times to get token_ids, since max limit is 100 in cw721 base
fn bundle_query_ids(
    deps: Deps,
    // Number of loops to complete
    // 1 loop = 100 token_ids to query
    loop_limit: u32,
    contract: String,
    start_after: Option<String>
) -> StdResult<Binary> {

    let mut token_ids: Vec<String> = vec![];

    let mut count = 0_u32;

    let mut start_after: Option<String> = start_after;

    while count < loop_limit {

        let msg = to_binary(&cw721::Cw721QueryMsg::AllTokens {
            start_after,
            // Default limit is 10, max limit is 100
            limit: Some(100_u32),
        })?;

        let response: TokensResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.clone(),
            msg
        }))?;
        
        // Update start_after for next query
        start_after = response.tokens.last().cloned();

        // If response length is less than 100, set count to loop_limit to stop firing queries
        if response.tokens.len() < 100 {
            count = loop_limit;
        } else {
            count += 1;
        }

        // Add token_ids from this query
        token_ids.extend(response.tokens);
    };

    to_binary(&token_ids)

}


#[cfg(test)]
#[allow(dead_code, unused)]
mod tests {
    use super::*;
    use cw_storage_plus::Map;
    use std::fmt::Display;
    use cosmwasm_std::Binary;

    #[test]
    fn null_byte() {
        let nullbyte = Binary::from(b"");

        assert!(nullbyte.len() == 0);
    }
}
