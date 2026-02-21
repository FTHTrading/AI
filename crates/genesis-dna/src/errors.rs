use thiserror::Error;

#[derive(Debug, Error)]
pub enum DnaError {
    #[error("Insufficient entropy for genome generation: need {need} bytes, got {got}")]
    InsufficientEntropy { need: usize, got: usize },

    #[error("Invalid genesis hash length: expected 32 bytes, got {0}")]
    InvalidHashLength(usize),

    #[error("Trait value out of range [0.0, 1.0]: {name} = {value}")]
    TraitOutOfRange { name: String, value: f64 },

    #[error("Lineage depth exceeds maximum ({max}): {actual}")]
    LineageTooDeep { max: usize, actual: usize },

    #[error("Duplicate agent ID in lineage: {0}")]
    DuplicateAncestor(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
