// Experiment Report — CSV export + text summary
//
// Every experiment should produce publishable output:
//   - CSV with per-step statistical summaries
//   - Text report with findings
//   - JSON manifest for reproducibility
//
// This module generates all three from an ExperimentResult.

use crate::runner::ExperimentResult;
use crate::manifest::ReplayManifest;
use serde::{Serialize, Deserialize};

/// Complete experiment report (text + CSV + manifest).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentReport {
    /// Human-readable text report.
    pub text_report: String,
    /// CSV data (parameter_value, metric_mean, metric_stddev, ...).
    pub csv_data: String,
    /// Replay manifest.
    pub manifest: ReplayManifest,
}

impl ExperimentReport {
    /// Generate a complete report from an experiment result.
    pub fn generate(result: &ExperimentResult, findings: Vec<String>) -> Self {
        let text_report = Self::generate_text(result, &findings);
        let csv_data = Self::generate_csv(result);
        let manifest = ReplayManifest::from_result(result, findings);

        Self {
            text_report,
            csv_data,
            manifest,
        }
    }

    /// Generate a human-readable text report.
    fn generate_text(result: &ExperimentResult, findings: &[String]) -> String {
        let mut s = String::new();

        // Header
        s.push_str("╔══════════════════════════════════════════════════════════╗\n");
        s.push_str(&format!("║  Genesis Experiment: {:<35} ║\n",
            truncate(&result.config.name, 35)));
        s.push_str("╚══════════════════════════════════════════════════════════╝\n\n");

        // Hypothesis
        s.push_str(&format!("Hypothesis: {}\n\n", result.config.hypothesis));

        // Protocol
        s.push_str("── Protocol ──\n");
        s.push_str(&format!("  Variable:      {}\n", result.config.sweep.variable.name()));
        s.push_str(&format!("  Range:         {:.6} → {:.6} (step {:.6})\n",
            result.config.sweep.start,
            result.config.sweep.end,
            result.config.sweep.step,
        ));
        s.push_str(&format!("  Runs/step:     {}\n", result.config.runs_per_step));
        s.push_str(&format!("  Epochs/run:    {}\n", result.config.epochs_per_run));
        s.push_str(&format!("  Base preset:   {:?}\n", result.config.base_preset));
        s.push_str(&format!("  Base seed:     {}\n", result.config.base_seed));
        s.push_str(&format!("  Total worlds:  {}\n", result.total_worlds));
        s.push_str(&format!("  Total epochs:  {}\n", result.total_epochs_run));
        let duration = result.finished_at - result.started_at;
        s.push_str(&format!("  Duration:      {}ms\n", duration.num_milliseconds()));
        s.push_str("\n");

        // Results table
        s.push_str("── Results ──\n\n");

        // Header row
        let metrics = &result.config.metrics;
        s.push_str(&format!("{:<12} {:<12}", "param_value", "collapse_%"));
        for metric in metrics {
            s.push_str(&format!(" {:<20}", metric.name()));
        }
        s.push('\n');
        s.push_str(&"-".repeat(24 + metrics.len() * 21));
        s.push('\n');

        // Data rows
        for step in &result.steps {
            s.push_str(&format!("{:<12.6} {:<12.1}",
                step.parameter_value,
                step.collapse_rate * 100.0,
            ));
            for metric in metrics {
                if let Some(summary) = step.metric_summaries.get(metric.name()) {
                    s.push_str(&format!(" {:<8.2} ± {:<8.2}", summary.mean, summary.std_dev));
                } else {
                    s.push_str(&format!(" {:<20}", "N/A"));
                }
            }
            s.push('\n');
        }
        s.push('\n');

        // Findings
        if !findings.is_empty() {
            s.push_str("── Findings ──\n\n");
            for (i, finding) in findings.iter().enumerate() {
                s.push_str(&format!("  {}. {}\n", i + 1, finding));
            }
            s.push('\n');
        }

        // Hash
        s.push_str("── Verification ──\n\n");
        s.push_str(&format!("  Result hash:   {}\n", result.result_hash));
        s.push('\n');

        s
    }

