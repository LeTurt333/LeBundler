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

/// ### Returns the raw map key to be used inside WasmQuery::Raw
/// ---
/// `namespace`: The namespace of the map
/// ```
/// // namespace is "some_map"
/// pub const MYMAP: Map<&str, u32> = Map::new("some_map");
/// ```
/// </br>
/// 
/// `pk`: The primary key of the map entry
/// ```
/// // pk is "first_key"
/// MYMAP.save(deps.storage, "first_key", &1_u32)?;
/// ```
pub fn raw_map_key(
    // cw721-base has namespace of "tokens"
    // cw20-base "Balances" has namespace of "balance"
    namespace: &str,
    pk: &str,
) -> StdResult<Binary> {
    if pk.is_empty() {
        return Err(StdError::GenericErr { msg: "Primary Key length cannot be 0".to_string() });
    }

    if namespace.is_empty() {
        return Err(StdError::GenericErr { msg: "Namespace length cannot be 0".to_string() });
    }

    let namespace_bytes = namespace.as_bytes();
    let encode_len_namespace = encode_length(namespace_bytes)?;
    let pk_bytes = pk.as_bytes();

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

