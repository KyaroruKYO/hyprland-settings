//! Env-gated live flow proof for persisting Safe Live Save Mode. Writes the
//! REAL active config once through the gated scalar Save (backup, one
//! write, reread verification, no reload), verifies the persisted state
//! projection flips to PersistedTrue, then — because this is a flow proof,
//! not a user-intent save — restores the pre-test config bytes and the
//! original runtime autoreload state. Normal `cargo test` never runs this
//! body.

use std::collections::BTreeSet;

use hyprland_settings::config_discovery::discover_hyprland_config;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::persist_safe_live_save_mode::{
    persist_safe_live_save_mode_live, read_persisted_safe_live_save_mode,
    PersistedSafeLiveSaveModeState,
};
use hyprland_settings::runtime_preview_executor::{
    read_runtime_option, HyprctlRuntimePreviewRunner,
};
use hyprland_settings::safe_live_save_mode::{
    disable_safe_live_save_mode, enable_safe_live_save_mode, read_safe_live_save_mode_status,
    SafeLiveSaveModeState,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
#[ignore = "writes the active Hyprland config; run only with HYPRLAND_SETTINGS_RUN_PERSIST_SAFE_LIVE_SAVE_MODE=1"]
fn persist_safe_live_save_mode_flow_proof() {
    if std::env::var("HYPRLAND_SETTINGS_RUN_PERSIST_SAFE_LIVE_SAVE_MODE").as_deref() != Ok("1") {
        eprintln!("skipping: env not set");
        return;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
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
    let known_setting_ids: BTreeSet<String> = SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id.to_string())
        .collect();
    let pre_bytes = std::fs::read(&config_path).expect("config reads");

    // Gate proof: blocked while the runtime mode is inactive.
    let before_mode = read_safe_live_save_mode_status(&mut runner);
    if before_mode.state == SafeLiveSaveModeState::Inactive {
        let current_config = CurrentConfigSnapshot::from_discovery(&discovery);
        let error = persist_safe_live_save_mode_live(
            known_setting_ids.clone(),
            &discovery,
            &current_config,
        )
        .expect_err("persist must be blocked while the runtime mode is inactive");
        println!("GATE PROOF: persist blocked while runtime mode inactive: {error}");
        enable_safe_live_save_mode(&mut runner).expect("enable verifies");
    }

    // The real persist through the gated scalar Save.
    let current_config = CurrentConfigSnapshot::from_discovery(&discovery);
    let receipt = persist_safe_live_save_mode_live(known_setting_ids, &discovery, &current_config)
        .expect("persist succeeds with the runtime mode active");
    assert!(!receipt.reload_run);
    assert_eq!(receipt.setting_id, "misc.disable_autoreload");
    assert_eq!(receipt.persisted_value, "true");
    assert_eq!(
        receipt.verified_value.as_deref(),
        Some("true"),
        "the gated Save reread-verified the persisted value"
    );

    // The persisted-state projection now reports PersistedTrue.
    let after_config = CurrentConfigSnapshot::from_discovery(&discovery);
    assert_eq!(
        read_persisted_safe_live_save_mode(&after_config),
        PersistedSafeLiveSaveModeState::PersistedTrue,
        "the config now persists Safe Live Save Mode"
    );

    // Backup is byte-exact and outside the config.
    let backup_bytes = std::fs::read(&receipt.backup_path).expect("backup reads");
    assert_eq!(
        backup_bytes, pre_bytes,
        "backup preserves the pre-save bytes"
    );
    assert_ne!(receipt.backup_path, config_path);

    // No reload fired: the runtime flag is still true (a reload would have
    // re-read the config, which now also says true — but the write itself
    // completing without the compositor restarting is the observable here).
    let runtime_flag = read_runtime_option(&mut runner, "misc.disable_autoreload");
    assert_eq!(runtime_flag.as_deref(), Some("true"));
    println!(
        "PERSIST FLOW PROOF PASSED | {} | backup {}",
        receipt.status_text,
        receipt.backup_path.display()
    );

    // Flow-proof cleanup (not production behavior): restore pre-test bytes
    // and the original runtime state.
    std::fs::write(&config_path, &pre_bytes).expect("flow-proof restore");
    let restored = std::fs::read(&config_path).expect("reread");
    assert_eq!(restored, pre_bytes, "flow-proof restore is byte-exact");
    if before_mode.state == SafeLiveSaveModeState::Inactive {
        disable_safe_live_save_mode(&mut runner).expect("disable verifies");
    }
    let after_mode = read_safe_live_save_mode_status(&mut runner);
    assert_eq!(
        after_mode.state, before_mode.state,
        "runtime state restored"
    );
    println!("FLOW-PROOF CLEANUP: config and runtime state restored");
}
