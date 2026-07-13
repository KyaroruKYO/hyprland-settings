use hyprland_settings::structured_family_preview_controller::{
    FamilyPreviewController, FamilyPreviewPhase, FamilyPreviewTarget,
};

/// Live UI-controller smoke: drives the same controller the GTK layer uses
/// for both proven records, verifying apply and exact restore. Ignored and
/// env-gated; normal cargo test never mutates the compositor.
#[test]
#[ignore = "mutates live animation/curve records; run only with HYPRLAND_SETTINGS_RUN_STRUCTURED_FAMILY_UI_LIVE=1"]
fn family_ui_controllers_preview_and_revert_live() {
    if std::env::var("HYPRLAND_SETTINGS_RUN_STRUCTURED_FAMILY_UI_LIVE").as_deref() != Ok("1") {
        eprintln!("skipping: env not set");
        return;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
        return;
    }
    let selector =
        std::env::var("HYPRLAND_SETTINGS_STRUCTURED_FAMILY").unwrap_or_else(|_| "all".to_string());

    if selector == "hl.animation" || selector == "all" {
        let mut controller =
            FamilyPreviewController::new_live(FamilyPreviewTarget::AnimationGlobalSpeed)
                .expect("controller builds");
        let before = controller.current_value().expect("readback");
        let preview = before.parse::<f64>().expect("parses") + 0.5;
        controller.preview(preview).expect("preview applies");
        assert_eq!(controller.phase(), FamilyPreviewPhase::CountingDown);
        let receipt = controller.revert_now().expect("revert verifies");
        assert!(receipt.status_text.contains("restored"));
        let after = controller.current_value().expect("readback");
        assert_eq!(after, before, "exact restore required");
        println!("FAMILY UI PROOF PASSED hl.animation | {before} -> {preview} -> {after}");
    }
    if selector == "hl.curve" || selector == "all" {
        let mut controller = FamilyPreviewController::new_live(FamilyPreviewTarget::CurveDefaultY0)
            .expect("controller builds");
        let before = controller.current_value().expect("readback");
        let preview = before.parse::<f64>().expect("parses") + 0.01;
        controller.preview(preview).expect("preview applies");
        let receipt = controller.cancel().expect("cancel verifies");
        assert!(receipt.status_text.contains("restored"));
        let after = controller.current_value().expect("readback");
        assert_eq!(after, before, "exact restore required");
        println!("FAMILY UI PROOF PASSED hl.curve | {before} -> {preview} -> {after}");
    }
}
