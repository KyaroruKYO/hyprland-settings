//! Env-gated live proofs for the family record picker's generalized record
//! shapes. Normal `cargo test` never runs these bodies (env-gated and
//! ignored); they mutate runtime animation/curve records reversibly and, in
//! the save flow proof, write the active config once and restore the
//! pre-test bytes as cleanup.
//!
//! The generalization proofs deliberately run on records that are NOT the
//! original family-proof records (global / default), so a passed run proves
//! the shape, not a repetition of the original proof.

use hyprland_settings::config_discovery::discover_hyprland_config;
use hyprland_settings::family_record_picker::{
    animation_record_entries, curve_record_entries, render_animation_preview_expression,
    render_curve_preview_expression, FamilyRecordPreviewController, PickedFamily,
    PickedRecordValues, RecordPickerPhase,
};
use hyprland_settings::runtime_preview_executor::{
    HyprctlRuntimePreviewRunner, RuntimePreviewRunner,
};
use hyprland_settings::safe_live_save_mode::{
    disable_safe_live_save_mode, enable_safe_live_save_mode, read_safe_live_save_mode_status,
    SafeLiveSaveModeState,
};
use hyprland_settings::structured_family_gated_persistence::{
    gated_family_record_save, FamilyRecordSaveRequest,
};
use hyprland_settings::structured_family_runtime_preview::{
    parse_animation_records, parse_bezier_records, AnimationRuntimeRecord, BezierRuntimeRecord,
};

fn live_gate(env: &str) -> bool {
    if std::env::var(env).as_deref() != Ok("1") {
        eprintln!("skipping: {env} not set");
        return false;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
        return false;
    }
    true
}

fn read_listing(runner: &mut dyn RuntimePreviewRunner) -> String {
    runner
        .run("hyprctl", &["animations".to_string()])
        .expect("animations listing reads")
}

fn animation_record(listing: &str, name: &str) -> AnimationRuntimeRecord {
    parse_animation_records(listing)
        .into_iter()
        .find(|record| record.name == name)
        .unwrap_or_else(|| panic!("animation record {name} present"))
}

fn bezier_record(listing: &str, name: &str) -> BezierRuntimeRecord {
    parse_bezier_records(listing)
        .into_iter()
        .find(|record| record.name == name)
        .unwrap_or_else(|| panic!("bezier record {name} present"))
}

/// Pick a proof record for the animation shape: explicitly overridden,
/// style-free, safe name, and NOT the original family-proof record.
fn pick_animation_proof_record(listing: &str) -> Option<String> {
    parse_animation_records(listing)
        .into_iter()
        .filter(|record| {
            record.overridden
                && record.style.is_empty()
                && !record.name.starts_with("__")
                && record.name != "global"
        })
        .map(|record| record.name)
        .find(|name| name == "fade")
        .or_else(|| {
            parse_animation_records(listing)
                .into_iter()
                .filter(|record| {
                    record.overridden
                        && record.style.is_empty()
                        && !record.name.starts_with("__")
                        && record.name != "global"
                })
                .map(|record| record.name)
                .next()
        })
}

/// Pick a proof record for the curve shape: an existing curve that is NOT
/// the original family-proof record.
fn pick_curve_proof_record(listing: &str) -> Option<String> {
    let candidates: Vec<String> = parse_bezier_records(listing)
        .into_iter()
        .map(|record| record.name)
        .filter(|name| name != "default")
        .collect();
    candidates
        .iter()
        .find(|name| name.as_str() == "quick")
        .cloned()
        .or_else(|| candidates.first().cloned())
}

/// Shape proof (primitives): modify-existing speed round trip on an
/// arbitrary overridden animation record with zero residue.
#[test]
#[ignore = "mutates runtime animation state reversibly; run only with HYPRLAND_SETTINGS_RUN_FAMILY_RECORD_PICKER_LIVE=1"]
fn record_shape_proof_animation_record_speed_round_trip() {
    if !live_gate("HYPRLAND_SETTINGS_RUN_FAMILY_RECORD_PICKER_LIVE") {
        return;
    }
    let mut runner = HyprctlRuntimePreviewRunner;
    let listing = read_listing(&mut runner);
    let name = pick_animation_proof_record(&listing)
        .expect("an overridden non-global style-free animation record exists");
    let original = animation_record(&listing, &name);
    let original_speed: f64 = original.speed.parse().expect("original speed parses");
    let preview_speed = original_speed + 0.25;

    let expression =
        render_animation_preview_expression(&original, preview_speed).expect("expression renders");
    runner
        .run("hyprctl", &["eval".to_string(), expression])
        .expect("preview applies");
    let applied = animation_record(&read_listing(&mut runner), &name);
    let applied_speed: f64 = applied.speed.parse().expect("applied speed parses");
    assert!(
        (applied_speed - preview_speed).abs() < 1e-3,
        "readback confirms the applied speed"
    );

    let restore =
        render_animation_preview_expression(&original, original_speed).expect("restore renders");
    runner
        .run("hyprctl", &["eval".to_string(), restore])
        .expect("restore applies");
    let restored = animation_record(&read_listing(&mut runner), &name);
    assert_eq!(
        restored, original,
        "zero residue: the full record matches the captured original"
    );
    println!(
        "RECORD SHAPE PROOF PASSED hl.animation {name}: speed {original_speed} -> {preview_speed} -> {original_speed}, full-record zero residue"
    );
}

