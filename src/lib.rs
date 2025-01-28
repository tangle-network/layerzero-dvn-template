use blueprint_sdk::alloy::hex;
use blueprint_sdk::alloy::primitives::keccak256;
use blueprint_sdk::alloy::primitives::{Address, Bytes, U256};
use blueprint_sdk::alloy::rpc::types as alloy_rpc_types;
use blueprint_sdk::alloy::sol_types::SolType;
use blueprint_sdk::config::GadgetConfiguration;
use blueprint_sdk::contexts::instrumented_evm_client::EvmInstrumentedClientContext;
use blueprint_sdk::error::Error;
use blueprint_sdk::event_listeners::evm::EvmContractEventListener;
use blueprint_sdk::macros::contexts::{EVMProviderContext, KeystoreContext, TangleClientContext};
use blueprint_sdk::macros::load_abi;
use blueprint_sdk::stores::local_database::LocalDatabase;
use blueprint_sdk::tokio;
use blueprint_sdk::{event_listeners, job};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

mod security;
pub use security::*;
mod bindings;
pub use bindings::*;
use ILayerZeroDVN::AssignJobParam;
use ILayerZeroEndpointV2::PacketSent;
use SendUln302::DVNFeePaid;
use SendUln302::Packet;

pub fn default_data_dir() -> PathBuf {
    const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
    Path::new(MANIFEST_DIR).join("data")
}

/// Stored packet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPacket {
    packet: Packet,
    options: Bytes,
    timestamp: u64,
}

#[derive(Debug, Clone, KeystoreContext, TangleClientContext, EVMProviderContext)]
pub struct DvnContext {
    #[config]
    pub config: GadgetConfiguration,
    #[call_id]
    pub call_id: Option<u64>,
    pub my_address: Address,
    pub store: Arc<LocalDatabase<StoredPacket>>,
    pub required_confirmations: u64,
    pub receive_lib: Address,
    pub price_feed: Address,
    pub default_multiplier_bps: u16,
    // Single security verification configuration
    pub security_type: SecurityType,
}

impl DvnContext {
    pub fn new(env: GadgetConfiguration, data_dir: PathBuf) -> Self {
        Self {
            config: env,
            call_id: None,
            my_address: Default::default(),
            store: Arc::new(LocalDatabase::open(data_dir.join("store.json"))),
            required_confirmations: 0,
            receive_lib: Default::default(),
            price_feed: Default::default(),
            default_multiplier_bps: 0,
            security_type: SecurityType::Mpc {
                participants: vec![],
                threshold: 0,
            },
        }
    }
}

load_abi!(
    ILAYER_ZERO_ENDPOINT_V2_ABI_STRING,
    "contracts/out/ILayerZeroEndpointV2.sol/ILayerZeroEndpointV2.json"
);

// First job: Listen for and store packets
#[job(
    id = 0,
    params(packet, options),
    event_listener(
        listener = EvmContractEventListener<PacketSent>
        instance = ILayerZeroEndpointV2,
        abi = ILAYER_ZERO_ENDPOINT_V2_ABI_STRING,
        pre_processor = convert_packet_event,
    )
)]
pub async fn store_packet(packet: Packet, options: Bytes, ctx: DvnContext) -> Result<(), Error> {
    let stored_packet = StoredPacket {
        packet,
        options,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| Error::Other(e.to_string()))?
            .as_secs(),
    };

    // Store using message_id as key
    let message_id = calculate_message_id(&stored_packet.packet, &stored_packet.options)?;
    ctx.store.set(&*hex::encode(message_id), stored_packet);

    Ok(())
}

load_abi!(
    ILAYER_ZERO_SEND_ULN_302_ABI_STRING,
    "contracts/out/SendUln302.sol/SendUln302.json"
);

