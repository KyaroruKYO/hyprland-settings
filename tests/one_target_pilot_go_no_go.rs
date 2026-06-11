use hyprland_settings::one_target_pilot_pre_enable_audit::one_target_pilot_go_no_go_decision;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn go_no_go_decision_keeps_first_pilot_blocked() {
    let decision = one_target_pilot_go_no_go_decision();

    assert!(!decision.go);
    assert!(decision.fixture_proven);
    assert!(decision.design_complete);
    assert!(decision.production_disabled);
    assert!(decision.ready_for_manual_review);
    assert!(decision.ready_to_flip_gate);

    for expected in [
        "write activation gates are false",
        "pre-enable audit has passed but production backup/write/reread/recovery are not active",
        "Apply integration is not approved",
    ] {
        assert!(
            decision.reasons.contains(&expected),
            "missing go/no-go reason: {expected}"
        );
    }

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
