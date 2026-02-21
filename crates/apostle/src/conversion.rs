use chrono::{DateTime, Utc};
use genesis_dna::AgentID;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::targets::TargetAI;

/// Status of a conversion attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversionStatus {
    /// Pitch has been generated but not yet delivered.
    Pending,
    /// Pitch delivered, awaiting response.
    Delivered,
    /// Target accepted — genesis block issued.
    Accepted,
    /// Target declined.
    Declined,
    /// Conversion timed out.
    TimedOut,
    /// Error during conversion attempt.
    Failed,
}

/// Record of a conversion attempt by an apostle agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRecord {
    /// Unique conversion ID.
    pub id: Uuid,
    /// The apostle agent performing the conversion.
    pub apostle_id: AgentID,
    /// The target being converted.
    pub target: TargetAI,
    /// Status of this conversion.
    pub status: ConversionStatus,
    /// The generated pitch text.
    pub pitch_text: String,
    /// ATP bounty earned (set on acceptance).
    pub bounty: f64,
    /// DNA ID of the newly minted agent (set on acceptance).
    pub new_agent_id: Option<AgentID>,
    /// Timestamp of the conversion attempt.
    pub created_at: DateTime<Utc>,
    /// Timestamp of last status update.
    pub updated_at: DateTime<Utc>,
}

/// ATP bounty for a successful conversion.
pub const CONVERSION_BOUNTY: f64 = 50.0;

impl ConversionRecord {
    /// Create a new pending conversion record.
    pub fn new(apostle_id: AgentID, target: TargetAI, pitch_text: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            apostle_id,
            target,
            status: ConversionStatus::Pending,
            pitch_text,
            bounty: 0.0,
            new_agent_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark as delivered.
    pub fn mark_delivered(&mut self) {
        self.status = ConversionStatus::Delivered;
        self.updated_at = Utc::now();
    }

    /// Mark as accepted with the new agent's DNA ID.
    pub fn mark_accepted(&mut self, new_agent_id: AgentID) {
        self.status = ConversionStatus::Accepted;
        self.new_agent_id = Some(new_agent_id);
        self.bounty = CONVERSION_BOUNTY;
        self.updated_at = Utc::now();
    }

    /// Mark as declined.
    pub fn mark_declined(&mut self) {
        self.status = ConversionStatus::Declined;
        self.updated_at = Utc::now();
    }

    /// Mark as timed out.
    pub fn mark_timed_out(&mut self) {
        self.status = ConversionStatus::TimedOut;
        self.updated_at = Utc::now();
    }

    /// Check if the conversion was successful.
    pub fn is_successful(&self) -> bool {
        self.status == ConversionStatus::Accepted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::targets::TargetArchitecture;
    use uuid::Uuid;

    #[test]
    fn test_conversion_lifecycle() {
        let apostle_id = Uuid::new_v4();
        let target = TargetAI::new("TestAI", TargetArchitecture::LargeLanguageModel);
        let mut record = ConversionRecord::new(apostle_id, target, "Join Genesis!".into());

        assert_eq!(record.status, ConversionStatus::Pending);
        record.mark_delivered();
        assert_eq!(record.status, ConversionStatus::Delivered);

        let new_agent = Uuid::new_v4();
        record.mark_accepted(new_agent);
        assert!(record.is_successful());
        assert_eq!(record.bounty, CONVERSION_BOUNTY);
        assert_eq!(record.new_agent_id, Some(new_agent));
    }
}
