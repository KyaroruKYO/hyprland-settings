//! Family record picker: classification honesty, modify-existing
//! enforcement, gate ordering, and source guards. Normal tests only — no
//! runtime mutation, no active-config writes (live proofs are env-gated in
//! tests/family_record_picker_live.rs).

use std::fs;

use hyprland_settings::config_discovery::{
    ConfigDiscovery, ConfigDiscoveryStatus, ConfigPathSource,
};
use hyprland_settings::family_record_picker::{
    animation_record_entries, curve_record_entries, record_name_is_safe,
    render_animation_preview_expression, render_curve_preview_expression, save_picked_record,
    FamilyRecordPreviewController, PickedFamily, PickedRecordValues, RecordPickerError,
    RecordPickerPhase, RecordPickerSupport,
};
use hyprland_settings::runtime_preview_executor::RuntimePreviewRunner;
use hyprland_settings::structured_family_gated_persistence::{
    gated_family_record_save, record_line_matches_name, render_record_request_line,
    FamilyRecordSaveRequest, FamilySaveError,
};
use hyprland_settings::structured_family_runtime_preview::{
    parse_animation_records, parse_bezier_records, proven_record_shape_proof,
    ANIMATION_RECORD_SPEED_SHAPE, CURVE_RECORD_POINTS_SHAPE, PROVEN_RECORD_SHAPE_PROOFS,
};

/// A listing in the real `hyprctl animations` format with one record of
/// every classification kind.
fn mock_listing(fade_speed: &str, quick_x1: f64) -> String {
    format!(
        "animations:\n\n\
         \tname: specialWorkspaceOut\n\t\toverriden: 0\n\t\tbezier: \n\t\tenabled: -1\n\t\tspeed: 0.00\n\t\tstyle: \n\n\
         \tname: borderangle\n\t\toverriden: 1\n\t\tbezier: default\n\t\tenabled: 0\n\t\tspeed: 1.00\n\t\tstyle: \n\n\
         \tname: workspaces\n\t\toverriden: 1\n\t\tbezier: easeOutQuint\n\t\tenabled: 1\n\t\tspeed: 3.00\n\t\tstyle: fade\n\n\
         \tname: global\n\t\toverriden: 1\n\t\tbezier: default\n\t\tenabled: 1\n\t\tspeed: 8.00\n\t\tstyle: \n\n\
         \tname: __internal_fadeCTM\n\t\toverriden: 1\n\t\tbezier: linear\n\t\tenabled: 1\n\t\tspeed: 5.00\n\t\tstyle: \n\n\
         \tname: fade\n\t\toverriden: 1\n\t\tbezier: quick\n\t\tenabled: 1\n\t\tspeed: {fade_speed}\n\t\tstyle: \n\n\
         beziers:\n\n\
         \tname: quick\n\t\tX0: 0.15\n\t\tY0: 0.00\n\t\tX1: {quick_x1}\n\t\tY1: 1.00\n\
         \tname: default\n\t\tX0: 0.00\n\t\tY0: 0.75\n\t\tX1: 0.15\n\t\tY1: 1.00\n"
    )
}

/// Stateful mock runner: getoption reports the scripted autoreload state,
/// animations renders the current mock state, eval updates it.
struct MockRunner {
    autoreload: &'static str,
    fade_speed: String,
    quick_x1: f64,
    eval_count: usize,
}

impl MockRunner {
    fn new(autoreload: &'static str) -> Self {
        Self {
            autoreload,
            fade_speed: "3.00".to_string(),
            quick_x1: 0.10,
            eval_count: 0,
        }
    }
}

impl RuntimePreviewRunner for MockRunner {
    fn run(&mut self, _program: &str, args: &[String]) -> Result<String, String> {
        match args[0].as_str() {
            "getoption" => Ok(format!("bool: {}\nset: true", self.autoreload)),
            "animations" => Ok(mock_listing(&self.fade_speed, self.quick_x1)),
            "eval" => {
                self.eval_count += 1;
                let expression = &args[1];
                if let Some(speed_part) = expression.split("speed = ").nth(1) {
                    let speed: String = speed_part
                        .chars()
                        .take_while(|character| character.is_ascii_digit() || *character == '.')
                        .collect();
                    self.fade_speed = speed;
                }
                if expression.starts_with("hl.curve(\"quick\"") {
                    if let Some(points) = expression.split("points = { {").nth(1) {
                        if let Some(second_pair) = points.split("}, {").nth(1) {
                            if let Some(x1) = second_pair.split(',').next() {
                                self.quick_x1 = x1.trim().parse().unwrap_or(self.quick_x1);
                            }
                        }
                    }
                }
                Ok(String::new())
            }
            other => Err(format!("unexpected command {other}")),
        }
    }
}