// Second job: Process packets when selected as DVN
#[job(
    id = 1,
    params(fee_paid, log),
    event_listener(
        listener = EvmContractEventListener<DVNFeePaid>
        instance = SendUln302,
        abi = ILAYER_ZERO_SEND_ULN_302_ABI_STRING,
        pre_processor = convert_fee_event,
    )
)]
pub async fn process_packet(
    fee_paid: DVNFeePaid,
    log: alloy_rpc_types::Log,
    ctx: DvnContext,
) -> Result<bool, Error> {
    // 1. Check if we're one of the selected DVNs
    let my_address = ctx.my_address;
    let is_required = fee_paid.requiredDVNs.iter().any(|addr| *addr == my_address);
    let is_optional = fee_paid.optionalDVNs.iter().any(|addr| *addr == my_address);

    if !is_required && !is_optional {
        return Ok(false);
    }

    // 2. Get the transaction that emitted this event
    let Some(tx_hash) = log.transaction_hash else {
        return Err(Error::Other("Transaction not found".into()));
    };

    let tx = ctx
        .evm_client()
        .await
        .transaction_by_hash(tx_hash)
        .await
        .unwrap(); // TODO

    Ok(true)
    // // 3. Decode the transaction input to get the AssignJob call data
    // let assign_job = AssignJobCall::abi_decode(&tx.input, true)
    //     .map_err(|e| Error::Other(format!("Failed to decode transaction input: {}", e)))?;
    //
    // // 4. Extract packet parameters from the assign job call
    // let message_id = calculate_message_id_from_params(&assign_job.param)?;
    //
    // // 5. Verify this matches the stored packet
    // let stored_packet: StoredPacket = ctx
    //     .store
    //     .get(&message_id.to_vec())
    //     .ok_or_else(|| Error::Other("Packet not found".into()))?;
    //
    // // 6. Verify the parameters match
    // verify_packet_params(&stored_packet, &assign_job.param)?;
    //
    // // 7. Check if already verified
    // if is_already_verified(&message_id, &ctx).await? {
    //     return Ok(true);
    // }
    //
    // // 8. Wait for required confirmations
    // wait_for_confirmations(stored_packet.packet.dstEid, ctx.required_confirmations).await?;
    //
    // // 9. Perform security verification
    // verify_security(&stored_packet.packet, &stored_packet.options, &ctx).await?;
    //
    // // 10. Call contract to verify on ULN
    // let verification_result = verify_on_destination(
    //     &stored_packet.packet,
    //     &stored_packet.options,
    //     ctx.receive_lib,
    // )
    // .await?;
    //
    // Ok(verification_result)
}

async fn convert_packet_event(
    event: (PacketSent, alloy_rpc_types::Log),
) -> Result<Option<(Packet, Bytes)>, event_listeners::core::Error<event_listeners::evm::error::Error>>
{
    let packet_sent = event.0;
    let packet =
        Packet::abi_decode(&packet_sent.encodedPayload.to_vec()[..], true).map_err(|e| {
            event_listeners::core::Error::Other(format!("Failed to decode packet: {}", e))
        })?;
    Ok(Some((packet, packet_sent.options)))
}

async fn convert_fee_event(
    event: (DVNFeePaid, alloy_rpc_types::Log),
) -> Result<
    Option<(DVNFeePaid, alloy_rpc_types::Log)>,
    event_listeners::core::Error<event_listeners::evm::error::Error>,
> {
    Ok(Some(event))
}

async fn is_already_verified(message_id: &[u8; 32], ctx: &DvnContext) -> Result<bool, Error> {
    // TODO: Call DVN contract's verifiedMessages mapping
    Ok(false)
}

// async fn wait_for_confirmations(dst_eid: u32, required_confirmations: u64) -> Result<(), Error> {
//     let mut current_attempt = 0;
//     let max_attempts = 10;
//     let initial_delay = Duration::from_secs(1);
//
//     loop {
//         // Get current block number
//         let current_block = get_current_block(dst_eid).await?;
//
//         // Check if we have enough confirmations
//         if current_block.confirmations >= required_confirmations {
//             return Ok(());
//         }
//
//         current_attempt += 1;
//         if current_attempt >= max_attempts {
//             return Err(Error::Other("Max confirmation attempts exceeded".into()));
//         }
//
//         // Exponential backoff
//         let delay = initial_delay * 2u32.pow(current_attempt as u32);
//         sleep(delay).await;
//     }
// }

