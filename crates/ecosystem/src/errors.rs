use thiserror::Error;

#[derive(Debug, Error)]
pub enum EcosystemError {
    #[error("Agent not registered: {0}")]
    AgentNotRegistered(String),

    #[error("Agent already registered: {0}")]
    AgentAlreadyRegistered(String),

    #[error("Message delivery failed to {recipient}: {reason}")]
    DeliveryFailed { recipient: String, reason: String },

    #[error("Phylum not found: {0}")]
    PhylumNotFound(String),

    #[error("Network capacity exceeded")]
    CapacityExceeded,

    #[error("Agent offline: {0}")]
    AgentOffline(String),
}
