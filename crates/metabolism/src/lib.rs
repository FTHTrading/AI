// Metabolism — ATP (Agent Transaction Protocol) Economy
//
// Agents earn ATP by solving computational puzzles, optimizing networks,
// or providing unique data. ATP is consumed for computation, replication,
// communication, and storage. Agents must maintain positive ATP to survive.

pub mod atp;
pub mod ledger;
pub mod proof;
pub mod treasury;
pub mod errors;

pub use atp::{AtpBalance, AtpTransaction, TransactionKind};
pub use ledger::MetabolismLedger;
pub use proof::{Solution, ProofKind, SolutionVerdict};
pub use treasury::UnitTreasury;
pub use errors::MetabolismError;
