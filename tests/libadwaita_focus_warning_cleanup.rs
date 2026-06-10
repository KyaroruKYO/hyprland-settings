use std::fs;

use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn read_json(path: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("report should exist"))
        .expect("report should parse")
}

#[test]
fn source_uses_libadwaita_style_manager_not_gtk_dark_theme_setting() {
    let app_source = fs::read_to_string("src/ui/app.rs").expect("app source should read");
    let all_ui_source = format!(
        "{}\n{}",
        app_source,
        fs::read_to_string("src/ui/window.rs").expect("window source should read")
    );

    assert!(app_source.contains("adw::StyleManager::default()"));
    assert!(app_source.contains("set_color_scheme(adw::ColorScheme::Default)"));
    assert!(app_source.contains("unsupported GtkSettings dark-theme flags"));

    for forbidden in [
        "gtk-application-prefer-dark-theme",
        "prefer-dark-theme",
        "gtk_application_prefer_dark_theme",
    ] {
        assert!(
            !all_ui_source.contains(forbidden),
            "unsupported GTK dark-theme setting should not be used: {forbidden}"
        );
    }
}

#[test]
fn initial_sidebar_selection_occurs_after_window_content_is_rooted() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let set_content = source
        .find("window.set_content(Some(&root));")
        .expect("window content assignment should exist");
    let select_row = source
        .find("sidebar.select_row(Some(&row));")
        .expect("initial sidebar selection should exist");
    let present = source
        .find("window.present();")
        .expect("window present should exist");

    assert!(
        set_content < select_row,
        "initial sidebar selection should happen after content is attached"
    );
    assert!(
        select_row < present,
        "initial sidebar selection should happen before presenting the fully initialized window"
    );
}

#[test]
fn dashboard_sidebar_search_and_details_behaviors_remain_wired() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");

    assert!(source.contains("const DASHBOARD_ID: &str = \"dashboard\""));
    assert!(source.contains("gtk::Button::with_label(\"Open\")"));
    assert!(source.contains("connect_clicked"));
    assert!(source.contains("sidebar.select_row(Some(&row))"));
    assert!(source.contains("connect_row_selected"));
    assert!(source.contains("Search settings"));
    assert!(source.contains("render_detail("));
    assert!(source.contains("&config_selection_state"));
    assert!(source.contains("Source / advanced metadata"));
}

#[test]
fn cleanup_report_preserves_final_counts() {
    let report = read_json("data/reports/libadwaita-focus-warning-cleanup.v0.55.2.json");

    assert_eq!(report["countsBefore"]["readableRows"], 341);
    assert_eq!(report["countsBefore"]["writableRows"], 341);
    assert_eq!(report["countsBefore"]["blockedRows"], 0);
    assert_eq!(report["countsAfter"]["readableRows"], 341);
    assert_eq!(report["countsAfter"]["writableRows"], 341);
    assert_eq!(report["countsAfter"]["blockedRows"], 0);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
