use cosmwasm_schema::cw_serde;
use cw20::{AllAccountsResponse, BalanceResponse};
use serde::{Serialize, de::DeserializeOwned};
use crate::{
    error::ErrorToMsg, 
    encoding::ToRawKey, msg::IntType
};
use cosmwasm_std::{
    to_binary, Binary, Deps, QueryRequest, Empty,
    StdResult, WasmQuery, StdError, ContractResult
};
use cw721::{NftInfoResponse, TokensResponse};
use cw721_metadata_onchain::Extension;

pub type QueryResRaw = BundleReturn<String>;
pub type Cw721QueryResSmart = BundleReturn<NftInfoResponse<Extension>>;
pub type Cw20QueryResSmart = BundleReturn<BalanceResponse>;

// ------------------------------------------------------------------------
// -------------------------------------------------------- Query Responses
// ------------------------------------------------------------------------
#[cw_serde]
pub enum BundleReturn<T> {
    Success(T),
    Error(String)
}

impl<T> BundleReturn<T>
where
    T: Sized + Serialize + DeserializeOwned
{
    pub fn success(val: T) -> Self {
        Self::Success(val)
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self::Error(msg.into())
    }
}

// ------------------------------------------------------------------------
// ------------------------------------------------------------ Query Logic
// ------------------------------------------------------------------------

// ----------------------------------------------------------------- CW-721

/// Smart Queries the provided contract with `Cw721QueryMsg::NftInfo`
/// - This function requires deserializing successful query responses into a provided type
/// - The type used here is [Extension](https://github.com/CosmWasm/cw-nfts/blob/f8600e6a760ce6ad340ce286262c55f471b2fb70/contracts/cw721-metadata-onchain/src/lib.rs#L31)
///   from the default `cw721-metadata-onchain` contract, but could be replaced with your own type
pub fn cw721_bundle_query_smart(
    deps: Deps,
    token_ids: Vec<String>,
    contract: String
) -> StdResult<Binary> {
    
    let _valid = deps.api.addr_validate(&contract)?;

    let mut res: Vec<(String, Cw721QueryResSmart)> = Vec::with_capacity(token_ids.len());

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
            Err(e) => res.push((id.to_owned(), Cw721QueryResSmart::error(e.to_msg()))),
            Ok(val) => res.push((id.to_owned(), Cw721QueryResSmart::success(val)))
        }
    }

    to_binary(&res)
}


/// Raw Queries the provided contract and token_id's
pub fn cw721_bundle_query_raw(
    deps: Deps,
    token_ids: Vec<String>,
    contract: String
) -> StdResult<Binary> {

    let _validate = deps.api.addr_validate(contract.as_str())?;

    let mut res: Vec<(String, QueryResRaw)> = Vec::with_capacity(token_ids.len());

    for id in token_ids.iter() {
        
        // If key encoding fails for any token_id, the entire call fails
        let key = id.to_raw_map_key("tokens")?;

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
pub fn cw721_bundle_query_ids(
    deps: Deps,
    // Number of loops to complete
    // 1 loop = 100 token_ids to query
    loop_limit: u32,
    max_limit: Option<u32>,
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
            limit: max_limit.or(Some(100u32)),
        })?;

        let response: TokensResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.clone(),
            msg
        }))?;
        
        // Update start_after for next query
        start_after = response.tokens.last().cloned();

        // If response length is less than 100 or max_limit, set count to loop_limit to stop firing queries
        if response.tokens.len() < max_limit.unwrap_or_else(|| 100).try_into().map_err(|_e| StdError::GenericErr { msg: "invalid max limit".to_string() })? {
            count = loop_limit;
        } else {
            count += 1;
        }

        // Add token_ids from this query
        token_ids.extend(response.tokens);
    };

    to_binary(&token_ids)

}


// ----------------------------------------------------------------- CW-20

/// Smart Queries the provided contract with `Cw20QueryMsg::Balance`
pub fn cw20_balances_bundle_query_smart(
    deps: Deps,
    accounts: Vec<String>,
    contract: String
) -> StdResult<Binary> {
    
    let _valid = deps.api.addr_validate(&contract)?;

    // Vec<(account, Cw20QueryResSmart)>
    let mut res: Vec<(String, Cw20QueryResSmart)> = Vec::with_capacity(accounts.len());

    for account in accounts.iter() {

        // The entire call will fail if base64 serialization fails for -any- Query Msg
        let msg = to_binary(&cw20::Cw20QueryMsg::Balance { 
            address: account.clone() 
        })?;

        // I do not propogate Query Errors, instead handle them individually
        // so that successful Query Responses can still be returned
        let response: StdResult<BalanceResponse> = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.clone(),
            msg
        }));

        match response {
            Err(e) => res.push((account.to_owned(), Cw20QueryResSmart::error(e.to_msg()))),
            Ok(val) => res.push((account.to_owned(), Cw20QueryResSmart::success(val)))
        }
    }

    to_binary(&res)
}

