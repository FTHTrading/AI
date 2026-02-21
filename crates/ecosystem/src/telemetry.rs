// Telemetry — UnitStatus & RiskState
//
// Provides ecosystem-level health monitoring. Computes aggregate
// statistics from the agent population and flags risk conditions
// before they cascade into collapse.

use std::collections::HashMap;
use genesis_dna::{AgentDNA, AgentRole};

/// Risk conditions the ecosystem can detect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskState {
    /// No immediate risks detected.
    Stable,
    /// One role exceeds 50% of population — evolutionary dead end risk.
    MonocultureEmerging,
    /// Top 10% of agents hold > 60% of total ATP — wealth concentration.
    ATPConcentrationHigh,
    /// Mean reputation below 0.3 — systemic quality erosion.
    ReputationDecay,
    /// Population below 10 — extinction-level event possible.
    PopulationCrashRisk,
}

/// Snapshot of ecosystem health at a point in time.
#[derive(Debug, Clone)]
pub struct UnitStatus {
    pub population: usize,
    pub avg_reputation: f64,
    pub atp_total: f64,
    pub role_distribution: HashMap<AgentRole, usize>,
    pub risks: Vec<RiskState>,
}

impl UnitStatus {
    /// Compute ecosystem health from agent list and their ATP balances.
    ///
    /// `atp_balances` is a parallel slice — atp_balances[i] is the ATP for agents[i].
    pub fn compute(agents: &[AgentDNA], atp_balances: &[f64]) -> Self {
        let population = agents.len();

        // Role distribution
        let mut role_distribution: HashMap<AgentRole, usize> = HashMap::new();
        for agent in agents {
            *role_distribution.entry(agent.role).or_insert(0) += 1;
        }

        // Average reputation
        let avg_reputation = if population > 0 {
            agents.iter().map(|a| a.reputation.score).sum::<f64>() / population as f64
        } else {
            0.0
        };

        // ATP total
        let atp_total: f64 = atp_balances.iter().sum();

        // --- Risk detection ---
        let mut risks = Vec::new();

        // Monoculture: any role > 50%
        if population > 0 {
            for &count in role_distribution.values() {
                if count as f64 / population as f64 > 0.5 {
                    risks.push(RiskState::MonocultureEmerging);
                    break;
                }
            }
        }

        // ATP concentration: top 10% holds > 60%
        if population >= 10 && atp_total > 0.0 {
            let mut sorted_atp: Vec<f64> = atp_balances.to_vec();
            sorted_atp.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            let top_n = (population as f64 * 0.1).ceil() as usize;
            let top_atp: f64 = sorted_atp.iter().take(top_n).sum();
            if top_atp / atp_total > 0.6 {
                risks.push(RiskState::ATPConcentrationHigh);
            }
        }

        // Reputation decay: mean below 0.3
        if avg_reputation < 0.3 && population > 0 {
            risks.push(RiskState::ReputationDecay);
        }

        // Population crash risk: fewer than 10 agents
        if population < 10 {
            risks.push(RiskState::PopulationCrashRisk);
        }

        // If no risks flagged, mark stable
        if risks.is_empty() {
            risks.push(RiskState::Stable);
        }

        UnitStatus {
            population,
            avg_reputation,
            atp_total,
            role_distribution,
            risks,
        }
    }

    /// True if ecosystem has no warning flags.
    pub fn is_stable(&self) -> bool {
        self.risks.len() == 1 && self.risks[0] == RiskState::Stable
    }

    /// True if a specific risk condition is present.
    pub fn has_risk(&self, risk: RiskState) -> bool {
        self.risks.contains(&risk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genesis_dna::AgentDNA;

    fn make_agents(count: usize) -> Vec<AgentDNA> {
        // Use different entropy seeds to get role variety
        // Seed must be >= 32 bytes for from_entropy
        (0..count)
            .map(|i| {
                let seed: Vec<u8> = (0..64).map(|j| (i * 7 + j * 13 + 42) as u8).collect();
                AgentDNA::from_entropy(&seed, false).unwrap()
            })
            .collect()
    }

    #[test]
    fn test_stable_ecosystem() {
        let agents = make_agents(20);
        let atp: Vec<f64> = vec![10.0; 20]; // Equal ATP
        let status = UnitStatus::compute(&agents, &atp);
        assert_eq!(status.population, 20);
        assert!(status.atp_total > 0.0);
        // With 20 agents and 5 roles, unlikely any single role > 50%
        // but we check for stability
        assert!(!status.has_risk(RiskState::PopulationCrashRisk));
    }

    #[test]
    fn test_population_crash_risk() {
        let agents = make_agents(5);
        let atp: Vec<f64> = vec![10.0; 5];
        let status = UnitStatus::compute(&agents, &atp);
        assert!(status.has_risk(RiskState::PopulationCrashRisk));
    }

    #[test]
    fn test_reputation_decay() {
        let mut agents = make_agents(15);
        // Trash everyone's reputation
        for agent in &mut agents {
            agent.reputation.score = 0.1;
        }
        let atp: Vec<f64> = vec![10.0; 15];
        let status = UnitStatus::compute(&agents, &atp);
        assert!(status.has_risk(RiskState::ReputationDecay));
        assert!(status.avg_reputation < 0.3);
    }

    #[test]
    fn test_atp_concentration() {
        let agents = make_agents(20);
        let mut atp: Vec<f64> = vec![1.0; 20];
        // Give top 2 agents (10%) most of the ATP
        atp[0] = 100.0;
        atp[1] = 100.0;
        // Total = 218, top 10% (2 agents) = 200, 200/218 = 91% > 60%
        let status = UnitStatus::compute(&agents, &atp);
        assert!(status.has_risk(RiskState::ATPConcentrationHigh));
    }

    #[test]
    fn test_empty_ecosystem() {
        let agents: Vec<AgentDNA> = vec![];
        let atp: Vec<f64> = vec![];
        let status = UnitStatus::compute(&agents, &atp);
        assert_eq!(status.population, 0);
        assert!(status.has_risk(RiskState::PopulationCrashRisk));
    }

    #[test]
    fn test_is_stable_helper() {
        let agents = make_agents(20);
        let atp: Vec<f64> = vec![10.0; 20];
        let status = UnitStatus::compute(&agents, &atp);
        // Might or might not be stable depending on role distribution,
        // but at minimum the helper should be consistent
        assert_eq!(status.is_stable(), !status.risks.iter().any(|r| *r != RiskState::Stable));
    }
}
