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
        max_limit: Option<u32>,
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
    },
    #[returns(Binary)]
    GenericStringBundleQueryRaw {
        keys: Vec<String>,
        namespace: String,
        contract: String,
    },
    #[returns(Binary)]
    GenericUIntBundleQueryRaw {
        keys: Vec<u64>,
        keytype: IntType,
        namespace: String,
        contract: String,
    },
}

#[cw_serde]
pub enum IntType {
    U8,
    U16,
    U32,
    U64,
    U128
}