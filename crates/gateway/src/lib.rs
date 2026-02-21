// Gateway — Organism-as-a-Service Layer
//
// Transforms Genesis Protocol from a CLI simulation into a long-running
// service with persistent world state, background survival loop,
// and HTTP API for external interaction.
//
// Shield provides layered defenses: rate limiting, emergency controls,
// security headers, and input validation.

pub mod world;
pub mod persistence;
pub mod runtime;
pub mod server;
pub mod shield;
pub mod moltbot;