/// Shape proof (primitives): modify-existing control-point round trip on an
/// arbitrary existing curve with zero residue.
#[test]
#[ignore = "mutates runtime curve state reversibly; run only with HYPRLAND_SETTINGS_RUN_FAMILY_RECORD_PICKER_LIVE=1"]
fn record_shape_proof_curve_control_points_round_trip() {
    if !live_gate("HYPRLAND_SETTINGS_RUN_FAMILY_RECORD_PICKER_LIVE") {
        return;
    }
    let mut runner = HyprctlRuntimePreviewRunner;
    let listing = read_listing(&mut runner);
    let name = pick_curve_proof_record(&listing).expect("a non-default curve exists");
    let original = bezier_record(&listing, &name);
    let preview_x1 = (original.x1 + 0.01).min(1.0);

    let expression =
        render_curve_preview_expression(&name, original.x0, original.y0, preview_x1, original.y1)
            .expect("expression renders");
    runner
        .run("hyprctl", &["eval".to_string(), expression])
        .expect("preview applies");
    let applied = bezier_record(&read_listing(&mut runner), &name);
    assert!(
        (applied.x1 - preview_x1).abs() < 1e-3,
        "readback confirms the applied X1"
    );

    let restore =
        render_curve_preview_expression(&name, original.x0, original.y0, original.x1, original.y1)
            .expect("restore renders");
    runner
        .run("hyprctl", &["eval".to_string(), restore])
        .expect("restore applies");
    let restored = bezier_record(&read_listing(&mut runner), &name);
    for (restored_point, original_point, label) in [
        (restored.x0, original.x0, "X0"),
        (restored.y0, original.y0, "Y0"),
        (restored.x1, original.x1, "X1"),
        (restored.y1, original.y1, "Y1"),
    ] {
        assert!(
            (restored_point - original_point).abs() < 1e-6,
            "zero residue on {label}"
        );
    }
    println!(
        "RECORD SHAPE PROOF PASSED hl.curve {name}: X1 {} -> {preview_x1} -> {}, zero residue on all four points",
        original.x1, original.x1
    );
}

/// Controller proof: the supervised picker controller itself round-trips
/// live (requires the shape receipts to be recorded, since the controller
/// refuses unproven shapes).
#[test]
#[ignore = "mutates runtime state reversibly; run only with HYPRLAND_SETTINGS_RUN_FAMILY_RECORD_PICKER_LIVE=1"]
fn record_picker_controller_round_trips_live() {
    if !live_gate("HYPRLAND_SETTINGS_RUN_FAMILY_RECORD_PICKER_LIVE") {
        return;
    }
    let mut probe = HyprctlRuntimePreviewRunner;
    let listing = read_listing(&mut probe);

    // Animation controller on a picker-supported non-global record.
    let animation_name = animation_record_entries(&listing)
        .into_iter()
        .filter(|entry| entry.preview_supported && entry.record.name != "global")
        .map(|entry| entry.record.name)
        .next()
        .expect("a preview-supported non-global animation record exists (receipts recorded)");
    let original = animation_record(&listing, &animation_name);
    let original_speed: f64 = original.speed.parse().expect("speed parses");
    let mut controller =
        FamilyRecordPreviewController::new_live(PickedFamily::Animation, &animation_name)
            .expect("controller arms for a supported record");
    let receipt = controller
        .preview(PickedRecordValues::AnimationRecord {
            enabled: original.enabled == "1",
            speed: original_speed + 0.25,
            bezier: if original.bezier.is_empty() {
                "default".to_string()
            } else {
                original.bezier.clone()
            },
        })
        .expect("preview applies and verifies");
    assert_eq!(receipt.phase, RecordPickerPhase::CountingDown);
    assert!(!receipt.config_written);
    assert!(!receipt.reload_run);
    let receipt = controller
        .revert_now()
        .expect("revert applies and verifies");
    assert_eq!(receipt.phase, RecordPickerPhase::CountingDown); // receipt captured pre-transition
    let restored = animation_record(&read_listing(&mut probe), &animation_name);
    assert_eq!(
        restored, original,
        "controller round trip leaves zero residue"
    );
    println!(
        "CONTROLLER PROOF PASSED hl.animation {animation_name}: preview + revert, zero residue"
    );

    // Curve controller on a picker-supported non-default record.
    let curve_name = curve_record_entries(&listing)
        .into_iter()
        .filter(|entry| entry.preview_supported && entry.record.name != "default")
        .map(|entry| entry.record.name)
        .next()
        .expect("a preview-supported non-default curve exists (receipts recorded)");
    let original = bezier_record(&listing, &curve_name);
    let mut controller = FamilyRecordPreviewController::new_live(PickedFamily::Curve, &curve_name)
        .expect("controller arms for a supported curve");
    controller
        .preview(PickedRecordValues::CurvePoints {
            x0: original.x0,
            y0: (original.y0 + 0.01).min(2.0),
            x1: original.x1,
            y1: original.y1,
        })
        .expect("preview applies and verifies");
    controller
        .revert_now()
        .expect("revert applies and verifies");
    let restored = bezier_record(&read_listing(&mut probe), &curve_name);
    for (restored_point, original_point) in [
        (restored.x0, original.x0),
        (restored.y0, original.y0),
        (restored.x1, original.x1),
        (restored.y1, original.y1),
    ] {
        assert!((restored_point - original_point).abs() < 1e-6);
    }
    println!("CONTROLLER PROOF PASSED hl.curve {curve_name}: preview + revert, zero residue");
}

