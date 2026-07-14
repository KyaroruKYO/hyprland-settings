//! UX simplification guards: the everyday pages stay settings-first, the
//! developer/proof surfaces stay on Safety Details, the quiet status-chip
//! vocabulary stays honest, and no safety gate weakened. Normal tests only.

use std::fs;

use hyprland_settings::runtime_preview::runtime_preview_capability_matrix;
use hyprland_settings::ux_presentation::{
    category_for_tab, short_description, status_chip_for_row, StatusChip, SAVE_GATE_CHIP,
    SIDEBAR_PAGE_LAYOUT,
};

fn source_slice<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source.find(start).expect("start marker present");
    let end_index = source[start_index..].find(end).expect("end marker present") + start_index;
    &source[start_index..end_index]
}

/// The seven developer/proof sections that moved off the Config page.
const DEVELOPER_SECTIONS: [&str; 7] = [
    "connected_files_review_section(",
    "profile_mode_detail_section()",
    "structured_family_editor_section(",
    "disabled_future_approval_cards_section()",
    "controlled_write_and_active_pilot_status_section()",
    "runtime_preview_readiness_section()",
    "structured_family_runtime_preview_status_section()",
];

#[test]
fn config_page_stays_settings_first() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    let config_view = source_slice(
        &window,
        "fn build_config_view",
        "fn build_safety_details_view",
    );
    for section in DEVELOPER_SECTIONS {
        assert!(
            !config_view.contains(section),
            "the Config page must not render the developer surface {section}"
        );
    }
    // The user-facing cards stay; the record picker card now lives on the
    // Animations page instead of here.
    for kept in [
        "config_file_selection_section(",
        "safe_live_save_mode_section(",
    ] {
        assert!(config_view.contains(kept), "Settings page keeps {kept}");
    }
    assert!(
        !config_view.contains("structured_family_preview_controls_section("),
        "the record picker card belongs to Animations, not Settings"
    );
    assert!(
        window.contains("sections_box.append(&structured_family_preview_controls_section(model))")
    );
}

#[test]
fn safety_details_page_hosts_every_moved_surface() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    let safety_view = source_slice(
        &window,
        "fn build_safety_details_view",
        "fn config_path_summary",
    );
    for section in DEVELOPER_SECTIONS {
        assert!(
            safety_view.contains(section),
            "Safety Details must render {section} (moved, not removed)"
        );
    }
    assert!(safety_view.contains("Nothing on this page changes behavior"));
    // The page is reachable: page layout entry, dashboard card, routing.
    assert!(window.contains("const SAFETY_ID: &str = \"safety-details\""));
    assert!(window.contains("title: \"Safety Details\""));
    assert!(window.contains("(SAFETY_ID, safety_view.clone())"));
    let layout = fs::read_to_string("src/ux_presentation.rs").expect("presentation reads");
    assert!(layout.contains("label: \"Safety Details\""));
}

#[test]
fn sidebar_is_grouped_by_task_categories() {
    // The page layout covers the HyprMod-style category set, every page id
    // resolves a category, and the window renders headers from it.
    let labels: Vec<&str> = SIDEBAR_PAGE_LAYOUT
        .iter()
        .map(|category| category.label)
        .collect();
    assert_eq!(
        labels,
        [
            "Look & Feel",
            "Input",
            "Display",
            "Window Management",
            "Startup",
            "Advanced"
        ]
    );
    for page_id in [
        "general",
        "decoration",
        "animations",
        "cursor",
        "keybinds",
        "devices",
        "gestures",
        "monitors",
        "workspaces",
        "layouts",
        "window-rules",
        "layer-rules",
        "autostart",
        "env-variables",
        "xwayland",
        "ecosystem",
        "system",
        "permissions",
        "profiles",
        "config",
        "safety-details",
    ] {
        assert!(
            category_for_tab(page_id).is_some(),
            "{page_id} must belong to a sidebar category"
        );
    }
    assert_eq!(category_for_tab("dashboard"), None);
    assert_eq!(category_for_tab("config"), Some("Advanced"));
    assert_eq!(category_for_tab("safety-details"), Some("Advanced"));
    assert_eq!(category_for_tab("autostart"), Some("Startup"));

    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(window.contains("SIDEBAR_PAGE_LAYOUT"));
    assert!(window.contains("set_header_func"));
    assert!(window.contains("hyprland-settings-nav-category-"));
    // Category headers render uppercase in a caption style, and nav rows
    // carry the legibility class backed by the app CSS provider.
    assert!(window.contains("label.to_uppercase()"));
    assert!(window.contains("caption-heading"));
    assert!(window.contains("hyprland-settings-nav-row-label"));
    assert!(window.contains("font-size: 1.08em"));
}

