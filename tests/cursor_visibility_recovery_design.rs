use std::collections::BTreeSet;
use std::fs;

use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

#[test]
fn cursor_visibility_reports_cover_exact_target_rows_without_enablement() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let analysis = read_json("data/reports/cursor-visibility-risk-class-analysis.v0.55.2.json")?;
    let proof = read_json("data/reports/cursor-visibility-recovery-proof-design.v0.55.2.json")?;
    let recommendation =
        read_json("data/reports/cursor-visibility-next-subset-recommendation.v0.55.2.json")?;

    assert_eq!(coverage["counts"]["writableRows"], 275);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 66);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 275);

    for report in [&analysis, &proof] {
        assert_eq!(report["counts"]["cursorVisibilityRowsAnalyzed"], 5);
        assert_eq!(report["counts"]["rowsRepresented"], 5);
        assert_eq!(report["counts"]["highRiskBlockedRows"], 5);
        assert_eq!(report["counts"]["safeWriteSupportedFalseRows"], 5);
        assert_eq!(report["counts"]["rowsEnabled"], 0);
        assert_eq!(report["counts"]["finalWritableRows"], 275);
        assert_eq!(report["counts"]["finalBlockedRows"], 66);
        assert_eq!(report["counts"]["cursorInputBlockedRows"], 21);
        assert_eq!(report["counts"]["displayRenderBlockedRows"], 23);
        assert_eq!(report["counts"]["debugCrashBlockedRows"], 22);
        assert_eq!(report["counts"]["writeAllowlistChanged"], false);
        assert_eq!(report["counts"]["productionBehaviorChanged"], false);
        assert_eq!(report["counts"]["recoveryGateWeakenedRows"], 0);
        assert_eq!(report["counts"]["hyprmodRecoveryEvidenceRows"], 0);
    }

    assert_eq!(recommendation["counts"]["rowsEvaluated"], 5);
    assert_eq!(recommendation["counts"]["selectedRows"], 2);
    assert_eq!(recommendation["counts"]["rejectedRows"], 3);
    assert_eq!(recommendation["counts"]["rowsEnabled"], 0);
    assert_eq!(recommendation["counts"]["finalWritableRows"], 275);
    assert_eq!(recommendation["counts"]["finalBlockedRows"], 66);
    assert_eq!(recommendation["counts"]["cursorInputBlockedRows"], 21);
    assert_eq!(recommendation["counts"]["displayRenderBlockedRows"], 23);
    assert_eq!(recommendation["counts"]["debugCrashBlockedRows"], 22);
    assert_eq!(recommendation["counts"]["writeAllowlistChanged"], false);
    assert_eq!(recommendation["counts"]["productionBehaviorChanged"], false);
    assert_eq!(recommendation["counts"]["recoveryGateWeakenedRows"], 0);

    let expected = [
        "cursor.invisible",
        "cursor.inactive_timeout",
        "cursor.hide_on_key_press",
        "cursor.hide_on_touch",
        "cursor.hide_on_tablet",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let actual = analysis["rows"]
        .as_array()
        .expect("analysis rows")
        .iter()
        .map(|row| row["rowId"].as_str().expect("row id"))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn cursor_visibility_rows_keep_high_risk_recovery_requirements() -> Result<()> {
    let analysis = read_json("data/reports/cursor-visibility-risk-class-analysis.v0.55.2.json")?;
    let proof = read_json("data/reports/cursor-visibility-recovery-proof-design.v0.55.2.json")?;

    let required = [
        "validator proof",
        "invalid rejection proof",
        "fixture write/reread proof",
        "temp config proof",
        "watchdog arm-before-mutation proof",
        "backup-before-mutation proof",
        "separate process confirm proof",
        "wrong-token failure proof",
        "timeout restore proof",
        "dry-run real config refusal proof",
        "UI/review projection with cursor visibility warning",
        "visible-cursor-independent recovery proof",
        "mouse-independent recovery proof",
        "app-UI-independent recovery proof",
        "Hyprland-keybind-independent recovery proof",
        "pointer-focus-independent recovery proof",
        "workspace-focus-independent recovery proof",
        "normal-pointer-behavior-independent recovery proof",
        "no reload/eval/Lua",
        "no active runtime mutation during tests",
    ];

    for row in analysis["rows"].as_array().expect("analysis rows") {
        assert_eq!(row["writeStatus"].as_str(), Some("high-risk"));
        assert_eq!(row["safeWriteSupported"].as_bool(), Some(false));
        assert_eq!(row["strongerWarningRequired"].as_bool(), Some(true));
        assert_eq!(row["hyprmodRecoveryEvidenceFound"].as_bool(), Some(false));
        assert!(row["hyprmodMetadataAdopted"]["label"].as_str().is_some());
        assert!(row["hyprmodMetadataAdopted"]["description"]
            .as_str()
            .is_some());

        let proof_requirements = row["proofRequirements"]
            .as_array()
            .expect("row proof requirements")
            .iter()
            .map(|value| value.as_str().expect("requirement"))
            .collect::<BTreeSet<_>>();
        for requirement in required {
            assert!(
                proof_requirements.contains(requirement),
                "{} missing requirement {requirement}",
                row["rowId"].as_str().unwrap_or("<unknown>")
            );
        }
    }

    let report_requirements = proof["proofRequirements"]
        .as_array()
        .expect("report proof requirements")
        .iter()
        .map(|value| value.as_str().expect("requirement"))
        .collect::<BTreeSet<_>>();
    for requirement in required {
        assert!(report_requirements.contains(requirement));
    }

    let invalid_dependencies = proof["invalidRecoveryDependencies"]
        .as_array()
        .expect("invalid dependencies")
        .iter()
        .map(|value| value.as_str().expect("dependency"))
        .collect::<BTreeSet<_>>();
    for dependency in [
        "visible cursor",
        "mouse click",
        "app UI only",
        "Hyprland keybind only",
        "pointer focus",
        "workspace focus",
        "normal pointer behavior",
    ] {
        assert!(invalid_dependencies.contains(dependency));
    }

    Ok(())
}

#[test]
fn cursor_visibility_future_subset_is_proof_only_and_excludes_invisible() -> Result<()> {
    let recommendation =
        read_json("data/reports/cursor-visibility-next-subset-recommendation.v0.55.2.json")?;

    assert_eq!(
        recommendation["recommendation"].as_str(),
        Some("two-row-future-proof-subset-selected")
    );
    assert_eq!(
        recommendation["selectedSubsetName"].as_str(),
        Some("cursor-visibility-conditional-touch-tablet-proof-subset")
    );
    assert_eq!(
        recommendation["noEnablementNotice"].as_str(),
        Some("Selected rows are for future proof planning only. They remain high-risk blocked.")
    );

    let selected = recommendation["selectedRows"]
        .as_array()
        .expect("selected rows");
    assert_eq!(selected.len(), 2);
    let selected_ids = selected
        .iter()
        .map(|row| row["rowId"].as_str().expect("row id"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        selected_ids,
        ["cursor.hide_on_touch", "cursor.hide_on_tablet"]
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
    assert!(!selected_ids.contains("cursor.invisible"));

    let rejected_ids = recommendation["rejectedRows"]
        .as_array()
        .expect("rejected rows")
        .iter()
        .map(|row| row["rowId"].as_str().expect("row id"))
        .collect::<BTreeSet<_>>();
    assert!(rejected_ids.contains("cursor.invisible"));
    assert!(rejected_ids.contains("cursor.inactive_timeout"));
    assert!(rejected_ids.contains("cursor.hide_on_key_press"));

    assert_eq!(recommendation["safety"]["realConfigModified"], false);
    assert_eq!(recommendation["safety"]["activeRuntimeModified"], false);
    assert_eq!(recommendation["safety"]["reloadRun"], false);
    assert_eq!(recommendation["safety"]["evalRun"], false);
    assert_eq!(recommendation["safety"]["luaExecuted"], false);
    assert_eq!(recommendation["safety"]["anythingPushed"], false);
    assert_eq!(recommendation["safety"]["mainTouched"], false);

    Ok(())
}
