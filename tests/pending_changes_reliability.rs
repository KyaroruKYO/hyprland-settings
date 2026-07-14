//! Pending-changes reliability guards plus the env-gated skeleton
//! exporter the live reliability-matrix harness consumes.
//!
//! The skeleton classifies every scalar row and structured family into the
//! reliability buckets BEFORE any live interaction; the harness
//! (tools/live_scenario_harness/pending_reliability_matrix.py) then drives
//! each editable control in the running app and fills in the live results.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::Result;
use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::pending_changes_ui::values_semantically_equal;
use hyprland_settings::runtime_preview_ui_projection::runtime_preview_ui_row_state;
use hyprland_settings::ui::model::UiProjection;
use hyprland_settings::ux_presentation::{page_for_row, status_chip_for_row, StatusChip};
use hyprland_settings::validation::validate_bundle;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn load_projection() -> Result<UiProjection> {
    let resolution = resolve_metadata_path_with_env(None, None)?;
    let bundle = ExportBundle::load(Path::new(&resolution.export_dir))?;
    let summary = validate_bundle(&bundle)?;
    Ok(UiProjection::from_bundle(
        &bundle,
        &summary,
        ConfigDiscovery {
            status: ConfigDiscoveryStatus::Missing,
            attempted_paths: Vec::new(),
        },
        CurrentConfigSnapshot::read_unavailable("test fixture has no live config"),
    ))
}

fn window_source() -> String {
    fs::read_to_string("src/ui/window.rs").expect("window source reads")
}

fn fn_slice<'a>(source: &'a str, name: &str) -> &'a str {
    let start = source
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("fn {name} exists"));
    let next = source[start + 3..]
        .find("\nfn ")
        .map(|offset| start + 3 + offset)
        .unwrap_or(source.len());
    &source[start..next]
}

/// The seven structured families with their pending-ledger classification.
const FAMILY_CLASSIFICATION: [(&str, &str, &str); 7] = [
    (
        "hl.animation",
        "outside-pending-ledger",
        "supervised record preview with explicit Preview/Keep/Revert/Save controls and auto-revert on timeout; the applied-live-until-discarded pending model does not describe it",
    ),
    (
        "hl.curve",
        "outside-pending-ledger",
        "supervised record preview with explicit Preview/Keep/Revert/Save controls and auto-revert on timeout; the applied-live-until-discarded pending model does not describe it",
    ),
    (
        "hl.bind",
        "visible-but-not-editable",
        "read-only source-entry view; no edit/preview/save path exists",
    ),
    (
        "hl.monitor",
        "visible-but-not-editable",
        "read-only source-entry view; no edit/preview/save path exists",
    ),
    (
        "hl.device",
        "visible-but-not-editable",
        "read-only source-entry view; no edit/preview/save path exists",
    ),
    (
        "hl.gesture",
        "visible-but-not-editable",
        "no runtime readback listing exists on this compositor version; no edit path",
    ),
    (
        "hl.permission",
        "visible-but-not-editable",
        "read-only source-entry view; no edit/preview/save path exists",
    ),
];

/// Reliability bucket for a scalar row before any live interaction.
fn preclassify(row_id: &str) -> (&'static str, String, String) {
    match runtime_preview_ui_row_state(row_id) {
        Some(state) if state.preview_enabled => (
            "editable-and-pending-required",
            format!("{:?}", state.control_kind),
            String::new(),
        ),
        Some(state) => {
            let chip = status_chip_for_row(row_id);
            let reason = match chip {
                StatusChip::Blocked => "blocked: high-risk row, live preview disabled".to_string(),
                StatusChip::HardwareRequired => {
                    "hardware-gated: proof requires unavailable hardware".to_string()
                }
                StatusChip::NotProvenYet => {
                    "not proven: no passed live proof arms this row".to_string()
                }
                StatusChip::SaveOnly => {
                    "save-only: no live preview stage exists; Save writes immediately through the gate, so there is never an unsaved intermediate state"
                        .to_string()
                }
                StatusChip::LivePreview => format!(
                    "preview control disabled: {:?} (dead-man supervised rows preview through their own confirm flow, outside the pending ledger by design)",
                    state.control_kind
                ),
            };
            (
                "visible-but-not-editable",
                format!("{:?}", state.control_kind),
                reason,
            )
        }
        None => (
            "visible-but-not-editable",
            "None".to_string(),
            "row has no runtime-preview row state".to_string(),
        ),
    }
}

