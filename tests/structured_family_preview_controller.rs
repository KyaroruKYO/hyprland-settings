use std::fs;

use hyprland_settings::runtime_preview_executor::RuntimePreviewRunner;
use hyprland_settings::structured_family_preview_controller::{
    FamilyPreviewController, FamilyPreviewError, FamilyPreviewPhase, FamilyPreviewTarget,
};

const ANIMATIONS_LISTING: &str = "animations:\n\n\tname: global\n\t\toverriden: 1\n\t\tbezier: default\n\t\tenabled: 1\n\t\tspeed: 8.00\n\t\tstyle: \n\nbeziers:\n\n\tname: default\n\t\tX0: 0.00\n\t\tY0: 0.75\n\t\tX1: 0.15\n\t\tY1: 1.00\n";
const MISSING_LISTING: &str = "animations:\n\nbeziers:\n";

/// Mock runner tracking the live speed/y0 state through eval commands.
struct SimulatedRunner {
    listing_template: &'static str,
    speed: f64,
    y0: f64,
    calls: Vec<Vec<String>>,
}

impl SimulatedRunner {
    fn new() -> Self {
        Self {
            listing_template: ANIMATIONS_LISTING,
            speed: 8.0,
            y0: 0.75,
            calls: Vec::new(),
        }
    }

    fn missing() -> Self {
        Self {
            listing_template: MISSING_LISTING,
            speed: 8.0,
            y0: 0.75,
            calls: Vec::new(),
        }
    }
}

impl RuntimePreviewRunner for SimulatedRunner {
    fn run(&mut self, _program: &str, args: &[String]) -> Result<String, String> {
        self.calls.push(args.to_vec());
        if args[0] == "animations" {
            let listing = self
                .listing_template
                .replace("speed: 8.00", &format!("speed: {:.2}", self.speed))
                .replace("Y0: 0.75", &format!("Y0: {:.2}", self.y0));
            return Ok(listing);
        }
        if args[0] == "eval" {
            let expression = &args[1];
            if let Some(index) = expression.find("speed = ") {
                let rest = &expression[index + 8..];
                let end = rest.find(',').unwrap_or(rest.len());
                self.speed = rest[..end].trim().parse().map_err(|_| "bad speed")?;
            } else if expression.contains("hl.curve") {
                // points = { {x0, y0}, ... } — the second number is y0.
                let inner = expression
                    .split("points = { {")
                    .nth(1)
                    .ok_or("bad curve expression")?;
                let first_pair = inner.split('}').next().ok_or("bad pair")?;
                let y0_text = first_pair.split(',').nth(1).ok_or("bad y0")?;
                self.y0 = y0_text.trim().parse().map_err(|_| "bad y0 value")?;
            }
            return Ok("ok".to_string());
        }
        Err("unexpected command".to_string())
    }
}

#[test]
fn family_controller_previews_keeps_and_reverts_with_verification() {
    for (target, preview_value, original_text) in [
        (FamilyPreviewTarget::AnimationGlobalSpeed, 9.5_f64, "8.00"),
        (FamilyPreviewTarget::CurveDefaultY0, 0.9_f64, "0.75"),
    ] {
        let mut controller = FamilyPreviewController::new(target, Box::new(SimulatedRunner::new()))
            .expect("proven families build controllers");
        assert_eq!(controller.phase(), FamilyPreviewPhase::Disarmed);
        assert_eq!(
            controller.current_value().expect("readback"),
            original_text
                .parse::<f64>()
                .unwrap()
                .to_string()
                .replace("8", "8.00")
                .replace("0.75", "0.75"), // normalize: current_value returns the raw listing value
        );

        // Preview applies, verifies, and starts the countdown.
        let receipt = controller.preview(preview_value).expect("preview applies");
        assert_eq!(receipt.action, "preview");
        assert!(!receipt.config_written);
        assert!(!receipt.reload_run);
        assert_eq!(controller.phase(), FamilyPreviewPhase::CountingDown);
        assert!(controller.remaining_seconds() > 0);

        // Keep confirms; countdown stops (tick past timeout does nothing).
        controller.keep().expect("keep confirms");
        assert_eq!(controller.phase(), FamilyPreviewPhase::Kept);
        assert!(controller.tick(60_000).expect("tick").is_none());

        // Revert restores the exact original with readback verification.
        let receipt = controller.revert_now().expect("revert verifies");
        assert!(receipt.status_text.contains("restored"));
        assert_eq!(controller.phase(), FamilyPreviewPhase::Reverted);
    }
}

