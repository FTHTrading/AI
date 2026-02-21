use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetabolismError {
    #[error("Insufficient ATP: required {required}, available {available}")]
    InsufficientAtp { required: f64, available: f64 },

    #[error("Agent not found in ledger: {0}")]
    AgentNotFound(String),

    #[error("Agent is in stasis (starvation): {0}")]
    AgentInStasis(String),

    #[error("Invalid ATP amount: {0} (must be positive)")]
    InvalidAmount(f64),

    #[error("Solution verification failed: {0}")]
    VerificationFailed(String),

    #[error("Duplicate transaction ID: {0}")]
    DuplicateTransaction(String),

    #[error("Metabolic overflow: balance would exceed maximum")]
    Overflow,
}
