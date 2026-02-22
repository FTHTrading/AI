// Genesis Multiverse — Multi-world evolutionary experiments
//
// Each world is a sovereign civilization:
//   - Its own State Chain (body)
//   - Its own Evolution Chain (mind)
//   - Its own PressureConfig lineage
//   - Its own ecological identity
//
// Operations:
//   spawn  — create a new primordial world with given physics
//   fork   — deep-clone a world at a cryptographic anchor point
//   merge  — transfer evolved pressure learnings between worlds
//   diverge — measure how two worlds' evolutionary paths have separated
//
// Fork is a cryptographically verifiable event: the fork record
// contains the exact State Chain root and Evolution Chain root
// at the branching point. You can prove which world descended
// from which, and where they diverged.
//
// This is not parallel simulation. This is comparative civilization science.

pub mod identity;
pub mod physics;
pub mod engine;
pub mod fork;
pub mod divergence;
pub mod merge;

pub use identity::WorldIdentity;
pub use physics::{WorldPhysics, PhysicsPreset};
pub use engine::{MultiverseEngine, ManagedWorld};
pub use fork::ForkEvent;
pub use divergence::DivergenceReport;
pub use merge::{MergeEvent, MergeStrategy};
