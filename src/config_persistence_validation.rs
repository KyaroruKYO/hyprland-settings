use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::config_parser::parse_hyprland_config_text;
use crate::write_classification::config_key_from_official_setting;

static TEMP_CONFIG_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub const DEFAULT_CANDIDATES_PATH: &str =
    "data/reports/batch-a-config-persistence-candidates.v0.55.2.json";
pub const DEFAULT_RESULTS_PATH: &str =
    "data/reports/config-persistence-validation-results.v0.55.2.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPersistenceCandidateReport {
    pub artifact_kind: String,
    pub counts: ConfigPersistenceCandidateCounts,
    pub rows: Vec<ConfigPersistenceCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPersistenceCandidateCounts {
    pub rows: usize,
    pub safe_to_implement_next_sprint: usize,
    pub safe_to_enable_now: usize,
    pub high_risk_rows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPersistenceCandidate {
    pub row_id: String,
    pub official_setting: String,
    #[serde(default)]
    pub candidate_value: Option<String>,
    #[serde(default)]
    pub control_kind: Option<String>,
    pub current_live_validation_status: String,
    pub proposed_config_persistence_status: String,
    pub safe_to_implement_next_sprint: bool,
    pub safe_to_enable_now: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPersistenceResults {
    pub artifact_kind: String,
    pub schema_version: u32,
    pub hyprland_version: String,
    pub mode: String,
    pub command_shape: String,
    pub counts: ConfigPersistenceResultCounts,
    pub rows: Vec<ConfigPersistenceResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPersistenceResultCounts {
    pub rows: usize,
    pub parser_roundtrip_passed: usize,
    pub writer_roundtrip_passed: usize,
    pub typed_validator_passed: usize,
    pub single_mutation_verified: usize,
    pub hyprland_verify_config_attempted: usize,
    pub hyprland_verify_config_passed: usize,
    pub safe_to_enable_by_config_persistence: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPersistenceResult {
    pub row_id: String,
    pub official_setting: String,
    pub batch: String,
    pub candidate_value: String,
    pub temp_directory: String,
    pub temp_config_path: String,
    pub parser_roundtrip_passed: bool,
    pub writer_roundtrip_passed: bool,
    pub typed_validator_passed: bool,
    pub single_mutation_verified: bool,
    pub hyprland_verify_config_attempted: bool,
    pub hyprland_verify_config_passed: bool,
    pub hyprland_verify_command: String,
    pub hyprland_verify_exit_status: Option<i32>,
    pub hyprland_verify_stdout: String,
    pub hyprland_verify_stderr: String,
    pub active_config_modified: bool,
    pub active_runtime_modified: bool,
    pub safe_to_enable_by_config_persistence: bool,
    pub requires_manual_observation: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyOutput {
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

pub trait HyprlandConfigVerifier {
    fn verify_config(&mut self, config_path: &Path, timeout: Duration) -> Result<VerifyOutput>;
    fn command_label(&self, config_path: &Path) -> String;
}

#[derive(Debug, Default)]
pub struct RealHyprlandConfigVerifier;

impl HyprlandConfigVerifier for RealHyprlandConfigVerifier {
    fn verify_config(&mut self, config_path: &Path, timeout: Duration) -> Result<VerifyOutput> {
        run_hyprland_verify_config(config_path, timeout)
    }

    fn command_label(&self, config_path: &Path) -> String {
        format!(
            "Hyprland --verify-config --config {}",
            config_path.display()
        )
    }
}

pub fn default_candidates_path() -> PathBuf {
    PathBuf::from(DEFAULT_CANDIDATES_PATH)
}

pub fn default_results_path() -> PathBuf {
    PathBuf::from(DEFAULT_RESULTS_PATH)
}

pub fn load_candidates(path: &Path) -> Result<ConfigPersistenceCandidateReport> {
    let text = fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read config persistence candidates {}",
            path.display()
        )
    })?;
    Ok(serde_json::from_str(&text)?)
}

pub fn save_results(path: &Path, results: &ConfigPersistenceResults) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(results)? + "\n").with_context(|| {
        format!(
            "failed to write config persistence results {}",
            path.display()
        )
    })
}

pub fn run_config_persistence_validation(
    candidate_report: &ConfigPersistenceCandidateReport,
    verify_hyprland: bool,
    verifier: &mut dyn HyprlandConfigVerifier,
    timeout: Duration,
) -> ConfigPersistenceResults {
    let rows = candidate_report
        .rows
        .iter()
        .map(|candidate| validate_candidate(candidate, verify_hyprland, verifier, timeout))
        .collect::<Vec<_>>();
    results_from_rows(
        if verify_hyprland {
            "verify-hyprland"
        } else {
            "dry-run"
        },
        rows,
    )
}

fn validate_candidate(
    candidate: &ConfigPersistenceCandidate,
    verify_hyprland: bool,
    verifier: &mut dyn HyprlandConfigVerifier,
    timeout: Duration,
) -> ConfigPersistenceResult {
    let candidate_value = candidate
        .candidate_value
        .clone()
        .unwrap_or_else(|| "true".to_string());
    let mut notes = Vec::new();
    let (temp_directory, temp_config_path) = match create_temp_config_paths(&candidate.row_id) {
        Ok(paths) => paths,
        Err(error) => {
            return failed_result(candidate, candidate_value, error.to_string());
        }
    };

    let config_key = config_key_from_official_setting(&candidate.official_setting);
    let initial = format!(
        "# Hyprland Settings config-persistence validation fixture\n{config_key} = false\n"
    );
    let parser_roundtrip_passed = fs::write(&temp_config_path, &initial)
        .map(|_| value_roundtrip_matches(&temp_config_path, &candidate.official_setting, "false"))
        .unwrap_or(false);
    if !parser_roundtrip_passed {
        notes.push("parser roundtrip failed before candidate mutation".to_string());
    }

    let typed_validator_passed = validate_candidate_value(
        candidate.control_kind.as_deref().unwrap_or("toggle"),
        &candidate_value,
    );
    if !typed_validator_passed {
        notes.push("typed boolean validator rejected candidate".to_string());
    }

    let writer_roundtrip_passed = if parser_roundtrip_passed && typed_validator_passed {
        match replace_candidate_value(&initial, &config_key, &candidate_value) {
            Ok(updated) => fs::write(&temp_config_path, &updated)
                .map(|_| {
                    value_roundtrip_matches(
                        &temp_config_path,
                        &candidate.official_setting,
                        &candidate_value,
                    )
                })
                .unwrap_or(false),
            Err(error) => {
                notes.push(error.to_string());
                false
            }
        }
    } else {
        false
    };
    if !writer_roundtrip_passed {
        notes.push("writer roundtrip failed after candidate mutation".to_string());
    }

    let single_mutation_verified = fs::read_to_string(&temp_config_path)
        .map(|updated| {
            updated
                == format!(
                    "# Hyprland Settings config-persistence validation fixture\n{config_key} = {candidate_value}\n"
                )
        })
        .unwrap_or(false);
    if !single_mutation_verified {
        notes.push("single mutation verification failed".to_string());
    }

    let mut hyprland_verify_config_attempted = false;
    let mut hyprland_verify_config_passed = false;
    let hyprland_verify_command = verifier.command_label(&temp_config_path);
    let mut hyprland_verify_exit_status = None;
    let mut hyprland_verify_stdout = String::new();
    let mut hyprland_verify_stderr = String::new();

    if verify_hyprland {
        hyprland_verify_config_attempted = true;
        match verifier.verify_config(&temp_config_path, timeout) {
            Ok(output) => {
                hyprland_verify_exit_status = output.exit_status;
                hyprland_verify_stdout = output.stdout;
                hyprland_verify_stderr = output.stderr;
                hyprland_verify_config_passed = !output.timed_out
                    && output.exit_status == Some(0)
                    && output_contains_config_ok(&hyprland_verify_stdout)
                    && !output_mentions_config_error(
                        &hyprland_verify_stdout,
                        &hyprland_verify_stderr,
                    );
                if !hyprland_verify_config_passed {
                    notes.push(
                        "Hyprland --verify-config did not accept the temp config".to_string(),
                    );
                }
            }
            Err(error) => {
                hyprland_verify_stderr = error.to_string();
                notes.push("Hyprland --verify-config failed to run".to_string());
            }
        }
    } else {
        notes.push("dry-run does not attempt Hyprland --verify-config".to_string());
    }

    let safe_to_enable_by_config_persistence = parser_roundtrip_passed
        && writer_roundtrip_passed
        && typed_validator_passed
        && single_mutation_verified
        && hyprland_verify_config_passed;

    ConfigPersistenceResult {
        row_id: candidate.row_id.clone(),
        official_setting: candidate.official_setting.clone(),
        batch: "batch-a-likely-safe-booleans".to_string(),
        candidate_value,
        temp_directory: temp_directory.display().to_string(),
        temp_config_path: temp_config_path.display().to_string(),
        parser_roundtrip_passed,
        writer_roundtrip_passed,
        typed_validator_passed,
        single_mutation_verified,
        hyprland_verify_config_attempted,
        hyprland_verify_config_passed,
        hyprland_verify_command,
        hyprland_verify_exit_status,
        hyprland_verify_stdout,
        hyprland_verify_stderr,
        active_config_modified: false,
        active_runtime_modified: false,
        safe_to_enable_by_config_persistence,
        requires_manual_observation: true,
        notes: if notes.is_empty() {
            "config-persistence proof passed".to_string()
        } else {
            notes.join("; ")
        },
    }
}

fn failed_result(
    candidate: &ConfigPersistenceCandidate,
    candidate_value: String,
    notes: String,
) -> ConfigPersistenceResult {
    ConfigPersistenceResult {
        row_id: candidate.row_id.clone(),
        official_setting: candidate.official_setting.clone(),
        batch: "batch-a-likely-safe-booleans".to_string(),
        candidate_value,
        temp_directory: String::new(),
        temp_config_path: String::new(),
        parser_roundtrip_passed: false,
        writer_roundtrip_passed: false,
        typed_validator_passed: false,
        single_mutation_verified: false,
        hyprland_verify_config_attempted: false,
        hyprland_verify_config_passed: false,
        hyprland_verify_command: String::new(),
        hyprland_verify_exit_status: None,
        hyprland_verify_stdout: String::new(),
        hyprland_verify_stderr: String::new(),
        active_config_modified: false,
        active_runtime_modified: false,
        safe_to_enable_by_config_persistence: false,
        requires_manual_observation: true,
        notes,
    }
}

fn results_from_rows(mode: &str, rows: Vec<ConfigPersistenceResult>) -> ConfigPersistenceResults {
    ConfigPersistenceResults {
        artifact_kind: "config-persistence-validation-results".to_string(),
        schema_version: 1,
        hyprland_version: "0.55.2".to_string(),
        mode: mode.to_string(),
        command_shape: "Hyprland --verify-config --config <temp-file>".to_string(),
        counts: ConfigPersistenceResultCounts {
            rows: rows.len(),
            parser_roundtrip_passed: rows
                .iter()
                .filter(|row| row.parser_roundtrip_passed)
                .count(),
            writer_roundtrip_passed: rows
                .iter()
                .filter(|row| row.writer_roundtrip_passed)
                .count(),
            typed_validator_passed: rows.iter().filter(|row| row.typed_validator_passed).count(),
            single_mutation_verified: rows
                .iter()
                .filter(|row| row.single_mutation_verified)
                .count(),
            hyprland_verify_config_attempted: rows
                .iter()
                .filter(|row| row.hyprland_verify_config_attempted)
                .count(),
            hyprland_verify_config_passed: rows
                .iter()
                .filter(|row| row.hyprland_verify_config_passed)
                .count(),
            safe_to_enable_by_config_persistence: rows
                .iter()
                .filter(|row| row.safe_to_enable_by_config_persistence)
                .count(),
        },
        rows,
    }
}

fn create_temp_config_paths(row_id: &str) -> Result<(PathBuf, PathBuf)> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let sequence = TEMP_CONFIG_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let safe_row = row_id
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-config-persistence-{safe_row}-{}-{stamp}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    let config_path = root.join("hyprland.conf");
    Ok((root, config_path))
}

