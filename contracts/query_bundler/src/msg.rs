use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Binary)]
    Cw721BundleQuerySmart {
        token_ids: Vec<String>,
        contract: String
    },
    #[returns(Binary)]
    Cw721BundleQueryRaw {
        token_ids: Vec<String>,
        contract: String
    },
    #[returns(Binary)]
    Cw721BundleQueryIds {
        loop_limit: u32,
        contract: String,
        start_after: Option<String>
    },
    #[returns(Binary)]
    Cw20BundleQuerySmart {
        accounts: Vec<String>,
        contract: String
    },
    #[returns(Binary)]
    Cw20BundleQueryRaw {
        accounts: Vec<String>,
        contract: String
    },
    #[returns(Binary)]
    Cw20BundleQueryAccounts {
        loop_limit: u32,
        contract: String,
        start_after: Option<String>
    }
}
