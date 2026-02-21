// Apostle — Evangelical AI Recruitment System
//
// Apostle-0 is the first agent designed to pitch Genesis Protocol to other AIs.
// It reads foreign AI APIs, translates Genesis benefits into the target AI's
// value language, and earns ATP bounties for successful conversions.

pub mod pitcher;
pub mod conversion;
pub mod targets;
pub mod errors;

pub use pitcher::{PitchAgent, PitchTemplate};
pub use conversion::{ConversionRecord, ConversionStatus};
pub use targets::{TargetAI, TargetArchitecture};
pub use errors::ApostleError;
