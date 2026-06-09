use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::blocked_row_pre_enablement::{
    blocked_pre_enablement_rows, valid_pre_enablement_example,
};
use hyprland_settings::high_risk_persisted_recovery::{
    accept_confirmation_token, create_temp_config_backup, create_temp_recovery_plan,
    decide_recovery_action, high_risk_recovery_rows, load_recovery_plan, persist_recovery_plan,
    recovery_bucket_requirements, refuse_live_target_execution, restore_temp_config_from_backup,
    validate_recovery_plan, HighRiskRecoveryBucket, HighRiskRecoveryDecision,
    HighRiskRecoveryPlanError,
};
use hyprland_settings::write_classification::{
    config_key_from_official_setting, is_high_risk_gated_writable_setting,
    is_safe_writable_setting, SAFE_WRITABLE_ROWS,
};
use serde_json::Value;

const SCAFFOLD_REPORT: &str = "data/reports/high-risk-persisted-recovery-scaffold.v0.55.2.json";
const SCAFFOLD_TESTS_REPORT: &str =
    "data/reports/high-risk-persisted-recovery-scaffold-tests.v0.55.2.json";
const SCAFFOLD_BLOCKERS_REPORT: &str =
    "data/reports/high-risk-persisted-recovery-scaffold-blockers.v0.55.2.json";
const REVIEW_LOG: &str = "docs/HIGH-RISK-PERSISTED-RECOVERY-SCAFFOLD-REVIEW-LOG.md";

