# LeBundler

A tooling contract to bundle `CW721` and `CW20` queries, as well as a basic example of using CosmWasm Raw Queries to save on gas

# The Problem

### Getting NFT information from a CW721 contract takes a lot of network requests, constrained by query gas limits 

</br>

Typically, when thinking about `cw721-base` (or any of it's variants), it will take you quite a few network requests (ie RPC calls) to get information about NFTs. First, you have to query the contract and get `Token IDs`, but you can only retrieve 100 at a time. Then, for each `Token ID`, you'll need an additional RPC call to get information like the NFT's metadata. The number of requests it would take to get **all** NFTs + their information can be depicted as:

```
let num_nfts = total number of NFTs;
let rem = num_nfts % 100;
let token_id_requests = match rem {
    0 => num_nfts / 100,
    _ => num_nfts / 100 + 1
}
let total_requests = token_id_requests + num_nfts
```

So, given a colleciton with 10,000 NFTs, you'd be looking at 10,100 requests. That's a lot!

</br>

# Solving the problem

## `1: QueryMsg::BundleQueryIds`

```
BundleQueryIds {
    loop_limit: u32,
    contract: String,
    start_after: Option<String>
}
```

This method will simply query the `contract` up to `loop_limit` times, and return the token IDs. If any of the queries return less than 100 results, the queries stop executing and the result is returned.

In manual tests, this method succeeded with a `loop_limit` of up to 7 (8+ hit query gas limits)

> **Note:** Manual tests were done on Juno testnet using an NFT contract with integer-incremented String token IDs - `"1", "2", "3"...`

If we take our equation from before, and give ourselves a bit of a safety margin at a `loop_limit` of 5, the new equation looks like this:

```
let num_nfts = total number of NFTs;
let rem = num_nfts % 100;
let token_id_requests = match rem {
    0 => (num_nfts / 5) / 100,
    _ => (num_nfts / 5) / 100 + 1
}
let total_requests = token_id_requests + num_nfts
```

So, given a collection with 10,000 NFTs, we'd be looking at 10,020 requests

| Method                            | Requests |
|-----------------------------------|----------|
| Without Bundler                   | 10,100   |
| BundleQueryIds                    | 10,020   |

**Reduction from original: 0.79%**

Any reduction is a good reduction, but we can do better

---

## `2: QueryMsg::BundleQuerySmart`

```
BundleQuerySmart {
    token_ids: Vec<String>,
    contract: String
}
```

This method iterates through `token_ids` and broadcasts a `TokenInfo` query to the `contract` for each. As you can tell by the name, this method uses CosmWasm's `SmartQuery` like you are probably used to

> **Note:** Because this method uses `SmartQuery`, you **must** know the type of `Extension` the CW721 contract is using if you want it to be included in the response. If you don't care about getting the Extension, you can cast the query results to `StdResult<NftInfoResponse<Empty>>` for it to be left out of the response

In manual tests, this method succeeded with up to about 40 token_ids before hitting query gas limits.

> **Note:** The NFT contract used for testing included the default `Extension` from `cw721-metadata-onchain` and 8 Trait values per NFT.

If we take our equation from before, and give ourselves a 10% safety margin at 36 `token_ids` per query, the new equation looks like this:


```
let num_nfts = total number of NFTs;
let rem = num_nfts % 100;
let token_id_requests = match rem {
    0 => (num_nfts / 5) / 100,
    _ => (num_nfts / 5) / 100 + 1
}

let rem_two = num_nfts % 36;
let nft_info_requests = match rem_two {
    0 => num_nfts / 36,
    _ => num_nfts / 36 + 1
}

let total_requests = token_id_requests + nft_info_requests;
```

Given a collection with 10,000 NFTs, using both the `BundleQueryIds` and `BundleQuerySmart` methods, we'd be looking at 298 requests


| Method                            | Requests |
|-----------------------------------|----------|
| Without Bundler                   | 10,100   |
| BundleQueryIds                    | 10,020   |
| BundleQueryIds + BundleQuerySmart | 298      |

**Reduction from previous: 97.03%**

**Reduction from original: 97.05%**

Now we're talking! Yet I yearn for more...

---

## `3: QueryMsg::BundleQueryRaw`

```
BundleQueryRaw {
    token_ids: Vec<String>,
    contract: String
}
```

This method is essentially the same as `BundleQuerySmart`, except it uses `RawQuery` instead of `SmartQuery`

In a nutshell, Raw Queries look up storage entries *directly* by key. You might be thinking "Aren't smart queries also looking up entries by key?", to which you'd be right, except that the *key* being referred to is a bit different

**It's important to note that Smart Contract storage in CosmWasm is essentially a *wrapper* around on-chain storage**

When you use a Smart Query, you only need to provide the wrapper-abstracted key (ie. the Key to a `cw_storage_plus::Map`), and the CosmWasm engine handles the on-chain storage look up for you

When using a Raw Query, you need to provide the actual underlying on-chain storage key (or, a less abstracted version of it at least), which requires more work from you, but less work from CosmWasm. You can get an idea of how these keys are constructed in `/query_bundler/src/encoding.rs`

The result is that Raw Queries require much less gas to complete than Smart Queries do. In regards to this tool, this means fewer network calls to get the NFT info we need. 

**TLDR:** Raw Queries are a lot less computationally expensive than Smart Queries. But...how much?

In manual tests (using the same CW721 contract as before), this method succeeded with up to about 440 token_ids before hitting query gas limits

If we take our equation from before, and give ourselves a 10% safety margin at 400 `token_ids` per query, the new equation looks like this:

```
let num_nfts = total number of NFTs;
let rem = num_nfts % 100;
let token_id_requests = match rem {
    0 => (num_nfts / 5) / 100,
    _ => (num_nfts / 5) / 100 + 1
}

let rem_two = num_nfts % 400;
let nft_info_requests = match rem_two {
    0 => num_nfts / 400,
    _ => num_nfts / 400 + 1
}

let total_requests = token_id_requests + nft_info_requests;
```

Given a collection with 10,000 NFTs, using both the `BundleQueryIds` and `BundleQueryRaw` methods, we'd now be looking at 45 requests

| Method                            | Requests |
|-----------------------------------|----------|
| Without Bundler                   | 10,100   |
| BundleQueryIds                    | 10,020   |
| BundleQueryIds + BundleQuerySmart | 298      |
| BundleQueryIds + BundleQueryRaw   | 45       |

**Reduction from previous: 84.90%**

**Reduction from original: 99.55%**

</br>

![image](https://github.com/LeTurt333/cw721-query-bundler/assets/89463679/e39f5a24-75f0-4420-93c3-32ec410ba3ea)

---

## Follow me on Twitter at [@LeTurt_](https://twitter.com/leturt_)