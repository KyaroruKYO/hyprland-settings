use std::path::PathBuf;

use hyprland_settings::config_layered_values::{LayeredSettingValues, LayeredValueOccurrence};
use hyprland_settings::current_config::{CurrentValueProjection, CurrentValueSourceStatus};
use hyprland_settings::guarded_write_review::{
    build_guarded_write_target_review, FixtureProofStatus, GuardedWriteReviewStatus,
};
use hyprland_settings::session_value_projection::compare_active_and_session_values;
use hyprland_settings::write_advanced_confirmation::advanced_confirmation_for_candidate;
use hyprland_settings::write_backup_plan::build_exact_backup_plan;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_review_walkthrough::{
    build_write_review_walkthrough, WriteReviewWalkthroughStepStatus,
    PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE,
};
use hyprland_settings::write_target_candidate::WriteTargetCandidate;
use hyprland_settings::write_target_recommendation::recommend_write_targets;
use hyprland_settings::write_verification_plan::planned_reread_verification;

fn layered() -> LayeredSettingValues {
    LayeredSettingValues {
        setting_id: "general.layout".to_string(),
        occurrences: vec![
            LayeredValueOccurrence {
                setting_id: "general.layout".to_string(),
                raw_value: "dwindle".to_string(),
                file_path: PathBuf::from("/tmp/main.conf"),
                resolved_path: None,
                line_number: 2,
                role_label: "Main config".to_string(),
                read_only: true,
                generated_or_script_managed: false,
                symlink_managed: false,
            },
            LayeredValueOccurrence {
                setting_id: "general.layout".to_string(),
                raw_value: "master".to_string(),
                file_path: PathBuf::from("/tmp/current.conf"),
                resolved_path: Some(PathBuf::from("/tmp/desktop.conf")),
                line_number: 1,
                role_label: "Current profile".to_string(),
                read_only: true,
                generated_or_script_managed: false,
                symlink_managed: true,
            },
        ],
        controlled_in_more_than_one_place: true,
        currently_active_value: Some("master".to_string()),
    }
}

fn candidate(label: &str, path: &str, safe: bool) -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: label.to_string(),
        file_path: PathBuf::from(path),
        resolved_path: None,
        line_number: Some(1),
        safe,
        generated_or_script_managed: !safe,
        symlink_managed: false,
        requires_advanced_confirmation: !safe,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn walkthrough_model_has_all_steps_and_safety_flags() {
    let active = CurrentValueProjection {
        status: CurrentValueSourceStatus::Configured,
        raw_value: Some("dwindle".to_string()),
        source_path: Some(PathBuf::from("/tmp/main.conf")),
        line_number: Some(2),
        raw_line: None,
        duplicate_lines: Vec::new(),
        warning: None,
    };
    let layered = layered();
    let session =
        compare_active_and_session_values("windows.layout", "general.layout", &active, &layered);
    let candidates = vec![
        candidate("Current profile", "/tmp/current.conf", true),
        candidate("Generated file", "/tmp/generated.conf", false),
    ];
    let recommendation = recommend_write_targets(&candidates);
    let selected = recommendation
        .recommended_target
        .clone()
        .expect("recommendation should include safe target");
    let backup = build_exact_backup_plan(&selected);
    let advanced = advanced_confirmation_for_candidate(&selected);
    let verification = planned_reread_verification(&selected, "general.layout", "master");
    let guarded = build_guarded_write_target_review(
        "windows.layout",
        "general.layout",
        "master",
        Some("dwindle".to_string()),
        Some("master".to_string()),
        &recommendation,
        Some(selected),
        true,
        FixtureProofStatus::Passed,
    );
    let walkthrough = build_write_review_walkthrough(
        Some(&session),
        Some(&layered),
        Some(&recommendation),
        Some(&guarded),
        Some(&backup),
        Some(&advanced),
        Some(&verification),
    );

    assert_eq!(walkthrough.steps.len(), 7);
    assert!(walkthrough.safety.read_only);
    assert!(walkthrough.safety.production_disabled);
    assert!(!walkthrough.safety.affects_apply);
    assert!(!walkthrough.safety.affects_writes);
    assert!(!walkthrough.safety.persists_selection);
    assert!(!PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE);
    assert_eq!(
        guarded.review_status,
        GuardedWriteReviewStatus::ProductionDisabled
    );
    assert!(walkthrough
        .steps
        .iter()
        .any(|step| step.status == WriteReviewWalkthroughStepStatus::ProductionDisabled));
    assert!(walkthrough
        .user_facing_lines()
        .iter()
        .any(|line| line == "Apply behavior has not changed."));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn missing_data_produces_friendly_unavailable_steps() {
    let walkthrough = build_write_review_walkthrough(None, None, None, None, None, None, None);

    assert_eq!(walkthrough.steps.len(), 7);
    assert!(walkthrough
        .steps
        .iter()
        .any(|step| step.status == WriteReviewWalkthroughStepStatus::NotAvailable));
    assert!(walkthrough
        .user_facing_lines()
        .iter()
        .any(|line| line.contains("not available yet")));
}
