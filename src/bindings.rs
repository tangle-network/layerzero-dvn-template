use blueprint_sdk::alloy::sol_types::sol;
use blueprint_sdk::macros::load_abi;
use serde::{Deserialize, Serialize};

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
    #[derive(Debug, Serialize, Deserialize)]
    SendUln302,
    "contracts/out/SendUln302.sol/SendUln302.json"
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
