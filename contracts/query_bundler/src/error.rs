use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Error Message: {0}")]
    GenericError(String),
}

pub trait ErrorToMsg {
    fn to_msg(self) -> String;
}

impl ErrorToMsg for cosmwasm_std::StdError {
    fn to_msg(self) -> String {
        match self {
            StdError::VerificationErr { .. } => {
                String::from("StdError::VerificationErr")
            },
            StdError::RecoverPubkeyErr { .. } => {
                String::from("StdError::RecoverPubkeyErr")
            },
            StdError::GenericErr { msg, .. } => {
                format!("StdError::GenericErr | Msg: {msg}")
            },
            StdError::InvalidBase64 { msg, .. } => {
                format!("StdError::InvalidBase64 | Msg: {msg}")
            },
            StdError::InvalidDataSize { expected, actual, ..} => {
                format!("StdError::InvalidDataSize | Invalid data size: expected={expected} actual={actual}")
            },
            StdError::InvalidHex { msg, .. } => {
                format!("StdError::InvalidHex | Invalid hex string: {msg}")
            },
            StdError::InvalidUtf8 { msg, .. } => {
                format!("StdError::InvalidUtf8 | Cannot decode UTF8 bytes into string: {msg}")
            },
            StdError::NotFound { kind, .. } => {
                format!("StdError::NotFound | {kind} not found")
            },
            StdError::ParseErr { target_type, msg, .. } => {
                format!("StdError::ParseErr | target_type={target_type} msg={msg}")
            },
            StdError::SerializeErr { source_type, msg, .. } => {
                format!("StdError::SerializeErr | source_type={source_type} msg={msg}")
            },
            StdError::Overflow { .. } => {
                String::from("StdError::Overflow")
            },
            StdError::DivideByZero { .. } => {
                String::from("StdError::DivideByZero")
            },
            StdError::ConversionOverflow { .. } => {
                String::from("StdError::ConversionOverflow {}")
            },
        }
    }
}
