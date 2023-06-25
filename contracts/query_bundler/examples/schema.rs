use cosmwasm_schema::write_api;
use query_bundler::msg::{InstantiateMsg, QueryMsg, ExecuteMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
