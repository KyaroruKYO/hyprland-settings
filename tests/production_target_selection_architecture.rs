use hyprland_settings::production_target_selection_architecture::{
    minimum_production_target_selection_architecture, production_apply_integration_boundary,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn minimum_target_selection_architecture_represents_required_boundaries() {
    let architecture = minimum_production_target_selection_architecture();

    assert_eq!(
        architecture.entry_point,
        "Disabled setting-detail write review for a layered scalar setting"
    );
    for required in [
        "LayeredSettingValues",
        "WriteTargetCandidate",
        "WriteTargetRecommendation",
        "GuardedWriteTargetReview",
        "WriteBackupPlan",
        "WriteAdvancedConfirmation",
        "WriteVerificationPlan",
    ] {
        assert!(
            architecture
                .required_input_models
                .iter()
                .any(|model| model == required),
            "missing required model: {required}"
        );
    }
    assert!(architecture
        .dependencies
        .iter()
        .any(
            |dependency| dependency.name == "rollback/recovery implementation"
                && dependency.needed_before_enablement
        ));
    assert!(!architecture.production_enabled);
    assert!(
        !architecture
            .apply_integration_boundary
            .production_apply_may_call_target_selection_after_all_gates
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn architecture_user_copy_states_preview_only_status() {
    let copy = minimum_production_target_selection_architecture()
        .user_facing_lines()
        .join("\n");

    assert!(copy.contains("Minimum production target-selection path is not enabled yet."));
    assert!(
        copy.contains("The current app can preview the review flow but cannot write through it.")
    );
    assert!(copy.contains("One fixture-proven target path may be used later as the first pilot."));

    let boundary = production_apply_integration_boundary();
    let lines = boundary.report_lines().join("\n");
    assert!(lines.contains("Production Apply must not call fixture proof."));
    assert!(lines.contains("Production Apply must not call the walkthrough directly."));
    assert!(lines.contains("Production Apply must not skip backup."));
    assert!(lines.contains("Production Apply must not skip reread verification."));
    assert!(lines.contains("Production Apply must not bypass high-risk policy."));
}
