use std::path::PathBuf;

use hyprland_settings::write_advanced_confirmation::advanced_confirmation_for_candidate;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;

fn candidate(generated: bool, symlink: bool) -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: "Managed file".to_string(),
        file_path: PathBuf::from("/tmp/current.conf"),
        resolved_path: Some(PathBuf::from("/tmp/desktop.conf")),
        line_number: Some(5),
        safe: !generated,
        generated_or_script_managed: generated,
        symlink_managed: symlink,
        requires_advanced_confirmation: generated || symlink,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn advanced_confirmation_represents_generated_script_and_symlink_warnings() {
    let confirmation = advanced_confirmation_for_candidate(&candidate(true, true));

    assert!(confirmation.requires_confirmation);
    assert!(confirmation.generated_file_warning);
    assert!(confirmation.script_managed_warning);
    assert!(confirmation.symlink_managed_warning);
    assert!(confirmation.advanced_mode_required);
    assert!(!confirmation.confirmed);
    assert!(confirmation.production_disabled);
    let lines = confirmation.user_facing_lines();
    assert!(lines
        .iter()
        .any(|line| line == "This file may be changed by scripts."));
    assert!(lines
        .iter()
        .any(|line| line == "This file appears to be generated."));
    assert!(lines.iter().any(|line| line == "This file is symlinked."));
    assert!(lines
        .iter()
        .any(|line| line == "Advanced confirmation would be required before writing here."));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn plain_candidate_does_not_require_advanced_confirmation() {
    let confirmation = advanced_confirmation_for_candidate(&candidate(false, false));

    assert!(!confirmation.requires_confirmation);
    assert!(!confirmation.advanced_mode_required);
    assert!(confirmation.production_disabled);
}
