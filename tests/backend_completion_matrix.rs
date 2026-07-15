//! Backend-completion scalar matrix generator + regression guard (Scopes A/K).
//!
//! Classifies every scalar row into its final backend bucket straight from
//! the live classifier (never a hand-written table), asserts the buckets
//! partition all 341 rows with no gaps, and — when
//! HYPRLAND_SETTINGS_WRITE_BACKEND_MATRIX=1 — writes the scalar matrix and
//! the final summary reports.

use std::collections::BTreeMap;

use hyprland_settings::runtime_preview_dead_man::dead_man_ui_state;
use hyprland_settings::runtime_preview_ui_projection::runtime_preview_ui_row_state;
use hyprland_settings::save_only_pending::{is_save_only_editable, save_only_control_kind};
use hyprland_settings::ux_presentation::{
    page_for_official_setting, status_chip_for_row, StatusChip,
};
use hyprland_settings::write_classification::{
    is_high_risk_gated_writable_setting, SAFE_WRITABLE_ROWS,
};

/// Rows whose runtime readback is upstream-broken on this compositor
/// (getoption returns no parsable value): they show a live control whose
/// preview no-ops, but Save writes config correctly, so they are editable.
/// Flagged in per-row output for honesty; not a separate bucket.
const UPSTREAM_READBACK_BROKEN: &[&str] = &[
    "group.groupbar.font_weight_active",
    "group.groupbar.font_weight_inactive",
];

fn final_bucket(row_id: &str) -> &'static str {
    let Some(state) = runtime_preview_ui_row_state(row_id) else {
        return "visible-blocked-other";
    };
    if state.preview_enabled {
        return "editable-live-preview-pending";
    }
    if is_save_only_editable(row_id) {
        return "editable-save-only-pending";
    }
    // A dead-man candidate that this machine can actually arm flows through
    // the modal rollback dialog and joins the pending ledger on Keep.
    if let Some(dm) = dead_man_ui_state(row_id) {
        if dm.arm_enabled {
            return "editable-dead-man-pending";
        }
    }
    // The one remaining blocked set: high-risk settings protected by the
    // production gate, which requires a recovery-plan proof to save. The
    // plain save-only path cannot supply that proof, so these stay blocked
    // (deliberately) rather than being faked as editable-but-unsavable.
    if is_high_risk_gated_writable_setting(row_id) {
        return "blocked-production-gated-high-risk";
    }
    let _ = status_chip_for_row(row_id);
    "visible-blocked-other"
}

fn counts() -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in SAFE_WRITABLE_ROWS.iter() {
        *counts
            .entry(final_bucket(row.row_id).to_string())
            .or_default() += 1;
    }
    counts
}

