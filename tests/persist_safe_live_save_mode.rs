//! Safe Live Save Mode persistence: gate behavior, persisted-state
//! projection, and source guards. Normal tests only — no runtime mutation
//! and no active-config writes (the live proof is env-gated in
//! tests/persist_safe_live_save_mode_live.rs).

use std::fs;

use hyprland_settings::config_discovery::{
    ConfigDiscovery, ConfigDiscoveryStatus, ConfigPathSource,
};
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::persist_safe_live_save_mode::{
    persist_safe_live_save_mode_gate, read_persisted_safe_live_save_mode,
    PersistedSafeLiveSaveModeState, SAFE_LIVE_SAVE_MODE_PERSIST_VALUE,
    SAFE_LIVE_SAVE_MODE_SETTING_ID,
};
use hyprland_settings::runtime_preview_executor::RuntimePreviewRunner;

struct MockRunner {
    autoreload: Result<&'static str, ()>,
}

impl RuntimePreviewRunner for MockRunner {
    fn run(&mut self, _program: &str, args: &[String]) -> Result<String, String> {
        match args[0].as_str() {
            "getoption" => match self.autoreload {
                Ok(value) => Ok(format!("bool: {value}\nset: true")),
                Err(()) => Err("getoption failed".to_string()),
            },
            other => Err(format!("unexpected command {other}")),
        }
    }
}

fn snapshot_for(config_text: &str) -> CurrentConfigSnapshot {
    let temp_dir = std::env::temp_dir().join(format!(
        "hyprland-settings-persist-safe-mode-{}",
        std::process::id()
    ));
    fs::create_dir_all(&temp_dir).expect("dir");
    let path = temp_dir.join("persisted-state.conf");
    fs::write(&path, config_text).expect("writes");
    let discovery = ConfigDiscovery {
        status: ConfigDiscoveryStatus::Found {
            path: path.clone(),
            source: ConfigPathSource::XdgConfigHome,
        },
        attempted_paths: vec![path],
    };
    CurrentConfigSnapshot::from_discovery(&discovery)
}

#[test]
fn the_module_can_only_persist_disable_autoreload_true() {
    assert_eq!(SAFE_LIVE_SAVE_MODE_SETTING_ID, "misc.disable_autoreload");
    assert_eq!(SAFE_LIVE_SAVE_MODE_PERSIST_VALUE, "true");
}

#[test]
fn persist_gate_is_blocked_when_runtime_mode_is_inactive_or_unreadable() {
    // Inactive: autoreload still active — a write would reload Hyprland.
    let mut runner = MockRunner {
        autoreload: Ok("false"),
    };
    let error = persist_safe_live_save_mode_gate(&mut runner)
        .expect_err("blocked while the runtime mode is inactive");
    assert!(error.contains("blocked"));
    assert!(error.contains("must be active at runtime"));

    // Unreadable: fail closed.
    let mut runner = MockRunner {
        autoreload: Err(()),
    };
    let error = persist_safe_live_save_mode_gate(&mut runner)
        .expect_err("blocked when the runtime state is unreadable");
    assert!(error.contains("blocked"));

    // Active: the gate opens (the gated Save re-verifies it internally).
    let mut runner = MockRunner {
        autoreload: Ok("true"),
    };
    persist_safe_live_save_mode_gate(&mut runner).expect("gate opens when the mode is active");
}

#[test]
fn persisted_state_projection_reads_the_config_snapshot() {
    // The projection reads the same flat syntax the gated Save writes.
    let persisted = snapshot_for("misc:disable_autoreload = true\n");
    assert_eq!(
        read_persisted_safe_live_save_mode(&persisted),
        PersistedSafeLiveSaveModeState::PersistedTrue
    );

    let persisted_other = snapshot_for("misc:disable_autoreload = false\n");
    assert_eq!(
        read_persisted_safe_live_save_mode(&persisted_other),
        PersistedSafeLiveSaveModeState::PersistedOther
    );

    let not_persisted = snapshot_for("general {\n    gaps_in = 5\n}\n");
    assert_eq!(
        read_persisted_safe_live_save_mode(&not_persisted),
        PersistedSafeLiveSaveModeState::NotPersisted
    );

    // Every state has explanatory user text.
    for state in [
        PersistedSafeLiveSaveModeState::PersistedTrue,
        PersistedSafeLiveSaveModeState::PersistedOther,
        PersistedSafeLiveSaveModeState::NotPersisted,
        PersistedSafeLiveSaveModeState::Unknown,
    ] {
        assert!(!state.user_text().is_empty());
    }
}

#[test]
fn persistence_sources_stay_guarded() {
    let module = fs::read_to_string("src/persist_safe_live_save_mode.rs").expect("module reads");

    // The only write path is the gated scalar Save; the module never
    // touches files, never spawns processes, never reloads.
    assert!(module.contains("gated_scalar_save_live("));
    for forbidden in [
        "fs::write",
        "fs::read",
        "File::create",
        "OpenOptions",
        "apply_setting_change",
        "atomic_controlled_write",
        "hyprctl reload",
        "\"reload\"",
        "Command::new",
        "std::process",
    ] {
        assert!(
            !module.contains(forbidden),
            "persist module must not contain {forbidden}"
        );
    }

    // The setting id and value are constants: no public function accepts a
    // setting id or value, so nothing else can be persisted through here.
    assert_eq!(
        module
            .matches("pub fn persist_safe_live_save_mode_live(")
            .count(),
        1
    );
    assert!(!module.contains("setting_id: &str"));
    assert!(!module.contains("proposed_value: &str"));

    // The gate is verified before the save and cannot be bypassed.
    assert!(module.contains("persist_safe_live_save_mode_gate(&mut runner)?;"));
    assert!(module.contains("require_safe_live_save_mode(runner)"));

    // The UI routes through the module's live wrapper only, and the user
    // must choose the persistence (a button, never automatic).
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(window.contains("persist_safe_live_save_mode_live("));
    assert!(window.contains("Save as default"));
    assert!(window.contains("hyprland-settings-safe-live-save-persist"));
    assert!(!window.contains("apply_setting_change("));
}
