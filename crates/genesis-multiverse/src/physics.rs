// World Physics — Alternative baseline parameters for civilizations
//
// Each world can run under different "laws of nature":
// different carrying capacities, catastrophe rates, entropy costs,
// wealth distribution rules. These are the initial conditions
// that the cortex may later evolve away from — but the starting
// point shapes the trajectory.
//
// Presets model qualitatively different universes:
//   Earth-Prime  — balanced defaults (our canonical world)
//   High Gravity — low capacity, high entropy (harsh survival)
//   Low Entropy  — generous resources, mild pressure (garden world)
//   Volcanic     — high catastrophe rate (unstable world)
//   Utopia       — no wealth correction, high overflow tolerance
//   Ice Age      — long seasons, extreme amplitude, low regen

use serde::{Serialize, Deserialize};
use gateway::world::PressureConfig;

/// Complete physics specification for a world.
///
/// Combines `PressureConfig` (evolutionary pressure) with ecological
/// parameters (environment shape). A world's physics determines
/// how hard life is, not what life does.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldPhysics {
    /// Evolutionary pressure parameters.
    pub pressure: PressureConfig,
    /// Base resource capacity per ecological niche.
    pub base_capacity: f64,
    /// Seasonal cycle length in epochs.
    pub season_length: u64,
    /// Seasonal amplitude (fraction of capacity, ±).
    pub season_amplitude: f64,
    /// Maximum population cap.
    pub pop_cap: usize,
    /// Human-readable label for this physics profile.
    pub label: String,
}

impl Default for WorldPhysics {
    fn default() -> Self {
        Self::preset(PhysicsPreset::EarthPrime)
    }
}

/// Named physics presets — qualitatively different universes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicsPreset {
    /// Balanced defaults — the canonical Genesis world.
    EarthPrime,
    /// Harsh survival: low capacity, high entropy, tight pressure.
    HighGravity,
    /// Garden world: generous resources, mild pressure.
    LowEntropy,
    /// Unstable world: high catastrophe probability.
    Volcanic,
    /// No wealth correction: laissez-faire economics.
    Utopia,
    /// Extreme seasons: long cycles, deep amplitude, low regeneration.
    IceAge,
}

impl PhysicsPreset {
    /// Name of this preset.
    pub fn name(&self) -> &'static str {
        match self {
            Self::EarthPrime => "Earth-Prime",
            Self::HighGravity => "High Gravity",
            Self::LowEntropy => "Low Entropy",
            Self::Volcanic => "Volcanic",
            Self::Utopia => "Utopia",
            Self::IceAge => "Ice Age",
        }
    }

    /// All available presets.
    pub fn all() -> &'static [PhysicsPreset] {
        &[
            Self::EarthPrime,
            Self::HighGravity,
            Self::LowEntropy,
            Self::Volcanic,
            Self::Utopia,
            Self::IceAge,
        ]
    }
}

