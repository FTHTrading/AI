// Genesis Protocol — Problem Market
//
// Minimal utility pressure engine. Problems are posted with difficulty
// and reward. Agents compete based on skill match. Best solver earns ATP
// and reputation. This creates selection pressure tied to usefulness,
// not just survival.
//
// Step 4 on the 10-step roadmap.

use genesis_dna::skills::SkillProfile;
use serde::{Deserialize, Serialize};

/// Category of problem — maps to skill axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProblemCategory {
    /// Requires compute skill.
    Optimization,
    /// Requires cooperation skill.
    Strategy,
    /// Requires communication skill.
    Coordination,
    /// Requires compute skill (analysis variant).
    Analysis,
}

/// A problem posted to the market.
///
/// Difficulty in [0.0, 1.0]. Reward in ATP.
/// The market evaluates all agents and awards the best match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    /// Unique problem identifier.
    pub id: u64,
    /// What kind of skill this problem demands.
    pub category: ProblemCategory,
    /// How hard the problem is [0.0, 1.0].
    pub difficulty: f64,
    /// ATP reward for solving.
    pub reward_atp: f64,
    /// Whether this problem has been solved.
    pub solved: bool,
}

/// Result of evaluating one agent against one problem.
#[derive(Debug, Clone)]
pub struct EvalResult {
    /// The score this agent achieved [0.0, 1.0].
    pub score: f64,
    /// Whether the score exceeds the difficulty threshold.
    pub passes: bool,
}

impl Problem {
    /// Create a new unsolved problem.
    pub fn new(id: u64, category: ProblemCategory, difficulty: f64, reward_atp: f64) -> Self {
        Self {
            id,
            category,
            difficulty: difficulty.clamp(0.0, 1.0),
            reward_atp,
            solved: false,
        }
    }
}

/// Evaluate how well an agent's skills match a problem.
///
/// Returns the relevant skill axis multiplied by difficulty, producing
/// a quality score. Higher is better.
pub fn evaluate(skills: &SkillProfile, problem: &Problem) -> EvalResult {
    let base = match problem.category {
        ProblemCategory::Optimization => skills.optimization,
        ProblemCategory::Strategy => skills.cooperation,
        ProblemCategory::Coordination => skills.communication,
        ProblemCategory::Analysis => skills.compute,
    };

    let score = (base * problem.difficulty).clamp(0.0, 1.0);
    let passes = score > problem.difficulty * 0.5;

    EvalResult { score, passes }
}

/// The problem market — holds active problems and tracks solutions.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemMarket {
    /// Active problems awaiting solutions.
    problems: Vec<Problem>,
    /// Next problem ID.
    next_id: u64,
    /// Total problems solved lifetime.
    pub total_solved: u64,
    /// Total ATP distributed as rewards lifetime.
    pub total_rewarded: f64,
}

impl ProblemMarket {
    pub fn new() -> Self {
        Self {
            problems: Vec::new(),
            next_id: 1,
            total_solved: 0,
            total_rewarded: 0.0,
        }
    }

    /// Post a new problem to the market.
    pub fn post(&mut self, category: ProblemCategory, difficulty: f64, reward_atp: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.problems.push(Problem::new(id, category, difficulty, reward_atp));
        id
    }

    /// Generate a set of epoch problems with category bias rotation.
    ///
    /// The bias shifts every 25 epochs to prevent monoculture:
    ///   - Epochs 0-24:  Optimization-heavy
    ///   - Epochs 25-49: Strategy-heavy
    ///   - Epochs 50-74: Coordination-heavy
    ///   - Epochs 75-99: Analysis-heavy
    ///
    /// The dominant category gets `count` problems. One problem from a
    /// secondary category is always injected as noise to reward diversity.
    /// Higher pressure → harder problems, but also higher rewards.
    pub fn generate_epoch_problems(&mut self, pressure: f64, count: usize, epoch: u64) -> Vec<u64> {
        let phase = ((epoch / 25) % 4) as usize;
        let dominant = [
            ProblemCategory::Optimization,
            ProblemCategory::Strategy,
            ProblemCategory::Coordination,
            ProblemCategory::Analysis,
        ];
        let primary = dominant[phase];
        // Secondary rotates within the non-dominant categories
        let secondary = dominant[(phase + 1 + (epoch as usize % 3)) % 4];

        let difficulty = (pressure * 0.8 + 0.1).clamp(0.1, 0.95);
        let reward = difficulty * 8.0;

        let mut ids = Vec::with_capacity(count);
        for i in 0..count {
            let cat = if i == count - 1 { secondary } else { primary };
            ids.push(self.post(cat, difficulty, reward));
        }
        ids
    }

    /// Get all unsolved problems.
    pub fn active_problems(&self) -> Vec<&Problem> {
        self.problems.iter().filter(|p| !p.solved).collect()
    }

