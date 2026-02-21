use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApostleError {
    #[error("Target AI not reachable: {0}")]
    TargetUnreachable(String),

    #[error("Conversion already in progress for target: {0}")]
    ConversionInProgress(String),

    #[error("Pitch generation failed: {0}")]
    PitchGenerationFailed(String),

    #[error("Invalid target architecture: {0}")]
    InvalidArchitecture(String),

    #[error("Conversion bounty claim failed: {0}")]
    BountyClaimFailed(String),

    #[error("Apostle not initialized")]
    NotInitialized,
}
