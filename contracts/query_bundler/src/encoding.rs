use cosmwasm_std::{
    Binary, StdResult, StdError
};

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

/// Returns the key to be used inside WasmQuery::Raw
pub fn token_key(
    // cw721-base has namespace of "tokens"
    namespace: &str,
    token_id: &str,
) -> StdResult<Binary> {
    if token_id.is_empty() {
        return Err(StdError::GenericErr { msg: "Token ID length cannot be 0".to_string() });
    }

    if namespace.is_empty() {
        return Err(StdError::GenericErr { msg: "Namespace length cannot be 0".to_string() });
    }

    let namespace_bytes = namespace.as_bytes();
    let encode_len_namespace = encode_length(namespace_bytes)?;
    let token_id_bytes = token_id.as_bytes();

    // [len(namespace) | namespace_bytes | token_id_bytes]
    let key: Vec<u8> = {
        // Panics if 2 + nmsp.len() + token_id.len() > isize::MAX
        let mut mkey: Vec<u8> = Vec::with_capacity(2 + namespace_bytes.len() + token_id_bytes.len());
        mkey.extend_from_slice(encode_len_namespace.as_slice());
        mkey.extend_from_slice(namespace_bytes);
        mkey.extend_from_slice(token_id_bytes);
        mkey
    };

    let bin_key = Binary::from(key);

    Ok(bin_key)
}

