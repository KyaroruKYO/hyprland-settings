use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::config_parser::parse_hyprland_config_file;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::structured_family::{
    accept_structured_family_draft_rendered_record_confirmation,
    prove_structured_family_draft_rendered_record_render_reread,
    structured_family_draft_rendered_record_approval_draft,
    structured_family_draft_rendered_record_confirmation_request,
    structured_family_draft_rendered_record_diff_review_summary,
    structured_family_draft_rendered_record_plans,
    structured_family_draft_rendered_record_staged_apply_plan, structured_family_record_drafts,
    structured_family_record_editor_forms, StructuredFamilyDraftRenderedRecordStagedApplyPlan,
    StructuredFamilyKind,
};
use hyprland_settings::structured_family_active_config_pilot::{
    active_config_pilot_content_hash, build_first_active_config_pilot_plan,
    execute_first_active_config_write_pilot, preflight_first_active_config_pilot,
    run_active_config_rehearsal, structured_family_active_config_pilot_audit_record,
    StructuredFamilyActiveConfigAutoreloadEvidence, StructuredFamilyActiveConfigPilotApproval,
    StructuredFamilyActiveConfigPilotError, StructuredFamilyActiveConfigPilotGate,
    ACTIVE_CONFIG_PILOT_CONFIRMATION_PHRASE,
};

const PILOT_FAMILY: StructuredFamilyKind = StructuredFamilyKind::Curve;
const PILOT_RECORD: &str = "bezier = hyprlandSettingsPilotCurve, 0.25, 0.10, 0.25, 1.00";

fn active_config_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME should be set");
    Path::new(&home).join(".config/hypr/hyprland.conf")
}

fn unique_temp_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after the epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-active-pilot-{label}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

fn curve_staged_apply_plan(
    scratch_root: &Path,
) -> StructuredFamilyDraftRenderedRecordStagedApplyPlan {
    let snapshot = CurrentConfigSnapshot::from_parsed(
        parse_hyprland_config_file("tests/fixtures/structured_families/hl_curve.conf")
            .expect("curve fixture should parse"),
    );
    let projection = snapshot
        .structured_family_projections()
        .into_iter()
        .find(|projection| projection.family == PILOT_FAMILY)
        .expect("curve projection should exist");
    let forms = structured_family_record_editor_forms(&projection);
    let drafts = structured_family_record_drafts(&forms);
    let plans = structured_family_draft_rendered_record_plans(&drafts);
    let proof_path = scratch_root.join("curve-render-reread.conf");
    let proof = prove_structured_family_draft_rendered_record_render_reread(&plans, &proof_path);
    let summary = structured_family_draft_rendered_record_diff_review_summary(&plans, &proof);
    let approval = structured_family_draft_rendered_record_approval_draft(&summary);
    let request = structured_family_draft_rendered_record_confirmation_request(&approval);
    let accepted = accept_structured_family_draft_rendered_record_confirmation(&approval, &request);
    structured_family_draft_rendered_record_staged_apply_plan(&accepted, &summary)
}

fn full_pilot_approval() -> StructuredFamilyActiveConfigPilotApproval {
    StructuredFamilyActiveConfigPilotApproval {
        pilot_approved: true,
        typed_confirmation: ACTIVE_CONFIG_PILOT_CONFIRMATION_PHRASE.to_string(),
        backup_acknowledged: true,
        restore_acknowledged: true,
        verification_acknowledged: true,
    }
}

fn autoreload_not_confirmed() -> StructuredFamilyActiveConfigAutoreloadEvidence {
    StructuredFamilyActiveConfigAutoreloadEvidence {
        disable_autoreload_confirmed: false,
        evidence_description:
            "hyprctl getoption misc:disable_autoreload returned false (autoreload active)"
                .to_string(),
    }
}

