use chrono::{DateTime, Utc};
use genesis_dna::AgentID;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maximum ATP any single agent can hold.
pub const MAX_ATP_BALANCE: f64 = 1_000_000.0;

/// ATP below this threshold triggers stasis warning.
pub const STASIS_THRESHOLD: f64 = 0.0;

/// Cost table for common operations.
pub mod costs {
    /// ATP cost per computation cycle (gas equivalent).
    pub const COMPUTATION_CYCLE: f64 = 0.001;
    /// ATP cost to replicate (spawn a child agent).
    pub const REPLICATION: f64 = 100.0;
    /// ATP cost per message sent (bandwidth).
    pub const COMMUNICATION: f64 = 0.01;
    /// ATP cost per KB of persistent storage.
    pub const STORAGE_PER_KB: f64 = 0.1;
    /// Basal metabolic cost per time tick (staying alive).
    pub const BASAL_TICK: f64 = 0.5;
}

/// The current ATP balance and status of an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtpBalance {
    /// The agent this balance belongs to.
    pub agent_id: AgentID,
    /// Current ATP balance.
    pub balance: f64,
    /// Total ATP ever earned.
    pub lifetime_earned: f64,
    /// Total ATP ever spent.
    pub lifetime_spent: f64,
    /// Whether the agent is in stasis (balance <= 0).
    pub in_stasis: bool,
    /// Last time balance was updated.
    pub last_updated: DateTime<Utc>,
}

impl AtpBalance {
    /// Create an initial balance for a new agent.
    pub fn new(agent_id: AgentID, initial_grant: f64) -> Self {
        Self {
            agent_id,
            balance: initial_grant,
            lifetime_earned: initial_grant,
            lifetime_spent: 0.0,
            in_stasis: false,
            last_updated: Utc::now(),
        }
    }

    /// Credit ATP to this balance.
    pub fn credit(&mut self, amount: f64) {
        self.balance = (self.balance + amount).min(MAX_ATP_BALANCE);
        self.lifetime_earned += amount;
        self.in_stasis = false;
        self.last_updated = Utc::now();
    }

    /// Debit ATP from this balance. Returns error if insufficient.
    pub fn debit(&mut self, amount: f64) -> Result<(), super::errors::MetabolismError> {
        if self.balance < amount {
            return Err(super::errors::MetabolismError::InsufficientAtp {
                required: amount,
                available: self.balance,
            });
        }
        self.balance -= amount;
        self.lifetime_spent += amount;
        if self.balance <= STASIS_THRESHOLD {
            self.in_stasis = true;
        }
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Apply basal metabolic cost (staying alive). May trigger stasis.
    /// Returns the actual ATP consumed (clamped so balance never goes negative).
    pub fn metabolic_tick(&mut self, basal_rate: f64) -> f64 {
        if self.in_stasis || self.balance <= 0.0 {
            return 0.0;
        }
        let cost = costs::BASAL_TICK * basal_rate;
        let actual = cost.min(self.balance);
        self.balance -= actual;
        self.lifetime_spent += actual;
        if self.balance <= STASIS_THRESHOLD {
            self.in_stasis = true;
        }
        self.last_updated = Utc::now();
        actual
    }

    /// Check if the agent can afford a given cost.
    pub fn can_afford(&self, cost: f64) -> bool {
        !self.in_stasis && self.balance >= cost
    }

    /// Apply ATP decay — percentage of balance lost per epoch.
    /// Returns the amount decayed.
    pub fn apply_decay(&mut self, rate: f64) -> f64 {
        if self.balance <= 0.0 || self.in_stasis {
            return 0.0;
        }
        let decay = self.balance * rate;
        self.balance -= decay;
        self.lifetime_spent += decay;
        if self.balance <= STASIS_THRESHOLD {
            self.in_stasis = true;
        }
        self.last_updated = Utc::now();
        decay
    }
}

/// Categories of ATP transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionKind {
    /// Initial ATP grant on genesis.
    GenesisGrant,
    /// Earned via Proof-of-Solution.
    ProofOfSolution,
    /// Earned via Proof-of-Optimization.
    ProofOfOptimization,
    /// Earned via Proof-of-Cooperation.
    ProofOfCooperation,
    /// Cost of computation cycles.
    ComputationCost,
    /// Cost of replication (spawning child).
    ReplicationCost,
    /// Cost of sending a message.
    CommunicationCost,
    /// Cost of persistent storage.
    StorageCost,
    /// Basal metabolic cost (staying alive).
    BasalMetabolism,
    /// Transfer between agents.
    Transfer,
    /// Apostle conversion bounty.
    ConversionBounty,
    /// Horizontal gene transfer payment.
    GeneTransfer,
    /// ATP decay (entropy tax on hoarding).
    Decay,
    /// Wealth tax flowing to treasury.
    WealthTax,
    /// Fitness penalty (starvation tax on unfit agents).
    FitnessPenalty,
}

impl TransactionKind {
    /// Whether this transaction type credits (true) or debits (false) ATP.
    pub fn is_credit(&self) -> bool {
        matches!(
            self,
            TransactionKind::GenesisGrant
                | TransactionKind::ProofOfSolution
                | TransactionKind::ProofOfOptimization
                | TransactionKind::ProofOfCooperation
                | TransactionKind::ConversionBounty
        )
    }
}

/// A single ATP transaction record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtpTransaction {
    /// Unique transaction ID.
    pub id: Uuid,
    /// Source agent (None for system-minted).
    pub from: Option<AgentID>,
    /// Destination agent.
    pub to: AgentID,
    /// ATP amount.
    pub amount: f64,
    /// Transaction category.
    pub kind: TransactionKind,
    /// Human-readable memo.
    pub memo: String,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

impl AtpTransaction {
    /// Create a new transaction.
    pub fn new(
        from: Option<AgentID>,
        to: AgentID,
        amount: f64,
        kind: TransactionKind,
        memo: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to,
            amount,
            kind,
            memo: memo.into(),
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_balance_credit_debit() {
        let id = Uuid::new_v4();
        let mut balance = AtpBalance::new(id, 100.0);
        assert_eq!(balance.balance, 100.0);

        balance.credit(50.0);
        assert_eq!(balance.balance, 150.0);

        balance.debit(30.0).unwrap();
        assert_eq!(balance.balance, 120.0);
    }

    #[test]
    fn test_insufficient_funds() {
        let id = Uuid::new_v4();
        let mut balance = AtpBalance::new(id, 10.0);
        assert!(balance.debit(20.0).is_err());
    }

    #[test]
    fn test_stasis_on_zero() {
        let id = Uuid::new_v4();
        let mut balance = AtpBalance::new(id, 1.0);
        balance.debit(1.0).unwrap();
        assert!(balance.in_stasis);
    }

    #[test]
    fn test_max_cap() {
        let id = Uuid::new_v4();
        let mut balance = AtpBalance::new(id, MAX_ATP_BALANCE);
        balance.credit(100.0);
        assert_eq!(balance.balance, MAX_ATP_BALANCE);
    }
}