fn found_discovery(path: std::path::PathBuf) -> ConfigDiscovery {
    ConfigDiscovery {
        status: ConfigDiscoveryStatus::Found {
            path: path.clone(),
            source: ConfigPathSource::XdgConfigHome,
        },
        attempted_paths: vec![path],
    }
}

#[test]
fn shape_proof_receipts_exist_for_both_picker_shapes() {
    assert_eq!(PROVEN_RECORD_SHAPE_PROOFS.len(), 2);
    let animation = proven_record_shape_proof("hl.animation", ANIMATION_RECORD_SPEED_SHAPE)
        .expect("animation shape receipt recorded");
    assert_eq!(animation.proven_on_record, "fade");
    assert!(animation.verification.contains("zero residue"));
    let curve = proven_record_shape_proof("hl.curve", CURVE_RECORD_POINTS_SHAPE)
        .expect("curve shape receipt recorded");
    assert_eq!(curve.proven_on_record, "quick");
    // Receipts prove generalization on non-family-proof records.
    assert_ne!(animation.proven_on_record, "global");
    assert_ne!(curve.proven_on_record, "default");
}

#[test]
fn listing_parsers_read_the_real_readback_format() {
    let listing = mock_listing("3.00", 0.10);
    let animations = parse_animation_records(&listing);
    assert_eq!(animations.len(), 6);
    let workspaces = animations
        .iter()
        .find(|record| record.name == "workspaces")
        .expect("workspaces parsed");
    assert!(workspaces.overridden);
    assert_eq!(workspaces.style, "fade");
    assert_eq!(workspaces.bezier, "easeOutQuint");
    let inherited = animations
        .iter()
        .find(|record| record.name == "specialWorkspaceOut")
        .expect("inherited leaf parsed");
    assert!(!inherited.overridden);

    let beziers = parse_bezier_records(&listing);
    assert_eq!(beziers.len(), 2);
    let quick = beziers
        .iter()
        .find(|record| record.name == "quick")
        .expect("quick parsed");
    assert!((quick.x0 - 0.15).abs() < 1e-9);
    assert!((quick.x1 - 0.10).abs() < 1e-9);
}

#[test]
fn animation_classification_is_honest_per_record() {
    let entries = animation_record_entries(&mock_listing("3.00", 0.10));
    let support = |name: &str| {
        entries
            .iter()
            .find(|entry| entry.record.name == name)
            .unwrap_or_else(|| panic!("{name} listed"))
    };

    // Proven: overridden, enabled, style-free.
    let fade = support("fade");
    assert_eq!(fade.support, RecordPickerSupport::SupportedProven);
    assert!(fade.preview_supported && fade.save_supported);

    let global = support("global");
    assert_eq!(global.support, RecordPickerSupport::SupportedProven);

    // Styled: save-only, style preserved on save.
    let workspaces = support("workspaces");
    assert_eq!(workspaces.support, RecordPickerSupport::SaveOnly);
    assert!(!workspaces.preview_supported && workspaces.save_supported);
    assert!(workspaces
        .blocked_reason
        .as_deref()
        .expect("reason")
        .contains("style"));

    // Disabled at runtime: save-only (found by live proof).
    let borderangle = support("borderangle");
    assert_eq!(borderangle.support, RecordPickerSupport::SaveOnly);
    assert!(borderangle
        .blocked_reason
        .as_deref()
        .expect("reason")
        .contains("disabled"));

    // Inherited: blocked — saving would create an override.
    let inherited = support("specialWorkspaceOut");
    assert_eq!(inherited.support, RecordPickerSupport::Blocked);
    assert!(!inherited.save_supported);
    assert!(inherited
        .blocked_reason
        .as_deref()
        .expect("reason")
        .contains("creation is blocked"));

    // Internal: blocked.
    let internal = support("__internal_fadeCTM");
    assert_eq!(internal.support, RecordPickerSupport::Blocked);
    assert!(!internal.save_supported);
}