/// Env-gated exporter: writes the static classification skeleton the live
/// harness merges its results into. Never runs in a normal suite pass.
#[test]
fn export_reliability_skeleton() -> Result<()> {
    if std::env::var("HYPRLAND_SETTINGS_EXPORT_PENDING_SKELETON").is_err() {
        return Ok(());
    }
    let projection = load_projection()?;
    let mut rows = Vec::new();
    for row in SAFE_WRITABLE_ROWS.iter() {
        let setting = projection
            .settings
            .iter()
            .find(|setting| setting.row_id == row.row_id)
            .expect("every safe-writable row exists in the projection");
        let page = page_for_row(&setting.tab_id, &setting.official_setting);
        let (bucket, control_kind, reason) = preclassify(row.row_id);
        rows.push(serde_json::json!({
            "rowId": row.row_id,
            "official": setting.official_setting,
            "officialColon": setting.official_setting.replace('.', ":"),
            "pageId": page.map(|page| page.id),
            "pageLabel": page.map(|page| page.label),
            "controlKind": control_kind,
            "bucket": bucket,
            "reason": reason,
            "configHasLine": setting.current_value.raw_value.is_some(),
        }));
    }
    let families: Vec<serde_json::Value> = FAMILY_CLASSIFICATION
        .iter()
        .map(|(family, bucket, reason)| {
            serde_json::json!({ "family": family, "bucket": bucket, "reason": reason })
        })
        .collect();
    let skeleton = serde_json::json!({
        "totalScalarRows": rows.len(),
        "totalStructuredFamilies": families.len(),
        "rows": rows,
        "families": families,
    });
    fs::write(
        "/tmp/hyprland-settings-pending-skeleton.json",
        serde_json::to_vec_pretty(&skeleton)?,
    )?;
    println!("skeleton exported: {} rows", rows.len());
    Ok(())
}

#[test]
fn every_row_gets_exactly_one_reliability_bucket() -> Result<()> {
    let projection = load_projection()?;
    let mut editable = 0usize;
    let mut not_editable = 0usize;
    let mut seen = BTreeSet::new();
    for row in SAFE_WRITABLE_ROWS.iter() {
        assert!(seen.insert(row.row_id), "row ids unique");
        let (bucket, _, reason) = preclassify(row.row_id);
        match bucket {
            "editable-and-pending-required" => editable += 1,
            "visible-but-not-editable" => {
                assert!(
                    !reason.is_empty(),
                    "{}: ineligible rows carry a reason",
                    row.row_id
                );
                not_editable += 1;
            }
            other => panic!("unexpected bucket {other}"),
        }
        // Every row still belongs to a page.
        let setting = projection
            .settings
            .iter()
            .find(|setting| setting.row_id == row.row_id)
            .expect("row exists in projection");
        assert!(
            page_for_row(&setting.tab_id, &setting.official_setting).is_some(),
            "{} maps to a page",
            row.row_id
        );
    }
    assert_eq!(editable + not_editable, 341);
    // The editable set is exactly the live-previewable control set.
    assert_eq!(editable, 135, "editable bucket = default-previewable rows");
    Ok(())
}

#[test]
fn semantic_no_op_rules_cover_the_readback_spellings() {
    // Booleans across spellings.
    assert!(values_semantically_equal("true", "1"));
    assert!(values_semantically_equal("off", "false"));
    assert!(!values_semantically_equal("true", "false"));
    // Numbers and formatting.
    assert!(values_semantically_equal("0.5", "0.500000"));
    assert!(values_semantically_equal("1", "1.0"));
    assert!(!values_semantically_equal("1", "2"));
    // Css-gap shorthand.
    assert!(values_semantically_equal("5", "5 5 5 5"));
    assert!(values_semantically_equal("5 10", "5 10 5 10"));
    assert!(!values_semantically_equal("5", "5 6 5 5"));
    // Colors: rgba() vs readback bare AARRGGBB hex; missing angle = 0deg.
    assert!(values_semantically_equal("rgba(ffffff4a)", "4affffff"));
    // Int-typed color readbacks: decimal u32 with AARRGGBB bits
    // (0x55000000 = 1426063360).
    assert!(values_semantically_equal("1426063360", "rgba(00000055)"));
    assert!(!values_semantically_equal("1426063360", "rgba(00000056)"));
    assert!(values_semantically_equal("rgba(ffffff4a)", "4affffff 0deg"));
    assert!(values_semantically_equal(
        "rgba(ffffff4a) rgba(ffffff1f) 45deg",
        "4affffff 1fffffff 45deg"
    ));
    assert!(!values_semantically_equal(
        "rgba(ffffff4a) 45deg",
        "4affffff 30deg"
    ));
    // Fail closed on unknown text.
    assert!(!values_semantically_equal("abc", "abd"));
}

#[test]
fn controls_seed_from_live_runtime_first() {
    let window = window_source();
    let seed = fn_slice(&window, "runtime_seed_initial_value");
    assert!(seed.contains("read_runtime_option_live"));
    assert!(seed.contains("raw_value"));
    assert!(seed.contains("official_default_value"));
    assert!(seed.contains("collapse_uniform_gap"));
    // Both control surfaces use the runtime-first seed.
    assert!(fn_slice(&window, "attach_inline_row_control").contains("runtime_seed_initial_value"));
    assert!(
        fn_slice(&window, "append_runtime_preview_controls").contains("read_runtime_option_live")
    );
    // The flip-suggestion is the LAST fallback everywhere, never the seed
    // for a value the runtime or config can provide.
    assert_eq!(collapse_order_ok(seed), true);
}

