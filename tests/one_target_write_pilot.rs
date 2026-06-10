use std::path::PathBuf;

use hyprland_settings::one_target_write_pilot::{
    minimum_one_target_write_pilot_design, one_target_write_pilot_for_candidate,
    PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;

fn candidate(generated: bool, symlink: bool) -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: "Normal config".to_string(),
        file_path: PathBuf::from("/tmp/hyprland.conf"),
        resolved_path: None,
        line_number: Some(3),
        safe: !generated && !symlink,
        generated_or_script_managed: generated,
        symlink_managed: symlink,
        requires_advanced_confirmation: generated || symlink,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn one_target_pilot_design_captures_minimum_safe_path_and_disabled_gate() {
    let pilot = minimum_one_target_write_pilot_design();

    for expected in [
        "one scalar setting",
        "one existing scalar line",
        "one target file",
        "non-generated file",
        "non-script-managed file",
        "non-symlink-managed file",
        "exact line number known",
        "backup required",
        "reread verification required",
        "high-risk policy clear",
        "fixture proof passed",
        "production gate still false",
    ] {
        assert!(
            pilot.target_constraints.iter().any(|item| item == expected),
            "missing target constraint: {expected}"
        );
    }
    assert!(pilot
        .blocked_conditions
        .iter()
        .any(|condition| condition == "high-risk row"));
    assert!(!pilot.production_enabled);
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn pilot_candidate_eligibility_blocks_generated_symlink_and_high_risk_targets() {
    let normal = one_target_write_pilot_for_candidate(
        "windows.layout",
        "general.layout",
        &candidate(false, false),
        false,
        true,
    );
    assert!(normal.candidate_eligible);
    assert!(!normal.production_enabled);

    let generated = one_target_write_pilot_for_candidate(
        "windows.layout",
        "general.layout",
        &candidate(true, false),
        false,
        true,
    );
    assert!(!generated.candidate_eligible);

    let symlink = one_target_write_pilot_for_candidate(
        "windows.layout",
        "general.layout",
        &candidate(false, true),
        false,
        true,
    );
    assert!(!symlink.candidate_eligible);

    let high_risk = one_target_write_pilot_for_candidate(
        "debug.manual_crash",
        "debug.manual_crash",
        &candidate(false, false),
        true,
        true,
    );
    assert!(!high_risk.candidate_eligible);
}