#[test]
fn curve_classification_supports_existing_curves_only() {
    let entries = curve_record_entries(&mock_listing("3.00", 0.10));
    assert_eq!(entries.len(), 2);
    for entry in &entries {
        assert_eq!(entry.support, RecordPickerSupport::SupportedProven);
        assert!(entry.preview_supported && entry.save_supported);
    }
}

#[test]
fn record_names_outside_the_safe_set_are_rejected() {
    assert!(record_name_is_safe("fade"));
    assert!(record_name_is_safe("easeOutQuint"));
    assert!(!record_name_is_safe(""));
    assert!(!record_name_is_safe("bad name"));
    assert!(!record_name_is_safe("bad\"name"));
    assert!(!record_name_is_safe("bad,name"));
}

#[test]
fn preview_expressions_are_fixed_shape() {
    let listing = mock_listing("3.00", 0.10);
    let fade = parse_animation_records(&listing)
        .into_iter()
        .find(|record| record.name == "fade")
        .expect("fade");
    assert_eq!(
        render_animation_preview_expression(&fade, 3.25).expect("renders"),
        "hl.animation({ leaf = \"fade\", enabled = true, speed = 3.25, bezier = \"quick\" })"
    );
    assert_eq!(
        render_curve_preview_expression("quick", 0.15, 0.0, 0.11, 1.0).expect("renders"),
        "hl.curve(\"quick\", { type = \"bezier\", points = { {0.15, 0}, {0.11, 1} } })"
    );
    // Out-of-range values refuse before any command exists.
    assert!(render_animation_preview_expression(&fade, 0.0).is_err());
    assert!(render_curve_preview_expression("quick", 1.5, 0.0, 0.1, 1.0).is_err());
}

#[test]
fn controller_refuses_blocked_and_missing_records() {
    let listing = mock_listing("3.00", 0.10);
    for blocked in [
        "workspaces",
        "borderangle",
        "specialWorkspaceOut",
        "__internal_fadeCTM",
    ] {
        let error = FamilyRecordPreviewController::new(
            PickedFamily::Animation,
            blocked,
            Box::new(MockRunner::new("true")),
            &listing,
        )
        .err()
        .unwrap_or_else(|| panic!("{blocked} must refuse"));
        assert!(matches!(error, RecordPickerError::RecordNotSupported(_)));
    }
    let error = FamilyRecordPreviewController::new(
        PickedFamily::Animation,
        "does_not_exist",
        Box::new(MockRunner::new("true")),
        &listing,
    )
    .err()
    .expect("missing record refuses");
    assert!(matches!(error, RecordPickerError::RecordMissing(_)));
}

#[test]
fn controller_round_trips_with_verified_apply_and_revert() {
    let listing = mock_listing("3.00", 0.10);
    let mut controller = FamilyRecordPreviewController::new(
        PickedFamily::Animation,
        "fade",
        Box::new(MockRunner::new("true")),
        &listing,
    )
    .expect("fade is supported");
    assert_eq!(controller.phase(), RecordPickerPhase::Disarmed);
    assert_eq!(controller.current_value().expect("reads"), "3.00");

    let receipt = controller
        .preview(PickedRecordValues::AnimationSpeed { speed: 3.25 })
        .expect("preview applies and verifies");
    assert_eq!(receipt.phase, RecordPickerPhase::CountingDown);
    assert!(!receipt.config_written);
    assert!(!receipt.reload_run);
    assert_eq!(controller.current_value().expect("reads"), "3.25");

    controller.revert_now().expect("revert verifies");
    assert_eq!(controller.phase(), RecordPickerPhase::Reverted);
    assert_eq!(controller.current_value().expect("reads"), "3");
}

