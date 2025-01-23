use alloy_sol_types::sol;
use gadget_sdk::load_abi;
use serde::{Deserialize, Serialize};

mod job;
mod security;

pub use job::*;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Serialize, Deserialize)]
    ILayerZeroEndpointV2,
    "contracts/out/ILayerZeroEndpointV2.sol/ILayerZeroEndpointV2.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Serialize, Deserialize)]
    ISendLib,
    "contracts/out/ISendLib.sol/ISendLib.json"
);

load_abi!(
    ILAYER_ZERO_ENDPOINT_V2_ABI_STRING,
    "contracts/out/ILayerZeroEndpointV2.sol/ILayerZeroEndpointV2.json"
);
