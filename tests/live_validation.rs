use anyhow::{anyhow, Result};
use hyprland_settings::live_validation::{
    known_plan_ids, parse_hyprctl_value, run_live_validation, DryRunRollbackWatchdog,
    HyprctlOutput, HyprctlRunner, LiveValidationPlan, LiveValidationPlanCounts, LiveValidationRow,
    RollbackWatchdog,
};

fn one_row_plan() -> LiveValidationPlan {
    LiveValidationPlan {
        artifact_kind: "live-validation-plan".to_string(),
        hyprland_version: "0.55.2".to_string(),
        batch: "batch-a-likely-safe-booleans".to_string(),
        counts: LiveValidationPlanCounts {
            rows: 1,
            rollback_timeout_seconds: 5,
        },
        rows: vec![LiveValidationRow {
            row_id: "misc.disable_hyprland_logo".to_string(),
            official_setting: "misc.disable_hyprland_logo".to_string(),
            batch: "batch-a-likely-safe-booleans".to_string(),
            value_kind: "boolean".to_string(),
            candidate_values: vec!["true".to_string(), "false".to_string()],
            rollback_deadline_seconds: 5,
            high_risk: false,
            notes: "test".to_string(),
        }],
    }
}

#[derive(Debug)]
struct FakeRunner {
    value: String,
    fail_keyword: bool,
    calls: Vec<String>,
}

impl FakeRunner {
    fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
            fail_keyword: false,
            calls: Vec::new(),
        }
    }
}

impl HyprctlRunner for FakeRunner {
    fn getoption(&mut self, setting: &str) -> Result<HyprctlOutput> {
        self.calls.push(format!("getoption {setting}"));
        Ok(HyprctlOutput {
            success: true,
            stdout: format!("int: {}\n", self.value),
            stderr: String::new(),
        })
    }

    fn keyword(&mut self, setting: &str, value: &str) -> Result<HyprctlOutput> {
        self.calls.push(format!("keyword {setting} {value}"));
        if self.fail_keyword {
            return Ok(HyprctlOutput {
                success: false,
                stdout: String::new(),
                stderr: "rejected".to_string(),
            });
        }
        self.value = value.to_string();
        Ok(HyprctlOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        })
    }

    fn configerrors(&mut self) -> Result<HyprctlOutput> {
        self.calls.push("configerrors".to_string());
        Ok(HyprctlOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

#[derive(Debug, Default)]
struct FakeWatchdog {
    armed: bool,
    fail_arm: bool,
    calls: Vec<String>,
}

impl RollbackWatchdog for FakeWatchdog {
    fn arm(&mut self, setting: &str, original_value: &str, timeout_seconds: u64) -> Result<()> {
        self.calls
            .push(format!("arm {setting} {original_value} {timeout_seconds}"));
        if self.fail_arm {
            return Err(anyhow!("watchdog unavailable"));
        }
        self.armed = true;
        Ok(())
    }

    fn disarm(&mut self) -> Result<()> {
        self.calls.push("disarm".to_string());
        self.armed = false;
        Ok(())
    }

    fn armed(&self) -> bool {
        self.armed
    }
}

#[test]
fn dry_watchdog_rejects_deadlines_over_ten_seconds() {
    let mut watchdog = DryRunRollbackWatchdog::default();

    assert!(watchdog
        .arm("misc.disable_hyprland_logo", "false", 11)
        .is_err());
    assert!(!watchdog.armed());
}

#[test]
fn live_validation_arms_watchdog_before_apply_and_reverts() {
    let plan = one_row_plan();
    let mut runner = FakeRunner::new("false");
    let mut watchdog = FakeWatchdog::default();

    let results = run_live_validation(&plan, &mut runner, &mut watchdog);

    assert_eq!(results.counts.level1_passed, 1);
    assert_eq!(results.counts.level2_passed, 1);
    assert_eq!(results.counts.level3_passed, 1);
    assert_eq!(results.counts.level4_passed, 1);
    assert_eq!(results.rows[0].accepted_values, vec!["true"]);
    assert_eq!(
        results.rows[0].original_live_value.as_deref(),
        Some("false")
    );
    assert!(results.rows[0].rollback_watchdog_armed);
    assert!(results.rows[0].revert_verified);
    assert!(results.rows[0].safe_to_enable);
    assert_eq!(
        watchdog.calls,
        vec!["arm misc.disable_hyprland_logo false 5", "disarm"]
    );
    assert_eq!(
        runner.calls,
        vec![
            "getoption misc.disable_hyprland_logo",
            "keyword misc.disable_hyprland_logo true",
            "getoption misc.disable_hyprland_logo",
            "keyword misc.disable_hyprland_logo false",
            "getoption misc.disable_hyprland_logo",
        ]
    );
}

#[test]
fn live_validation_blocks_when_watchdog_cannot_arm() {
    let plan = one_row_plan();
    let mut runner = FakeRunner::new("false");
    let mut watchdog = FakeWatchdog {
        fail_arm: true,
        ..FakeWatchdog::default()
    };

    let results = run_live_validation(&plan, &mut runner, &mut watchdog);

    assert_eq!(
        results.rows[0].level3_hyprland_accepts_value_status,
        "blocked-watchdog-not-armed"
    );
    assert_eq!(results.rows[0].level4_revert_status, "not-run");
    assert!(!results.rows[0].rollback_watchdog_armed);
    assert_eq!(runner.calls, vec!["getoption misc.disable_hyprland_logo"]);
}

#[test]
fn live_validation_logs_rejected_candidate_and_still_reverts() {
    let plan = one_row_plan();
    let mut runner = FakeRunner::new("false");
    runner.fail_keyword = true;
    let mut watchdog = FakeWatchdog::default();

    let results = run_live_validation(&plan, &mut runner, &mut watchdog);

    assert_eq!(
        results.rows[0].level3_hyprland_accepts_value_status,
        "rejected"
    );
    assert_eq!(results.rows[0].rejected_values, vec!["true"]);
    assert_eq!(results.rows[0].level4_revert_status, "failed");
    assert!(!results.rows[0].safe_to_enable);
}

#[test]
fn hyprctl_value_parser_extracts_typed_values() {
    assert_eq!(parse_hyprctl_value("int: 1\n").as_deref(), Some("1"));
    assert_eq!(
        parse_hyprctl_value("str: hello\n").as_deref(),
        Some("hello")
    );
    assert_eq!(
        parse_hyprctl_value("option type: custom type\nset: true\n").as_deref(),
        Some("true")
    );
}

#[test]
fn plan_ids_are_unique_set() {
    let plan = one_row_plan();

    assert!(known_plan_ids(&plan).contains("misc.disable_hyprland_logo"));
}
