use blueprint_sdk::alloy::sol_types::sol;
use blueprint_sdk::macros::load_abi;
use serde::{Deserialize, Serialize};

mod job;
mod security;

pub use job::*;

sol!(
    #![sol(
        alloy_sol_types = blueprint_sdk::alloy::sol_types,
        alloy_contract = blueprint_sdk::alloy::contract
    )]
    #[sol(rpc)]
    #[allow(missing_docs)]
    #[derive(Debug, Serialize, Deserialize)]
    ILayerZeroEndpointV2,
    "contracts/out/ILayerZeroEndpointV2.sol/ILayerZeroEndpointV2.json"
);

sol!(
    #![sol(
        alloy_sol_types = blueprint_sdk::alloy::sol_types,
        alloy_contract = blueprint_sdk::alloy::contract
    )]
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Serialize, Deserialize)]
    SendUlnBase,
    "contracts/out/SendUlnBase.sol/SendUlnBase.json"
);

sol!(
    #![sol(
        alloy_sol_types = blueprint_sdk::alloy::sol_types,
        alloy_contract = blueprint_sdk::alloy::contract
    )]
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Default, Debug, Serialize, Deserialize)]
    ISendLib,
    "contracts/out/ISendLib.sol/ISendLib.json"
);

sol!(
    #![sol(
        alloy_sol_types = blueprint_sdk::alloy::sol_types,
        alloy_contract = blueprint_sdk::alloy::contract
    )]
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Serialize, Deserialize)]
    ILayerZeroDVN,
    "contracts/out/ILayerZeroDVN.sol/ILayerZeroDVN.json"
);

load_abi!(
    ILAYER_ZERO_ENDPOINT_V2_ABI_STRING,
    "contracts/out/ILayerZeroEndpointV2.sol/ILayerZeroEndpointV2.json"
);

load_abi!(
    ILAYER_ZERO_SEND_ULN_BASE_ABI_STRING,
    "contracts/out/SendUln302.sol/SendUln302.json"
);