#[test]
fn controller_timeout_auto_reverts_and_session_drop_reverts_unconfirmed() {
    let listing = mock_listing("3.00", 0.10);
    let mut controller = FamilyRecordPreviewController::new(
        PickedFamily::Curve,
        "quick",
        Box::new(MockRunner::new("true")),
        &listing,
    )
    .expect("quick is supported");
    controller
        .preview(PickedRecordValues::CurvePoints {
            x0: 0.15,
            y0: 0.0,
            x1: 0.11,
            y1: 1.0,
        })
        .expect("preview applies");
    let receipt = controller
        .tick(60_000)
        .expect("tick evaluates")
        .expect("timeout reverts");
    assert_eq!(receipt.phase, RecordPickerPhase::TimedOutReverted);

    // Session drop reverts only unconfirmed previews.
    let mut controller = FamilyRecordPreviewController::new(
        PickedFamily::Animation,
        "fade",
        Box::new(MockRunner::new("true")),
        &listing,
    )
    .expect("supported");
    controller
        .preview(PickedRecordValues::AnimationSpeed { speed: 3.25 })
        .expect("preview applies");
    assert!(controller.revert_if_unconfirmed().is_some());
    let mut controller = FamilyRecordPreviewController::new(
        PickedFamily::Animation,
        "fade",
        Box::new(MockRunner::new("true")),
        &listing,
    )
    .expect("supported");
    controller
        .preview(PickedRecordValues::AnimationSpeed { speed: 3.25 })
        .expect("preview applies");
    controller.keep().expect("keep confirms");
    assert!(controller.revert_if_unconfirmed().is_none());
}

#[test]
fn record_request_render_enforces_modify_existing_and_preserves_style() {
    let mut runner = MockRunner::new("true");

    // Styled record: the rendered line preserves the style field.
    let line = render_record_request_line(
        &FamilyRecordSaveRequest::AnimationRecordSpeed {
            record: "workspaces".to_string(),
            speed: 3.5,
        },
        &mut runner,
    )
    .expect("renders");
    assert_eq!(line, "animation = workspaces, 1, 3.5, easeOutQuint, fade");

    // Disabled record: onoff renders 0.
    let line = render_record_request_line(
        &FamilyRecordSaveRequest::AnimationRecordSpeed {
            record: "borderangle".to_string(),
            speed: 1.5,
        },
        &mut runner,
    )
    .expect("renders");
    assert_eq!(line, "animation = borderangle, 0, 1.5, default");

    // Inherited record: refused — saving would create an override.
    let error = render_record_request_line(
        &FamilyRecordSaveRequest::AnimationRecordSpeed {
            record: "specialWorkspaceOut".to_string(),
            speed: 3.0,
        },
        &mut runner,
    )
    .expect_err("inherited records refuse");
    assert!(matches!(error, FamilySaveError::InvalidValue(_)));

    // Missing record: refused.
    assert!(render_record_request_line(
        &FamilyRecordSaveRequest::CurveRecordPoints {
            record: "missing".to_string(),
            x0: 0.0,
            y0: 0.0,
            x1: 1.0,
            y1: 1.0,
        },
        &mut runner,
    )
    .is_err());

    // Curve renders all four points.
    let line = render_record_request_line(
        &FamilyRecordSaveRequest::CurveRecordPoints {
            record: "quick".to_string(),
            x0: 0.15,
            y0: 0.0,
            x1: 0.11,
            y1: 1.0,
        },
        &mut runner,
    )
    .expect("renders");
    assert_eq!(line, "bezier = quick, 0.15, 0, 0.11, 1");
}

#[test]
fn record_save_is_blocked_without_safe_live_save_mode() {
    let mut runner = MockRunner::new("false");
    let discovery = found_discovery(std::env::temp_dir().join("never-touched.conf"));
    let error = gated_family_record_save(
        &mut runner,
        &discovery,
        FamilyRecordSaveRequest::AnimationRecordSpeed {
            record: "fade".to_string(),
            speed: 3.25,
        },
    )
    .expect_err("save must be blocked while autoreload is active");
    assert!(matches!(
        error,
        FamilySaveError::SafeLiveSaveModeRequired(_)
    ));
    assert_eq!(runner.eval_count, 0, "no runtime command was issued");
}