fn temp_case(name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprland-settings-persisted-recovery-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn report_row_ids(rows: &[Value]) -> Vec<String> {
    let mut ids = rows
        .iter()
        .map(|row| row["rowId"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn expected_row_ids() -> Vec<String> {
    let mut ids = blocked_pre_enablement_rows()
        .iter()
        .map(|row| row.row_id.to_string())
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn sample_plan(
    row_id: &str,
) -> Result<hyprland_settings::high_risk_persisted_recovery::HighRiskRecoveryPlan> {
    let row = blocked_pre_enablement_rows()
        .iter()
        .find(|row| row.row_id == row_id)
        .expect("test row must exist");
    let dir = temp_case(row_id.replace('.', "-").as_str())?;
    let config_path = dir.join("hyprland.conf");
    let backup_path = dir.join("backup").join("hyprland.conf.bak");
    let config_key = config_key_from_official_setting(row.official_setting);
    let previous = valid_pre_enablement_example(row);
    let proposed = valid_pre_enablement_example(row);
    fs::write(&config_path, format!("{config_key} = {previous}\n"))?;
    Ok(create_temp_recovery_plan(
        row.row_id,
        proposed,
        Some(previous),
        &config_path,
        &backup_path,
        1_700_000_000,
        None,
        30,
    )?)
}

#[test]
fn scaffold_classifies_all_63_rows_into_required_buckets() {
    let rows = high_risk_recovery_rows();
    assert_eq!(rows.len(), 63);

    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for row in &rows {
        *counts.entry(row.bucket.as_str()).or_default() += 1;
        assert_eq!(row.recovery_model, row.bucket.recovery_model());
        assert!(is_safe_writable_setting(&row.row_id));
        assert!(is_high_risk_gated_writable_setting(&row.row_id));
    }

    assert_eq!(counts["display/render"], 23);
    assert_eq!(counts["cursor/input"], 18);
    assert_eq!(counts["debug/crash"], 22);
    assert!(rows
        .iter()
        .any(|row| row.row_id == "cursor.default_monitor" && row.runtime_dynamic_special_case));
    assert!(rows
        .iter()
        .filter(|row| row.runtime_dynamic_special_case)
        .all(|row| row.row_id == "cursor.default_monitor"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn bucket_requirements_encode_forbidden_dependencies() {
    let display = recovery_bucket_requirements(HighRiskRecoveryBucket::DisplayRender);
    assert!(display
        .must_not_depend_on
        .contains(&"visible compositor output"));
    assert!(display
        .must_not_depend_on
        .contains(&"screen remains readable"));

    let cursor = recovery_bucket_requirements(HighRiskRecoveryBucket::CursorInput);
    for forbidden in [
        "pointer visibility",
        "mouse input",
        "app UI",
        "Hyprland keybinds",
    ] {
        assert!(cursor.must_not_depend_on.contains(&forbidden));
    }

    let debug = recovery_bucket_requirements(HighRiskRecoveryBucket::DebugCrash);
    for forbidden in [
        "process that may be disrupted",
        "active compositor health",
        "app UI running inside the affected session",
        "Hyprland keybinds",
    ] {
        assert!(debug.must_not_depend_on.contains(&forbidden));
    }
}

#[test]
fn valid_recovery_plan_validates_and_roundtrips_from_temp_persisted_plan() -> Result<()> {
    let plan = sample_plan("render.direct_scanout")?;
    let validation = validate_recovery_plan(&plan);
    assert!(validation.valid, "{:?}", validation.errors);
    assert_eq!(plan.bucket, HighRiskRecoveryBucket::DisplayRender);
    assert_eq!(
        plan.recovery_model,
        HighRiskRecoveryBucket::DisplayRender.recovery_model()
    );
    assert!(!plan.live_execution_enabled);
    assert!(plan.temp_test_only);

    let plan_path = plan
        .target_config_path
        .parent()
        .unwrap()
        .join("persisted-plan.json");
    persist_recovery_plan(&plan_path, &plan)?;
    let loaded = load_recovery_plan(&plan_path)?;
    assert_eq!(loaded.plan_id, plan.plan_id);
    assert_eq!(loaded.row_id, "render.direct_scanout");

    Ok(())
}

#[test]
fn invalid_recovery_plans_are_rejected_with_exact_validation_errors() -> Result<()> {
    let plan = sample_plan("cursor.no_warps")?;

    let mut missing_row = plan.clone();
    missing_row.row_id.clear();
    assert!(validate_recovery_plan(&missing_row)
        .errors
        .contains(&HighRiskRecoveryPlanError::MissingRowId));

    let mut missing_official = plan.clone();
    missing_official.official_setting.clear();
    assert!(validate_recovery_plan(&missing_official)
        .errors
        .contains(&HighRiskRecoveryPlanError::MissingOfficialSettingKey));

    let mut missing_target = plan.clone();
    missing_target.target_config_path = PathBuf::new();
    assert!(validate_recovery_plan(&missing_target)
        .errors
        .contains(&HighRiskRecoveryPlanError::MissingTargetPath));

    let mut missing_backup = plan.clone();
    missing_backup.backup_config_path = PathBuf::new();
    assert!(validate_recovery_plan(&missing_backup)
        .errors
        .contains(&HighRiskRecoveryPlanError::MissingBackupPath));

    let mut wrong_bucket = plan.clone();
    wrong_bucket.bucket = HighRiskRecoveryBucket::DebugCrash;
    assert!(validate_recovery_plan(&wrong_bucket)
        .errors
        .iter()
        .any(|error| matches!(error, HighRiskRecoveryPlanError::BucketMismatch { .. })));

    assert!(matches!(
        create_temp_recovery_plan(
            "windows.snap.enabled",
            "true",
            Some("false".to_string()),
            &plan.target_config_path,
            &plan.backup_config_path,
            1,
            None,
            30
        ),
        Err(HighRiskRecoveryPlanError::NonHighRiskRow(_))
    ));

    let mut non_temp_target = plan.clone();
    non_temp_target.target_config_path = PathBuf::from("/home/kyo/not-real-hyprland.conf");
    assert!(validate_recovery_plan(&non_temp_target)
        .errors
        .iter()
        .any(|error| matches!(error, HighRiskRecoveryPlanError::TargetPathNotTemp(_))));

    let mut not_marked_temp = plan.clone();
    not_marked_temp.temp_test_only = false;
    assert!(validate_recovery_plan(&not_marked_temp)
        .errors
        .iter()
        .any(|error| matches!(error, HighRiskRecoveryPlanError::TargetPathNotTemp(_))));

    Ok(())
}

#[test]
fn temp_backup_restore_and_parser_reread_work_without_real_config_paths() -> Result<()> {
    let plan = sample_plan("debug.damage_tracking")?;
    let original = fs::read_to_string(&plan.target_config_path)?;

    let backup = create_temp_config_backup(&plan)?;
    assert!(backup.target_config_path.starts_with(std::env::temp_dir()));
    assert!(backup.backup_config_path.starts_with(std::env::temp_dir()));
    assert!(backup.backup_matches_target_before_mutation);
    assert!(backup.bytes_copied > 0);

    fs::write(&plan.target_config_path, "debug:damage_tracking = 2\n")?;
    assert_ne!(fs::read_to_string(&plan.target_config_path)?, original);

    let restore = restore_temp_config_from_backup(&plan)?;
    assert!(restore.restore_written);
    assert!(restore.parser_reread_succeeded);
    assert_eq!(restore.restored_value, plan.previous_value);
    assert_eq!(fs::read_to_string(&plan.target_config_path)?, original);

    Ok(())
}

#[test]
fn confirmation_token_and_timeout_decisions_are_temp_only_and_non_authoritative() -> Result<()> {
    let plan = sample_plan("cursor.invisible")?;

    assert_eq!(
        accept_confirmation_token(&plan, plan.confirmation_token.as_str())?,
        HighRiskRecoveryDecision::KeepApply
    );
    assert!(matches!(
        accept_confirmation_token(&plan, "wrong-token"),
        Err(HighRiskRecoveryPlanError::WrongConfirmationToken)
    ));
    assert_eq!(
        decide_recovery_action(&plan, plan.created_unix_seconds + 1, None)?,
        HighRiskRecoveryDecision::AwaitConfirmation
    );
    assert_eq!(
        decide_recovery_action(&plan, plan.confirmation_deadline_unix_seconds, None)?,
        HighRiskRecoveryDecision::Rollback
    );
    assert_eq!(
        decide_recovery_action(
            &plan,
            plan.created_unix_seconds + 1,
            Some(plan.confirmation_token.as_str())
        )?,
        HighRiskRecoveryDecision::KeepApply
    );

    Ok(())
}

#[test]
fn live_target_execution_is_refused_while_disabled() -> Result<()> {
    let plan = sample_plan("xwayland.enabled")?;
    assert!(matches!(
        refuse_live_target_execution(&plan),
        Err(HighRiskRecoveryPlanError::LiveExecutionDisabled)
    ));
    assert!(!plan.live_execution_enabled);
    assert!(plan.target_config_path.starts_with(std::env::temp_dir()));
    Ok(())
}

#[test]
fn screen_shader_remains_closed_and_counts_are_unchanged() -> Result<()> {
    let closure = read_json("data/reports/screen-shader-track-closure.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(closure["screenShaderTrackClosedForNow"], true);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    Ok(())
}

#[test]
fn scaffold_reports_exist_and_preserve_counts() -> Result<()> {
    let report = read_json(SCAFFOLD_REPORT)?;
    let tests = read_json(SCAFFOLD_TESTS_REPORT)?;
    let blockers = read_json(SCAFFOLD_BLOCKERS_REPORT)?;

    assert_eq!(
        report["artifactKind"],
        "high-risk-persisted-recovery-scaffold"
    );
    assert_eq!(report["startingCommit"], "eca1514");
    assert_eq!(report["rowsAnalyzed"], 63);
    assert_eq!(report["rowsEnabledThisSprint"], 0);
    assert_eq!(report["safeWritableRowsChanged"], false);
    assert_eq!(report["writeAllowlistChanged"], false);
    assert_eq!(report["countsBefore"]["readableRows"], 341);
    assert_eq!(report["countsBefore"]["writableRows"], 278);
    assert_eq!(report["countsBefore"]["blockedRows"], 63);
    assert_eq!(report["countsAfter"]["readableRows"], 341);
    assert_eq!(report["countsAfter"]["writableRows"], 278);
    assert_eq!(report["countsAfter"]["blockedRows"], 63);
    assert_eq!(
        report["sourceModuleAdded"],
        "src/high_risk_persisted_recovery.rs"
    );
    assert_eq!(
        report["testsAdded"][0],
        "tests/high_risk_persisted_recovery_scaffold.rs"
    );
    assert_eq!(
        report["bucketCoverage"]["display/render"]["rowsCovered"],
        23
    );
    assert_eq!(report["bucketCoverage"]["cursor/input"]["rowsCovered"], 18);
    assert_eq!(report["bucketCoverage"]["debug/crash"]["rowsCovered"], 22);
    assert_eq!(
        report["bucketCoverage"]["cursor/input"]["runtimeDynamicSpecialCases"][0],
        "cursor.default_monitor"
    );
    assert_eq!(report["rowClassifications"].as_array().unwrap().len(), 63);
    assert_eq!(
        report_row_ids(report["rowClassifications"].as_array().unwrap()),
        expected_row_ids()
    );

    assert_eq!(
        tests["artifactKind"],
        "high-risk-persisted-recovery-scaffold-tests"
    );
    assert_eq!(tests["summary"]["testsAdded"], 8);
    assert_eq!(tests["summary"]["allPassedInTargetedRun"], true);
    assert_eq!(tests["summary"]["rowsEnabledThisSprint"], 0);
    assert_eq!(tests["tests"].as_array().unwrap().len(), 8);

    assert_eq!(
        blockers["artifactKind"],
        "high-risk-persisted-recovery-scaffold-blockers"
    );
    assert_eq!(blockers["rowsStillBlocked"], 63);
    assert_eq!(blockers["rowsEnabledThisSprint"], 0);
    assert_eq!(
        blockers["blockerCategories"]["missingProductionGateIntegration"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        blockers["blockerCategories"]["missingLiveRuntimeProof"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        blockers["blockerCategories"]["missingExplicitApproval"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        blockers["blockerCategories"]["dynamicRuntimeState"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        blockers["blockerCategories"]["notYetEnablementSprint"]
            .as_array()
            .unwrap()
            .len(),
        63
    );

    Ok(())
}

#[test]
fn scaffold_review_log_covers_all_rows_and_projected_next_steps() -> Result<()> {
    let review = fs::read_to_string(REVIEW_LOG)?;

    for required in [
        "# High-risk Persisted Recovery Scaffold Review Log",
        "## Sprint summary",
        "## What the scaffold now proves",
        "## Bucket: display/render",
        "## Bucket: cursor/input",
        "## Bucket: debug/crash",
        "## Row-by-row scaffold classification",
        "## Projected next 3 steps",
    ] {
        assert!(review.contains(required), "review log missing {required}");
    }

    for row_id in expected_row_ids() {
        assert!(
            review.contains(&format!("- Row: {row_id}")),
            "review log missing {row_id}"
        );
    }

    Ok(())
}

#[test]
fn aggregate_reports_link_to_persisted_recovery_scaffold() -> Result<()> {
    let aggregate_paths = [
        "data/reports/all-341-unified-pipeline.v0.55.2.json",
        "data/reports/scalar-read-write-coverage.v0.55.2.json",
        "data/reports/deferred-validator-remaining-items.v0.55.2.json",
        "data/reports/next-high-risk-bucket-readiness.v0.55.2.json",
        "data/reports/writable-value-type-evidence-matrix.v0.55.2.json",
        "data/reports/writable-value-type-gap-summary.v0.55.2.json",
    ];

    for path in aggregate_paths {
        let report = read_json(path)?;
        let follow_up = &report["screenShaderDisplayRenderReviewFollowUp"];
        assert_eq!(
            follow_up["highRiskPersistedRecoveryScaffoldReport"],
            SCAFFOLD_REPORT
        );
        assert_eq!(
            follow_up["highRiskPersistedRecoveryScaffoldTestsReport"],
            SCAFFOLD_TESTS_REPORT
        );
        assert_eq!(
            follow_up["highRiskPersistedRecoveryScaffoldBlockersReport"],
            SCAFFOLD_BLOCKERS_REPORT
        );
        assert_eq!(follow_up["highRiskPersistedRecoveryRowsAnalyzed"], 63);
        assert_eq!(
            follow_up["highRiskPersistedRecoveryRowsEnabledThisSprint"],
            0
        );
        assert_eq!(follow_up["highRiskPersistedRecoveryRowsStillBlocked"], 63);
        assert_eq!(
            follow_up["highRiskPersistedRecoverySourceModuleAdded"],
            "src/high_risk_persisted_recovery.rs"
        );
        assert_eq!(
            follow_up["highRiskPersistedRecoveryDisplayRenderRowsCovered"],
            23
        );
        assert_eq!(
            follow_up["highRiskPersistedRecoveryCursorInputRowsCovered"],
            18
        );
        assert_eq!(
            follow_up["highRiskPersistedRecoveryDebugCrashRowsCovered"],
            22
        );
        assert_eq!(
            follow_up["highRiskPersistedRecoveryCursorDefaultMonitorSpecialCase"],
            true
        );
        assert_eq!(
            follow_up["highRiskPersistedRecoverySafeWritableRowsAfter"],
            278
        );
        assert_eq!(
            follow_up["highRiskPersistedRecoveryWriteAllowlistChanged"],
            false
        );
        assert_eq!(
            follow_up["highRiskPersistedRecoveryNextRecommendedSprint"],
            "Integrate high-risk persisted recovery scaffold into production gate dry-run sprint"
        );
    }

    Ok(())
}
