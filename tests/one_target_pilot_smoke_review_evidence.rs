use hyprland_settings::one_target_pilot_manual_review::{
    one_target_pilot_safe_smoke_review_evidence, ManualSmokeReviewItemResult,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn safe_smoke_review_evidence_covers_required_source_review_items() {
    let evidence = one_target_pilot_safe_smoke_review_evidence();
    let requirements = evidence
        .iter()
        .map(|item| item.requirement)
        .collect::<Vec<_>>();

    for expected in [
        "normal scalar row",
        "normal file",
        "generated/script/symlink exclusions",
        "high-risk exclusion",
        "line number known",
        "backup contract",
        "verification contract",
        "recovery contract",
        "advanced confirmation inactive",
        "high-risk approval inactive",
        "real writing inactive",
        "Apply unchanged",
    ] {
        assert!(
            requirements.contains(&expected),
            "missing smoke evidence: {expected}"
        );
    }

    assert!(evidence.iter().all(|item| {
        item.result == ManualSmokeReviewItemResult::SourceProven && !item.evidence.is_empty()
    }));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
