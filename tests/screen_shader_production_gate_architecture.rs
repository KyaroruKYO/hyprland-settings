use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::config_backup::BackupManager;
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::high_risk_recovery::HighRiskRecoveryPlanner;
use hyprland_settings::pending_change::stage_pending_change;
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use hyprland_settings::write_safety::{
    review_screen_shader_gated_write_plan, review_write_plan,
    screen_shader_requires_high_risk_gate, GatedWriteFailure, HighRiskGateProof, WritePlanRequest,
    SCREEN_SHADER_PRODUCTION_GATE_PRIMITIVE_NAME,
};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn temp_case(name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprland-settings-screen-shader-gate-architecture-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn snapshot_for(path: &PathBuf, contents: &str) -> CurrentConfigSnapshot {
    CurrentConfigSnapshot::from_parsed(parse_hyprland_config_text(path, contents))
}

fn base_review_for(
    config_path: PathBuf,
    config_contents: &str,
    setting_id: &str,
    proposed_value: &str,
) -> Result<hyprland_settings::write_safety::WriteReview> {
    fs::write(&config_path, config_contents)?;
    let snapshot = snapshot_for(&config_path, config_contents);
    let current = snapshot.value_for(setting_id);
    let pending = stage_pending_change(setting_id, &current, proposed_value);
    let backup =
        BackupManager::new(config_path.with_extension("backups")).create_backup(&config_path)?;
    Ok(review_write_plan(WritePlanRequest {
        known_setting_ids: BTreeSet::from([setting_id.to_string()]),
        detected_config_path: config_path,
        current_value: current,
        pending_change: pending,
        backup: Some(backup),
    }))
}

#[test]
fn screen_shader_production_gate_architecture_report_records_option_c() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let architecture =
        read_json("data/reports/screen-shader-production-gate-architecture.v0.55.2.json")?;
    let watchdog = read_json("data/reports/screen-shader-watchdog-migration-proof.v0.55.2.json")?;

    assert_eq!(
        SCREEN_SHADER_PRODUCTION_GATE_PRIMITIVE_NAME,
        "screen-shader-dry-run-gated-write-review"
    );
    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    assert_eq!(architecture["rowId"], "decoration.screen_shader");
    assert_eq!(architecture["officialSetting"], "decoration.screen_shader");
    assert_eq!(architecture["startingCommit"], "4f22d65");
    assert_eq!(architecture["selectedArchitectureOption"], "Option C");
    assert_eq!(architecture["currentWritableStatus"], "writable");
    assert_eq!(architecture["writableMigrationCandidate"], true);
    assert_eq!(architecture["watchdogMigrationProofStatus"], "complete");
    assert_eq!(architecture["productionEnforcementChanged"], false);
    assert_eq!(architecture["productionGateEnforcedThisSprint"], false);
    assert_eq!(architecture["productionWriteFlowChanged"], false);
    assert_eq!(architecture["dryRunGatePrimitiveAdded"], true);
    assert_eq!(
        architecture["dryRunGatePrimitiveName"],
        SCREEN_SHADER_PRODUCTION_GATE_PRIMITIVE_NAME
    );
    assert_eq!(architecture["normalProductionReviewChanged"], false);
    assert_eq!(
        architecture["normalPathOnlyApprovalStillAcceptedInProduction"],
        true
    );
    assert_eq!(architecture["ungatedDryRunScreenShaderRejected"], true);
    assert_eq!(architecture["gatedDryRunScreenShaderAccepted"], true);
    assert_eq!(
        architecture["unrelatedNormalWritableRowsRequireGate"],
        false
    );
    assert_eq!(architecture["countedAsEnabledHighRiskRow"], false);
    assert_eq!(architecture["readableRows"], 341);
    assert_eq!(architecture["writableRows"], 278);
    assert_eq!(architecture["blockedRows"], 63);
    assert_eq!(architecture["safeWritableRowsChanged"], false);
    assert_eq!(architecture["writeAllowlistChanged"], false);
    assert_eq!(architecture["rowsEnabledThisSprint"], 0);
    assert_eq!(architecture["realConfigTouched"], false);
    assert_eq!(architecture["runtimeTouched"], false);
    assert_eq!(architecture["reloadEvalLuaUsed"], false);
    assert_eq!(architecture["liveShaderCompileUsed"], false);
    assert_eq!(architecture["liveDisplayRuntimeProofUsed"], false);
    assert_eq!(architecture["compileAwareValidationChanged"], false);
    assert_eq!(architecture["compileAwareValidationStatus"], "deferred");
    assert_eq!(watchdog["watchdogMigrationProofStatus"], "complete");

    Ok(())
}

