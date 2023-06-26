#[cfg(not(feature = "library"))]
use crate::{
    error::{ContractError}, 
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query::{
        cw721_bundle_query_ids, cw721_bundle_query_raw, cw721_bundle_query_smart,
        cw20_bundle_query_accounts, cw20_balances_bundle_query_raw, cw20_balances_bundle_query_smart
    },
};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult
};
use cw2::set_contract_version;

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
        QueryMsg::Cw721BundleQuerySmart {
            token_ids,
            contract
        } => to_binary(&cw721_bundle_query_smart(deps, token_ids, contract)?),
        QueryMsg::Cw721BundleQueryRaw { 
            token_ids, 
            contract 
        } => to_binary(&cw721_bundle_query_raw(deps, token_ids, contract)?),
        QueryMsg::Cw721BundleQueryIds {
            loop_limit,
            contract,
            start_after
        } => to_binary(&cw721_bundle_query_ids(deps, loop_limit, contract, start_after)?),
        QueryMsg::Cw20BundleQuerySmart { 
            accounts, 
            contract 
        } => to_binary(&cw20_balances_bundle_query_smart(deps, accounts, contract)?),
        QueryMsg::Cw20BundleQueryRaw { 
            accounts, 
            contract 
        } => to_binary(&cw20_balances_bundle_query_raw(deps, accounts, contract)?),
        QueryMsg::Cw20BundleQueryAccounts { 
            loop_limit, 
            contract, 
            start_after 
        } => to_binary(&cw20_bundle_query_accounts(deps, loop_limit, contract, start_after)?)
    }
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