impl WorldPhysics {
    /// Create a physics profile from a named preset.
    pub fn preset(preset: PhysicsPreset) -> Self {
        match preset {
            PhysicsPreset::EarthPrime => Self {
                pressure: PressureConfig::default(),
                base_capacity: 150.0,
                season_length: 100,
                season_amplitude: 0.25,
                pop_cap: 200,
                label: "Earth-Prime".into(),
            },
            PhysicsPreset::HighGravity => Self {
                pressure: PressureConfig {
                    soft_cap: 80,
                    entropy_coeff: 0.00008,
                    catastrophe_base_prob: 0.005,
                    catastrophe_pop_scale: 0.00003,
                    gini_wealth_tax_threshold: 0.35,
                    gini_wealth_tax_rate: 0.04,
                    treasury_overflow_threshold: 0.40,
                },
                base_capacity: 80.0,
                season_length: 60,
                season_amplitude: 0.35,
                pop_cap: 100,
                label: "High Gravity".into(),
            },
            PhysicsPreset::LowEntropy => Self {
                pressure: PressureConfig {
                    soft_cap: 300,
                    entropy_coeff: 0.000005,
                    catastrophe_base_prob: 0.0005,
                    catastrophe_pop_scale: 0.000002,
                    gini_wealth_tax_threshold: 0.55,
                    gini_wealth_tax_rate: 0.01,
                    treasury_overflow_threshold: 0.70,
                },
                base_capacity: 250.0,
                season_length: 150,
                season_amplitude: 0.15,
                pop_cap: 400,
                label: "Low Entropy".into(),
            },
            PhysicsPreset::Volcanic => Self {
                pressure: PressureConfig {
                    soft_cap: 150,
                    entropy_coeff: 0.00003,
                    catastrophe_base_prob: 0.015,
                    catastrophe_pop_scale: 0.00005,
                    gini_wealth_tax_threshold: 0.40,
                    gini_wealth_tax_rate: 0.02,
                    treasury_overflow_threshold: 0.50,
                },
                base_capacity: 130.0,
                season_length: 80,
                season_amplitude: 0.40,
                pop_cap: 180,
                label: "Volcanic".into(),
            },
            PhysicsPreset::Utopia => Self {
                pressure: PressureConfig {
                    soft_cap: 250,
                    entropy_coeff: 0.00001,
                    catastrophe_base_prob: 0.001,
                    catastrophe_pop_scale: 0.000005,
                    gini_wealth_tax_threshold: 0.90,  // almost never triggers
                    gini_wealth_tax_rate: 0.005,
                    treasury_overflow_threshold: 0.90,
                },
                base_capacity: 200.0,
                season_length: 120,
                season_amplitude: 0.10,
                pop_cap: 350,
                label: "Utopia".into(),
            },
            PhysicsPreset::IceAge => Self {
                pressure: PressureConfig {
                    soft_cap: 120,
                    entropy_coeff: 0.00004,
                    catastrophe_base_prob: 0.003,
                    catastrophe_pop_scale: 0.00002,
                    gini_wealth_tax_threshold: 0.38,
                    gini_wealth_tax_rate: 0.03,
                    treasury_overflow_threshold: 0.45,
                },
                base_capacity: 100.0,
                season_length: 200,
                season_amplitude: 0.50,
                pop_cap: 150,
                label: "Ice Age".into(),
            },
        }
    }

    /// Create a custom physics profile.
    pub fn custom(
        label: impl Into<String>,
        pressure: PressureConfig,
        base_capacity: f64,
        season_length: u64,
        season_amplitude: f64,
        pop_cap: usize,
    ) -> Self {
        Self {
            pressure,
            base_capacity,
            season_length,
            season_amplitude,
            pop_cap,
            label: label.into(),
        }
    }

    /// Apply this physics profile to a World, overwriting its pressure
    /// config and environment parameters.
    pub fn apply_to(&self, world: &mut gateway::world::World) {
        world.pressure = self.pressure.clone();
        world.pop_cap = self.pop_cap;
        world.environment.base_capacity = self.base_capacity;
        world.environment.season_length = self.season_length;
        world.environment.season_amplitude = self.season_amplitude;
    }

