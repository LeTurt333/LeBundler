#[cfg(not(feature = "library"))]
use crate::{
    msg::*, error::*
};
use cosmwasm_std::{
    entry_point, to_binary, DepsMut, Env,
    MessageInfo, Response, StdResult, Binary, Deps, QueryRequest, WasmQuery,
};
use cw2::set_contract_version;
use serde::{de::DeserializeOwned, Serialize};
use cw721::NftInfoResponse;

const CONTRACT_NAME: &str = "crates.io:cw721-query-bundler";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("Instantiate", "qbundler")
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query<T: Serialize + DeserializeOwned + Clone>(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::BundleQuery { token_ids, contract } => bundle_query::<T>(deps, env, token_ids, contract)
    }
}

fn bundle_query<T: Serialize + DeserializeOwned + Clone>(
    deps: Deps,
    _env: Env,
    token_ids: Vec<String>,
    contract: String
) -> StdResult<Binary> {

    let _valid = deps.api.addr_validate(&contract)?;

    // if token_ids.len() > 5 {
    //     return Err(ContractError::GenericError("Max query 5 token ids".to_string()));
    // }

    let mut res: Vec<(String, NftInfoResponse<T>)> = vec![];

    for id in token_ids.iter() {
        let resp: NftInfoResponse<T> = deps
            .querier
            .query(&QueryRequest::Wasm(
                WasmQuery::Smart {
                    contract_addr: contract.clone(),
                    msg: to_binary(&cw721::Cw721QueryMsg::NftInfo { token_id: id.clone() })?
                }
            ))?;
        res.push((id.to_string(), resp));
    }

    to_binary(&res)
}



#[cfg(test)]
mod tests {

    #[test]
    fn test1() {
        let a = true;
        assert_eq!(a, true);
    }
}

