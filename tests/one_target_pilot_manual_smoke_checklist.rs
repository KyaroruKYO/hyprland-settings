use hyprland_settings::one_target_pilot_pre_enable_audit::{
    one_target_pilot_manual_smoke_checklist, ManualSmokeChecklistStatus,
    PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn manual_smoke_checklist_covers_safe_one_target_review_items() {
    let checklist = one_target_pilot_manual_smoke_checklist();
    let labels = checklist.labels();

    for expected in [
        "Open the app.",
        "Open a normal settings category.",
        "Select a known safe scalar row.",
        "Confirm the row has one current scalar value.",
        "Confirm the target file is a normal config file.",
        "Confirm the target is not generated.",
        "Confirm the target is not script-managed.",
        "Confirm the target is not script-referenced.",
        "Confirm the target is not symlink-managed.",
        "Confirm the target is not a symlink target.",
        "Confirm the row is not high-risk.",
        "Confirm exact line number is known.",
        "Confirm backup contract is shown as required.",
        "Confirm verification contract is shown as required.",
        "Confirm recovery contract is shown as required.",
        "Confirm advanced confirmation is inactive.",
        "Confirm high-risk approval is inactive.",
        "Confirm real writing is not active.",
        "Confirm Apply behavior has not changed.",
    ] {
        assert!(
            labels.contains(&expected),
            "missing checklist item: {expected}"
        );
    }

    assert!(!checklist.manual_review_completed);
    assert!(!checklist.production_enabled);
    assert!(!PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED);
    assert!(checklist
        .items
        .iter()
        .all(|item| item.required_before_gate_flip
            && item.status == ManualSmokeChecklistStatus::ReviewRequired));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
