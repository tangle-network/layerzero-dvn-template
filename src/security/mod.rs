//! Custom security verification implementations for DVN

mod mpc;
mod oracle;
mod signature;
mod zk_proof;

pub use mpc::MpcVerifier;
pub use oracle::OracleVerifier;
pub use signature::SignatureVerifier;
pub use zk_proof::ZkProofVerifier;

use async_trait::async_trait;
use blueprint_sdk::alloy::primitives::{Address, Bytes};
use blueprint_sdk::error::Error;
use serde::{Deserialize, Serialize};

/// Type of security verification this DVN performs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityType {
    Signature {
        required_signers: Vec<Address>,
        threshold: usize,
    },
    ZkProof {
        verification_key: Bytes,
        proof_system: String,
    },
    Oracle {
        providers: Vec<Address>,
        threshold: usize,
    },
    Mpc {
        participants: Vec<Address>,
        threshold: usize,
    },
}

/// Common trait for all security verifiers
#[async_trait]
pub trait SecurityVerifier: Send + Sync {
    /// Verify the security requirement
    async fn verify(&self, data: &[u8], context: &VerificationContext) -> Result<bool, Error>;
}

/// Context for verification operations
#[derive(Debug, Clone)]
pub struct VerificationContext {
    pub chain_id: u64,
    pub verifier_address: Address,
    pub extra_data: Bytes,
}
