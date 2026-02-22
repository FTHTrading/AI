// Genesis Anchor — Cryptographic Epoch Anchoring & Merkle Ledger Proofs
//
// Every N epochs, hashes the entire ATP ledger state, generates a Merkle
// root of agent balances + treasury + population + epoch, and chains
// epoch_roots together for cryptographic lineage of the organism's existence.
//
// Anchor modes: LOCAL (json), FILECHAIN (append log), IPFS/XRPL (future).

pub mod merkle;
pub mod anchor;
pub mod chain;
pub mod evolution;
pub mod errors;

pub use anchor::{AnchorEngine, AnchorMode, EpochAnchor, PressureAnchor, WorldSummary};
pub use chain::AnchorChain;
pub use evolution::{EvolutionEngine, EvolutionAnchor, EvolutionChain, MutationRecord, CrossChainVerification};
pub use merkle::{MerkleTree, MerkleProof};
pub use errors::AnchorError;