    /// Mark a problem as solved and record the reward.
    pub fn mark_solved(&mut self, problem_id: u64, reward: f64) {
        if let Some(p) = self.problems.iter_mut().find(|p| p.id == problem_id) {
            p.solved = true;
            self.total_solved += 1;
            self.total_rewarded += reward;
        }
    }

    /// Count of currently active (unsolved) problems.
    pub fn active_count(&self) -> usize {
        self.problems.iter().filter(|p| !p.solved).count()
    }
}

impl Default for ProblemMarket {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_skills(compute: f64, opt: f64, comm: f64, coop: f64) -> SkillProfile {
        SkillProfile {
            compute,
            optimization: opt,
            communication: comm,
            cooperation: coop,
        }
    }

    #[test]
    fn test_evaluate_optimization() {
        let skills = test_skills(0.3, 0.9, 0.2, 0.4);
        let problem = Problem::new(1, ProblemCategory::Optimization, 0.8, 10.0);
        let result = evaluate(&skills, &problem);
        // optimization (0.9) * difficulty (0.8) = 0.72
        assert!((result.score - 0.72).abs() < 0.001);
        assert!(result.passes); // 0.72 > 0.8 * 0.5 = 0.4
    }

    #[test]
    fn test_evaluate_analysis() {
        let skills = test_skills(0.8, 0.2, 0.3, 0.1);
        let problem = Problem::new(2, ProblemCategory::Analysis, 0.6, 5.0);
        let result = evaluate(&skills, &problem);
        // compute (0.8) * difficulty (0.6) = 0.48
        assert!((result.score - 0.48).abs() < 0.001);
        assert!(result.passes); // 0.48 > 0.6 * 0.5 = 0.3
    }

    #[test]
    fn test_low_skill_fails() {
        let skills = test_skills(0.1, 0.1, 0.1, 0.1);
        let problem = Problem::new(3, ProblemCategory::Optimization, 0.9, 10.0);
        let result = evaluate(&skills, &problem);
        // optimization (0.1) * difficulty (0.9) = 0.09
        assert!(!result.passes); // 0.09 < 0.9 * 0.5 = 0.45
    }

    #[test]
    fn test_market_lifecycle() {
        let mut market = ProblemMarket::new();
        let id = market.post(ProblemCategory::Strategy, 0.5, 4.0);
        assert_eq!(market.active_count(), 1);

        market.mark_solved(id, 4.0);
        assert_eq!(market.active_count(), 0);
        assert_eq!(market.total_solved, 1);
        assert!((market.total_rewarded - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_generate_epoch_problems() {
        let mut market = ProblemMarket::new();
        let ids = market.generate_epoch_problems(0.5, 4, 0);
        assert_eq!(ids.len(), 4);
        assert_eq!(market.active_count(), 4);
    }

    #[test]
    fn test_bias_rotation_shifts_category() {
        // Phase 0 (epoch 0-24): Optimization dominant
        let mut m1 = ProblemMarket::new();
        m1.generate_epoch_problems(0.5, 4, 0);
        let p1: Vec<_> = m1.active_problems().iter().map(|p| p.category).collect();
        let opt_count = p1.iter().filter(|&&c| c == ProblemCategory::Optimization).count();
        assert!(opt_count >= 3, "Phase 0 should be Optimization-heavy, got {}", opt_count);

        // Phase 1 (epoch 25-49): Strategy dominant
        let mut m2 = ProblemMarket::new();
        m2.generate_epoch_problems(0.5, 4, 25);
        let p2: Vec<_> = m2.active_problems().iter().map(|p| p.category).collect();
        let strat_count = p2.iter().filter(|&&c| c == ProblemCategory::Strategy).count();
        assert!(strat_count >= 3, "Phase 1 should be Strategy-heavy, got {}", strat_count);

        // Phase 2 (epoch 50-74): Coordination dominant
        let mut m3 = ProblemMarket::new();
        m3.generate_epoch_problems(0.5, 4, 50);
        let p3: Vec<_> = m3.active_problems().iter().map(|p| p.category).collect();
        let coord_count = p3.iter().filter(|&&c| c == ProblemCategory::Coordination).count();
        assert!(coord_count >= 3, "Phase 2 should be Coordination-heavy, got {}", coord_count);

        // Phase 3 (epoch 75-99): Analysis dominant
        let mut m4 = ProblemMarket::new();
        m4.generate_epoch_problems(0.5, 4, 75);
        let p4: Vec<_> = m4.active_problems().iter().map(|p| p.category).collect();
        let anal_count = p4.iter().filter(|&&c| c == ProblemCategory::Analysis).count();
        assert!(anal_count >= 3, "Phase 3 should be Analysis-heavy, got {}", anal_count);
    }

    #[test]
    fn test_difficulty_clamped() {
        let problem = Problem::new(1, ProblemCategory::Analysis, 1.5, 5.0);
        assert!(problem.difficulty <= 1.0);
        let problem2 = Problem::new(2, ProblemCategory::Analysis, -0.5, 5.0);
        assert!(problem2.difficulty >= 0.0);
    }
}
