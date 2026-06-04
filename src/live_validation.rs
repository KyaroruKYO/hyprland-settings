use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::config_parser::parse_hyprland_config_text;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveValidationPlan {
    pub artifact_kind: String,
    pub hyprland_version: String,
    pub batch: String,
    pub counts: LiveValidationPlanCounts,
    pub rows: Vec<LiveValidationRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveValidationPlanCounts {
    pub rows: usize,
    pub rollback_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveValidationRow {
    pub row_id: String,
    pub official_setting: String,
    pub batch: String,
    pub value_kind: String,
    pub candidate_values: Vec<String>,
    pub rollback_deadline_seconds: u64,
    pub high_risk: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveValidationResults {
    pub artifact_kind: String,
    pub hyprland_version: String,
    pub mode: String,
    pub counts: LiveValidationResultCounts,
    pub rows: Vec<LiveValidationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveValidationResultCounts {
    pub rows: usize,
    pub level1_passed: usize,
    pub level2_passed: usize,
    pub level3_passed: usize,
    pub level4_passed: usize,
    pub level5_manual_observation: usize,
    pub enabled_rows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveValidationResult {
    pub row_id: String,
    pub official_setting: String,
    pub batch: String,
    pub value_kind: String,
    pub level1_parse_read_status: String,
    pub level2_fixture_write_reread_status: String,
    pub level3_hyprland_accepts_value_status: String,
    pub level4_revert_status: String,
    pub level5_behavior_status: String,
    pub original_live_value: Option<String>,
    pub candidate_values: Vec<String>,
    pub accepted_values: Vec<String>,
    pub rejected_values: Vec<String>,
    pub rollback_watchdog_armed: bool,
    pub rollback_deadline_seconds: u64,
    pub revert_verified: bool,
    pub safe_to_enable: bool,
    pub requires_manual_observation: bool,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyprctlOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub trait HyprctlRunner {
    fn getoption(&mut self, setting: &str) -> Result<HyprctlOutput>;
    fn keyword(&mut self, setting: &str, value: &str) -> Result<HyprctlOutput>;
    fn configerrors(&mut self) -> Result<HyprctlOutput>;
}

pub trait RollbackWatchdog {
    fn arm(&mut self, setting: &str, original_value: &str, timeout_seconds: u64) -> Result<()>;
    fn disarm(&mut self) -> Result<()>;
    fn armed(&self) -> bool;
}

#[derive(Debug, Default)]
pub struct RealHyprctlRunner;

impl HyprctlRunner for RealHyprctlRunner {
    fn getoption(&mut self, setting: &str) -> Result<HyprctlOutput> {
        run_hyprctl(["getoption", setting])
    }

    fn keyword(&mut self, setting: &str, value: &str) -> Result<HyprctlOutput> {
        run_hyprctl(["keyword", setting, value])
    }

    fn configerrors(&mut self) -> Result<HyprctlOutput> {
        run_hyprctl(["configerrors"])
    }
}

fn run_hyprctl<const N: usize>(args: [&str; N]) -> Result<HyprctlOutput> {
    let output = Command::new("hyprctl")
        .args(args)
        .output()
        .context("failed to execute hyprctl")?;
    Ok(HyprctlOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

#[derive(Debug, Default)]
pub struct ProcessRollbackWatchdog {
    child: Option<Child>,
}

impl RollbackWatchdog for ProcessRollbackWatchdog {
    fn arm(&mut self, setting: &str, original_value: &str, timeout_seconds: u64) -> Result<()> {
        if timeout_seconds == 0 || timeout_seconds > 10 {
            return Err(anyhow!("rollback timeout must be between 1 and 10 seconds"));
        }
        if self.child.is_some() {
            return Err(anyhow!("rollback watchdog already armed"));
        }
        let command = format!(
            "sleep {}; hyprctl keyword {} {} >/dev/null 2>&1",
            timeout_seconds,
            shell_quote(setting),
            shell_quote(original_value)
        );
        let child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("failed to arm rollback watchdog")?;
        self.child = Some(child);
        Ok(())
    }

    fn disarm(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    fn armed(&self) -> bool {
        self.child.is_some()
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[derive(Debug, Default)]
pub struct DryRunRollbackWatchdog {
    armed: bool,
}

impl RollbackWatchdog for DryRunRollbackWatchdog {
    fn arm(&mut self, _setting: &str, _original_value: &str, timeout_seconds: u64) -> Result<()> {
        if timeout_seconds == 0 || timeout_seconds > 10 {
            return Err(anyhow!("rollback timeout must be between 1 and 10 seconds"));
        }
        self.armed = true;
        Ok(())
    }

    fn disarm(&mut self) -> Result<()> {
        self.armed = false;
        Ok(())
    }

    fn armed(&self) -> bool {
        self.armed
    }
}

pub fn load_plan(path: &Path) -> Result<LiveValidationPlan> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read live validation plan {}", path.display()))?;
    Ok(serde_json::from_str(&text)?)
}

pub fn save_plan(path: &Path, plan: &LiveValidationPlan) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(plan)? + "\n")
        .with_context(|| format!("failed to write live validation plan {}", path.display()))
}

pub fn save_results(path: &Path, results: &LiveValidationResults) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(results)? + "\n")
        .with_context(|| format!("failed to write live validation results {}", path.display()))
}

pub fn build_batch_a_plan_from_manual_report(path: &Path) -> Result<LiveValidationPlan> {
    let report: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    let items = report["items"]
        .as_array()
        .ok_or_else(|| anyhow!("manual-review report items must be an array"))?;
    let mut rows = Vec::new();
    for item in items {
        if item["recommendedBatch"].as_str() != Some("batch-a-likely-safe-booleans") {
            continue;
        }
        rows.push(LiveValidationRow {
            row_id: required_string(item, "rowId")?,
            official_setting: required_string(item, "officialSetting")?,
            batch: "batch-a-likely-safe-booleans".to_string(),
            value_kind: "boolean".to_string(),
            candidate_values: vec!["true".to_string(), "false".to_string()],
            rollback_deadline_seconds: 5,
            high_risk: false,
            notes: "Batch A boolean live validation candidate; planning does not enable writes."
                .to_string(),
        });
    }
    Ok(LiveValidationPlan {
        artifact_kind: "live-validation-plan".to_string(),
        hyprland_version: "0.55.2".to_string(),
        batch: "batch-a-likely-safe-booleans".to_string(),
        counts: LiveValidationPlanCounts {
            rows: rows.len(),
            rollback_timeout_seconds: 5,
        },
        rows,
    })
}

fn required_string(item: &serde_json::Value, key: &str) -> Result<String> {
    item[key]
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("missing {key}"))
}

pub fn run_dry_validation(plan: &LiveValidationPlan) -> LiveValidationResults {
    let rows = plan
        .rows
        .iter()
        .map(|row| {
            let level1 = level1_parse_read(row);
            let level2 = level2_fixture_write_reread(row);
            LiveValidationResult {
                row_id: row.row_id.clone(),
                official_setting: row.official_setting.clone(),
                batch: row.batch.clone(),
                value_kind: row.value_kind.clone(),
                level1_parse_read_status: status_label(level1),
                level2_fixture_write_reread_status: status_label(level2),
                level3_hyprland_accepts_value_status: "not-run-dry-run".to_string(),
                level4_revert_status: "not-run-dry-run".to_string(),
                level5_behavior_status: "requires-manual-observation".to_string(),
                original_live_value: None,
                candidate_values: row.candidate_values.clone(),
                accepted_values: Vec::new(),
                rejected_values: Vec::new(),
                rollback_watchdog_armed: false,
                rollback_deadline_seconds: row.rollback_deadline_seconds,
                revert_verified: false,
                safe_to_enable: false,
                requires_manual_observation: true,
                notes: "Dry run proves Levels 1-2 only; no Hyprland runtime mutation attempted."
                    .to_string(),
            }
        })
        .collect::<Vec<_>>();
    results_from_rows("dry-run", rows)
}

pub fn run_live_validation<R: HyprctlRunner, W: RollbackWatchdog>(
    plan: &LiveValidationPlan,
    runner: &mut R,
    watchdog: &mut W,
) -> LiveValidationResults {
    let mut rows = Vec::new();
    for row in &plan.rows {
        eprintln!("live-validating {} ({})", row.row_id, row.official_setting);
        let level1 = level1_parse_read(row);
        let level2 = level2_fixture_write_reread(row);
        let mut result = LiveValidationResult {
            row_id: row.row_id.clone(),
            official_setting: row.official_setting.clone(),
            batch: row.batch.clone(),
            value_kind: row.value_kind.clone(),
            level1_parse_read_status: status_label(level1),
            level2_fixture_write_reread_status: status_label(level2),
            level3_hyprland_accepts_value_status: "not-run".to_string(),
            level4_revert_status: "not-run".to_string(),
            level5_behavior_status: "requires-manual-observation".to_string(),
            original_live_value: None,
            candidate_values: row.candidate_values.clone(),
            accepted_values: Vec::new(),
            rejected_values: Vec::new(),
            rollback_watchdog_armed: false,
            rollback_deadline_seconds: row.rollback_deadline_seconds,
            revert_verified: false,
            safe_to_enable: false,
            requires_manual_observation: true,
            notes: String::new(),
        };

        let original = match runner.getoption(&row.official_setting) {
            Ok(output) if output.success => match parse_hyprctl_value(&output.stdout) {
                Some(value) => value,
                None => {
                    result.level3_hyprland_accepts_value_status =
                        "blocked-original-value-unparsed".to_string();
                    result.level4_revert_status = "not-run".to_string();
                    result.notes =
                        "Could not parse original value from hyprctl getoption output.".to_string();
                    rows.push(result);
                    continue;
                }
            },
            Ok(output) => {
                result.level3_hyprland_accepts_value_status =
                    "blocked-getoption-failed".to_string();
                result.notes = output.stderr;
                rows.push(result);
                continue;
            }
            Err(error) => {
                result.level3_hyprland_accepts_value_status = "blocked-getoption-error".to_string();
                result.notes = error.to_string();
                rows.push(result);
                continue;
            }
        };
        result.original_live_value = Some(original.clone());
        let candidate = opposite_bool(&original).unwrap_or_else(|| row.candidate_values[0].clone());

        if let Err(error) = watchdog.arm(
            &row.official_setting,
            &original,
            row.rollback_deadline_seconds,
        ) {
            result.level3_hyprland_accepts_value_status = "blocked-watchdog-not-armed".to_string();
            result.level4_revert_status = "not-run".to_string();
            result.notes = error.to_string();
            rows.push(result);
            continue;
        }
        result.rollback_watchdog_armed = watchdog.armed();

        let apply = runner.keyword(&row.official_setting, &candidate);
        let accepted = match apply {
            Ok(output) if output.success => {
                let after = runner.getoption(&row.official_setting).ok();
                let observed = after
                    .as_ref()
                    .and_then(|output| parse_hyprctl_value(&output.stdout))
                    .is_some_and(|value| values_equivalent(&value, &candidate));
                if !observed {
                    result.rejected_values.push(candidate.clone());
                }
                observed
            }
            Ok(output) => {
                result.rejected_values.push(candidate.clone());
                result.notes = output.stderr;
                false
            }
            Err(error) => {
                result.rejected_values.push(candidate.clone());
                result.notes = error.to_string();
                false
            }
        };
        if accepted {
            result.accepted_values.push(candidate);
            result.level3_hyprland_accepts_value_status = "passed".to_string();
        } else {
            result.level3_hyprland_accepts_value_status = "rejected".to_string();
        }

        let revert = runner.keyword(&row.official_setting, &original);
        let restored = revert.ok().is_some_and(|output| output.success)
            && runner
                .getoption(&row.official_setting)
                .ok()
                .and_then(|output| parse_hyprctl_value(&output.stdout))
                .is_some_and(|value| values_equivalent(&value, &original));
        result.revert_verified = restored;
        result.level4_revert_status = if restored { "passed" } else { "failed" }.to_string();
        result.safe_to_enable = level1 && level2 && accepted && restored;
        result.notes = if result.safe_to_enable {
            "Live runtime accepted the candidate and rollback verification restored the original value."
                .to_string()
        } else if result.notes.is_empty() {
            "Live runtime probe did not pass all gates.".to_string()
        } else {
            result.notes
        };
        if restored {
            let _ = watchdog.disarm();
        }
        let should_stop = !restored;
        rows.push(result);
        if should_stop {
            break;
        }
    }
    results_from_rows("live", rows)
}

fn level1_parse_read(row: &LiveValidationRow) -> bool {
    let key = row.official_setting.replace('.', ":");
    let text = format!("{key} = true\n");
    let parsed = parse_hyprland_config_text("/tmp/live-validation-level1.conf", &text);
    let found = parsed
        .scalar_records()
        .any(|record| record.normalized_setting_id.as_deref() == Some(&row.official_setting));
    found
}

fn level2_fixture_write_reread(row: &LiveValidationRow) -> bool {
    let key = row.official_setting.replace('.', ":");
    row.candidate_values.iter().all(|candidate| {
        let text = format!("{key} = {candidate}\n");
        let parsed = parse_hyprland_config_text("/tmp/live-validation-level2.conf", &text);
        let found = parsed.scalar_records().any(|record| {
            record.normalized_setting_id.as_deref() == Some(&row.official_setting)
                && record.raw_value.as_deref() == Some(candidate.as_str())
        });
        found
    })
}

fn results_from_rows(mode: &str, rows: Vec<LiveValidationResult>) -> LiveValidationResults {
    LiveValidationResults {
        artifact_kind: "live-validation-results".to_string(),
        hyprland_version: "0.55.2".to_string(),
        mode: mode.to_string(),
        counts: LiveValidationResultCounts {
            rows: rows.len(),
            level1_passed: rows
                .iter()
                .filter(|row| row.level1_parse_read_status == "passed")
                .count(),
            level2_passed: rows
                .iter()
                .filter(|row| row.level2_fixture_write_reread_status == "passed")
                .count(),
            level3_passed: rows
                .iter()
                .filter(|row| row.level3_hyprland_accepts_value_status == "passed")
                .count(),
            level4_passed: rows
                .iter()
                .filter(|row| row.level4_revert_status == "passed")
                .count(),
            level5_manual_observation: rows
                .iter()
                .filter(|row| row.requires_manual_observation)
                .count(),
            enabled_rows: 0,
        },
        rows,
    }
}

fn status_label(passed: bool) -> String {
    if passed { "passed" } else { "failed" }.to_string()
}

pub fn parse_hyprctl_value(output: &str) -> Option<String> {
    let mut typed_values = BTreeMap::new();
    for line in output.lines() {
        let trimmed = line.trim();
        for prefix in ["int:", "float:", "str:", "data:", "set:"] {
            if let Some(value) = trimmed.strip_prefix(prefix) {
                typed_values.insert(prefix, value.trim().to_string());
            }
        }
    }
    for prefix in ["int:", "float:", "str:", "data:", "set:"] {
        if let Some(value) = typed_values.get(prefix) {
            return Some(value.clone());
        }
    }
    output.lines().find_map(|line| {
        line.split_once(':')
            .map(|(_, value)| value.trim().to_string())
    })
}

fn opposite_bool(value: &str) -> Option<String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some("false".to_string()),
        "0" | "false" | "no" | "off" => Some("true".to_string()),
        _ => None,
    }
}

fn values_equivalent(left: &str, right: &str) -> bool {
    if left.trim() == right.trim() {
        return true;
    }
    match (bool_value(left), bool_value(right)) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

fn bool_value(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub fn known_plan_ids(plan: &LiveValidationPlan) -> BTreeSet<String> {
    plan.rows.iter().map(|row| row.row_id.clone()).collect()
}

pub fn default_plan_path() -> PathBuf {
    PathBuf::from("data/reports/live-validation-plan.v0.55.2.json")
}

pub fn default_results_path() -> PathBuf {
    PathBuf::from("data/reports/live-validation-results.v0.55.2.json")
}
