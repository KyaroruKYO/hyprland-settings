use hyprland_settings::one_target_pilot_pre_enable_audit::{
    one_target_pilot_pre_enable_audit, PreEnableAuditStatus,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn final_pre_enable_audit_tracks_all_required_categories_and_stays_not_ready() {
    let audit = one_target_pilot_pre_enable_audit();
    let categories = audit.category_names();

    for expected in [
        "target eligibility",
        "target risk exclusions",
        "high-risk exclusion",
        "backup contract",
        "backup path/collision policy",
        "backup integrity",
        "reread verification",
        "verification failure behavior",
        "recovery/rollback",
        "advanced confirmation policy",
        "high-risk approval boundary",
        "manual smoke checklist",
        "Apply/write isolation",
        "UI disabled-state proof",
        "fixture-only proof",
        "real user config safety",
        "production gate inventory",
    ] {
        assert!(
            categories.contains(&expected),
            "missing audit category: {expected}"
        );
    }

    assert!(!audit.readiness);
    assert!(audit.production_disabled);
    assert!(audit
        .categories
        .iter()
        .any(
            |category| category.category_name == "manual smoke checklist"
                && category.status == PreEnableAuditStatus::NotStarted
                && category.blocking_reason == "manual smoke review is not complete"
        ));
    assert!(audit
        .categories
        .iter()
        .any(
            |category| category.category_name == "production gate inventory"
                && category.status == PreEnableAuditStatus::ProductionDisabled
                && category.blocking_reason == "all gates remain false"
        ));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