    /// Compute a delta summary against another physics profile.
    pub fn delta(&self, other: &WorldPhysics) -> Vec<(String, f64, f64)> {
        let mut diffs = Vec::new();

        if self.pressure.soft_cap != other.pressure.soft_cap {
            diffs.push(("soft_cap".into(), self.pressure.soft_cap as f64, other.pressure.soft_cap as f64));
        }
        if (self.pressure.entropy_coeff - other.pressure.entropy_coeff).abs() > 1e-12 {
            diffs.push(("entropy_coeff".into(), self.pressure.entropy_coeff, other.pressure.entropy_coeff));
        }
        if (self.pressure.catastrophe_base_prob - other.pressure.catastrophe_base_prob).abs() > 1e-12 {
            diffs.push(("catastrophe_base_prob".into(), self.pressure.catastrophe_base_prob, other.pressure.catastrophe_base_prob));
        }
        if (self.pressure.catastrophe_pop_scale - other.pressure.catastrophe_pop_scale).abs() > 1e-12 {
            diffs.push(("catastrophe_pop_scale".into(), self.pressure.catastrophe_pop_scale, other.pressure.catastrophe_pop_scale));
        }
        if (self.pressure.gini_wealth_tax_threshold - other.pressure.gini_wealth_tax_threshold).abs() > 1e-12 {
            diffs.push(("gini_wealth_tax_threshold".into(), self.pressure.gini_wealth_tax_threshold, other.pressure.gini_wealth_tax_threshold));
        }
        if (self.pressure.gini_wealth_tax_rate - other.pressure.gini_wealth_tax_rate).abs() > 1e-12 {
            diffs.push(("gini_wealth_tax_rate".into(), self.pressure.gini_wealth_tax_rate, other.pressure.gini_wealth_tax_rate));
        }
        if (self.pressure.treasury_overflow_threshold - other.pressure.treasury_overflow_threshold).abs() > 1e-12 {
            diffs.push(("treasury_overflow_threshold".into(), self.pressure.treasury_overflow_threshold, other.pressure.treasury_overflow_threshold));
        }
        if (self.base_capacity - other.base_capacity).abs() > 1e-6 {
            diffs.push(("base_capacity".into(), self.base_capacity, other.base_capacity));
        }
        if self.season_length != other.season_length {
            diffs.push(("season_length".into(), self.season_length as f64, other.season_length as f64));
        }
        if (self.season_amplitude - other.season_amplitude).abs() > 1e-6 {
            diffs.push(("season_amplitude".into(), self.season_amplitude, other.season_amplitude));
        }
        if self.pop_cap != other.pop_cap {
            diffs.push(("pop_cap".into(), self.pop_cap as f64, other.pop_cap as f64));
        }

        diffs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earth_prime_is_default() {
        let ep = WorldPhysics::preset(PhysicsPreset::EarthPrime);
        let default_p = PressureConfig::default();
        assert_eq!(ep.pressure.soft_cap, default_p.soft_cap);
        assert_eq!(ep.pop_cap, 200);
        assert_eq!(ep.label, "Earth-Prime");
    }

    #[test]
    fn high_gravity_is_harsher() {
        let ep = WorldPhysics::preset(PhysicsPreset::EarthPrime);
        let hg = WorldPhysics::preset(PhysicsPreset::HighGravity);
        assert!(hg.pressure.soft_cap < ep.pressure.soft_cap);
        assert!(hg.pressure.entropy_coeff > ep.pressure.entropy_coeff);
        assert!(hg.base_capacity < ep.base_capacity);
    }

    #[test]
    fn low_entropy_is_generous() {
        let ep = WorldPhysics::preset(PhysicsPreset::EarthPrime);
        let le = WorldPhysics::preset(PhysicsPreset::LowEntropy);
        assert!(le.pressure.soft_cap > ep.pressure.soft_cap);
        assert!(le.pressure.entropy_coeff < ep.pressure.entropy_coeff);
        assert!(le.base_capacity > ep.base_capacity);
    }

    #[test]
    fn volcanic_high_catastrophe() {
        let ep = WorldPhysics::preset(PhysicsPreset::EarthPrime);
        let vol = WorldPhysics::preset(PhysicsPreset::Volcanic);
        assert!(vol.pressure.catastrophe_base_prob > ep.pressure.catastrophe_base_prob);
    }

    #[test]
    fn physics_delta_detects_differences() {
        let ep = WorldPhysics::preset(PhysicsPreset::EarthPrime);
        let hg = WorldPhysics::preset(PhysicsPreset::HighGravity);
        let delta = ep.delta(&hg);
        assert!(!delta.is_empty());
        // Should detect soft_cap difference
        assert!(delta.iter().any(|(name, _, _)| name == "soft_cap"));
    }

    #[test]
    fn identical_physics_no_delta() {
        let a = WorldPhysics::preset(PhysicsPreset::EarthPrime);
        let b = WorldPhysics::preset(PhysicsPreset::EarthPrime);
        assert!(a.delta(&b).is_empty());
    }

    #[test]
    fn all_presets_listed() {
        assert_eq!(PhysicsPreset::all().len(), 6);
    }

    #[test]
    fn apply_physics_to_world() {
        let hg = WorldPhysics::preset(PhysicsPreset::HighGravity);
        let mut world = gateway::world::World::new();
        hg.apply_to(&mut world);
        assert_eq!(world.pressure.soft_cap, 80);
        assert_eq!(world.pop_cap, 100);
    }
}