/// Raw Queries the provided contract and accounts
pub fn cw20_balances_bundle_query_raw(
    deps: Deps,
    accounts: Vec<String>,
    contract: String
) -> StdResult<Binary> {

    let _validate = deps.api.addr_validate(contract.as_str())?;

    let mut res: Vec<(String, QueryResRaw)> = Vec::with_capacity(accounts.len());

    for account in accounts.iter() {

        // NOTE: Primary Key of cw20-base Balances is &Addr, but I skip
        // address validation of each account to minimize query gas usage
        // You may want to add that here depending on your preference
         
        // If key encoding fails for any account, the entire call fails
        let key = account.to_raw_map_key("balance")?;

        let request: QueryRequest<Empty> = WasmQuery::Raw {
            contract_addr: contract.clone(),
            key
        }.into();
    
        // If query request serialization fails for any account, the entire call fails
        let raw = cosmwasm_std::to_vec(&request).map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
        })?;

        // Entire call only fails if raw_query results in a System Error,
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

        res.push((account.to_owned(), response));
    }

    to_binary(&res)
}

/// Loops up to `loop_limit` times to get accounts that have balances
pub fn cw20_bundle_query_accounts(
    deps: Deps,
    // Number of loops to complete
    // 1 loop = 30 accounts
    loop_limit: u32,
    contract: String,
    start_after: Option<String>
) -> StdResult<Binary> {

    let mut accounts: Vec<String> = vec![];

    let mut count = 0_u32;

    let mut start_after: Option<String> = start_after;

    while count < loop_limit {

        let msg = to_binary(&cw20::Cw20QueryMsg::AllAccounts { 
            start_after, 
            limit: Some(30_u32)
        })?;

        let response: AllAccountsResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.clone(),
            msg
        }))?;

        // Update start_after for next query
        start_after = response.accounts.last().cloned();

        // If response length is less than 30, set count to loop limit to stop firing queries
        if response.accounts.len() < 30 {
            count = loop_limit;
        } else {
            count += 1;
        }

        // Add accounts from this query
        accounts.extend(response.accounts);

    };

    to_binary(&accounts)

}



// ----------------------------------------------------------------- Generic

pub fn generic_string_bundle_query_raw(
    deps: Deps,
    keys: Vec<String>,
    namespace: String,
    contract: String
) -> StdResult<Binary> {

    let _validate = deps.api.addr_validate(contract.as_str())?;

    let mut res: Vec<(String, QueryResRaw)> = Vec::with_capacity(keys.len());

    for k in keys.iter() {

        // If key encoding fails for any token_id, the entire call fails
        let key = k.to_raw_map_key(namespace.as_str())?;

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

        res.push((k.to_owned(), response));
    }

    to_binary(&res)
}



pub fn generic_uint_bundle_query_raw(
    deps: Deps,
    keys: Vec<u64>,
    keytype: IntType,
    namespace: String,
    contract: String
) -> StdResult<Binary> {

    let _validate = deps.api.addr_validate(contract.as_str())?;

    let mut res: Vec<(String, QueryResRaw)> = Vec::with_capacity(keys.len());

    for k in keys.iter() {
        // If key encoding fails for any token_id, the entire call fails
        // The cast is necessary because different int sizes have different
        // byte representations thus produce differnt keys
        let key = match keytype {
            IntType::U8 => u8::try_from(*k)
                .map_err(|_e| StdError::generic_err("u8 key overflow"))?
                .to_raw_map_key(namespace.as_str()),
            IntType::U16 => u16::try_from(*k)
                .map_err(|_e| StdError::generic_err("u16 key overflow"))?
                .to_raw_map_key(namespace.as_str()),
            IntType::U32 => u32::try_from(*k)
                .map_err(|_e| StdError::generic_err("u32 key overflow"))?
                .to_raw_map_key(namespace.as_str()),
            IntType::U64 => k.to_raw_map_key(namespace.as_str()),
            IntType::U128 => (*k as u128).to_raw_map_key(namespace.as_str()),
        }?;

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

        res.push((k.to_string(), response));
    }

    to_binary(&res)
}

