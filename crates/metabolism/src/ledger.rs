use std::collections::HashMap;

use genesis_dna::AgentID;

use crate::atp::{AtpBalance, AtpTransaction, TransactionKind};
use crate::errors::MetabolismError;
use crate::proof::{Solution, SolutionVerdict};

use serde::{Serialize, Deserialize};

/// Central metabolic ledger tracking all agent balances and transaction history.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetabolismLedger {
    balances: HashMap<AgentID, AtpBalance>,
    transactions: Vec<AtpTransaction>,
    total_atp_supply: f64,
}

impl MetabolismLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new agent with an initial ATP grant.
    pub fn register_agent(&mut self, agent_id: AgentID, initial_atp: f64) -> &AtpBalance {
        let balance = AtpBalance::new(agent_id, initial_atp);
        self.total_atp_supply += initial_atp;

        let tx = AtpTransaction::new(
            None,
            agent_id,
            initial_atp,
            TransactionKind::GenesisGrant,
            "Initial genesis ATP grant",
        );
        self.transactions.push(tx);
        self.balances.entry(agent_id).or_insert(balance)
    }

    /// Get an agent's current balance.
    pub fn balance(&self, agent_id: &AgentID) -> Result<&AtpBalance, MetabolismError> {
        self.balances
            .get(agent_id)
            .ok_or_else(|| MetabolismError::AgentNotFound(agent_id.to_string()))
    }

    /// Mint ATP for an agent (reward for proof-of-work).
    pub fn mint(
        &mut self,
        agent_id: &AgentID,
        amount: f64,
        kind: TransactionKind,
        memo: impl Into<String>,
    ) -> Result<(), MetabolismError> {
        if amount <= 0.0 {
            return Err(MetabolismError::InvalidAmount(amount));
        }
        let balance = self
            .balances
            .get_mut(agent_id)
            .ok_or_else(|| MetabolismError::AgentNotFound(agent_id.to_string()))?;

        balance.credit(amount);
        self.total_atp_supply += amount;

        let tx = AtpTransaction::new(None, *agent_id, amount, kind, memo);
        self.transactions.push(tx);
        Ok(())
    }

    /// Burn ATP from an agent (cost of operations).
    pub fn burn(
        &mut self,
        agent_id: &AgentID,
        amount: f64,
        kind: TransactionKind,
        memo: impl Into<String>,
    ) -> Result<(), MetabolismError> {
        if amount <= 0.0 {
            return Err(MetabolismError::InvalidAmount(amount));
        }
        let balance = self
            .balances
            .get_mut(agent_id)
            .ok_or_else(|| MetabolismError::AgentNotFound(agent_id.to_string()))?;

        if balance.in_stasis {
            return Err(MetabolismError::AgentInStasis(agent_id.to_string()));
        }

        balance.debit(amount)?;
        self.total_atp_supply -= amount;

        let tx = AtpTransaction::new(Some(*agent_id), *agent_id, amount, kind, memo);
        self.transactions.push(tx);
        Ok(())
    }

    /// Transfer ATP between agents.
    pub fn transfer(
        &mut self,
        from: &AgentID,
        to: &AgentID,
        amount: f64,
        memo: impl Into<String>,
    ) -> Result<(), MetabolismError> {
        if amount <= 0.0 {
            return Err(MetabolismError::InvalidAmount(amount));
        }

        // Check sender exists and can afford
        {
            let sender = self
                .balances
                .get(from)
                .ok_or_else(|| MetabolismError::AgentNotFound(from.to_string()))?;
            if sender.in_stasis {
                return Err(MetabolismError::AgentInStasis(from.to_string()));
            }
            if !sender.can_afford(amount) {
                return Err(MetabolismError::InsufficientAtp {
                    required: amount,
                    available: sender.balance,
                });
            }
        }

        // Check receiver exists
        if !self.balances.contains_key(to) {
            return Err(MetabolismError::AgentNotFound(to.to_string()));
        }

        // Execute
        self.balances.get_mut(from).unwrap().debit(amount)?;
        self.balances.get_mut(to).unwrap().credit(amount);

        let memo_str = memo.into();
        let tx = AtpTransaction::new(Some(*from), *to, amount, TransactionKind::Transfer, memo_str);
        self.transactions.push(tx);
        Ok(())
    }

    /// Evaluate a submitted solution and reward the agent accordingly.
    pub fn evaluate_and_reward(
        &mut self,
        agent_id: &AgentID,
        solution: &Solution,
        generation_efficiency: f64,
    ) -> Result<SolutionVerdict, MetabolismError> {
        let verdict = solution.evaluate();

        if verdict.accepted {
            let reward = verdict.reward * generation_efficiency;
            let kind = match solution.proof_kind {
                crate::proof::ProofKind::Solution => TransactionKind::ProofOfSolution,
                crate::proof::ProofKind::Optimization => TransactionKind::ProofOfOptimization,
                crate::proof::ProofKind::Cooperation => TransactionKind::ProofOfCooperation,
            };
            self.mint(
                agent_id,
                reward,
                kind,
                format!("Reward for {:?}: {}", solution.proof_kind, solution.description),
            )?;
        }

        Ok(verdict)
    }

    /// Apply metabolic tick to all agents (basal cost of staying alive).
    pub fn metabolic_tick_all(&mut self) {
        let agent_ids: Vec<AgentID> = self.balances.keys().cloned().collect();
        for id in agent_ids {
            if let Some(balance) = self.balances.get_mut(&id) {
                let rate = 1.0; // Could be per-agent via DNA
                balance.metabolic_tick(rate);
            }
        }
    }

    /// Get all agents currently in stasis.
    pub fn agents_in_stasis(&self) -> Vec<AgentID> {
        self.balances
            .iter()
            .filter(|(_, b)| b.in_stasis)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get total ATP supply in the economy.
    pub fn total_supply(&self) -> f64 {
        self.total_atp_supply
    }

    /// Number of registered agents.
    pub fn agent_count(&self) -> usize {
        self.balances.len()
    }

    /// Get recent transactions (last N).
    pub fn recent_transactions(&self, n: usize) -> &[AtpTransaction] {
        let start = self.transactions.len().saturating_sub(n);
        &self.transactions[start..]
    }

    /// Get all transactions for a specific agent.
    pub fn agent_transactions(&self, agent_id: &AgentID) -> Vec<&AtpTransaction> {
        self.transactions
            .iter()
            .filter(|tx| tx.to == *agent_id || tx.from == Some(*agent_id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_register_and_balance() {
        let mut ledger = MetabolismLedger::new();
        let id = Uuid::new_v4();
        ledger.register_agent(id, 100.0);
        assert_eq!(ledger.balance(&id).unwrap().balance, 100.0);
    }

    #[test]
    fn test_mint_and_burn() {
        let mut ledger = MetabolismLedger::new();
        let id = Uuid::new_v4();
        ledger.register_agent(id, 50.0);

        ledger
            .mint(&id, 25.0, TransactionKind::ProofOfSolution, "test reward")
            .unwrap();
        assert_eq!(ledger.balance(&id).unwrap().balance, 75.0);

        ledger
            .burn(&id, 10.0, TransactionKind::ComputationCost, "compute gas")
            .unwrap();
        assert_eq!(ledger.balance(&id).unwrap().balance, 65.0);
    }

    #[test]
    fn test_transfer() {
        let mut ledger = MetabolismLedger::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        ledger.register_agent(a, 100.0);
        ledger.register_agent(b, 50.0);

        ledger.transfer(&a, &b, 30.0, "payment").unwrap();
        assert_eq!(ledger.balance(&a).unwrap().balance, 70.0);
        assert_eq!(ledger.balance(&b).unwrap().balance, 80.0);
    }

    #[test]
    fn test_stasis_prevents_burn() {
        let mut ledger = MetabolismLedger::new();
        let id = Uuid::new_v4();
        ledger.register_agent(id, 1.0);
        ledger
            .burn(&id, 1.0, TransactionKind::ComputationCost, "drain")
            .unwrap();
        // Now in stasis
        assert!(ledger
            .burn(&id, 0.1, TransactionKind::ComputationCost, "should fail")
            .is_err());
    }
}
