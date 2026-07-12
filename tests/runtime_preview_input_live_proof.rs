use hyprland_settings::runtime_preview_executor::{
    build_runtime_preview_command, parse_getoption_value, runtime_option_query,
    HyprctlRuntimePreviewRunner, RuntimePreviewRunner,
};
use hyprland_settings::runtime_preview_input_proof::{
    input_proof_plans, live_proof_gate, proof_preview_value_for,
    RuntimePreviewInputProofClassification, RuntimePreviewInputProofPlan,
};

fn prove_row(
    runner: &mut HyprctlRuntimePreviewRunner,
    plan: &RuntimePreviewInputProofPlan,
) -> Result<(String, String), String> {
    let query = runtime_option_query(plan.official_setting);
    let read_value = |runner: &mut HyprctlRuntimePreviewRunner| -> Result<String, String> {
        let output = runner
            .run("hyprctl", &query)
            .map_err(|error| format!("read-only getoption failed: {error}"))?;
        parse_getoption_value(&output).ok_or_else(|| "option value did not parse".to_string())
    };

    // 1. Capture original (read-only).
    let original = read_value(runner)?;
    // 2. Derive a preview value guaranteed to differ from the original.
    let preview_value = proof_preview_value_for(plan.official_setting, &original)
        .ok_or_else(|| "no safe differing preview value could be derived".to_string())?;

    // 3. Apply through the executor's own supervised command builder — the
    //    same construction and grammar validation the dead-man path uses.
    let apply = build_runtime_preview_command(plan.row_id, &preview_value, true)
        .map_err(|error| format!("apply command rejected: {error:?}"))?;
    runner
        .run(apply.program, &apply.args)
        .map_err(|error| format!("apply failed: {error}"))?;

    // 4. Verify apply.
    let during = read_value(runner)?;
    let applied_matches = during == preview_value
        || during
            .parse::<f64>()
            .ok()
            .zip(preview_value.parse::<f64>().ok())
            .map(|(lhs, rhs)| (lhs - rhs).abs() < 1e-6)
            .unwrap_or(false)
        || (during.starts_with("true") && preview_value == "true")
        || (during.starts_with("false") && preview_value == "false");

    // 5. Revert automatically regardless of the verification outcome. The
    //    original comes from getoption, so normalize floats for the builder.
    let original_rendered = original
        .parse::<f64>()
        .map(|value| {
            if value.fract() == 0.0 {
                format!("{}", value as i64)
            } else {
                format!("{value}")
            }
        })
        .unwrap_or_else(|_| original.clone());
    let revert = build_runtime_preview_command(plan.row_id, &original_rendered, true)
        .map_err(|error| format!("revert command rejected: {error:?}"))?;
    runner
        .run(revert.program, &revert.args)
        .map_err(|error| format!("revert failed: {error}"))?;

    // 6. Verify revert.
    let after = read_value(runner)?;
    if after != original {
        return Err(format!(
            "revert verification failed: expected {original:?}, observed {after:?}"
        ));
    }
    if !applied_matches {
        return Err(format!(
            "apply verification failed: expected {preview_value:?}, observed {during:?} (reverted cleanly)"
        ));
    }
    Ok((original, preview_value))
}

/// Per-row env-gated live proof harness for input/cursor rows.
///
/// `HYPRLAND_SETTINGS_INPUT_PROOF_ROW` names one row, or `all` to iterate
/// every proof-ready row, reverting each before moving to the next. The gate
/// fails closed: rows without a plan, without a usable fallback, or not
/// classified ProofReadyForEnvGatedLiveTest refuse before any command exists.
/// Never writes config, never reloads, never persists.
#[test]
#[ignore = "mutates live input/cursor behavior; run only with HYPRLAND_SETTINGS_RUN_INPUT_LIVE_PROOF=1 and HYPRLAND_SETTINGS_INPUT_PROOF_ROW=<official_setting|all>"]
fn per_row_input_live_proof_applies_and_reverts() {
    if std::env::var("HYPRLAND_SETTINGS_RUN_INPUT_LIVE_PROOF").as_deref() != Ok("1") {
        eprintln!("skipping: HYPRLAND_SETTINGS_RUN_INPUT_LIVE_PROOF is not set");
        return;
    }
    let Ok(row_selector) = std::env::var("HYPRLAND_SETTINGS_INPUT_PROOF_ROW") else {
        panic!("HYPRLAND_SETTINGS_INPUT_PROOF_ROW must name the row to prove (or 'all')");
    };
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
        return;
    }

    let targets: Vec<RuntimePreviewInputProofPlan> = if row_selector == "all" {
        input_proof_plans()
            .into_iter()
            .filter(|plan| {
                plan.proof_classification
                    == RuntimePreviewInputProofClassification::ProofReadyForEnvGatedLiveTest
            })
            .collect()
    } else {
        vec![match live_proof_gate(&row_selector) {
            Ok(plan) => plan,
            Err(reason) => panic!("live proof refused: {reason}"),
        }]
    };
    assert!(!targets.is_empty(), "no proof-ready rows to prove");

    let mut runner = HyprctlRuntimePreviewRunner;
    let mut passed = 0usize;
    let mut failed: Vec<(String, String)> = Vec::new();
    for plan in &targets {
        assert!(
            plan.fallback.keyboard_remains_usable || plan.fallback.pointer_remains_usable,
            "{} lost its fallback",
            plan.official_setting
        );
        assert!(plan.fallback.timeout_auto_revert_needs_no_input);
        match prove_row(&mut runner, plan) {
            Ok((original, preview)) => {
                passed += 1;
                println!(
                    "PROOF PASSED {} | original {original:?} | preview {preview:?} | reverted-and-verified | fallback: {}",
                    plan.official_setting, plan.fallback.summary
                );
            }
            Err(reason) => {
                failed.push((plan.official_setting.to_string(), reason.clone()));
                println!("PROOF FAILED {} | {reason}", plan.official_setting);
            }
        }
    }
    println!(
        "PROOF SUMMARY: {passed} passed, {} failed of {} attempted",
        failed.len(),
        targets.len()
    );
    assert!(
        failed.is_empty(),
        "some proofs failed (originals restored): {failed:?}"
    );
}

#[test]
fn live_proof_gate_refuses_unsupported_rows() {
    // Unknown row.
    assert!(live_proof_gate("no.such.row").is_err());
    // A row outside the input/cursor proof set.
    assert!(live_proof_gate("general.gaps_in").is_err());
    // Held-back rows are not proof-ready.
    let err = live_proof_gate("input.left_handed").expect_err("held back");
    assert!(err.contains("not proof-ready"));
    let err = live_proof_gate("input.rotation").expect_err("held back");
    assert!(err.contains("not proof-ready"));
    // Blocked rows refuse.
    let err = live_proof_gate("cursor.invisible").expect_err("too dangerous");
    assert!(err.contains("not proof-ready"));
    // Touch-family rows without hardware refuse.
    let err = live_proof_gate("input.touchpad.tap-to-click").expect_err("needs hardware");
    assert!(err.contains("not proof-ready"));
}
