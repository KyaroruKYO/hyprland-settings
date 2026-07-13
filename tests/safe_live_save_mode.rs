use std::fs;

use hyprland_settings::runtime_preview_executor::RuntimePreviewRunner;
use hyprland_settings::safe_live_save_mode::{
    disable_safe_live_save_mode, enable_safe_live_save_mode, read_safe_live_save_mode_status,
    SafeLiveSaveModeState, SAFE_LIVE_SAVE_MODE_PROOF,
};

/// Mock runner scripting getoption responses and recording every call.
struct ScriptedRunner {
    getoption_values: Vec<&'static str>,
    calls: Vec<Vec<String>>,
}

impl RuntimePreviewRunner for ScriptedRunner {
    fn run(&mut self, _program: &str, args: &[String]) -> Result<String, String> {
        self.calls.push(args.to_vec());
        if args[0] == "getoption" {
            if self.getoption_values.is_empty() {
                return Err("no scripted value".to_string());
            }
            let value = self.getoption_values.remove(0);
            Ok(format!("bool: {value}\nset: true"))
        } else {
            Ok("ok".to_string())
        }
    }
}

#[test]
fn status_reads_runtime_autoreload_and_fails_closed() {
    // Autoreload active -> mode inactive, save gate closed, reason shown.
    let mut runner = ScriptedRunner {
        getoption_values: vec!["false"],
        calls: Vec::new(),
    };
    let status = read_safe_live_save_mode_status(&mut runner);
    assert_eq!(status.state, SafeLiveSaveModeState::Inactive);
    assert!(!status.save_gate_open);
    assert!(status.blocked_reason.is_some());
    assert!(!status.one_time_reload_warning_needed);

    // Autoreload disabled -> mode active, save gate open.
    let mut runner = ScriptedRunner {
        getoption_values: vec!["true"],
        calls: Vec::new(),
    };
    let status = read_safe_live_save_mode_status(&mut runner);
    assert_eq!(status.state, SafeLiveSaveModeState::ActiveViaRuntime);
    assert!(status.save_gate_open);
    assert!(status.blocked_reason.is_none());

    // Unreadable -> Unknown, fail closed.
    let mut runner = ScriptedRunner {
        getoption_values: vec![],
        calls: Vec::new(),
    };
    let status = read_safe_live_save_mode_status(&mut runner);
    assert_eq!(status.state, SafeLiveSaveModeState::Unknown);
    assert!(!status.save_gate_open);
}

#[test]
fn transitions_are_runtime_only_and_verified() {
    // Enable: read before, eval, read after (true) -> verified receipt.
    let mut runner = ScriptedRunner {
        getoption_values: vec!["false", "true"],
        calls: Vec::new(),
    };
    let receipt = enable_safe_live_save_mode(&mut runner).expect("enable verifies");
    assert!(receipt.verified);
    assert!(!receipt.config_written);
    assert!(!receipt.reload_run);
    assert_eq!(receipt.before.as_deref(), Some("false"));
    assert_eq!(receipt.after.as_deref(), Some("true"));
    // Exactly one eval between the two reads; the eval is the fixed constant.
    let evals: Vec<_> = runner
        .calls
        .iter()
        .filter(|call| call[0] == "eval")
        .collect();
    assert_eq!(evals.len(), 1);
    assert_eq!(
        evals[0][1],
        "hl.config({ misc = { disable_autoreload = true } })"
    );

    // Verification failure fails closed with an error.
    let mut runner = ScriptedRunner {
        getoption_values: vec!["false", "false"],
        calls: Vec::new(),
    };
    assert!(enable_safe_live_save_mode(&mut runner).is_err());

    // Disable: symmetric.
    let mut runner = ScriptedRunner {
        getoption_values: vec!["true", "false"],
        calls: Vec::new(),
    };
    let receipt = disable_safe_live_save_mode(&mut runner).expect("disable verifies");
    assert!(receipt.verified);
    assert!(!receipt.config_written);
}

#[test]
fn safe_mode_sources_never_write_config_or_reload() {
    let module = fs::read_to_string("src/safe_live_save_mode.rs").expect("module reads");
    for forbidden in [
        "fs::write",
        "File::create",
        "std::fs",
        ".config/hypr",
        "hyprctl reload",
        "\"reload\"",
        "Command::new",
        "std::process",
        "write_flow::",
        "apply_setting_change(",
    ] {
        assert!(
            !module.contains(forbidden),
            "safe mode module must not contain {forbidden}"
        );
    }
    assert!(
        !SAFE_LIVE_SAVE_MODE_PROOF.is_empty(),
        "the proof evidence must be recorded"
    );
    // The UI routes through the module: no autoreload runtime expression is
    // built in UI code (instructional copy may mention the setting name).
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(!window.contains("misc = { disable_autoreload"));
    assert!(!window.contains("hl.config"));
}