#[test]
fn family_controller_timeout_auto_reverts() {
    let mut controller = FamilyPreviewController::new(
        FamilyPreviewTarget::AnimationGlobalSpeed,
        Box::new(SimulatedRunner::new()),
    )
    .expect("controller builds");
    controller.preview(9.0).expect("preview");
    assert!(controller.tick(9_999).expect("tick").is_none());
    let receipt = controller
        .tick(1)
        .expect("tick")
        .expect("timeout fires a receipt");
    assert_eq!(receipt.phase, FamilyPreviewPhase::TimedOutReverted);
    assert_eq!(controller.phase(), FamilyPreviewPhase::TimedOutReverted);
}

#[test]
fn family_controller_enforces_modify_existing_and_validation() {
    // Missing record: refuses before any mutation.
    let mut controller = FamilyPreviewController::new(
        FamilyPreviewTarget::AnimationGlobalSpeed,
        Box::new(SimulatedRunner::missing()),
    )
    .expect("controller builds");
    assert!(matches!(
        controller.preview(9.0),
        Err(FamilyPreviewError::RecordMissing(_))
    ));

    // Out-of-range and non-finite values refuse.
    let mut controller = FamilyPreviewController::new(
        FamilyPreviewTarget::AnimationGlobalSpeed,
        Box::new(SimulatedRunner::new()),
    )
    .expect("controller builds");
    assert!(matches!(
        controller.preview(0.0),
        Err(FamilyPreviewError::InvalidValue(_))
    ));
    assert!(matches!(
        controller.preview(f64::NAN),
        Err(FamilyPreviewError::InvalidValue(_))
    ));
    // Out-of-phase actions refuse.
    assert!(controller.keep().is_err());
    assert!(controller.revert_now().is_err());

    // Session-drop reverts only unconfirmed previews.
    let mut controller = FamilyPreviewController::new(
        FamilyPreviewTarget::CurveDefaultY0,
        Box::new(SimulatedRunner::new()),
    )
    .expect("controller builds");
    controller.preview(0.9).expect("preview");
    assert!(controller.revert_if_unconfirmed().is_some());
    let mut kept = FamilyPreviewController::new(
        FamilyPreviewTarget::CurveDefaultY0,
        Box::new(SimulatedRunner::new()),
    )
    .expect("controller builds");
    kept.preview(0.9).expect("preview");
    kept.keep().expect("keep");
    assert!(kept.revert_if_unconfirmed().is_none());
}

#[test]
fn family_controller_sources_stay_guarded() {
    let module =
        fs::read_to_string("src/structured_family_preview_controller.rs").expect("module reads");
    // Modify-existing only: no creation or deletion operations exist.
    for forbidden in [
        "fs::write",
        "File::create",
        "std::fs",
        ".config/hypr",
        "hyprctl reload",
        "\"reload\"",
        "Command::new",
        "std::process",
        "delete",
        "remove_record",
        "create_record",
    ] {
        assert!(
            !module.contains(forbidden),
            "family controller must not contain {forbidden}"
        );
    }
    // Only the two proven targets exist; no monitor/bind/device/permission
    // path can be expressed.
    for forbidden in [
        "hl.monitor",
        "hl.bind",
        "hl.device",
        "hl.permission",
        "hl.gesture",
    ] {
        assert!(
            !module.contains(forbidden),
            "family controller must not reference {forbidden}"
        );
    }
    // UI never builds family commands directly (guarded already, re-checked).
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(!window.contains("hl.animation("));
    assert!(!window.contains("hl.curve("));
}
