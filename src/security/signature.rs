use super::{SecurityVerifier, VerificationContext};
use alloy_primitives::{keccak256, Address, Bytes};
use async_trait::async_trait;
use gadget_sdk::Error;
use k256::{
    ecdsa::{RecoveryId, Signature, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
};

/// ECDSA signature verification implementation
pub struct SignatureVerifier {
    required_signers: Vec<Address>,
    threshold: usize,
}

impl SignatureVerifier {
    pub fn new(required_signers: Vec<Address>, threshold: usize) -> Self {
        Self {
            required_signers,
            threshold,
        }
    }

    fn recover_signer(
        message_hash: [u8; 32],
        signature: &[u8],
        recovery_id: u8,
    ) -> Result<Address, Error> {
        let sig = Signature::from_slice(signature)
            .map_err(|e| Error::Client(format!("Invalid signature: {}", e)))?;

        let recovery_id = RecoveryId::from_byte(recovery_id)
            .ok_or_else(|| Error::Client("Invalid recovery ID".into()))?;

        let verifying_key = VerifyingKey::recover_from_prehash(&message_hash, &sig, recovery_id)
            .map_err(|e| Error::Client(format!("Signature recovery failed: {}", e)))?;

        let public_key = verifying_key.to_encoded_point(false);
        let public_key_bytes = public_key.as_bytes();
        let address = keccak256(&public_key_bytes[1..]);

        Ok(Address::from_slice(&address[12..]))
    }
}

#[async_trait]
impl SecurityVerifier for SignatureVerifier {
    async fn verify(&self, data: &[u8], context: &VerificationContext) -> Result<bool, Error> {
        let message_hash = keccak256(data);
        let mut valid_signatures = 0;

        // Extract signatures from context.extra_data
        // Format: [signature (65 bytes) || recovery_id (1 byte)]*
        let signatures = context.extra_data.chunks_exact(66);

        for signature_data in signatures {
            let signature = &signature_data[..65];
            let recovery_id = signature_data[65];

            let signer = Self::recover_signer(message_hash, signature, recovery_id)?;

            if self.required_signers.contains(&signer) {
                valid_signatures += 1;
            }
        }

        Ok(valid_signatures >= self.threshold)
    }
}
