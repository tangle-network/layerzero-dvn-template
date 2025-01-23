use super::{SecurityVerifier, VerificationContext};
use async_trait::async_trait;
use blueprint_sdk::alloy::primitives::{keccak256, Address};
use blueprint_sdk::error::Error;
use serde::{Deserialize, Serialize};

/// Oracle verification implementation
pub struct OracleVerifier {
    /// Required oracle providers
    providers: Vec<Address>,
    /// Minimum number of matching oracle responses required
    threshold: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct OracleResponse {
    provider: Address,
    timestamp: u64,
    data: Vec<u8>,
    signature: Vec<u8>,
}

impl OracleVerifier {
    pub fn new(providers: Vec<Address>, threshold: usize) -> Self {
        Self {
            providers,
            threshold,
        }
    }

    fn verify_oracle_response(
        &self,
        response: &OracleResponse,
        expected_data: &[u8],
    ) -> Result<bool, Error> {
        // 1. Verify the oracle provider is authorized
        if !self.providers.contains(&response.provider) {
            return Ok(false);
        }

        // 2. Verify the timestamp is recent enough
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| Error::Other(format!("Time error: {}", e)))?
            .as_secs();

        if current_time - response.timestamp > 3600 {
            // Response older than 1 hour
            return Ok(false);
        }

        // 3. Verify the data matches
        if response.data != expected_data {
            return Ok(false);
        }

        // 4. Verify the signature
        // TODO: Implement actual signature verification for oracle responses

        Ok(true)
    }
}

#[async_trait]
impl SecurityVerifier for OracleVerifier {
    async fn verify(&self, data: &[u8], context: &VerificationContext) -> Result<bool, Error> {
        let mut valid_responses = 0;

        // Decode oracle responses from context.extra_data
        let responses: Vec<OracleResponse> = serde_json::from_slice(&context.extra_data)
            .map_err(|e| Error::Other(format!("Failed to decode oracle responses: {}", e)))?;

        for response in responses {
            if self.verify_oracle_response(&response, data)? {
                valid_responses += 1;
            }
        }

        Ok(valid_responses >= self.threshold)
    }
}
