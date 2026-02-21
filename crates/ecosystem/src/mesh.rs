use std::collections::{HashMap, VecDeque};

use genesis_dna::AgentID;

use crate::errors::EcosystemError;
use crate::messages::Message;
use crate::registry::{AgentRegistry, AgentStatus};

/// Maximum message queue depth per agent.
const MAX_QUEUE_DEPTH: usize = 1000;

use serde::{Serialize, Deserialize};

/// The ecosystem mesh — manages message routing between agents.
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemMesh {
    /// Agent registry.
    pub registry: AgentRegistry,
    /// Per-agent message inboxes.
    inboxes: HashMap<AgentID, VecDeque<Message>>,
    /// Global message log (for auditing).
    message_log: Vec<Message>,
    /// Total messages processed.
    pub total_messages: u64,
}

impl EcosystemMesh {
    pub fn new() -> Self {
        Self {
            registry: AgentRegistry::new(),
            inboxes: HashMap::new(),
            message_log: Vec::new(),
            total_messages: 0,
        }
    }

    /// Initialize an inbox for a newly registered agent.
    pub fn init_inbox(&mut self, agent_id: AgentID) {
        self.inboxes.entry(agent_id).or_insert_with(VecDeque::new);
    }

    /// Send a direct message to a specific agent.
    pub fn send_direct(&mut self, message: Message) -> Result<(), EcosystemError> {
        let to = message.to.ok_or_else(|| EcosystemError::DeliveryFailed {
            recipient: "broadcast".into(),
            reason: "send_direct requires a recipient".into(),
        })?;

        // Check recipient exists and is online
        let recipient = self
            .registry
            .get(&to)
            .ok_or_else(|| EcosystemError::AgentNotRegistered(to.to_string()))?;

        if recipient.status == AgentStatus::Dead {
            return Err(EcosystemError::AgentOffline(to.to_string()));
        }

        // Deliver to inbox
        let inbox = self.inboxes.entry(to).or_insert_with(VecDeque::new);
        if inbox.len() >= MAX_QUEUE_DEPTH {
            return Err(EcosystemError::CapacityExceeded);
        }

        inbox.push_back(message.clone());
        self.message_log.push(message);
        self.total_messages += 1;
        Ok(())
    }

    /// Broadcast a gossip message to all neighbors of the sender.
    pub fn broadcast_gossip(&mut self, message: Message) -> Result<u32, EcosystemError> {
        let sender = self
            .registry
            .get(&message.from)
            .ok_or_else(|| EcosystemError::AgentNotRegistered(message.from.to_string()))?
            .clone();

        let mut delivered = 0u32;

        for &neighbor_id in &sender.neighbors {
            if let Some(neighbor) = self.registry.get(&neighbor_id) {
                if neighbor.status != AgentStatus::Dead {
                    let mut fwd = message.clone();
                    fwd.to = Some(neighbor_id);
                    let inbox = self.inboxes.entry(neighbor_id).or_insert_with(VecDeque::new);
                    if inbox.len() < MAX_QUEUE_DEPTH {
                        inbox.push_back(fwd);
                        delivered += 1;
                    }
                }
            }
        }

        self.message_log.push(message);
        self.total_messages += delivered as u64;
        Ok(delivered)
    }

    /// Retrieve pending messages for an agent (drains the inbox).
    pub fn receive(&mut self, agent_id: &AgentID) -> Vec<Message> {
        self.inboxes
            .get_mut(agent_id)
            .map(|inbox| inbox.drain(..).collect())
            .unwrap_or_default()
    }

    /// Peek at the number of pending messages for an agent.
    pub fn pending_count(&self, agent_id: &AgentID) -> usize {
        self.inboxes
            .get(agent_id)
            .map(|inbox| inbox.len())
            .unwrap_or(0)
    }

    /// Get recent messages from the global log.
    pub fn recent_messages(&self, n: usize) -> &[Message] {
        let start = self.message_log.len().saturating_sub(n);
        &self.message_log[start..]
    }
}

impl Default for EcosystemMesh {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::MessageKind;
    use genesis_dna::AgentDNA;

    fn setup_mesh_with_agents() -> (EcosystemMesh, AgentID, AgentID) {
        let mut mesh = EcosystemMesh::new();
        let dna1 = AgentDNA::from_entropy(&[0x11u8; 64], false).unwrap();
        let dna2 = AgentDNA::from_entropy(&[0x22u8; 64], false).unwrap();

        mesh.registry.register(&dna1, "A", "coding").unwrap();
        mesh.registry.register(&dna2, "B", "coding").unwrap();
        mesh.init_inbox(dna1.id);
        mesh.init_inbox(dna2.id);
        mesh.registry.connect(&dna1.id, &dna2.id).unwrap();

        (mesh, dna1.id, dna2.id)
    }

    #[test]
    fn test_direct_message() {
        let (mut mesh, a, b) = setup_mesh_with_agents();

        let msg = Message::direct(a, b, MessageKind::Request, b"hello".to_vec());
        mesh.send_direct(msg).unwrap();
        assert_eq!(mesh.pending_count(&b), 1);

        let received = mesh.receive(&b);
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].payload, b"hello");
    }

    #[test]
    fn test_gossip_broadcast() {
        let (mut mesh, a, b) = setup_mesh_with_agents();

        let msg = Message::broadcast(a, MessageKind::Gossip, b"news".to_vec(), 3);
        let delivered = mesh.broadcast_gossip(msg).unwrap();
        assert_eq!(delivered, 1); // only one neighbor
        assert_eq!(mesh.pending_count(&b), 1);
    }
}
