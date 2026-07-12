use hyprland_settings::runtime_preview_dead_man::{
    RuntimePreviewDeadManController, RuntimePreviewDeadManUiPhase,
};
use hyprland_settings::runtime_preview_executor::{
    parse_getoption_value, runtime_option_query, HyprctlRuntimePreviewRunner, RuntimePreviewRunner,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

/// Live supervised preview proof on the safest candidate row
/// (animations.enabled): arm, apply, verify live change, tick past the
/// timeout, and verify the automatic revert restored the original value.
/// Ignored by default and env-gated; normal `cargo test` never mutates the
/// live compositor.
#[test]
#[ignore = "mutates the live compositor; run only with HYPRLAND_SETTINGS_RUN_DEAD_MAN_PREVIEW_LIVE=1"]
fn live_dead_man_preview_times_out_and_auto_reverts_animations_enabled() {
    if std::env::var("HYPRLAND_SETTINGS_RUN_DEAD_MAN_PREVIEW_LIVE").as_deref() != Ok("1") {
        eprintln!("skipping: HYPRLAND_SETTINGS_RUN_DEAD_MAN_PREVIEW_LIVE is not set");
        return;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
        return;
    }

    let row_id = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "animations.enabled")
        .expect("animations.enabled row should exist")
        .row_id;

    let read_enabled = || -> String {
        let mut runner = HyprctlRuntimePreviewRunner;
        let output = runner
            .run("hyprctl", &runtime_option_query("animations.enabled"))
            .expect("getoption should succeed");
        parse_getoption_value(&output).expect("animations.enabled should parse")
    };

    let before = read_enabled();
    let preview_value = if before.starts_with("true") || before == "1" {
        "false"
    } else {
        "true"
    };

    let mut controller =
        RuntimePreviewDeadManController::new_live(row_id).expect("live controller builds");
    controller.arm().expect("arm succeeds");
    controller.apply(preview_value).expect("apply succeeds");
    let during = read_enabled();
    assert_ne!(
        during, before,
        "supervised preview must change the live value"
    );

    // Do not confirm: drive the countdown past the timeout and let the
    // dead-man auto-revert fire.
    let receipt = controller
        .tick(11_000)
        .expect("tick succeeds")
        .expect("timeout fires");
    assert_eq!(
        receipt.phase,
        RuntimePreviewDeadManUiPhase::TimedOutReverted
    );

    let after = read_enabled();
    assert_eq!(
        after, before,
        "timeout auto-revert must restore the original live value"
    );
}