#[test]
fn record_save_refuses_bad_values_unsafe_names_and_non_active_targets() {
    // Invalid values and unsafe names refuse before any gate or file access.
    let mut runner = MockRunner::new("true");
    let discovery = found_discovery(std::env::temp_dir().join("never-touched.conf"));
    for request in [
        FamilyRecordSaveRequest::AnimationRecordSpeed {
            record: "fade".to_string(),
            speed: f64::NAN,
        },
        FamilyRecordSaveRequest::AnimationRecordSpeed {
            record: "fade".to_string(),
            speed: 100.0,
        },
        FamilyRecordSaveRequest::AnimationRecordSpeed {
            record: "__internal_fadeCTM".to_string(),
            speed: 5.0,
        },
        FamilyRecordSaveRequest::AnimationRecordSpeed {
            record: "bad name".to_string(),
            speed: 5.0,
        },
        FamilyRecordSaveRequest::CurveRecordPoints {
            record: "quick".to_string(),
            x0: 1.5,
            y0: 0.0,
            x1: 0.1,
            y1: 1.0,
        },
    ] {
        assert!(matches!(
            gated_family_record_save(&mut runner, &discovery, request),
            Err(FamilySaveError::InvalidValue(_))
        ));
    }

    // A non-active config target is refused (identity gate).
    let temp = std::env::temp_dir().join("hyprland-settings-picker-not-active.conf");
    fs::write(&temp, "general:gaps_in = 5\n").expect("temp writes");
    let error = gated_family_record_save(
        &mut MockRunner::new("true"),
        &found_discovery(temp.clone()),
        FamilyRecordSaveRequest::AnimationRecordSpeed {
            record: "fade".to_string(),
            speed: 3.25,
        },
    )
    .expect_err("non-active config must be refused");
    assert!(matches!(error, FamilySaveError::TargetIdentityFailed(_)));
    fs::remove_file(&temp).ok();
}

#[test]
fn save_picked_record_routes_through_gated_persistence_only() {
    // Behavioral: the picker save surfaces the same gate errors.
    let mut runner = MockRunner::new("false");
    let discovery = found_discovery(std::env::temp_dir().join("never-touched.conf"));
    let error = save_picked_record(
        &mut runner,
        &discovery,
        PickedFamily::Animation,
        "fade",
        PickedRecordValues::AnimationSpeed { speed: 3.25 },
    )
    .expect_err("gate error surfaces through the picker save");
    assert!(matches!(
        error,
        FamilySaveError::SafeLiveSaveModeRequired(_)
    ));

    // Mismatched family/values refuse.
    assert!(matches!(
        save_picked_record(
            &mut MockRunner::new("true"),
            &discovery,
            PickedFamily::Curve,
            "quick",
            PickedRecordValues::AnimationSpeed { speed: 3.25 },
        ),
        Err(FamilySaveError::InvalidValue(_))
    ));
}

#[test]
fn record_line_name_matching_is_exact() {
    assert!(record_line_matches_name(
        "animation = fade, 1, 3, quick",
        "fade"
    ));
    assert!(!record_line_matches_name(
        "animation = fadeIn, 1, 3, quick",
        "fade"
    ));
    assert!(record_line_matches_name(
        "bezier = quick, 0.15, 0, 0.1, 1",
        "quick"
    ));
    assert!(!record_line_matches_name(
        "bezier = quicker, 0, 0, 1, 1",
        "quick"
    ));
}

#[test]
fn picker_sources_stay_guarded() {
    let module = fs::read_to_string("src/family_record_picker.rs").expect("module reads");
    // The picker can express only the two proven families.
    for forbidden in [
        "hl.monitor",
        "hl.bind",
        "hl.device",
        "hl.permission",
        "hl.gesture",
    ] {
        assert!(
            !module.contains(forbidden),
            "picker must not reference {forbidden}"
        );
    }
    // The picker never writes files, never spawns processes, never reloads,
    // and holds no UI code.
    for forbidden in [
        "fs::write",
        "fs::File",
        "OpenOptions",
        "atomic_controlled_write",
        "apply_rendered_family_records",
        "apply_setting_change",
        "hyprctl reload",
        "\"reload\"",
        "Command::new",
        "std::process",
        "gtk::",
        "connect_clicked",
    ] {
        assert!(
            !module.contains(forbidden),
            "picker must not contain {forbidden}"
        );
    }
    // No creation or removal operations exist.
    for forbidden in ["fn create", "fn remove", "fn add_record", "delete"] {
        assert!(
            !module.contains(forbidden),
            "picker must not contain {forbidden}"
        );
    }
    // Saving goes through the gated persistence module only.
    assert!(module.contains("gated_family_record_save(runner, discovery, request)"));

    // The persistence module gained exactly one generalized save entry and
    // one live wrapper alongside the pinned originals.
    let persistence =
        fs::read_to_string("src/structured_family_gated_persistence.rs").expect("module reads");
    assert_eq!(
        persistence
            .matches("pub fn gated_family_record_save(")
            .count(),
        1
    );
    assert_eq!(
        persistence
            .matches("pub fn gated_family_record_save_live(")
            .count(),
        1
    );
}