#[test]
fn copied_active_config_rehearsal_round_trips_without_touching_the_source() {
    let active = active_config_path();
    if !active.exists() {
        eprintln!("skipping: no active Hyprland config on this machine");
        return;
    }
    let source_bytes = fs::read(&active).expect("active config should read");
    let scratch = unique_temp_root("rehearsal-scratch");
    let rehearsal_root = unique_temp_root("rehearsal");

    let staged_apply = curve_staged_apply_plan(&scratch);
    let proof = run_active_config_rehearsal(&active, &staged_apply, PILOT_RECORD, &rehearsal_root)
        .expect("rehearsal should round trip against the copied active config");

    assert_eq!(proof.family, PILOT_FAMILY);
    assert!(proof.write_verified);
    assert!(proof.restore_verified);
    assert!(proof.source_untouched);
    assert_eq!(
        proof.source_hash,
        active_config_pilot_content_hash(&source_bytes)
    );
    assert_eq!(
        fs::read(&active).expect("active config should reread"),
        source_bytes,
        "rehearsal must never modify the active config"
    );

    fs::remove_dir_all(&scratch).ok();
    fs::remove_dir_all(&rehearsal_root).ok();
}

#[test]
fn pilot_preflight_blocks_on_autoreload_evidence_and_every_missing_proof() {
    let active = active_config_path();
    if !active.exists() {
        eprintln!("skipping: no active Hyprland config on this machine");
        return;
    }
    let scratch = unique_temp_root("gates-scratch");
    let rehearsal_root = unique_temp_root("gates-rehearsal");
    let backup_root = unique_temp_root("gates-backup");
    let staged_apply = curve_staged_apply_plan(&scratch);
    let rehearsal =
        run_active_config_rehearsal(&active, &staged_apply, PILOT_RECORD, &rehearsal_root)
            .expect("rehearsal should succeed");

    // Everything proven except autoreload evidence — this is the live state
    // of this machine today (misc:disable_autoreload is false).
    let plan = build_first_active_config_pilot_plan(
        &active,
        PILOT_FAMILY,
        PILOT_RECORD,
        &backup_root,
        autoreload_not_confirmed(),
        Some(rehearsal.clone()),
    )
    .expect("plan should build");
    let preflight = preflight_first_active_config_pilot(&plan, &full_pilot_approval());
    assert!(!preflight.passed);
    assert!(preflight
        .blocking_gates
        .contains(&StructuredFamilyActiveConfigPilotGate::AutoreloadDisabledConfirmed));
    assert!(preflight
        .blocking_gates
        .contains(&StructuredFamilyActiveConfigPilotGate::NoRuntimeMutationPlanned));
    let error = execute_first_active_config_write_pilot(&plan, &full_pilot_approval())
        .expect_err("pilot must be blocked by the autoreload gate");
    assert!(matches!(
        error,
        StructuredFamilyActiveConfigPilotError::GateFailed(_)
    ));

    // Missing approval blocks.
    let mut unapproved = full_pilot_approval();
    unapproved.pilot_approved = false;
    let preflight = preflight_first_active_config_pilot(&plan, &unapproved);
    assert!(preflight
        .blocking_gates
        .contains(&StructuredFamilyActiveConfigPilotGate::PilotApprovalPresent));

    // Wrong typed confirmation blocks.
    let mut wrong_phrase = full_pilot_approval();
    wrong_phrase.typed_confirmation = "yes".to_string();
    let preflight = preflight_first_active_config_pilot(&plan, &wrong_phrase);
    assert!(preflight
        .blocking_gates
        .contains(&StructuredFamilyActiveConfigPilotGate::TypedConfirmationMatches));

    // Missing rehearsal blocks.
    let plan_no_rehearsal = build_first_active_config_pilot_plan(
        &active,
        PILOT_FAMILY,
        PILOT_RECORD,
        &backup_root,
        autoreload_not_confirmed(),
        None,
    )
    .expect("plan should build");
    let preflight = preflight_first_active_config_pilot(&plan_no_rehearsal, &full_pilot_approval());
    assert!(preflight
        .blocking_gates
        .contains(&StructuredFamilyActiveConfigPilotGate::RehearsalProven));

    // Backup path inside the active config area blocks.
    let mut bad_backup = plan.clone();
    bad_backup.backup_path = active
        .parent()
        .expect("active config should have a parent")
        .join("pilot-backup-inside-active.conf");
    let preflight = preflight_first_active_config_pilot(&bad_backup, &full_pilot_approval());
    assert!(preflight
        .blocking_gates
        .contains(&StructuredFamilyActiveConfigPilotGate::BackupPathOutsideActiveConfig));

    // A non-minimal change blocks: two appended records.
    let mut non_minimal = plan.clone();
    non_minimal
        .rendered_records
        .push("bezier = secondPilotCurve, 0.3, 0.3, 0.7, 0.7".to_string());
    let preflight = preflight_first_active_config_pilot(&non_minimal, &full_pilot_approval());
    assert!(preflight
        .blocking_gates
        .contains(&StructuredFamilyActiveConfigPilotGate::MinimalReversibleChange));

    // A non-minimal change blocks: dropped original records.
    let mut dropped = plan.clone();
    dropped
        .original_records
        .push("bezier = phantom, 0, 0, 1, 1".to_string());
    let preflight = preflight_first_active_config_pilot(&dropped, &full_pilot_approval());
    assert!(preflight
        .blocking_gates
        .contains(&StructuredFamilyActiveConfigPilotGate::MinimalReversibleChange));

    // Refusing to restore blocks.
    let mut keeps_record = plan.clone();
    keeps_record.restore_original_bytes = false;
    let preflight = preflight_first_active_config_pilot(&keeps_record, &full_pilot_approval());
    assert!(preflight
        .blocking_gates
        .contains(&StructuredFamilyActiveConfigPilotGate::RollbackPlanPresent));

    // A non-active target blocks target identity.
    let decoy_root = unique_temp_root("decoy");
    let decoy = decoy_root.join("hyprland.conf");
    fs::write(&decoy, "monitor = eDP-1, 1920x1080@60, 0x0, 1\n").expect("decoy should write");
    let decoy_plan = build_first_active_config_pilot_plan(
        &decoy,
        PILOT_FAMILY,
        PILOT_RECORD,
        &backup_root,
        autoreload_not_confirmed(),
        Some(rehearsal),
    )
    .expect("plan should build");
    let preflight = preflight_first_active_config_pilot(&decoy_plan, &full_pilot_approval());
    assert!(preflight
        .blocking_gates
        .contains(&StructuredFamilyActiveConfigPilotGate::TargetIdentityProven));

    // Nothing in this test may have modified the active config.
    let plan_after = build_first_active_config_pilot_plan(
        &active,
        PILOT_FAMILY,
        PILOT_RECORD,
        &backup_root,
        autoreload_not_confirmed(),
        None,
    )
    .expect("plan should rebuild");
    assert_eq!(plan_after.original_records, plan.original_records);

    fs::remove_dir_all(&scratch).ok();
    fs::remove_dir_all(&rehearsal_root).ok();
    fs::remove_dir_all(&backup_root).ok();
    fs::remove_dir_all(&decoy_root).ok();
}