#[test]
fn buckets_partition_all_341_rows() {
    let counts = counts();
    let total: usize = counts.values().sum();
    assert_eq!(
        total, 341,
        "every scalar row must fall in exactly one bucket"
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    // Broadened save-only: every non-gated, non-live, non-armable row with a
    // control + write path is now editable.
    assert_eq!(counts.get("editable-save-only-pending"), Some(&117));
    assert_eq!(counts.get("editable-live-preview-pending"), Some(&135));
    assert_eq!(counts.get("editable-dead-man-pending"), Some(&38));
    // The only blocked rows are the production-gated high-risk set.
    assert_eq!(counts.get("blocked-production-gated-high-risk"), Some(&51));
    // Nothing is blocked for any other reason.
    assert_eq!(counts.get("visible-blocked-other"), None);
    let editable = counts.get("editable-live-preview-pending").unwrap()
        + counts.get("editable-save-only-pending").unwrap()
        + counts.get("editable-dead-man-pending").unwrap();
    assert_eq!(editable, 290, "290 of 341 scalar rows are editable");
}

#[test]
fn write_backend_matrix_reports() {
    if std::env::var("HYPRLAND_SETTINGS_WRITE_BACKEND_MATRIX").is_err() {
        return;
    }
    let mut rows = Vec::new();
    for row in SAFE_WRITABLE_ROWS.iter() {
        let bucket = final_bucket(row.row_id);
        let state = runtime_preview_ui_row_state(row.row_id);
        let page = page_for_official_setting(row.official_setting);
        rows.push(serde_json::json!({
            "rowId": row.row_id,
            "official": row.official_setting,
            "officialColon": row.official_setting.replace('.', ":"),
            "pageId": page.map(|p| p.id),
            "valueKind": format!("{:?}", row.value_kind),
            "statusChip": status_chip_for_row(row.row_id).label(),
            "previewEnabled": state.as_ref().map(|s| s.preview_enabled).unwrap_or(false),
            "saveOnlyEditable": is_save_only_editable(row.row_id),
            "saveOnlyControl": save_only_control_kind(row.row_id).map(|k| k.as_str()),
            "deadManArmable": dead_man_ui_state(row.row_id).map(|d| d.arm_enabled).unwrap_or(false),
            "saveSupported": state.as_ref().map(|s| s.save_state.available()).unwrap_or(false),
            "productionGated": is_high_risk_gated_writable_setting(row.row_id),
            "upstreamReadbackBroken": UPSTREAM_READBACK_BROKEN.contains(&row.row_id),
            "finalBucket": bucket,
        }));
    }
    let bucket_counts = counts();
    let scalar = serde_json::json!({
        "report": "backend-completion-scalar-matrix",
        "modelVersion": "v0.55.2",
        "generatedAt": "2026-07-14",
        "method": "classified live from status_chip_for_row + is_save_only_editable + dead_man arm_enabled; no hand-written table",
        "totalScalarRows": rows.len(),
        "buckets": bucket_counts,
        "rows": rows,
    });
    std::fs::write(
        "data/reports/backend-completion-scalar-matrix.v0.55.2.json",
        serde_json::to_vec_pretty(&scalar).unwrap(),
    )
    .unwrap();

    let counts = &bucket_counts;
    let editable = counts
        .get("editable-live-preview-pending")
        .copied()
        .unwrap_or(0)
        + counts
            .get("editable-save-only-pending")
            .copied()
            .unwrap_or(0)
        + counts
            .get("editable-dead-man-pending")
            .copied()
            .unwrap_or(0);
    let scalar_summary = serde_json::json!({
        "visible": 341,
        "editableTotal": editable,
        "editableLivePreviewPending": counts.get("editable-live-preview-pending"),
        "editableSaveOnlyPending": counts.get("editable-save-only-pending"),
        "editableDeadManPending": counts.get("editable-dead-man-pending"),
        "blockedProductionGatedHighRisk": counts.get("blocked-production-gated-high-risk"),
        "blockedOther": counts.get("visible-blocked-other").copied().unwrap_or(0),
        "hardwareGatedDeferred": 0,
        "notProven": 0,
        "upstreamReadbackBrokenButSavable": UPSTREAM_READBACK_BROKEN.len(),
    });
    let final_matrix = serde_json::json!({
        "report": "backend-completion-final-matrix",
        "modelVersion": "v0.55.2",
        "generatedAt": "2026-07-14",
        "scalar": scalar_summary,
        "productionGatedRationale": "51 high-risk settings (accepted display/render, cursor/input, debug/crash lists) require a HighRiskProductionGate recovery-plan proof to save. The plain gated scalar save cannot supply that proof, so exposing a save-only control would be editable-but-unsavable. They stay blocked deliberately; making them editable needs the production-gate recovery flow, NOT a gate bypass.",
        "structuredFamilies": {
            "total": 7,
            "editableSupervised": ["hl.animation", "hl.curve"],
            "notFullyEditable": ["hl.monitor", "hl.bind", "hl.gesture", "hl.device", "hl.permission"],
            "participatingInPendingLedger": ["scalar save-only", "scalar live-preview", "scalar dead-man-kept"],
        },
        "pendingLedgerSources": ["LivePreview", "SaveOnlyDraft", "DeadManKept"],
        "styleEditing": "blocked — no trusted style-grammar evidence in the installed Hyprland package (stub enumerates no styles; styles are leaf-specific and parameterized)",
    });
    for path in [
        "data/reports/backend-completion-final-matrix.v0.55.2.json",
        "data/reports/full-editability-final.v0.55.2.json",
    ] {
        std::fs::write(path, serde_json::to_vec_pretty(&final_matrix).unwrap()).unwrap();
    }
    std::fs::write(
        "data/reports/full-editability-scalar-matrix.v0.55.2.json",
        serde_json::to_vec_pretty(&scalar).unwrap(),
    )
    .unwrap();
}