    /// Generate CSV data for all steps and metrics.
    fn generate_csv(result: &ExperimentResult) -> String {
        let mut csv = String::new();
        let metrics = &result.config.metrics;

        // Header
        csv.push_str("parameter_value,collapse_rate,mean_survival_epochs");
        for metric in metrics {
            csv.push_str(&format!(",{}_mean,{}_median,{}_stddev,{}_min,{}_max,{}_p10,{}_p90",
                metric.name(), metric.name(), metric.name(),
                metric.name(), metric.name(), metric.name(), metric.name(),
            ));
        }
        csv.push('\n');

        // Data rows
        for step in &result.steps {
            csv.push_str(&format!("{},{},{}",
                step.parameter_value,
                step.collapse_rate,
                step.mean_survival_epochs,
            ));
            for metric in metrics {
                if let Some(summary) = step.metric_summaries.get(metric.name()) {
                    csv.push_str(&format!(",{},{},{},{},{},{},{}",
                        summary.mean, summary.median, summary.std_dev,
                        summary.min, summary.max, summary.p10, summary.p90,
                    ));
                } else {
                    csv.push_str(",0,0,0,0,0,0,0");
                }
            }
            csv.push('\n');
        }

        csv
    }

    /// Save report files to a directory.
    /// Optionally provide a slug to use as the filename prefix.
    pub fn save_to_dir(&self, dir: &str) -> std::io::Result<()> {
        self.save_to_dir_with_slug(dir, None)
    }

    /// Save report files to a directory with a custom slug prefix.
    pub fn save_to_dir_with_slug(&self, dir: &str, slug: Option<&str>) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;

        let safe_name = match slug {
            Some(s) => s.to_string(),
            None => self.manifest.config.name
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                .collect::<String>()
                .to_lowercase(),
        };

        // Text report
        std::fs::write(
            format!("{}/{}_report.txt", dir, safe_name),
            &self.text_report,
        )?;

        // CSV data
        std::fs::write(
            format!("{}/{}_data.csv", dir, safe_name),
            &self.csv_data,
        )?;

        // JSON manifest
        std::fs::write(
            format!("{}/{}_manifest.json", dir, safe_name),
            self.manifest.to_json(),
        )?;

        Ok(())
    }
}

/// Truncate a string to max_len characters.
fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ExperimentConfig, ParameterSweep, Metric, SweepVariable};
    use crate::runner::ExperimentRunner;
    use genesis_multiverse::PhysicsPreset;

    fn mini_result() -> ExperimentResult {
        let config = ExperimentConfig {
            name: "Report Test".into(),
            hypothesis: "Reports generate correctly".into(),
            sweep: ParameterSweep::new(SweepVariable::EntropyCoeff, 0.00002, 0.00004, 0.00001),
            runs_per_step: 2,
            epochs_per_run: 5,
            metrics: vec![Metric::FinalPopulation, Metric::Collapsed, Metric::MeanFitness],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: None,
            mutation_rate_override: None,
            cortex_enabled_override: None,
            base_stress_override: None,
            base_seed: 42,
        };
        ExperimentRunner::run(&config)
    }

    #[test]
    fn text_report_generated() {
        let result = mini_result();
        let report = ExperimentReport::generate(&result, vec!["Finding 1".into()]);

        assert!(report.text_report.contains("Report Test"));
        assert!(report.text_report.contains("entropy_coeff"));
        assert!(report.text_report.contains("Finding 1"));
        assert!(report.text_report.contains("Verification"));
    }

    #[test]
    fn csv_has_correct_columns() {
        let result = mini_result();
        let report = ExperimentReport::generate(&result, vec![]);
        let lines: Vec<&str> = report.csv_data.lines().collect();

        // Header + 3 data rows
        assert_eq!(lines.len(), 4);
        assert!(lines[0].contains("parameter_value"));
        assert!(lines[0].contains("collapse_rate"));
        assert!(lines[0].contains("final_population_mean"));
        assert!(lines[0].contains("mean_fitness_stddev"));
    }

    #[test]
    fn csv_data_rows_match_steps() {
        let result = mini_result();
        let report = ExperimentReport::generate(&result, vec![]);
        let data_lines = report.csv_data.lines().count() - 1; // minus header
        assert_eq!(data_lines, result.steps.len());
    }

    #[test]
    fn report_includes_manifest() {
        let result = mini_result();
        let report = ExperimentReport::generate(&result, vec![]);
        assert!(report.manifest.verify());
        assert_eq!(report.manifest.result_hash, result.result_hash);
    }
}
