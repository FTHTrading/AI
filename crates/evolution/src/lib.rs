// Evolution — Trait Mutation, Replication, Natural Selection & Speciation
//
// Agents evolve through environmental pressure: mutation rates adapt to task
// difficulty, high-fitness agents replicate, low-fitness agents enter stasis.
// Horizontal gene transfer allows sharing successful code modules for ATP.

pub mod mutation;
pub mod selection;
pub mod gene_transfer;
pub mod errors;

pub use mutation::{MutationEngine, MutationEvent};
pub use selection::{SelectionEngine, SelectionOutcome};
pub use gene_transfer::{GeneModule, GeneTransferOffer};
pub use errors::EvolutionError;