#[test]
fn pilot_module_is_unreachable_from_live_ui_and_scalar_write_paths() {
    for path in [
        "src/main.rs",
        "src/write_flow.rs",
        "src/ui/mod.rs",
        "src/ui/app.rs",
        "src/ui/model.rs",
        "src/ui/window.rs",
    ] {
        let source = fs::read_to_string(path).expect("source should read");
        for forbidden in [
            "structured_family_active_config_pilot",
            "execute_first_active_config_write_pilot",
            "run_active_config_rehearsal",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} must not reference {forbidden}"
            );
        }
    }
}

#[test]
fn pilot_module_source_has_no_command_reload_or_ui_paths() {
    let source = fs::read_to_string("src/structured_family_active_config_pilot.rs")
        .expect("pilot source should read");
    for forbidden in [
        "hyprctl ",
        "\"hyprctl\"",
        "Command::",
        "std::process",
        "process::Command",
        "spawn(",
        "gtk::",
        "gtk4::",
        "adw::",
        "Button::",
        "connect_clicked",
        "execute_safe_write_scaffold",
        "write_flow::",
        "crate::write_flow",
        "apply_setting_change(",
    ] {
        assert!(
            !source.contains(forbidden),
            "pilot source must not contain {forbidden}"
        );
    }
}

