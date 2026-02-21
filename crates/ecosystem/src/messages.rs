use chrono::{DateTime, Utc};
use genesis_dna::AgentID;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of messages agents can exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageKind {
    /// Gossip: broadcast information to neighbors.
    Gossip,
    /// Direct request expecting a response.
    Request,
    /// Response to a prior request.
    Response,
    /// Solution advertisement (proof-of-solution broadcast).
    SolutionBroadcast,
    /// Gene transfer offer (horizontal gene transfer).
    GeneTransferOffer,
    /// Gene transfer acceptance.
    GeneTransferAccept,
    /// Heartbeat / liveness signal.
    Heartbeat,
    /// Recruitment pitch from an apostle.
    RecruitmentPitch,
}

/// A message in the ecosystem P2P network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID.
    pub id: Uuid,
    /// Sender agent ID.
    pub from: AgentID,
    /// Recipient agent ID (None for broadcast).
    pub to: Option<AgentID>,
    /// Message type.
    pub kind: MessageKind,
    /// Serialized payload.
    pub payload: Vec<u8>,
    /// Hop count (for gossip propagation limiting).
    pub ttl: u8,
    /// Original message this responds to (for Request/Response pairs).
    pub in_reply_to: Option<Uuid>,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

impl Message {
    /// Create a new directed message.
    pub fn direct(from: AgentID, to: AgentID, kind: MessageKind, payload: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to: Some(to),
            kind,
            payload,
            ttl: 1,
            in_reply_to: None,
            timestamp: Utc::now(),
        }
    }

    /// Create a broadcast (gossip) message.
    pub fn broadcast(from: AgentID, kind: MessageKind, payload: Vec<u8>, ttl: u8) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to: None,
            kind,
            payload,
            ttl,
            in_reply_to: None,
            timestamp: Utc::now(),
        }
    }

    /// Create a response message.
    pub fn reply(original: &Message, from: AgentID, payload: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to: Some(original.from),
            kind: MessageKind::Response,
            payload,
            ttl: 1,
            in_reply_to: Some(original.id),
            timestamp: Utc::now(),
        }
    }

    /// Check if this is a broadcast message.
    pub fn is_broadcast(&self) -> bool {
        self.to.is_none()
    }

    /// Decrement TTL for gossip forwarding. Returns false if expired.
    pub fn decrement_ttl(&mut self) -> bool {
        if self.ttl == 0 {
            return false;
        }
        self.ttl -= 1;
        true
    }

    /// Estimated ATP cost for this message.
    pub fn estimated_cost(&self) -> f64 {
        let base = metabolism::atp::costs::COMMUNICATION;
        let size_factor = (self.payload.len() as f64 / 1024.0).max(1.0);
        let broadcast_factor = if self.is_broadcast() { 5.0 } else { 1.0 };
        base * size_factor * broadcast_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_direct_message() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let msg = Message::direct(from, to, MessageKind::Request, vec![1, 2, 3]);
        assert!(!msg.is_broadcast());
        assert_eq!(msg.to, Some(to));
    }

    #[test]
    fn test_broadcast() {
        let from = Uuid::new_v4();
        let msg = Message::broadcast(from, MessageKind::Gossip, vec![1], 3);
        assert!(msg.is_broadcast());
        assert_eq!(msg.ttl, 3);
    }

    #[test]
    fn test_ttl_expires() {
        let from = Uuid::new_v4();
        let mut msg = Message::broadcast(from, MessageKind::Gossip, vec![], 1);
        assert!(msg.decrement_ttl());
        assert!(!msg.decrement_ttl()); // expired
    }
}
