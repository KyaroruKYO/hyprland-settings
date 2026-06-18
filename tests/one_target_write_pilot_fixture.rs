use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::guarded_write_review::{
    build_guarded_write_target_review, FixtureProofStatus, GuardedWriteReviewStatus,
};
use hyprland_settings::one_target_write_pilot::{
    one_target_write_pilot_for_candidate, PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED,
};
use hyprland_settings::write_advanced_confirmation::advanced_confirmation_for_candidate;
use hyprland_settings::write_backup_plan::build_exact_backup_plan;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;
use hyprland_settings::write_target_fixture_proof::{
    prove_fixture_target_write, FixtureTargetWriteProofRequest,
};
use hyprland_settings::write_target_recommendation::recommend_write_targets;
use hyprland_settings::write_verification_plan::{
    fixture_verification_passed, planned_reread_verification, WriteVerificationStatus,
};

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-one-target-pilot-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("fixture root should be created");
    root
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture parent should be created");
    }
    fs::write(path, content).expect("fixture file should be written");
}

fn normal_candidate(path: PathBuf) -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: "Normal config".to_string(),
        file_path: path,
        resolved_path: None,
        line_number: Some(3),
        safe: true,
        generated_or_script_managed: false,
        symlink_managed: false,
        requires_advanced_confirmation: false,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn one_target_pilot_fixture_proves_normal_scalar_path_without_real_files() {
    let root = temp_fixture("normal");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "# keep header\nwindowrule = float, class:test\ngeneral:layout = dwindle\nmisc:disable_hyprland_logo = true\n",
    );
    let candidate = normal_candidate(config.clone());
    let pilot = one_target_write_pilot_for_candidate(
        "windows.layout",
        "general.layout",
        &candidate,
        false,
        true,
    );
    let recommendation = recommend_write_targets(&[candidate.clone()]);
    let backup_plan = build_exact_backup_plan(&candidate);
    let advanced = advanced_confirmation_for_candidate(&candidate);
    let verification = planned_reread_verification(&candidate, "general.layout", "master");
    let proof = prove_fixture_target_write(&FixtureTargetWriteProofRequest {
        target: candidate.clone(),
        setting_id: "general.layout".to_string(),
        new_value: "master".to_string(),
        advanced_fixture_approval: false,
    })
    .expect("normal fixture target should write and reread");
    let verification = fixture_verification_passed(&verification);
    let guarded = build_guarded_write_target_review(
        "windows.layout",
        "general.layout",
        "master",
        Some("dwindle".to_string()),
        Some("dwindle".to_string()),
        &recommendation,
        Some(candidate),
        true,
        FixtureProofStatus::Passed,
    );

    assert!(pilot.candidate_eligible);
    assert!(!pilot.production_enabled);
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert!(backup_plan.backup_required);
    assert!(backup_plan.production_backup_disabled);
    assert!(!advanced.requires_confirmation);
    assert!(advanced.production_disabled);
    assert_eq!(
        verification.verification_status,
        WriteVerificationStatus::PassedInFixture
    );
    assert_eq!(proof.reread_value.as_deref(), Some("master"));
    assert!(proof.unrelated_lines_preserved);
    assert!(!proof.target_path.starts_with("/home/kyo/.config/hypr"));
    assert_eq!(
        guarded.review_status,
        GuardedWriteReviewStatus::ReadyForReview
    );
    assert!(guarded.production_enabled);
    assert!(guarded.required_gates.production_write_integration_allowed);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