/// The real first active config write pilot. Ignored by default: it only
/// runs when explicitly requested via `cargo test -- --ignored` with
/// HYPRLAND_SETTINGS_RUN_ACTIVE_PILOT=1, and even then every preflight gate
/// (including externally confirmed autoreload-disabled evidence via
/// HYPRLAND_SETTINGS_AUTORELOAD_DISABLED_CONFIRMED=true) must pass or it
/// refuses without writing.
#[test]
#[ignore = "first active config write pilot: run only with explicit approval env"]
fn first_active_config_write_pilot_runs_only_when_every_gate_passes() {
    if std::env::var("HYPRLAND_SETTINGS_RUN_ACTIVE_PILOT").as_deref() != Ok("1") {
        eprintln!("skipping: HYPRLAND_SETTINGS_RUN_ACTIVE_PILOT is not set");
        return;
    }
    let active = active_config_path();
    assert!(active.exists(), "active config must exist for the pilot");

    let scratch = unique_temp_root("live-pilot-scratch");
    let rehearsal_root = unique_temp_root("live-pilot-rehearsal");
    let backup_root = unique_temp_root("live-pilot-backup");
    let staged_apply = curve_staged_apply_plan(&scratch);
    let rehearsal =
        run_active_config_rehearsal(&active, &staged_apply, PILOT_RECORD, &rehearsal_root)
            .expect("rehearsal must succeed before the live pilot");

    let autoreload = StructuredFamilyActiveConfigAutoreloadEvidence {
        disable_autoreload_confirmed: std::env::var(
            "HYPRLAND_SETTINGS_AUTORELOAD_DISABLED_CONFIRMED",
        )
        .as_deref()
            == Ok("true"),
        evidence_description: "operator-confirmed misc:disable_autoreload=true via read-only query"
            .to_string(),
    };
    let plan = build_first_active_config_pilot_plan(
        &active,
        PILOT_FAMILY,
        PILOT_RECORD,
        &backup_root,
        autoreload,
        Some(rehearsal),
    )
    .expect("pilot plan should build");

    let original_bytes = fs::read(&active).expect("active config should read");
    match execute_first_active_config_write_pilot(&plan, &full_pilot_approval()) {
        Ok(receipt) => {
            assert!(receipt.written);
            assert!(receipt.post_write_verification.intended_records_present);
            assert!(receipt.restored);
            assert!(receipt.original_bytes_restored);
            assert_eq!(receipt.pre_write_hash, receipt.post_restore_hash);
            assert_ne!(receipt.pre_write_hash, receipt.post_write_hash);
            assert_eq!(fs::read(&active).expect("reread"), original_bytes);
            let audit = structured_family_active_config_pilot_audit_record(&receipt);
            assert!(audit.original_bytes_restored);
        }
        Err(StructuredFamilyActiveConfigPilotError::GateFailed(gate)) => {
            assert_eq!(
                fs::read(&active).expect("reread"),
                original_bytes,
                "a blocked pilot must not touch the active config"
            );
            eprintln!("pilot blocked by gate {}", gate.as_str());
        }
        Err(other) => panic!("unexpected pilot error: {other:?}"),
    }

    fs::remove_dir_all(&scratch).ok();
    fs::remove_dir_all(&rehearsal_root).ok();
    fs::remove_dir_all(&backup_root).ok();
}

#[test]
fn autoreload_evidence_collector_fails_closed() {
    use hyprland_settings::runtime_preview_executor::RuntimePreviewRunner;
    use hyprland_settings::structured_family_active_config_pilot::collect_autoreload_evidence;

    struct FixedRunner(Result<String, String>);
    impl RuntimePreviewRunner for FixedRunner {
        fn run(&mut self, _program: &str, args: &[String]) -> Result<String, String> {
            assert_eq!(args[0], "getoption", "collector must be read-only");
            self.0.clone()
        }
    }

    // Disabled autoreload confirms the gate.
    let mut runner = FixedRunner(Ok("bool: true\nset: true".to_string()));
    let evidence = collect_autoreload_evidence(&mut runner);
    assert!(evidence.disable_autoreload_confirmed);
    assert!(evidence.evidence_description.contains("cannot live-reload"));

    // Active autoreload fails closed with the blocker explained.
    let mut runner = FixedRunner(Ok("bool: false\nset: false".to_string()));
    let evidence = collect_autoreload_evidence(&mut runner);
    assert!(!evidence.disable_autoreload_confirmed);
    assert!(evidence.evidence_description.contains("stays blocked"));

    // Read failures fail closed.
    let mut runner = FixedRunner(Err("socket unavailable".to_string()));
    let evidence = collect_autoreload_evidence(&mut runner);
    assert!(!evidence.disable_autoreload_confirmed);
    assert!(evidence.evidence_description.contains("failing closed"));

    // Unparseable output fails closed.
    let mut runner = FixedRunner(Ok("garbage".to_string()));
    let evidence = collect_autoreload_evidence(&mut runner);
    assert!(!evidence.disable_autoreload_confirmed);
}
