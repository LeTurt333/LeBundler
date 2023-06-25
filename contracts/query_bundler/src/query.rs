use cosmwasm_schema::cw_serde;
use serde::{Serialize, de::DeserializeOwned};

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
