use hyprland_settings::runtime_preview_executor::{
    apply_runtime_preview_value, parse_getoption_value, revert_runtime_preview_session,
    runtime_option_query, start_runtime_preview_session, HyprctlRuntimePreviewRunner,
    RuntimePreviewRunner, RuntimePreviewSessionState,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

/// Live runtime preview proof against the running compositor. Ignored by
/// default and additionally env-gated: normal `cargo test` never mutates the
/// live session. When run, it previews a known low-risk value
/// (general.gaps_in), verifies the change through a read-only getoption,
/// reverts to the captured original, and verifies the restoration.
#[test]
#[ignore = "mutates the live compositor; run only with HYPRLAND_SETTINGS_RUN_RUNTIME_PREVIEW_LIVE=1"]
fn live_gaps_in_preview_applies_and_reverts() {
    if std::env::var("HYPRLAND_SETTINGS_RUN_RUNTIME_PREVIEW_LIVE").as_deref() != Ok("1") {
        eprintln!("skipping: HYPRLAND_SETTINGS_RUN_RUNTIME_PREVIEW_LIVE is not set");
        return;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
        return;
    }

    let gaps_row = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "general.gaps_in")
        .expect("gaps_in row should exist");
    let mut runner = HyprctlRuntimePreviewRunner;

    let read_gaps = |runner: &mut HyprctlRuntimePreviewRunner| -> String {
        let output = runner
            .run("hyprctl", &runtime_option_query("general.gaps_in"))
            .expect("getoption should succeed");
        parse_getoption_value(&output).expect("gaps_in should parse")
    };

    let before = read_gaps(&mut runner);
    let mut session = start_runtime_preview_session(&mut runner, gaps_row.row_id, false)
        .expect("live session should start");
    assert_eq!(session.original_value, before);

    // Preview a visibly different, harmless gap value.
    let preview_value = "9";
    assert_ne!(
        before.split_whitespace().next(),
        Some(preview_value),
        "preview value must differ from the original"
    );
    let receipt = apply_runtime_preview_value(&mut runner, &mut session, preview_value)
        .expect("live preview apply should succeed");
    assert!(!receipt.config_written);
    assert!(!receipt.reload_run);
    let during = read_gaps(&mut runner);
    assert!(
        during.split_whitespace().all(|part| part == preview_value),
        "live gaps_in should reflect the preview value, got {during}"
    );

    // Revert and verify the original value is restored live.
    let revert = revert_runtime_preview_session(&mut runner, &mut session)
        .expect("live revert should succeed");
    assert_eq!(revert.restored_value, before);
    assert_eq!(session.state, RuntimePreviewSessionState::Reverted);
    let after = read_gaps(&mut runner);
    assert_eq!(
        after, before,
        "live gaps_in must return to its original value"
    );
}