/// Save flow proof: gated_family_record_save really persists a picked
/// record (backup, one atomic write, reread verification, no restore by
/// production code), then the flow proof restores the pre-test bytes.
#[test]
#[ignore = "writes the active Hyprland config; run only with HYPRLAND_SETTINGS_RUN_FAMILY_RECORD_SAVE_LIVE=1"]
fn gated_family_record_save_flow_proof() {
    if !live_gate("HYPRLAND_SETTINGS_RUN_FAMILY_RECORD_SAVE_LIVE") {
        return;
    }
    let mut runner = HyprctlRuntimePreviewRunner;
    let discovery = discover_hyprland_config();
    let config_path = match &discovery.status {
        hyprland_settings::config_discovery::ConfigDiscoveryStatus::Found { path, .. } => {
            path.clone()
        }
        other => panic!("config not discovered: {other:?}"),
    };

    let before_mode = read_safe_live_save_mode_status(&mut runner);
    if before_mode.state == SafeLiveSaveModeState::Inactive {
        enable_safe_live_save_mode(&mut runner).expect("enable verifies");
    }

    let listing = read_listing(&mut runner);
    let animation_name =
        pick_animation_proof_record(&listing).expect("animation proof record exists");
    let animation = animation_record(&listing, &animation_name);
    let animation_speed: f64 = animation.speed.parse().expect("speed parses");
    let curve_name = pick_curve_proof_record(&listing).expect("curve proof record exists");
    let curve = bezier_record(&listing, &curve_name);

    let requests = vec![
        FamilyRecordSaveRequest::AnimationRecordFields {
            record: animation_name.clone(),
            enabled: animation.enabled == "1",
            speed: animation_speed,
            bezier: if animation.bezier.is_empty() {
                "default".to_string()
            } else {
                animation.bezier.clone()
            },
        },
        FamilyRecordSaveRequest::CurveRecordPoints {
            record: curve_name.clone(),
            x0: curve.x0,
            y0: curve.y0,
            x1: curve.x1,
            y1: curve.y1,
        },
    ];

    for request in requests {
        let label = format!("{} {}", request.family_id(), request.record());
        let pre_bytes = std::fs::read(&config_path).expect("config reads");
        let receipt = gated_family_record_save(&mut runner, &discovery, request)
            .expect("gated record save succeeds with the mode active");
        assert!(receipt.reread_verified);
        assert!(!receipt.restored_after_success);
        assert!(!receipt.reload_run);
        assert_ne!(receipt.pre_save_hash, receipt.post_save_hash);
        let saved_text = std::fs::read_to_string(&receipt.config_path).expect("config rereads");
        assert!(saved_text.contains(&receipt.rendered_line));
        let backup_bytes = std::fs::read(&receipt.backup_path).expect("backup reads");
        assert_eq!(backup_bytes, pre_bytes, "backup is byte-exact");
        println!(
            "RECORD SAVE FLOW PROOF PASSED {label} | wrote {:?} | backup {} | reread-verified",
            receipt.rendered_line,
            receipt.backup_path.display()
        );

        // Flow-proof cleanup (not production behavior): restore pre-test bytes.
        std::fs::write(&receipt.config_path, &pre_bytes).expect("flow-proof restore");
        let restored = std::fs::read(&receipt.config_path).expect("reread");
        assert_eq!(restored, pre_bytes, "flow-proof restore is byte-exact");
        println!("FLOW-PROOF CLEANUP: config restored byte-exactly");
    }

    if before_mode.state == SafeLiveSaveModeState::Inactive {
        disable_safe_live_save_mode(&mut runner).expect("disable verifies");
    }
    let after_mode = read_safe_live_save_mode_status(&mut runner);
    assert_eq!(
        after_mode.state, before_mode.state,
        "autoreload state restored"
    );
}