fn collapse_order_ok(seed: &str) -> bool {
    let runtime = seed.find("read_runtime_option_live").unwrap_or(usize::MAX);
    let raw = seed.find("raw_value").unwrap_or(usize::MAX);
    let default = seed.find("official_default_value").unwrap_or(usize::MAX);
    let proposed = seed.find("proposed_value").unwrap_or(usize::MAX);
    runtime < raw && raw < default && default < proposed
}

#[test]
fn pending_ledger_owns_sessions_and_reuses_them() {
    let window = window_source();
    // Strong ownership: sessions survive page re-renders.
    assert!(window.contains("controller: Rc<RefCell<RuntimePreviewUiController>>"));
    // Creation paths reuse the ledger's controller for the row.
    assert!(window.contains("fn pending_controller_for_row"));
    let inline = fn_slice(&window, "inline_preview_apply");
    assert!(inline.contains("pending_controller_for_row"));
    let detail = fn_slice(&window, "append_runtime_preview_controls");
    assert!(detail.contains("pending_controller_for_row"));
    // The pending predicate is semantic, not string equality.
    let snapshots = fn_slice(&window, "pending_change_snapshots");
    assert!(snapshots.contains("values_semantically_equal"));
    assert!(!snapshots.contains("current == original"));
}

#[test]
fn color_previews_use_the_proven_runtime_color_grammar() {
    let executor = fs::read_to_string("src/runtime_preview_executor.rs").expect("executor reads");
    // Session originals parse the gradient readback prefix.
    assert!(executor.contains("\"gradient data:\""));
    // Applies and reverts render the proven lua table form.
    assert!(executor.contains("fn rendered_color_lua_table"));
    assert!(executor.contains("colors = {"));
    assert!(executor.contains("ScalarWriteValueKind::Color | ScalarWriteValueKind::Gradient =>"));
    // Number-classified rows with css-gap-shaped runtime readbacks revert
    // through the gap table instead of failing the numeric parse.
    assert!(executor.contains("fn rendered_css_gap_table"));
    // Token normalization covers the readback's AARRGGBB order and the
    // decimal u32 form int-typed color options report.
    assert!(executor.contains("fn color_token_to_rgba"));
    assert!(executor.contains("token.parse::<u32>()") || executor.contains("parse::<u32>()"));
    // Shape picks the grammar: single stop without angle is a plain color
    // string (int-typed color options reject the gradient table).
    assert!(executor.contains("colors.len() == 1 && angle.is_none()"));
    // Bare-hex spellings parse in the presentation layer too (swatches and
    // semantic comparison for runtime-seeded values).
    use hyprland_settings::ux_presentation::parse_hyprland_color;
    let bare = parse_hyprland_color("4affffff").expect("bare hex8 parses");
    assert_eq!(
        (bare.alpha, bare.red, bare.green, bare.blue),
        (0x4a, 0xff, 0xff, 0xff)
    );
    let rgba = parse_hyprland_color("rgba(ffffff4a)").expect("rgba parses");
    assert_eq!(bare, rgba, "AARRGGBB and RRGGBBAA spellings agree");
    use hyprland_settings::ux_presentation::parse_hyprland_gradient;
    let single = parse_hyprland_gradient("18ffffff 0deg").expect("single stop + angle parses");
    assert_eq!(single.0.len(), 1);
    assert_eq!(single.1, Some(0));
}

#[test]
fn pending_rows_expose_an_accessible_marker() {
    let window = window_source();
    let notify = fn_slice(&window, "notify_pending_changed");
    assert!(notify.contains("gtk::accessible::Property::Description"));
    assert!(notify.contains("Pending change"));
    let row_builder = fn_slice(&window, "build_setting_row");
    assert!(row_builder.contains("Official key:") || window.contains("Official key:"));
}

#[test]
fn matrix_report_shows_zero_bugs() {
    // The committed matrix is the live harness output; its totals must
    // stay coherent and its bug bucket empty.
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read("data/reports/pending-changes-reliability-matrix.v0.55.2.json")
            .expect("reliability matrix report exists"),
    )
    .expect("report parses");
    assert_eq!(report["totals"]["scalarRows"], 341);
    assert_eq!(report["totals"]["structuredFamilies"], 7);
    let editable = report["totals"]["editableAndPendingRequired"]
        .as_u64()
        .expect("editable count");
    let not_editable = report["totals"]["visibleButNotEditable"]
        .as_u64()
        .expect("not-editable count");
    let no_op = report["totals"]["editableButNoOpInThisSession"]
        .as_u64()
        .expect("no-op count");
    let bugs = report["totals"]["bug"].as_u64().expect("bug count");
    assert_eq!(editable + not_editable + no_op + bugs, 341);
    assert_eq!(bugs, 0, "the committed matrix must contain zero bugs");
    assert!(report["rows"].as_array().expect("rows").len() == 341);
}
