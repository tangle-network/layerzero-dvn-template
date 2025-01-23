use alloy_primitives::keccak256;
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::sol;
use alloy_sol_types::SolType;
use gadget_sdk::{
    config::StdGadgetConfiguration,
    ctx::{KeystoreContext, TangleClientContext},
    event_listener::evm::contracts::EvmContractEventListener,
    job, Error,
};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use tokio::time::{sleep, Duration};

use crate::{
    security::{SecurityType, SecurityVerifier, VerificationContext},
    ILayerZeroEndpointV2::{self, PacketSent},
    ISendLib::Packet,
    ILAYER_ZERO_ENDPOINT_V2_ABI_STRING,
};

#[derive(Debug, Clone, KeystoreContext, TangleClientContext)]
pub struct DvnContext {
    #[config]
    pub config: StdGadgetConfiguration,
    #[call_id]
    pub call_id: Option<u64>,
    pub required_confirmations: u64,
    pub receive_lib: Address,
    pub price_feed: Address,
    pub default_multiplier_bps: u16,
    // Single security verification configuration
    pub security_type: SecurityType,
}

#[job(
    id = 0,
    params(packet, options),
    result(verification_result),
    event_listener(
        listener = EvmContractEventListener<ILayerZeroEndpointV2::PacketSent>
        instance = ILayerZeroEndpointV2,
        abi = ILAYER_ZERO_ENDPOINT_V2_ABI_STRING,
        pre_processor = convert_event_to_inputs,
    )
)]
pub async fn verify_packet(packet: Packet, options: Bytes, ctx: DvnContext) -> Result<bool, Error> {
    // 1. Extract verification parameters from packet
    let message_id = calculate_message_id(&packet, &options)?;

    // 2. Check if already verified
    if is_already_verified(&message_id, &ctx).await? {
        return Ok(true);
    }

    // 3. Wait for required confirmations
    wait_for_confirmations(packet.dstEid, ctx.required_confirmations).await?;

    // 4. Perform security verification based on type
    verify_security(&packet, &options, &ctx).await?;

    // 5. Call contract to verify on ULN
    let verification_result = verify_on_destination(&packet, &options, ctx.receive_lib).await?;

    Ok(verification_result)
}

async fn convert_event_to_inputs(
    event: (PacketSent, gadget_sdk::alloy_rpc_types::Log),
) -> Result<(Packet, Bytes), Error> {
    let packet_sent = event.0;

    // Decode the packet from the encoded payload
    let packet = Packet::abi_decode(&packet_sent.encodedPayload.to_vec()[..], true)
        .map_err(|e| Error::Client(format!("Failed to decode packet: {}", e)))?;

    Ok((packet, packet_sent.options))
}

async fn is_already_verified(message_id: &[u8; 32], ctx: &DvnContext) -> Result<bool, Error> {
    // TODO: Call DVN contract's verifiedMessages mapping
    Ok(false)
}

async fn wait_for_confirmations(dst_eid: u32, required_confirmations: u64) -> Result<(), Error> {
    let mut current_attempt = 0;
    let max_attempts = 10;
    let initial_delay = Duration::from_secs(1);

    loop {
        // Get current block number
        let current_block = get_current_block(dst_eid).await?;

        // Check if we have enough confirmations
        if current_block.confirmations >= required_confirmations {
            return Ok(());
        }

        current_attempt += 1;
        if current_attempt >= max_attempts {
            return Err(Error::Client("Max confirmation attempts exceeded".into()));
        }

        // Exponential backoff
        let delay = initial_delay * 2u32.pow(current_attempt as u32);
        sleep(delay).await;
    }
}

async fn verify_security(packet: &Packet, options: &Bytes, ctx: &DvnContext) -> Result<(), Error> {
    let verification_context = VerificationContext {
        chain_id: ctx.config.chain_id,
        verifier_address: ctx.receive_lib,
        extra_data: options.clone(),
    };

    let data = encode_verification_data(packet)?;

    // Create and use the appropriate verifier based on security type
    let verified = match &ctx.security_type {
        SecurityType::Signature {
            required_signers,
            threshold,
        } => {
            let verifier =
                crate::security::SignatureVerifier::new(required_signers.clone(), *threshold);
            verifier.verify(&data, &verification_context).await?
        }
        SecurityType::ZkProof {
            verification_key,
            proof_system,
        } => {
            let verifier = crate::security::ZkProofVerifier::new(
                verification_key.clone(),
                proof_system.clone(),
            );
            verifier.verify(&data, &verification_context).await?
        }
        SecurityType::Oracle {
            providers,
            threshold,
        } => {
            let verifier = crate::security::OracleVerifier::new(providers.clone(), *threshold);
            verifier.verify(&data, &verification_context).await?
        }
        SecurityType::Mpc {
            participants,
            threshold,
        } => {
            let verifier = crate::security::MpcVerifier::new(participants.clone(), *threshold);
            verifier.verify(&data, &verification_context).await?
        }
    };

    if !verified {
        return Err(Error::Client("Security verification failed".into()));
    }

    Ok(())
}

async fn verify_on_destination(
    packet: &Packet,
    options: &Bytes,
    receive_lib: Address,
) -> Result<bool, Error> {
    let message_id = calculate_message_id(packet, options)?;
    let encoded_message = encode_verification_message(packet, receive_lib)?;

    // Call the DVN contract's verifyMessageHash function
    let result = call_verify_message_hash(message_id, encoded_message).await?;

    Ok(result)
}

fn calculate_message_id(packet: &Packet, options: &Bytes) -> Result<[u8; 32], Error> {
    // Encode packet header
    let packet_header = encode_packet_header(packet)?;

    // Calculate payload hash
    let payload_hash = keccak256(&packet.message);

    // Calculate message ID: keccak256(abi.encodePacked(packet_header, payload_hash))
    let mut message_data = Vec::with_capacity(packet_header.len() + 32);
    message_data.extend_from_slice(&packet_header);
    message_data.extend_from_slice(&payload_hash);

    Ok(keccak256(&message_data))
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
    data.extend_from_slice(&packet.sender.to_fixed_bytes());
    data.extend_from_slice(&packet.dstEid.to_be_bytes());
    data.extend_from_slice(&packet.receiver);

    Ok(data.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_packet_verification() {
        // TODO: Add tests for:
        // 1. Message ID calculation
        // 2. Packet verification flow
        // 3. Custom security checks
    }
}
