use cosmwasm_std::{
    Binary, StdResult, StdError
};
use cw_storage_plus::IntKey;

/// Tries to encode the length of a given namespace as a 2 byte big endian encoded integer
/// - Modified from [cw-storage-plus/helpers/L84](https://github.com/CosmWasm/cw-storage-plus/blob/69300779519d8ba956fb53725e44e2b59c317b1c/src/helpers.rs#L84)
fn encode_length(namespace: &[u8]) -> StdResult<[u8; 2]> {
    // Cannot return over [255, 255]
    if namespace.len() > 0xFFFF {
        return Err(StdError::GenericErr { msg: "Namespace length cannot be > 0xFFFF".to_string()});
    }

    let length_bytes = (namespace.len() as u32).to_be_bytes();
    // Return third and fourth byte
    // First 2 are always zero as we checked for over 0xFFFF above
    Ok([length_bytes[2], length_bytes[3]])
}


pub trait ToRawKey: Sized + Clone {
    fn to_raw_map_key(&self, namespace: &str) -> StdResult<Binary>;
}

impl ToRawKey for String {
    fn to_raw_map_key(&self, namespace: &str) -> StdResult<Binary> {
        if self.is_empty() {
            return Err(StdError::GenericErr { msg: "Primary Key length cannot be 0".to_string() });
        }
    
        if namespace.is_empty() {
            return Err(StdError::GenericErr { msg: "Namespace length cannot be 0".to_string() });
        }
    
        let namespace_bytes = namespace.as_bytes();
        let encode_len_namespace = encode_length(namespace_bytes)?;
        let pk_bytes = self.as_bytes();
    
        // [len(namespace) | namespace_bytes | pk_bytes]
        let key: Vec<u8> = {
            // Panics if 2 + nmsp.len() + pk.len() > isize::MAX
            let mut mkey: Vec<u8> = Vec::with_capacity(2 + namespace_bytes.len() + pk_bytes.len());
            mkey.extend_from_slice(encode_len_namespace.as_slice());
            mkey.extend_from_slice(namespace_bytes);
            mkey.extend_from_slice(pk_bytes);
            mkey
        };
    
        let bin_key = Binary::from(key);
    
        Ok(bin_key)
    }
}

macro_rules! to_raw_key_uint {
    (for $($t:ty),+) => {
        $(impl ToRawKey for $t {
            fn to_raw_map_key(&self, namespace: &str) -> StdResult<Binary> {
                if namespace.is_empty() {
                    return Err(StdError::GenericErr { msg: "Namespace length cannot be 0".to_string() });
                }
            
                let namespace_bytes = namespace.as_bytes();
                let encode_len_namespace = encode_length(namespace_bytes)?;
                let pk_bytes = self.to_cw_bytes();
            
                // [len(namespace) | namespace_bytes | pk_bytes]
                let key: Vec<u8> = {
                    // Panics if 2 + nmsp.len() + pk.len() > isize::MAX
                    let mut mkey: Vec<u8> = Vec::with_capacity(2 + namespace_bytes.len() + pk_bytes.len());
                    mkey.extend_from_slice(encode_len_namespace.as_slice());
                    mkey.extend_from_slice(namespace_bytes);
                    mkey.extend_from_slice(&pk_bytes);
                    mkey
                };
            
                let bin_key = Binary::from(key);
            
                Ok(bin_key)
            }
        })*
    }
}

to_raw_key_uint!(for u8, u16, u32, u64, u128);