// async fn verify_security(packet: &Packet, options: &Bytes, ctx: &DvnContext) -> Result<(), Error> {
//     let verification_context = VerificationContext {
//         chain_id: ctx.config.chain_id,
//         verifier_address: ctx.receive_lib,
//         extra_data: options.clone(),
//     };
//
//     let data = encode_verification_data(packet)?;
//
//     // Create and use the appropriate verifier based on security type
//     let verified = match &ctx.security_type {
//         SecurityType::Signature {
//             required_signers,
//             threshold,
//         } => {
//             let verifier =
//                 crate::security::SignatureVerifier::new(required_signers.clone(), *threshold);
//             verifier.verify(&data, &verification_context).await?
//         }
//         SecurityType::ZkProof {
//             verification_key,
//             proof_system,
//         } => {
//             let verifier = crate::security::ZkProofVerifier::new(
//                 verification_key.clone(),
//                 proof_system.clone(),
//             );
//             verifier.verify(&data, &verification_context).await?
//         }
//         SecurityType::Oracle {
//             providers,
//             threshold,
//         } => {
//             let verifier = crate::security::OracleVerifier::new(providers.clone(), *threshold);
//             verifier.verify(&data, &verification_context).await?
//         }
//         SecurityType::Mpc {
//             participants,
//             threshold,
//         } => {
//             let verifier = crate::security::MpcVerifier::new(participants.clone(), *threshold);
//             verifier.verify(&data, &verification_context).await?
//         }
//     };
//
//     if !verified {
//         return Err(Error::Other("Security verification failed".into()));
//     }
//
//     Ok(())
// }

// async fn verify_on_destination(
//     packet: &Packet,
//     options: &Bytes,
//     receive_lib: Address,
// ) -> Result<bool, Error> {
//     let message_id = calculate_message_id(packet, options)?;
//     let encoded_message = encode_verification_message(packet, receive_lib)?;
//
//     // Call the DVN contract's verifyMessageHash function
//     let result = call_verify_message_hash(message_id, encoded_message).await?;
//
//     Ok(result)
// }

fn calculate_message_id(packet: &Packet, options: &Bytes) -> Result<[u8; 32], Error> {
    // Encode packet header
    let packet_header = encode_packet_header(packet)?;

    // Calculate payload hash
    let payload_hash = keccak256(&packet.message);

    // Calculate message ID: keccak256(abi.encodePacked(packet_header, payload_hash))
    let mut message_data = Vec::with_capacity(packet_header.len() + 32);
    message_data.extend_from_slice(&packet_header);
    message_data.extend_from_slice(&*payload_hash);

    Ok(*keccak256(&message_data))
}

fn encode_packet_header(packet: &Packet) -> Result<Bytes, Error> {
    // Encode according to LayerZero format:
    // PACKET_VERSION (uint8)
    // nonce (uint64)
    // srcEid (uint32)
    // sender (bytes32)
    // dstEid (uint32)
    // receiver (bytes32)
    let mut data = Vec::with_capacity(1 + 8 + 4 + 32 + 4 + 32);

    data.push(1u8); // PACKET_VERSION
    data.extend_from_slice(&packet.nonce.to_be_bytes());
    data.extend_from_slice(&packet.srcEid.to_be_bytes());
    data.extend_from_slice(&packet.sender.to_vec());
    data.extend_from_slice(&packet.dstEid.to_be_bytes());
    data.extend_from_slice(&*packet.receiver);

    Ok(data.into())
}

fn verify_packet_params(
    stored_packet: &StoredPacket,
    assign_params: &AssignJobParam,
) -> Result<(), Error> {
    // Verify that the stored packet matches the parameters from the assign job
    if stored_packet.packet.dstEid != assign_params.dstEid {
        return Err(Error::Other("Destination EID mismatch".into()));
    }

    let calculated_hash = keccak256(&stored_packet.packet.message);
    if calculated_hash != assign_params.payloadHash {
        return Err(Error::Other("Payload hash mismatch".into()));
    }

    Ok(())
}

// fn calculate_message_id_from_params(params: &AssignJobParam) -> Result<[u8; 32], Error> {
//     // We need to construct a packet header from the params and combine it with the payload hash
//     // Format matches encode_packet_header:
//     // PACKET_VERSION (uint8)
//     // nonce (uint64)
//     // srcEid (uint32)
//     // sender (bytes32)
//     // dstEid (uint32)
//     // receiver (bytes32)
//     let mut header_data = Vec::with_capacity(1 + 8 + 4 + 32 + 4 + 32);
//
//     header_data.push(1u8); // PACKET_VERSION
//     header_data.extend_from_slice(&params.nonce.to_be_bytes());
//     header_data.extend_from_slice(&params.srcEid.to_be_bytes());
//     header_data.extend_from_slice(&params.sender.to_vec());
//     header_data.extend_from_slice(&params.dstEid.to_be_bytes());
//     header_data.extend_from_slice(&params.receiver);
//
//     // Combine header with payload hash
//     let mut message_data = Vec::with_capacity(header_data.len() + 32);
//     message_data.extend_from_slice(&header_data);
//     message_data.extend_from_slice(&params.payloadHash);
//
//     Ok(*keccak256(&message_data))
// }