fn value_roundtrip_matches(path: &Path, official_setting: &str, expected_value: &str) -> bool {
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    let parsed = parse_hyprland_config_text(path, &text);
    let matches = parsed.scalar_records().any(|record| {
        record.normalized_setting_id.as_deref() == Some(official_setting)
            && record.raw_value.as_deref() == Some(expected_value)
    });
    matches
}

fn replace_candidate_value(
    contents: &str,
    config_key: &str,
    candidate_value: &str,
) -> Result<String> {
    let mut replaced = false;
    let mut lines = Vec::new();
    for line in contents.lines() {
        if line.trim_start().starts_with(config_key) && line.contains('=') {
            lines.push(format!("{config_key} = {candidate_value}"));
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !replaced {
        return Err(anyhow!(
            "candidate config key was not present in temp fixture"
        ));
    }
    let mut updated = lines.join("\n");
    updated.push('\n');
    Ok(updated)
}

fn validate_candidate_value(control_kind: &str, value: &str) -> bool {
    match control_kind {
        "toggle" => is_bool_literal(value),
        "percent-slider" => value
            .trim()
            .parse::<f64>()
            .is_ok_and(|number| number.is_finite() && (0.0..=1.0).contains(&number)),
        "slider" | "number-input" => value
            .trim()
            .parse::<f64>()
            .is_ok_and(|number| number.is_finite() && number >= 0.0),
        _ => false,
    }
}

fn is_bool_literal(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "false" | "1" | "0" | "yes" | "no" | "on" | "off"
    )
}

fn output_contains_config_ok(stdout: &str) -> bool {
    strip_ansi(stdout)
        .to_ascii_lowercase()
        .contains("config ok")
}

fn output_mentions_config_error(stdout: &str, stderr: &str) -> bool {
    let combined = strip_ansi(&format!("{stdout}\n{stderr}")).to_ascii_lowercase();
    combined.contains("config error") || combined.contains("failed")
}

fn strip_ansi(text: &str) -> String {
    let mut stripped = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for code in chars.by_ref() {
                if code.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            stripped.push(ch);
        }
    }
    stripped
}

fn run_hyprland_verify_config(config_path: &Path, timeout: Duration) -> Result<VerifyOutput> {
    let mut child = Command::new("Hyprland")
        .arg("--verify-config")
        .arg("--config")
        .arg(config_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to execute Hyprland --verify-config")?;
    let started = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            let output = child.wait_with_output()?;
            return Ok(VerifyOutput {
                exit_status: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                timed_out: false,
            });
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let output = child.wait_with_output()?;
            return Ok(VerifyOutput {
                exit_status: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                timed_out: true,
            });
        }
        thread::sleep(Duration::from_millis(25));
    }
}