#[test]
fn status_chips_stay_quiet_and_honest() {
    // The vocabulary is fixed and short.
    assert_eq!(StatusChip::LivePreview.label(), "Live Preview");
    assert_eq!(StatusChip::SaveOnly.label(), "Save Only");
    assert_eq!(StatusChip::HardwareRequired.label(), "Hardware Required");
    assert_eq!(StatusChip::NotProvenYet.label(), "Not Proven Yet");
    assert_eq!(StatusChip::Blocked.label(), "Blocked");
    assert_eq!(SAVE_GATE_CHIP, "Requires Safe Live Save Mode");

    // Every one of the 341 rows maps to a chip, and the distribution
    // matches the honest classification (no chip inflates capability).
    let matrix = runtime_preview_capability_matrix();
    assert_eq!(matrix.len(), 341);
    let mut live = 0;
    let mut hardware = 0;
    let mut blocked = 0;
    let mut save_only = 0;
    let mut not_proven = 0;
    for row in &matrix {
        match status_chip_for_row(&row.row_id) {
            StatusChip::LivePreview => live += 1,
            StatusChip::HardwareRequired => hardware += 1,
            StatusChip::Blocked => blocked += 1,
            StatusChip::SaveOnly => save_only += 1,
            StatusChip::NotProvenYet => not_proven += 1,
        }
    }
    // 135 default-previewable + the armed dead-man candidates.
    assert_eq!(live, 135 + 38, "live chips = live-previewable + armed rows");
    assert!(hardware >= 18, "touch-family rows read Hardware Required");
    assert!(blocked >= 74, "high-risk rows stay Blocked");
    assert!(save_only >= 43, "config-write rows read Save Only");
    assert!(not_proven >= 1);
    assert_eq!(live + hardware + blocked + save_only + not_proven, 341);

    // Known anchors (row ids are UI-tab-based).
    assert_eq!(
        status_chip_for_row("appearance.gaps_in"),
        StatusChip::LivePreview
    );
    assert_eq!(
        status_chip_for_row("does.not.exist"),
        StatusChip::NotProvenYet
    );

    // The row list uses the chip and the shortened description.
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(window.contains("status_chip_for_row(&setting.row_id).label()"));
    assert!(window.contains("ux_presentation::short_description("));
}

#[test]
fn short_descriptions_are_one_line() {
    assert_eq!(
        short_description("Gaps between windows. Also see workspace rules."),
        "Gaps between windows."
    );
    let long = "word ".repeat(60);
    let shortened = short_description(&long);
    assert!(shortened.chars().count() <= 110);
    assert!(shortened.ends_with('…'));
    assert_eq!(
        short_description("  plain text without period  "),
        "plain text without period"
    );
}

#[test]
fn presentation_layer_is_presentational_only_and_gates_unchanged() {
    let module = fs::read_to_string("src/ux_presentation.rs").expect("module reads");
    // Presentational only: no file access, no process spawning, no runtime
    // commands, no save paths.
    for forbidden in [
        "fs::",
        "Command::new",
        "std::process",
        "hyprctl",
        "gated_scalar_save",
        "gated_family",
        "apply_setting_change",
        "atomic_controlled_write",
    ] {
        assert!(
            !module.contains(forbidden),
            "presentation module must not contain {forbidden}"
        );
    }

    // The UX pass did not weaken the gates: the gated save routes and the
    // no-direct-apply rule still hold in the window source.
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(window.contains("gated_scalar_save_live("));
    assert!(window.contains("persist_safe_live_save_mode_live("));
    assert!(window.contains("save_picked_record_live("));
    assert!(!window.contains("apply_setting_change("));
    assert!(!window.contains("\"reload\""));
}
