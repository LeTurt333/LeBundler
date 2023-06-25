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
    BundleQuerySmart {
        token_ids: Vec<String>,
        contract: String
    },
    #[returns(Binary)]
    BundleQueryRaw {
        token_ids: Vec<String>,
        contract: String
    },
    #[returns(Binary)]
    BundleQueryIds {
        loop_limit: u32,
        contract: String,
        start_after: Option<String>
    }
}
