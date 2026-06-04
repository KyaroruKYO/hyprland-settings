use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use hyprland_settings::config_persistence_validation::{
    load_candidates, run_config_persistence_validation, ConfigPersistenceCandidateReport,
    HyprlandConfigVerifier, VerifyOutput,
};
use serde_json::Value;

fn candidates() -> Result<ConfigPersistenceCandidateReport> {
    load_candidates(Path::new(
        "data/reports/batch-a-config-persistence-candidates.v0.55.2.json",
    ))
}

fn committed_results() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/config-persistence-validation-results.v0.55.2.json"
    ))?)
}

#[derive(Debug)]
struct FakeVerifier {
    output: VerifyOutput,
}

impl HyprlandConfigVerifier for FakeVerifier {
    fn verify_config(&mut self, _config_path: &Path, _timeout: Duration) -> Result<VerifyOutput> {
        Ok(self.output.clone())
    }

    fn command_label(&self, config_path: &Path) -> String {
        format!(
            "fake Hyprland --verify-config --config {}",
            config_path.display()
        )
    }
}

fn passing_verifier() -> FakeVerifier {
    FakeVerifier {
        output: VerifyOutput {
            exit_status: Some(0),
            stdout: "======== Config parsing result:\n\nconfig ok\n".to_string(),
            stderr: String::new(),
            timed_out: false,
        },
    }
}

#[test]
fn dry_run_proves_batch_a_temp_parser_writer_and_single_mutation() -> Result<()> {
    let report = candidates()?;
    let mut verifier = passing_verifier();

    let results =
        run_config_persistence_validation(&report, false, &mut verifier, Duration::from_secs(1));

    assert_eq!(results.mode, "dry-run");
    assert_eq!(results.counts.rows, 39);
    assert_eq!(results.counts.parser_roundtrip_passed, 39);
    assert_eq!(results.counts.writer_roundtrip_passed, 39);
    assert_eq!(results.counts.typed_validator_passed, 39);
    assert_eq!(results.counts.single_mutation_verified, 39);
    assert_eq!(results.counts.hyprland_verify_config_attempted, 0);
    assert_eq!(results.counts.hyprland_verify_config_passed, 0);
    assert_eq!(results.counts.safe_to_enable_by_config_persistence, 0);
    for row in &results.rows {
        assert!(!row.active_config_modified);
        assert!(!row.active_runtime_modified);
        assert!(!row.safe_to_enable_by_config_persistence);
        assert!(row.temp_config_path.starts_with("/tmp/"));
        assert_ne!(row.temp_config_path, "/home/kyo/.config/hypr/hyprland.conf");
    }

    Ok(())
}

#[test]
fn fake_hyprland_verify_success_marks_batch_a_safe_by_config_persistence() -> Result<()> {
    let report = candidates()?;
    let mut verifier = passing_verifier();

    let results =
        run_config_persistence_validation(&report, true, &mut verifier, Duration::from_secs(1));

    assert_eq!(results.mode, "verify-hyprland");
    assert_eq!(results.counts.rows, 39);
    assert_eq!(results.counts.hyprland_verify_config_attempted, 39);
    assert_eq!(results.counts.hyprland_verify_config_passed, 39);
    assert_eq!(results.counts.safe_to_enable_by_config_persistence, 39);
    for row in &results.rows {
        assert!(row
            .hyprland_verify_command
            .contains("Hyprland --verify-config"));
        assert!(row.safe_to_enable_by_config_persistence);
        assert!(!row.active_config_modified);
        assert!(!row.active_runtime_modified);
    }

    Ok(())
}

#[test]
fn fake_hyprland_verify_failure_keeps_rows_blocked() -> Result<()> {
    let report = candidates()?;
    let mut verifier = FakeVerifier {
        output: VerifyOutput {
            exit_status: Some(1),
            stdout: "Config parsing result:\n\nconfig error at line 2\n".to_string(),
            stderr: String::new(),
            timed_out: false,
        },
    };

    let results =
        run_config_persistence_validation(&report, true, &mut verifier, Duration::from_secs(1));

    assert_eq!(results.counts.hyprland_verify_config_attempted, 39);
    assert_eq!(results.counts.hyprland_verify_config_passed, 0);
    assert_eq!(results.counts.safe_to_enable_by_config_persistence, 0);
    assert!(results
        .rows
        .iter()
        .all(|row| !row.safe_to_enable_by_config_persistence));

    Ok(())
}

#[test]
fn fake_hyprland_verify_timeout_keeps_rows_blocked() -> Result<()> {
    let report = candidates()?;
    let mut verifier = FakeVerifier {
        output: VerifyOutput {
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
            timed_out: true,
        },
    };

    let results =
        run_config_persistence_validation(&report, true, &mut verifier, Duration::from_secs(1));

    assert_eq!(results.counts.hyprland_verify_config_attempted, 39);
    assert_eq!(results.counts.hyprland_verify_config_passed, 0);
    assert_eq!(results.counts.safe_to_enable_by_config_persistence, 0);

    Ok(())
}

#[test]
fn committed_results_enforce_config_persistence_policy() -> Result<()> {
    let results = committed_results()?;
    let rows = results["rows"]
        .as_array()
        .expect("results rows should be an array");

    assert_eq!(results["mode"].as_str(), Some("verify-hyprland"));
    assert_eq!(results["counts"]["rows"], 39);
    assert_eq!(results["counts"]["parserRoundtripPassed"], 39);
    assert_eq!(results["counts"]["writerRoundtripPassed"], 39);
    assert_eq!(results["counts"]["typedValidatorPassed"], 39);
    assert_eq!(results["counts"]["singleMutationVerified"], 39);
    assert_eq!(results["counts"]["hyprlandVerifyConfigAttempted"], 39);
    assert_eq!(results["counts"]["hyprlandVerifyConfigPassed"], 39);
    assert_eq!(results["counts"]["safeToEnableByConfigPersistence"], 39);

    for row in rows {
        assert_eq!(row["batch"].as_str(), Some("batch-a-likely-safe-booleans"));
        assert_eq!(row["hyprlandVerifyConfigPassed"].as_bool(), Some(true));
        assert_eq!(row["safeToEnableByConfigPersistence"].as_bool(), Some(true));
        assert_eq!(row["activeConfigModified"].as_bool(), Some(false));
        assert_eq!(row["activeRuntimeModified"].as_bool(), Some(false));
        assert!(row["tempConfigPath"]
            .as_str()
            .expect("tempConfigPath should exist")
            .starts_with("/tmp/"));
    }

    Ok(())
}
