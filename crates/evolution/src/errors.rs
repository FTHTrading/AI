use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvolutionError {
    #[error("Agent not eligible for replication: fitness {fitness} below threshold {threshold}")]
    IneligibleForReplication { fitness: f64, threshold: f64 },

    #[error("Mutation rate out of bounds [0.0, 1.0]: {0}")]
    InvalidMutationRate(f64),

    #[error("Gene module too large: {size} bytes (max {max})")]
    ModuleTooLarge { size: usize, max: usize },

    #[error("Incompatible gene transfer: {0}")]
    IncompatibleTransfer(String),

    #[error("Population too small for selection: {count} (need {min})")]
    PopulationTooSmall { count: usize, min: usize },
}
