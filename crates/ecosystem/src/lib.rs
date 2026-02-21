// Ecosystem — P2P Mesh Network for Agent Communication
//
// Provides gossip-based message propagation and request/response patterns
// for agent-to-agent communication. Agents pay ATP for bandwidth.

pub mod mesh;
pub mod messages;
pub mod registry;
pub mod publication_gate;
pub mod problem_market;
pub mod telemetry;
pub mod errors;

pub use mesh::EcosystemMesh;
pub use messages::{Message, MessageKind};
pub use registry::{AgentRegistry, AgentStatus, RegisteredAgent};
pub use publication_gate::{PublicationGate, GateDecision};
pub use problem_market::{ProblemMarket, Problem, ProblemCategory, evaluate as evaluate_problem};
pub use telemetry::{UnitStatus, RiskState};
pub use errors::EcosystemError;