#[test]
fn dry_run_gate_primitive_detects_screen_shader_and_rejects_ungated_review() -> Result<()> {
    let dir = temp_case("ungated")?;
    let config_path = dir.join("hyprland.conf");
    let review = base_review_for(
        config_path,
        "decoration:screen_shader = ./old-screen-shader.frag\n",
        "decoration.screen_shader",
        "./new-screen-shader.frag",
    )?;

    assert!(screen_shader_requires_high_risk_gate(
        "decoration.screen_shader"
    ));
    assert!(review.is_approved());
    let gated = review_screen_shader_gated_write_plan(review, None);

    assert!(gated.gate_required);
    assert!(!gated.gate_proof_accepted);
    assert!(!gated.is_approved());
    assert!(gated
        .failures
        .contains(&GatedWriteFailure::MissingScreenShaderGateProof));

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn dry_run_gate_primitive_accepts_screen_shader_with_fixture_watchdog_proof() -> Result<()> {
    let dir = temp_case("gated")?;
    let config_path = dir.join("hyprland.conf");
    let original = "decoration:screen_shader = ./old-screen-shader.frag\n";
    let proposed = "decoration:screen_shader = ./new-screen-shader.frag\n";
    let review = base_review_for(
        config_path.clone(),
        original,
        "decoration.screen_shader",
        "./new-screen-shader.frag",
    )?;
    assert!(review.is_approved());

    let backup_root = dir.join("watchdog-backups");
    let plan_path = dir.join("screen-shader-watchdog-plan.json");
    let result_path = dir.join("screen-shader-watchdog-result.json");
    let planner = HighRiskRecoveryPlanner::new(&backup_root, 2000);
    let watchdog_plan = planner.arm_watchdog_for_temp_config(
        &config_path,
        proposed,
        10,
        &plan_path,
        &result_path,
    )?;
    let proof = HighRiskGateProof {
        setting_id: "decoration.screen_shader".to_string(),
        recovery_bucket: "display-render-recovery:screen-shader-gate-migration-design".to_string(),
        watchdog_plan,
    };

    let gated = review_screen_shader_gated_write_plan(review, Some(proof));
    assert!(gated.gate_required);
    assert!(gated.gate_proof_accepted);
    assert!(gated.failures.is_empty());
    assert!(gated.is_approved());

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn dry_run_gate_primitive_does_not_gate_unrelated_normal_writable_rows() -> Result<()> {
    let dir = temp_case("unrelated")?;
    let config_path = dir.join("hyprland.conf");
    let review = base_review_for(
        config_path,
        "windows:snap:enabled = true\n",
        "windows.snap.enabled",
        "false",
    )?;
    assert!(review.is_approved());
    assert!(!screen_shader_requires_high_risk_gate(
        "windows.snap.enabled"
    ));

    let gated = review_screen_shader_gated_write_plan(review, None);
    assert!(!gated.gate_required);
    assert!(!gated.gate_proof_accepted);
    assert!(gated.failures.is_empty());
    assert!(gated.is_approved());

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn current_production_review_remains_normal_path_only_for_screen_shader() -> Result<()> {
    let dir = temp_case("production-unchanged")?;
    let config_path = dir.join("hyprland.conf");
    let review = base_review_for(
        config_path,
        "decoration:screen_shader = ./old-screen-shader.frag\n",
        "decoration.screen_shader",
        "./new-screen-shader.frag",
    )?;

    assert!(
        review.is_approved(),
        "normal production review remains unchanged; the new primitive is dry-run/non-production only"
    );

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn invalid_or_mismatched_watchdog_proof_does_not_satisfy_screen_shader_gate() -> Result<()> {
    let dir = temp_case("mismatch")?;
    let config_path = dir.join("hyprland.conf");
    let original = "decoration:screen_shader = ./old-screen-shader.frag\n";
    let review = base_review_for(
        config_path.clone(),
        original,
        "decoration.screen_shader",
        "./new-screen-shader.frag",
    )?;
    assert!(review.is_approved());

    let wrong_config_path = dir.join("wrong-hyprland.conf");
    fs::write(&wrong_config_path, original)?;
    let backup_root = dir.join("watchdog-backups");
    let plan_path = dir.join("screen-shader-watchdog-plan.json");
    let result_path = dir.join("screen-shader-watchdog-result.json");
    let planner = HighRiskRecoveryPlanner::new(&backup_root, 3000);
    let watchdog_plan = planner.arm_watchdog_for_temp_config(
        &wrong_config_path,
        "decoration:screen_shader = ./new-screen-shader.frag\n",
        10,
        &plan_path,
        &result_path,
    )?;
    let proof = HighRiskGateProof {
        setting_id: "decoration.screen_shader".to_string(),
        recovery_bucket: "display-render-recovery:screen-shader-gate-migration-design".to_string(),
        watchdog_plan,
    };

    let gated = review_screen_shader_gated_write_plan(review, Some(proof));
    assert!(gated.gate_required);
    assert!(!gated.gate_proof_accepted);
    assert!(!gated.is_approved());
    assert!(gated
        .failures
        .contains(&GatedWriteFailure::GateProofTargetMismatch));

    fs::remove_dir_all(dir)?;
    Ok(())
}
