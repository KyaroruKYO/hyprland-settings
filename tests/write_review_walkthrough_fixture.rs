use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::config_graph::{
    inspect_config_graph_with_options, ConfigGraphOptions, SourceFollowPolicy,
};
use hyprland_settings::config_layered_values::layered_values_for_setting;
use hyprland_settings::current_config::{CurrentValueProjection, CurrentValueSourceStatus};
use hyprland_settings::guarded_write_review::{
    build_guarded_write_target_review, FixtureProofStatus, GuardedWriteReviewStatus,
};
use hyprland_settings::session_value_projection::compare_active_and_session_values;
use hyprland_settings::write_advanced_confirmation::advanced_confirmation_for_candidate;
use hyprland_settings::write_backup_plan::build_exact_backup_plan;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_review_walkthrough::build_write_review_walkthrough;
use hyprland_settings::write_target_candidate::write_target_candidates_for_layered_setting;
use hyprland_settings::write_target_fixture_proof::{
    prove_fixture_target_write, FixtureTargetWriteProofError, FixtureTargetWriteProofRequest,
};
use hyprland_settings::write_target_recommendation::recommend_write_targets;
use hyprland_settings::write_verification_plan::{
    fixture_verification_passed, planned_reread_verification, WriteVerificationStatus,
};

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-walkthrough-{name}-{}",
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

#[test]
fn fixture_walkthrough_composes_full_review_and_uses_safe_batch_gate() {
    let root = temp_fixture("full");
    let config = root.join("hyprland.conf");
    let profile = root.join("profiles/desktop.conf");
    write_file(
        &config,
        "source = profiles/desktop.conf\n# keep root\ngeneral:layout = dwindle\n",
    );
    write_file(
        &profile,
        "# keep profile\ngeneral:layout = dwindle\nmisc:disable_hyprland_logo = true\n",
    );

    let graph = inspect_config_graph_with_options(
        &config,
        ConfigGraphOptions {
            home_dir: Some(root.clone()),
            script_dirs: Vec::new(),
            max_depth: 16,
            source_follow_policy: SourceFollowPolicy::ReviewAll,
        },
    );
    let layered = layered_values_for_setting(&graph, "general.layout");
    let active = CurrentValueProjection {
        status: CurrentValueSourceStatus::Configured,
        raw_value: Some("dwindle".to_string()),
        source_path: Some(config.clone()),
        line_number: Some(3),
        raw_line: None,
        duplicate_lines: Vec::new(),
        warning: None,
    };
    let session =
        compare_active_and_session_values("windows.layout", "general.layout", &active, &layered);
    let candidates = write_target_candidates_for_layered_setting(&layered, &graph.files);
    let recommendation = recommend_write_targets(&candidates);
    let selected = recommendation
        .recommended_target
        .clone()
        .expect("safe target should be recommended");
    let backup = build_exact_backup_plan(&selected);
    let advanced = advanced_confirmation_for_candidate(&selected);
    let verification = planned_reread_verification(&selected, "general.layout", "master");
    let proof = prove_fixture_target_write(&FixtureTargetWriteProofRequest {
        target: selected.clone(),
        setting_id: "general.layout".to_string(),
        new_value: "master".to_string(),
        advanced_fixture_approval: false,
    })
    .expect("fixture proof should pass");
    let verification = fixture_verification_passed(&verification);
    let guarded = build_guarded_write_target_review(
        "windows.layout",
        "general.layout",
        "master",
        Some("dwindle".to_string()),
        Some("dwindle".to_string()),
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
    assert!(!walkthrough.safety.production_disabled);
    assert!(walkthrough.safety.affects_apply);
    assert_eq!(
        guarded.review_status,
        GuardedWriteReviewStatus::ReadyForReview
    );
    assert!(guarded.production_enabled);
    assert_eq!(
        verification.verification_status,
        WriteVerificationStatus::PassedInFixture
    );
    assert!(proof.fixture_only);
    assert!(proof.unrelated_lines_preserved);
    assert!(!proof.target_path.starts_with("/home/kyo/.config/hypr"));
    assert_eq!(proof.reread_value.as_deref(), Some("master"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn generated_fixture_target_requires_advanced_approval_for_walkthrough_proof() {
    let root = temp_fixture("generated");
    let config = root.join("hyprland.conf");
    let generated = root.join("profiles/gaming.conf");
    write_file(
        &config,
        "source = profiles/gaming.conf\ngeneral:layout = dwindle\n",
    );
    write_file(
        &generated,
        "# generated by test\n# do not edit\ngeneral:layout = dwindle\n",
    );

    let graph = inspect_config_graph_with_options(
        &config,
        ConfigGraphOptions {
            home_dir: Some(root.clone()),
            script_dirs: Vec::new(),
            max_depth: 16,
            source_follow_policy: SourceFollowPolicy::ReviewAll,
        },
    );
    let layered = layered_values_for_setting(&graph, "general.layout");
    let candidates = write_target_candidates_for_layered_setting(&layered, &graph.files);
    let generated_target = candidates
        .iter()
        .find(|candidate| candidate.generated_or_script_managed)
        .cloned()
        .expect("generated target should be represented");

    let rejected = prove_fixture_target_write(&FixtureTargetWriteProofRequest {
        target: generated_target.clone(),
        setting_id: "general.layout".to_string(),
        new_value: "master".to_string(),
        advanced_fixture_approval: false,
    })
    .expect_err("generated target should require approval");
    assert_eq!(
        rejected,
        FixtureTargetWriteProofError::UnsafeTargetRequiresAdvancedApproval
    );

    let approved = prove_fixture_target_write(&FixtureTargetWriteProofRequest {
        target: generated_target,
        setting_id: "general.layout".to_string(),
        new_value: "master".to_string(),
        advanced_fixture_approval: true,
    })
    .expect("advanced fixture approval should allow fixture proof");
    assert_eq!(approved.reread_value.as_deref(), Some("master"));
    assert!(!approved.target_path.starts_with("/home/kyo/.config/hypr"));
}
