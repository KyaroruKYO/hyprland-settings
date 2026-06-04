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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LiveValidationDiagnostics {
    pub artifact_kind: String,
    pub schema_version: u32,
    pub hyprland_version: String,
    pub source_plan: String,
    pub source_results: String,
    pub counts: LiveValidationDiagnosticCounts,
    pub diagnosis_summary: String,
    pub items: Vec<LiveValidationDiagnosticItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LiveValidationDiagnosticCounts {
    pub rows: usize,
    pub accepted: usize,
    pub revert_verified: usize,
    pub safe_to_retest: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LiveValidationDiagnosticItem {
    pub row_id: String,
    pub official_setting: String,
    pub runtime_setting: String,
    pub original_getoption_raw: String,
    pub original_parsed_value: Option<String>,
    pub candidate_value: Option<String>,
    pub rollback_watchdog_armed: bool,
    pub keyword_exit_success: bool,
    pub keyword_stdout: String,
    pub keyword_stderr: String,
    pub post_apply_getoption_raw: String,
    pub post_apply_parsed_value: Option<String>,
    pub values_equivalent: bool,
    pub config_errors_raw: String,
    pub revert_keyword_exit_success: bool,
    pub post_revert_getoption_raw: String,
    pub post_revert_parsed_value: Option<String>,
    pub revert_verified: bool,
    pub diagnosis: String,
    pub recommended_harness_fix: String,
    pub safe_to_retest: bool,
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

pub fn save_diagnostics(path: &Path, diagnostics: &LiveValidationDiagnostics) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(diagnostics)? + "\n").with_context(|| {
        format!(
            "failed to write live validation diagnostics {}",
            path.display()
        )
    })
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
        let runtime_setting = runtime_setting_name(&row.official_setting);
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

        let original = match runner.getoption(&runtime_setting) {
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

        if let Err(error) = watchdog.arm(&runtime_setting, &original, row.rollback_deadline_seconds)
        {
            result.level3_hyprland_accepts_value_status = "blocked-watchdog-not-armed".to_string();
            result.level4_revert_status = "not-run".to_string();
            result.notes = error.to_string();
            rows.push(result);
            continue;
        }
        result.rollback_watchdog_armed = watchdog.armed();

        let apply = runner.keyword(&runtime_setting, &candidate);
        let accepted = match apply {
            Ok(output) if output.success => {
                let after = runner.getoption(&runtime_setting).ok();
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

        let revert = runner.keyword(&runtime_setting, &original);
        let restored = revert.ok().is_some_and(|output| output.success)
            && runner
                .getoption(&runtime_setting)
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

pub fn run_live_diagnostics<R: HyprctlRunner, W: RollbackWatchdog>(
    plan: &LiveValidationPlan,
    selected_row_ids: &BTreeSet<String>,
    timeout_override: Option<u64>,
    runner: &mut R,
    watchdog: &mut W,
) -> LiveValidationDiagnostics {
    let mut items = Vec::new();
    for row in &plan.rows {
        if !selected_row_ids.contains(&row.row_id) {
            continue;
        }
        eprintln!("live-diagnosing {} ({})", row.row_id, row.official_setting);
        let item = diagnose_one_row(row, timeout_override, runner, watchdog);
        let should_stop = item.rollback_watchdog_armed && !item.revert_verified;
        items.push(item);
        if should_stop {
            break;
        }
    }
    diagnostics_from_items(items)
}

fn diagnose_one_row<R: HyprctlRunner, W: RollbackWatchdog>(
    row: &LiveValidationRow,
    timeout_override: Option<u64>,
    runner: &mut R,
    watchdog: &mut W,
) -> LiveValidationDiagnosticItem {
    let runtime_setting = runtime_setting_name(&row.official_setting);
    let timeout_seconds = timeout_override.unwrap_or(row.rollback_deadline_seconds);
    let mut item = LiveValidationDiagnosticItem {
        row_id: row.row_id.clone(),
        official_setting: row.official_setting.clone(),
        runtime_setting: runtime_setting.clone(),
        original_getoption_raw: String::new(),
        original_parsed_value: None,
        candidate_value: None,
        rollback_watchdog_armed: false,
        keyword_exit_success: false,
        keyword_stdout: String::new(),
        keyword_stderr: String::new(),
        post_apply_getoption_raw: String::new(),
        post_apply_parsed_value: None,
        values_equivalent: false,
        config_errors_raw: String::new(),
        revert_keyword_exit_success: false,
        post_revert_getoption_raw: String::new(),
        post_revert_parsed_value: None,
        revert_verified: false,
        diagnosis: String::new(),
        recommended_harness_fix: String::new(),
        safe_to_retest: false,
    };

    let original_output = match runner.getoption(&runtime_setting) {
        Ok(output) if output.success => output,
        Ok(output) => {
            item.original_getoption_raw = output.stdout;
            item.keyword_stderr = output.stderr;
            item.diagnosis = "getoption-failed-before-apply".to_string();
            item.recommended_harness_fix = "verify runtime setting name".to_string();
            return item;
        }
        Err(error) => {
            item.diagnosis = "getoption-error-before-apply".to_string();
            item.recommended_harness_fix = error.to_string();
            return item;
        }
    };
    item.original_getoption_raw = original_output.stdout.clone();
    let Some(original) = parse_hyprctl_value(&original_output.stdout) else {
        item.diagnosis = "original-value-unparsed".to_string();
        item.recommended_harness_fix = "extend hyprctl getoption parser".to_string();
        return item;
    };
    item.original_parsed_value = Some(original.clone());
    let candidate = opposite_bool(&original).unwrap_or_else(|| row.candidate_values[0].clone());
    item.candidate_value = Some(candidate.clone());

    if let Err(error) = watchdog.arm(&runtime_setting, &original, timeout_seconds) {
        item.diagnosis = "watchdog-not-armed".to_string();
        item.recommended_harness_fix = error.to_string();
        return item;
    }
    item.rollback_watchdog_armed = watchdog.armed();

    match runner.keyword(&runtime_setting, &candidate) {
        Ok(output) => {
            item.keyword_exit_success = output.success;
            item.keyword_stdout = output.stdout;
            item.keyword_stderr = output.stderr;
        }
        Err(error) => {
            item.keyword_stderr = error.to_string();
        }
    }

    if let Ok(output) = runner.getoption(&runtime_setting) {
        item.post_apply_getoption_raw = output.stdout.clone();
        item.post_apply_parsed_value = parse_hyprctl_value(&output.stdout);
    }
    item.values_equivalent = item
        .post_apply_parsed_value
        .as_deref()
        .is_some_and(|value| values_equivalent(value, &candidate));

    if let Ok(output) = runner.configerrors() {
        item.config_errors_raw = if output.stdout.trim().is_empty() {
            output.stderr
        } else {
            output.stdout
        };
    }

    match runner.keyword(&runtime_setting, &original) {
        Ok(output) => {
            item.revert_keyword_exit_success = output.success;
            if !output.stderr.trim().is_empty() {
                item.keyword_stderr = join_nonempty(&item.keyword_stderr, &output.stderr);
            }
        }
        Err(error) => {
            item.keyword_stderr = join_nonempty(&item.keyword_stderr, &error.to_string());
        }
    }
    if let Ok(output) = runner.getoption(&runtime_setting) {
        item.post_revert_getoption_raw = output.stdout.clone();
        item.post_revert_parsed_value = parse_hyprctl_value(&output.stdout);
    }
    item.revert_verified = item.revert_keyword_exit_success
        && item
            .post_revert_parsed_value
            .as_deref()
            .is_some_and(|value| values_equivalent(value, &original));

    if item.revert_verified {
        let _ = watchdog.disarm();
    }
    item.safe_to_retest = item.revert_verified;
    item.diagnosis = if item.values_equivalent {
        "accepted-value-detected".to_string()
    } else if !item.keyword_exit_success {
        "keyword-rejected-or-failed".to_string()
    } else if item.post_apply_parsed_value == item.original_parsed_value {
        "keyword-succeeded-but-getoption-stayed-original".to_string()
    } else {
        "keyword-succeeded-but-post-apply-value-did-not-match-candidate".to_string()
    };
    item.recommended_harness_fix = if item.values_equivalent {
        "no-level3-parser-fix-needed-for-this-row".to_string()
    } else if !item.keyword_exit_success {
        "inspect-keyword-stderr-and-runtime-setting-name".to_string()
    } else {
        "inspect-raw-getoption-output-and-accepted-value-normalization".to_string()
    };

    item
}

fn diagnostics_from_items(items: Vec<LiveValidationDiagnosticItem>) -> LiveValidationDiagnostics {
    LiveValidationDiagnostics {
        artifact_kind: "live-validation-level3-diagnostics".to_string(),
        schema_version: 1,
        hyprland_version: "0.55.2".to_string(),
        source_plan: "data/reports/live-validation-plan.v0.55.2.json".to_string(),
        source_results: "data/reports/live-validation-results.v0.55.2.json".to_string(),
        counts: LiveValidationDiagnosticCounts {
            rows: items.len(),
            accepted: items.iter().filter(|item| item.values_equivalent).count(),
            revert_verified: items.iter().filter(|item| item.revert_verified).count(),
            safe_to_retest: items.iter().filter(|item| item.safe_to_retest).count(),
        },
        diagnosis_summary: "Diagnostic mode captures raw hyprctl output and does not enable rows."
            .to_string(),
        items,
    }
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

pub fn runtime_setting_name(official_setting: &str) -> String {
    official_setting.replace('.', ":")
}

fn join_nonempty(left: &str, right: &str) -> String {
    match (left.trim().is_empty(), right.trim().is_empty()) {
        (true, true) => String::new(),
        (true, false) => right.to_string(),
        (false, true) => left.to_string(),
        (false, false) => format!("{left}\n{right}"),
    }
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

pub fn default_diagnostics_path() -> PathBuf {
    PathBuf::from("data/reports/live-validation-level3-diagnostics.v0.55.2.json")
}
