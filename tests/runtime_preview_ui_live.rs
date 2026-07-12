use hyprland_settings::runtime_preview_executor::{
    parse_getoption_value, runtime_option_query, HyprctlRuntimePreviewRunner, RuntimePreviewRunner,
};
use hyprland_settings::runtime_preview_ui_projection::{
    RuntimePreviewUiController, RuntimePreviewUiSessionState,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

/// Live UI-model smoke: drives the same controller the GTK layer uses
/// against the running compositor for one known low-risk row, and restores
/// the original value before exiting. Ignored by default and env-gated so
/// normal `cargo test` never mutates the live session.
#[test]
#[ignore = "mutates the live compositor; run only with HYPRLAND_SETTINGS_RUN_RUNTIME_PREVIEW_UI_LIVE=1"]
fn ui_controller_live_smoke_applies_and_reverts_gaps_in() {
    if std::env::var("HYPRLAND_SETTINGS_RUN_RUNTIME_PREVIEW_UI_LIVE").as_deref() != Ok("1") {
        eprintln!("skipping: HYPRLAND_SETTINGS_RUN_RUNTIME_PREVIEW_UI_LIVE is not set");
        return;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
        return;
    }

    let row_id = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "general.gaps_in")
        .expect("gaps_in row should exist")
        .row_id;

    let read_gaps = || -> String {
        let mut runner = HyprctlRuntimePreviewRunner;
        let output = runner
            .run("hyprctl", &runtime_option_query("general.gaps_in"))
            .expect("getoption should succeed");
        parse_getoption_value(&output).expect("gaps_in should parse")
    };

    let before = read_gaps();
    let mut controller =
        RuntimePreviewUiController::new_live(row_id).expect("live controller builds");

    // Simulate a short slider drag: several offers, throttled.
    let mut now = 0;
    for value in ["7", "8", "9"] {
        let _ = controller.offer_value(value, now).expect("offer succeeds");
        now += 40;
    }
    let _ = controller.drain_pending(now + 200).expect("drain succeeds");
    assert_eq!(
        controller.session_state(),
        RuntimePreviewUiSessionState::PreviewingLive
    );
    let during = read_gaps();
    assert_ne!(during, before, "preview must change the live value");

    // Cancel restores the original value.
    let receipt = controller.cancel().expect("cancel succeeds");
    assert_eq!(
        receipt.session_state,
        RuntimePreviewUiSessionState::Cancelled
    );
    let after = read_gaps();
    assert_eq!(after, before, "cancel must restore the original live value");
}
