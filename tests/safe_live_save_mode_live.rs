use hyprland_settings::runtime_preview_executor::HyprctlRuntimePreviewRunner;
use hyprland_settings::safe_live_save_mode::{
    disable_safe_live_save_mode, enable_safe_live_save_mode, read_safe_live_save_mode_status,
    SafeLiveSaveModeState,
};

/// Live Safe Live Save Mode round trip: enable at runtime (verified), then
/// restore the original state (verified). Ignored and env-gated; normal
/// cargo test never mutates the compositor. No file is written and no
/// reload command exists in this path.
#[test]
#[ignore = "mutates the runtime autoreload flag; run only with HYPRLAND_SETTINGS_RUN_SAFE_LIVE_SAVE_MODE_PROOF=1"]
fn safe_live_save_mode_round_trips_at_runtime() {
    if std::env::var("HYPRLAND_SETTINGS_RUN_SAFE_LIVE_SAVE_MODE_PROOF").as_deref() != Ok("1") {
        eprintln!("skipping: env not set");
        return;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
        return;
    }
    let mut runner = HyprctlRuntimePreviewRunner;
    let before = read_safe_live_save_mode_status(&mut runner);

    match before.state {
        SafeLiveSaveModeState::Inactive => {
            let receipt = enable_safe_live_save_mode(&mut runner).expect("enable verifies");
            assert!(receipt.verified);
            let during = read_safe_live_save_mode_status(&mut runner);
            assert_eq!(during.state, SafeLiveSaveModeState::ActiveViaRuntime);
            assert!(during.save_gate_open);
            let receipt = disable_safe_live_save_mode(&mut runner).expect("disable verifies");
            assert!(receipt.verified);
            let after = read_safe_live_save_mode_status(&mut runner);
            assert_eq!(after.state, SafeLiveSaveModeState::Inactive);
            println!(
                "SAFE MODE PROOF PASSED | inactive -> active -> inactive, all readback-verified"
            );
        }
        SafeLiveSaveModeState::ActiveViaRuntime => {
            eprintln!("mode already active; leaving state untouched");
        }
        SafeLiveSaveModeState::Unknown => panic!("autoreload state unreadable"),
    }
}
