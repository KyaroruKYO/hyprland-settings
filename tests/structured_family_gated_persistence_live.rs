use hyprland_settings::config_discovery::discover_hyprland_config;
use hyprland_settings::runtime_preview_executor::HyprctlRuntimePreviewRunner;
use hyprland_settings::safe_live_save_mode::{
    disable_safe_live_save_mode, enable_safe_live_save_mode, read_safe_live_save_mode_status,
    SafeLiveSaveModeState,
};
use hyprland_settings::structured_family_gated_persistence::gated_family_save;
use hyprland_settings::structured_family_preview_controller::FamilyPreviewTarget;

/// Live production-save flow proof. Enables Safe Live Save Mode at runtime,
/// performs a REAL gated save of the current runtime value to the active
/// config (backup, one atomic write, reread verification, no restore-on-
/// success in production code), then — because this is a flow proof, not a
/// user-intent save — restores the pre-test config bytes from the receipt's
/// backup and restores the original autoreload state. Ignored and env-gated;
/// normal cargo test never writes the active config.
#[test]
#[ignore = "writes the active Hyprland config; run only with HYPRLAND_SETTINGS_RUN_STRUCTURED_FAMILY_SAVE_LIVE=1 and HYPRLAND_SETTINGS_STRUCTURED_FAMILY_SAVE_TARGET=<hl.animation.global.speed|hl.curve.default.y0|all>"]
fn gated_family_save_flow_proof() {
    if std::env::var("HYPRLAND_SETTINGS_RUN_STRUCTURED_FAMILY_SAVE_LIVE").as_deref() != Ok("1") {
        eprintln!("skipping: env not set");
        return;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
        return;
    }
    let selector = std::env::var("HYPRLAND_SETTINGS_STRUCTURED_FAMILY_SAVE_TARGET")
        .unwrap_or_else(|_| "all".to_string());
    let mut runner = HyprctlRuntimePreviewRunner;
    let discovery = discover_hyprland_config();

    // Blocked while autoreload is active: prove the gate live first.
    let before_mode = read_safe_live_save_mode_status(&mut runner);
    if before_mode.state == SafeLiveSaveModeState::Inactive {
        let error = gated_family_save(
            &mut runner,
            &discovery,
            FamilyPreviewTarget::AnimationGlobalSpeed,
            8.0,
        )
        .expect_err("save must be blocked while autoreload is active");
        println!(
            "GATE PROOF: save blocked while autoreload active: {}",
            error.user_text()
        );
        enable_safe_live_save_mode(&mut runner).expect("enable verifies");
    }

    let targets: Vec<(FamilyPreviewTarget, f64, &str)> = match selector.as_str() {
        "hl.animation.global.speed" => {
            vec![(FamilyPreviewTarget::AnimationGlobalSpeed, 8.0, "animation")]
        }
        "hl.curve.default.y0" => vec![(FamilyPreviewTarget::CurveDefaultY0, 0.75, "curve")],
        _ => vec![
            (FamilyPreviewTarget::AnimationGlobalSpeed, 8.0, "animation"),
            (FamilyPreviewTarget::CurveDefaultY0, 0.75, "curve"),
        ],
    };

    for (target, value, label) in targets {
        let pre_bytes = std::fs::read(match &discovery.status {
            hyprland_settings::config_discovery::ConfigDiscoveryStatus::Found { path, .. } => {
                path.clone()
            }
            _ => panic!("config not discovered"),
        })
        .expect("config reads");

        let receipt = gated_family_save(&mut runner, &discovery, target, value)
            .expect("gated save should succeed with the mode active");
        assert!(receipt.reread_verified);
        assert!(
            !receipt.restored_after_success,
            "production save must not restore"
        );
        assert!(!receipt.reload_run);
        assert_ne!(
            receipt.pre_save_hash, receipt.post_save_hash,
            "the save must change the config"
        );
        let saved_text = std::fs::read_to_string(&receipt.config_path).expect("config rereads");
        assert!(
            saved_text.contains(&receipt.rendered_line),
            "saved line must be present in the config"
        );
        println!(
            "SAVE FLOW PROOF PASSED {label} | wrote {:?} | backup {} | reread-verified | no restore by production code",
            receipt.rendered_line,
            receipt.backup_path.display()
        );

        // Flow-proof cleanup (not production behavior): restore pre-test bytes.
        std::fs::write(&receipt.config_path, &pre_bytes).expect("flow-proof restore");
        let restored = std::fs::read(&receipt.config_path).expect("reread");
        assert_eq!(restored, pre_bytes, "flow-proof restore must be byte-exact");
        println!("FLOW-PROOF CLEANUP: config restored byte-exactly");
    }

    // Restore the original autoreload state.
    if before_mode.state == SafeLiveSaveModeState::Inactive {
        disable_safe_live_save_mode(&mut runner).expect("disable verifies");
    }
    let after_mode = read_safe_live_save_mode_status(&mut runner);
    assert_eq!(
        after_mode.state, before_mode.state,
        "autoreload state restored"
    );
}
