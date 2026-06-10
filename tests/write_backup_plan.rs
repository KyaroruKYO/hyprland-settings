use std::path::PathBuf;

use hyprland_settings::write_backup_plan::build_exact_backup_plan;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;

fn candidate() -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: "Main config".to_string(),
        file_path: PathBuf::from("/tmp/hyprland.conf"),
        resolved_path: Some(PathBuf::from("/tmp/hyprland.conf")),
        line_number: Some(10),
        safe: true,
        generated_or_script_managed: false,
        symlink_managed: false,
        requires_advanced_confirmation: false,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn exact_backup_plan_represents_target_backup_and_disabled_production() {
    let plan = build_exact_backup_plan(&candidate());

    assert_eq!(plan.target_file_path, PathBuf::from("/tmp/hyprland.conf"));
    assert_eq!(
        plan.resolved_target_path,
        Some(PathBuf::from("/tmp/hyprland.conf"))
    );
    assert_eq!(
        plan.backup_path,
        PathBuf::from("/tmp/hyprland.review-plan.bak")
    );
    assert!(plan.backup_required);
    assert!(plan.fixture_only);
    assert!(plan.production_backup_disabled);
    assert!(plan
        .user_facing_lines()
        .iter()
        .any(|line| line == "The app will back up this exact file before saving changes."));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
