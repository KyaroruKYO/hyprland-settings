use hyprland_settings::runtime_preview_executor::{
    parse_getoption_value, runtime_option_query, HyprctlRuntimePreviewRunner, RuntimePreviewRunner,
};
use hyprland_settings::runtime_preview_input_proof::live_proof_gate;

/// Per-row env-gated live proof harness for input/cursor rows.
///
/// Refuses to run without both env vars, refuses rows without a proof plan,
/// refuses rows without a usable fallback path, and refuses rows not
/// classified ProofReadyForEnvGatedLiveTest. When it runs, it captures the
/// original runtime value, applies the plan's minimal preview value through
/// the supervised runtime-set path, verifies the apply via a read-only
/// getoption, reverts automatically, and verifies the revert. It never writes
/// config, never reloads, and never persists anything.
#[test]
#[ignore = "mutates live input/cursor behavior; run only with HYPRLAND_SETTINGS_RUN_INPUT_LIVE_PROOF=1 and HYPRLAND_SETTINGS_INPUT_PROOF_ROW=<official_setting>"]
fn per_row_input_live_proof_applies_and_reverts() {
    if std::env::var("HYPRLAND_SETTINGS_RUN_INPUT_LIVE_PROOF").as_deref() != Ok("1") {
        eprintln!("skipping: HYPRLAND_SETTINGS_RUN_INPUT_LIVE_PROOF is not set");
        return;
    }
    let Ok(official_setting) = std::env::var("HYPRLAND_SETTINGS_INPUT_PROOF_ROW") else {
        panic!("HYPRLAND_SETTINGS_INPUT_PROOF_ROW must name the row to prove");
    };
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
        return;
    }

    // The gate fails closed: no plan, no fallback, or not proof-ready all
    // refuse before any command exists.
    let plan = match live_proof_gate(&official_setting) {
        Ok(plan) => plan,
        Err(reason) => panic!("live proof refused: {reason}"),
    };
    assert!(
        plan.fallback.timeout_auto_revert_needs_no_input,
        "proof rows must revert without user input"
    );
    eprintln!(
        "proving {} ({}): fallback = {}",
        plan.official_setting,
        plan.category.as_str(),
        plan.fallback.summary
    );

    let mut runner = HyprctlRuntimePreviewRunner;
    let query = runtime_option_query(plan.official_setting);
    let read_value = |runner: &mut HyprctlRuntimePreviewRunner| -> String {
        let output = runner
            .run("hyprctl", &query)
            .expect("read-only getoption should succeed");
        parse_getoption_value(&output).expect("option value should parse")
    };

    // 1. Capture original (read-only).
    let original = read_value(&mut runner);
    let preview_value = plan.minimal_preview_value.clone();
    assert!(
        preview_value.parse::<f64>().is_ok() || preview_value == "true" || preview_value == "false",
        "proof-ready rows must have a concrete minimal preview value, got {preview_value:?}"
    );

    // 2. Apply the minimal preview value through the supervised runtime-set
    //    path (same command shape the dead-man executor uses).
    let mut segments: Vec<&str> = plan.official_setting.split('.').collect();
    let option = segments.pop().expect("option name");
    let mut expression = format!("{option} = {preview_value}");
    for segment in segments.iter().rev() {
        expression = format!("{segment} = {{ {expression} }}");
    }
    let apply_args = vec!["eval".to_string(), format!("hl.config({{ {expression} }})")];
    runner
        .run("hyprctl", &apply_args)
        .expect("supervised apply should succeed");

    // 3. Verify apply.
    let during = read_value(&mut runner);
    let applied_matches = during == preview_value
        || during
            .parse::<f64>()
            .ok()
            .zip(preview_value.parse::<f64>().ok())
            .map(|(lhs, rhs)| (lhs - rhs).abs() < 1e-6)
            .unwrap_or(false);

    // 4. Revert automatically regardless of the verification outcome.
    let mut segments: Vec<&str> = plan.official_setting.split('.').collect();
    let option = segments.pop().expect("option name");
    let original_rendered = original
        .parse::<f64>()
        .map(|value| value.to_string())
        .unwrap_or_else(|_| original.clone());
    let mut expression = format!("{option} = {original_rendered}");
    for segment in segments.iter().rev() {
        expression = format!("{segment} = {{ {expression} }}");
    }
    let revert_args = vec!["eval".to_string(), format!("hl.config({{ {expression} }})")];
    runner
        .run("hyprctl", &revert_args)
        .expect("automatic revert should succeed");

    // 5. Verify revert.
    let after = read_value(&mut runner);
    assert_eq!(
        after, original,
        "revert must restore the exact original runtime value"
    );
    assert!(
        applied_matches,
        "apply verification failed: expected {preview_value}, observed {during}"
    );

    eprintln!(
        "PROOF PASSED for {}: original {original:?} -> preview {preview_value:?} (observed {during:?}) -> reverted {after:?}",
        plan.official_setting
    );
}

#[test]
fn live_proof_gate_refuses_unsupported_rows() {
    // Unknown row.
    assert!(live_proof_gate("no.such.row").is_err());
    // A row outside the input/cursor proof set.
    assert!(live_proof_gate("general.gaps_in").is_err());
    // A needs-live-proof row that is not proof-ready yet.
    let err = live_proof_gate("input.sensitivity").expect_err("not proof-ready");
    assert!(err.contains("not proof-ready"));
    // Blocked rows refuse.
    let err = live_proof_gate("cursor.invisible").expect_err("too dangerous");
    assert!(err.contains("not proof-ready"));
}
