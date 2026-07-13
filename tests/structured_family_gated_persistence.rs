use std::fs;

use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::runtime_preview_executor::RuntimePreviewRunner;
use hyprland_settings::structured_family::StructuredFamilyKind;
use hyprland_settings::structured_family_gated_persistence::{
    gated_family_save, record_matches_target, render_target_line, verify_saved_record,
    FamilySaveError,
};
use hyprland_settings::structured_family_preview_controller::FamilyPreviewTarget;

const LISTING: &str = "animations:\n\n\tname: global\n\t\toverriden: 1\n\t\tbezier: default\n\t\tenabled: 1\n\t\tspeed: 8.00\n\t\tstyle: \n\nbeziers:\n\n\tname: default\n\t\tX0: 0.00\n\t\tY0: 0.75\n\t\tX1: 0.15\n\t\tY1: 1.00\n";

/// Mock runner: scripted autoreload state plus the animations listing.
struct MockRunner {
    autoreload: &'static str,
}

impl RuntimePreviewRunner for MockRunner {
    fn run(&mut self, _program: &str, args: &[String]) -> Result<String, String> {
        match args[0].as_str() {
            "getoption" => Ok(format!("bool: {}\nset: true", self.autoreload)),
            "animations" => Ok(LISTING.to_string()),
            other => Err(format!("unexpected command {other}")),
        }
    }
}

fn found_discovery(path: std::path::PathBuf) -> ConfigDiscovery {
    ConfigDiscovery {
        status: ConfigDiscoveryStatus::Found {
            path: path.clone(),
            source: hyprland_settings::config_discovery::ConfigPathSource::XdgConfigHome,
        },
        attempted_paths: vec![path],
    }
}

#[test]
fn save_is_blocked_without_safe_live_save_mode() {
    let mut runner = MockRunner {
        autoreload: "false",
    };
    let discovery = found_discovery(std::env::temp_dir().join("never-touched.conf"));
    let error = gated_family_save(
        &mut runner,
        &discovery,
        FamilyPreviewTarget::AnimationGlobalSpeed,
        8.5,
    )
    .expect_err("save must be blocked while autoreload is active");
    assert!(matches!(
        error,
        FamilySaveError::SafeLiveSaveModeRequired(_)
    ));
    assert!(error.user_text().contains("Enable Safe Live Save Mode"));
}

#[test]
fn save_refuses_non_active_config_targets_and_bad_values() {
    // With the gate open, a non-active config path is refused (identity).
    let mut runner = MockRunner { autoreload: "true" };
    let temp = std::env::temp_dir().join("hyprland-settings-not-active.conf");
    fs::write(&temp, "general:gaps_in = 5\n").expect("temp writes");
    let discovery = found_discovery(temp.clone());
    let error = gated_family_save(
        &mut runner,
        &discovery,
        FamilyPreviewTarget::AnimationGlobalSpeed,
        8.5,
    )
    .expect_err("non-active config must be refused");
    assert!(matches!(error, FamilySaveError::TargetIdentityFailed(_)));
    fs::remove_file(&temp).ok();

    // Invalid values are refused before anything else happens.
    let mut runner = MockRunner { autoreload: "true" };
    let discovery = found_discovery(std::env::temp_dir().join("x.conf"));
    for bad in [f64::NAN, 0.0, 100.0] {
        assert!(matches!(
            gated_family_save(
                &mut runner,
                &discovery,
                FamilyPreviewTarget::AnimationGlobalSpeed,
                bad
            ),
            Err(FamilySaveError::InvalidValue(_))
        ));
    }
}

#[test]
fn rendered_lines_match_config_grammar_and_runtime_fields() {
    let mut runner = MockRunner { autoreload: "true" };
    let line = render_target_line(FamilyPreviewTarget::AnimationGlobalSpeed, 8.5, &mut runner)
        .expect("renders");
    assert_eq!(line, "animation = global, 1, 8.5, default");
    let line =
        render_target_line(FamilyPreviewTarget::CurveDefaultY0, 0.9, &mut runner).expect("renders");
    assert_eq!(line, "bezier = default, 0, 0.9, 0.15, 1");

    assert!(record_matches_target(
        FamilyPreviewTarget::AnimationGlobalSpeed,
        "animation = global, 1, 8, default"
    ));
    assert!(!record_matches_target(
        FamilyPreviewTarget::AnimationGlobalSpeed,
        "animation = windows, 1, 7, default"
    ));
    assert!(record_matches_target(
        FamilyPreviewTarget::CurveDefaultY0,
        "bezier = default, 0, 0.75, 0.15, 1"
    ));
}

#[test]
fn verification_requires_exactly_the_saved_record() {
    let temp_dir = std::env::temp_dir().join(format!(
        "hyprland-settings-persistence-verify-{}",
        std::process::id()
    ));
    fs::create_dir_all(&temp_dir).expect("dir");
    let path = temp_dir.join("verify.conf");

    // The saved record present exactly once verifies.
    fs::write(
        &path,
        "# c\nanimation = global, 1, 8.5, default\nbind = SUPER, K, exec, x\n",
    )
    .expect("writes");
    verify_saved_record(
        &path,
        StructuredFamilyKind::Animation,
        FamilyPreviewTarget::AnimationGlobalSpeed,
        "animation = global, 1, 8.5, default",
    )
    .expect("verifies");

    // A mismatched value fails.
    assert!(verify_saved_record(
        &path,
        StructuredFamilyKind::Animation,
        FamilyPreviewTarget::AnimationGlobalSpeed,
        "animation = global, 1, 9, default",
    )
    .is_err());

    // A missing record fails.
    fs::write(&path, "# nothing here\n").expect("writes");
    assert!(verify_saved_record(
        &path,
        StructuredFamilyKind::Animation,
        FamilyPreviewTarget::AnimationGlobalSpeed,
        "animation = global, 1, 8.5, default",
    )
    .is_err());

    fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn persistence_sources_stay_guarded() {
    let module =
        fs::read_to_string("src/structured_family_gated_persistence.rs").expect("module reads");
    // Only the two proven targets can be expressed; no other family appears.
    for forbidden in [
        "hl.monitor",
        "hl.bind",
        "hl.device",
        "hl.permission",
        "hl.gesture",
    ] {
        assert!(
            !module.contains(forbidden),
            "persistence must not reference {forbidden}"
        );
    }
    // No reload path and no record deletion exist.
    for forbidden in [
        "hyprctl reload",
        "\"reload\"",
        "Command::new",
        "std::process",
        "delete",
    ] {
        assert!(
            !module.contains(forbidden),
            "persistence must not contain {forbidden}"
        );
    }
    // The gate cannot be bypassed: require_safe_live_save_mode is called
    // before any filesystem access, and there is no alternate save entry.
    assert!(module.contains("require_safe_live_save_mode(runner)"));
    assert_eq!(
        module.matches("pub fn gated_family_save(").count(),
        1,
        "exactly one save entry point"
    );
    assert_eq!(
        module.matches("pub fn gated_family_save_live(").count(),
        1,
        "exactly one live wrapper"
    );
    // UI routes through the module and never calls apply_setting_change or
    // the persistence internals directly.
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(!window.contains("apply_setting_change("));
    assert!(!window.contains("atomic_controlled_write"));
    assert!(!window.contains("apply_rendered_family_records"));
}
