use alloy_sol_types::{sol, SolType};
use gadget_sdk::event_listener::evm::contracts::EvmContractEventListener;
use gadget_sdk::{job, load_abi};
use serde::{Deserialize, Serialize};
use ISendLib::Packet;
use std::convert::Infallible;

use alloy_primitives::Bytes;
use std::ops::Deref;

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

#[derive(Debug, Clone)]
struct MyContext;

#[job(
    id = 0,
    params(payload, options),
    result(_),
    event_listener(
        listener = EvmContractEventListener<ILayerZeroEndpointV2::PacketSent>
        instance = ILayerZeroEndpointV2,
        abi = ILAYER_ZERO_ENDPOINT_V2_ABI_STRING,
        pre_processor = convert_event_to_inputs,
    )
)]
pub async fn handle_packet_sent(payload: Packet, options: Bytes, ctx: MyContext) -> Result<u32, Infallible> {
    // Extract necessary fields from the task and options
    let task = payload;
    let options = options;
    Ok(0)
}

async fn convert_event_to_inputs(event: (ILayerZeroEndpointV2::PacketSent, gadget_sdk::alloy_rpc_types::Log)) -> Result<(Packet, Bytes), gadget_sdk::Error> {
    // Extract necessary fields from the event to convert into task and task_index
    let task = event.0.encodedPayload;
    let packet = Packet::abi_decode(&task.to_vec()[..], true).unwrap();
    let options = event.0.options;

    Ok((packet, options))
}
