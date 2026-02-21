// Treasury — Ecosystem Reserve & Redistribution
//
// Collects a skim from market rewards to build a communal reserve.
// The reserve funds baseline stipends for underrepresented roles
// and crisis stabilization when the ecosystem detects risk.

use std::collections::HashMap;
use genesis_dna::AgentRole;

/// Ecosystem-level ATP reserve with redistribution logic.
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitTreasury {
    /// Current ATP reserve.
    pub reserve: f64,
    /// Fraction of market rewards skimmed into the reserve (0.0 – 1.0).
    pub skim_rate: f64,
    /// Base stipend paid per epoch to agents in underrepresented roles.
    pub stipend_amount: f64,
    /// Total ATP ever collected via skim.
    pub total_collected: f64,
    /// Total ATP ever distributed via stipends/crisis.
    pub total_distributed: f64,
}

impl UnitTreasury {
    /// Create a treasury with default parameters:
    /// - 5% skim rate
    /// - 1.0 ATP base stipend
    pub fn new() -> Self {
        UnitTreasury {
            reserve: 0.0,
            skim_rate: 0.05,
            stipend_amount: 1.0,
            total_collected: 0.0,
            total_distributed: 0.0,
        }
    }

    /// Skim a fraction of a market reward into the treasury.
    /// Returns the amount skimmed (caller should subtract from agent reward).
    pub fn skim(&mut self, reward: f64) -> f64 {
        let amount = reward * self.skim_rate;
        self.reserve += amount;
        self.total_collected += amount;
        amount
    }

    /// Compute stipends for underrepresented roles.
    ///
    /// A role is "underrepresented" if its population share is below
    /// the fair share (1/number_of_roles = 20% for 5 roles).
    ///
    /// Returns a map of role → per-agent stipend amount.
    /// The caller is responsible for distributing to individual agents.
    pub fn compute_stipends(
        &self,
        role_distribution: &HashMap<AgentRole, usize>,
        population: usize,
    ) -> HashMap<AgentRole, f64> {
        let mut stipends = HashMap::new();
        if population == 0 {
            return stipends;
        }

        let num_roles = 5usize; // 5 archetypes
        let fair_share = 1.0 / num_roles as f64;

        for (&role, &count) in role_distribution {
            let share = count as f64 / population as f64;
            if share < fair_share {
                // Stipend scales with how underrepresented the role is
                let deficit_ratio = (fair_share - share) / fair_share;
                stipends.insert(role, self.stipend_amount * (1.0 + deficit_ratio));
            }
        }

        stipends
    }

    /// Distribute stipends from the treasury reserve.
    ///
    /// Returns a map of role → total ATP distributed for that role.
    /// The treasury pays out up to its reserve — it never goes negative.
    pub fn distribute_stipends(
        &mut self,
        role_distribution: &HashMap<AgentRole, usize>,
        population: usize,
    ) -> HashMap<AgentRole, f64> {
        let stipend_plan = self.compute_stipends(role_distribution, population);
        let mut distributed: HashMap<AgentRole, f64> = HashMap::new();

        for (&role, &per_agent) in &stipend_plan {
            let count = *role_distribution.get(&role).unwrap_or(&0);
            let total_needed = per_agent * count as f64;
            let actual = total_needed.min(self.reserve);
            if actual > 0.0 {
                self.reserve -= actual;
                self.total_distributed += actual;
                distributed.insert(role, actual);
            }
        }

        distributed
    }

    /// Emergency injection: spend up to `amount` from reserves
    /// for crisis stabilization. Returns actual amount spent.
    pub fn crisis_spend(&mut self, amount: f64) -> f64 {
        let actual = amount.min(self.reserve);
        self.reserve -= actual;
        self.total_distributed += actual;
        actual
    }
}

impl Default for UnitTreasury {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skim() {
        let mut t = UnitTreasury::new();
        let skimmed = t.skim(100.0);
        assert!((skimmed - 5.0).abs() < 1e-9);
        assert!((t.reserve - 5.0).abs() < 1e-9);
        assert!((t.total_collected - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_multiple_skims_accumulate() {
        let mut t = UnitTreasury::new();
        t.skim(100.0);
        t.skim(200.0);
        assert!((t.reserve - 15.0).abs() < 1e-9); // 5 + 10
    }

    #[test]
    fn test_compute_stipends_balanced() {
        let t = UnitTreasury::new();
        // Perfectly balanced: 4 agents per role, 20 total
        let mut dist = HashMap::new();
        dist.insert(AgentRole::Optimizer, 4);
        dist.insert(AgentRole::Strategist, 4);
        dist.insert(AgentRole::Communicator, 4);
        dist.insert(AgentRole::Archivist, 4);
        dist.insert(AgentRole::Executor, 4);
        let stipends = t.compute_stipends(&dist, 20);
        // No role is underrepresented — should be empty
        assert!(stipends.is_empty());
    }

    #[test]
    fn test_compute_stipends_unbalanced() {
        let t = UnitTreasury::new();
        // Optimizer monoculture: 12 Opt, 2 each for others
        let mut dist = HashMap::new();
        dist.insert(AgentRole::Optimizer, 12);
        dist.insert(AgentRole::Strategist, 2);
        dist.insert(AgentRole::Communicator, 2);
        dist.insert(AgentRole::Archivist, 2);
        dist.insert(AgentRole::Executor, 2);
        let stipends = t.compute_stipends(&dist, 20);
        // Strategist, Communicator, Archivist, Executor all below 20%
        assert_eq!(stipends.len(), 4);
        // Optimizer is at 60%, above fair share — no stipend
        assert!(!stipends.contains_key(&AgentRole::Optimizer));
    }

    #[test]
    fn test_distribute_stipends_respects_reserve() {
        let mut t = UnitTreasury::new();
        t.reserve = 2.0; // Very small reserve
        let mut dist = HashMap::new();
        dist.insert(AgentRole::Optimizer, 12);
        dist.insert(AgentRole::Strategist, 2);
        dist.insert(AgentRole::Communicator, 2);
        dist.insert(AgentRole::Archivist, 2);
        dist.insert(AgentRole::Executor, 2);
        let paid = t.distribute_stipends(&dist, 20);
        // Should have paid something but not more than 2.0 total
        let total_paid: f64 = paid.values().sum();
        assert!(total_paid <= 2.0 + 1e-9);
        assert!(t.reserve >= -1e-9);
    }

    #[test]
    fn test_crisis_spend() {
        let mut t = UnitTreasury::new();
        t.reserve = 10.0;
        let spent = t.crisis_spend(7.0);
        assert!((spent - 7.0).abs() < 1e-9);
        assert!((t.reserve - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_crisis_spend_capped_by_reserve() {
        let mut t = UnitTreasury::new();
        t.reserve = 3.0;
        let spent = t.crisis_spend(10.0);
        assert!((spent - 3.0).abs() < 1e-9);
        assert!((t.reserve).abs() < 1e-9);
    }

    #[test]
    fn test_empty_population() {
        let t = UnitTreasury::new();
        let stipends = t.compute_stipends(&HashMap::new(), 0);
        assert!(stipends.is_empty());
    }
}
