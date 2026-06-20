use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk4 as gtk;

use crate::config_discovery::ConfigDiscovery;
use crate::config_graph::{
    inspect_config_graph, inspect_config_graph_with_options, ConfigGraphFile, ConfigGraphOptions,
    ConfigGraphSummary, ConfigManagementHintKind, ConfigSourceReference, SourceFollowPolicy,
};
use crate::config_layered_values::layered_values_for_setting;
use crate::config_selection::{ConfigSelectionState, SourceFollowChoice};
use crate::current_config::{
    CurrentConfigLoadStatus, CurrentConfigSnapshot, CurrentValueSourceStatus,
};
use crate::export::ExportBundle;
use crate::future_capability::{
    duplicate_production_approval_gate, source_include_insertion_review,
    source_include_selected_target_dry_run_plan, source_include_target_selection_fixture_proof,
    DuplicateOccurrence, DuplicateProductionGateStatus, SourceIncludeInsertionReadiness,
    SourceIncludeSelectedTargetDryRunStatus, SourceIncludeTargetCandidate,
};
use crate::guarded_write_review::{
    build_guarded_write_target_review, FixtureProofStatus, PRODUCTION_WRITE_TARGET_REVIEW_ENABLED,
};
use crate::missing_default_insertion::{
    build_missing_default_insertion_plan, MissingDefaultInsertionRequest,
};
use crate::one_target_pilot_live_visual_smoke::disabled_live_visual_smoke_review_ui_lines;
use crate::one_target_pilot_manual_review::disabled_manual_smoke_review_ui_lines;
use crate::one_target_pilot_pre_enable_audit::disabled_pre_enable_audit_ui_lines;
use crate::one_target_pilot_readiness::current_one_target_pilot_readiness_mapping;
use crate::production_advanced_confirmation::disabled_advanced_confirmation_ui_lines;
use crate::production_high_risk_approval::disabled_high_risk_approval_ui_lines;
use crate::safe_batch_write::safe_batch_write_user_facing_lines;
use crate::search::{search_projection, SearchRank, SearchResult};
use crate::session_config_preview::build_session_config_preview;
use crate::session_value_projection::{
    compare_active_and_session_values, SessionValueComparisonStatus, SessionValueProjection,
};
use crate::ui::model::{
    initial_screen_shader_advisory_ui_action, run_screen_shader_advisory_ui_action,
    RowDetailProjection, ScreenShaderAdvisoryUiActionRequest, UiProjection,
};
use crate::validation::ValidationSummary;
use crate::write_advanced_confirmation::advanced_confirmation_for_candidate;
use crate::write_backup_plan::build_exact_backup_plan;
use crate::write_classification::{high_risk_write_policy, ScalarWriteValueKind};
use crate::write_enablement_readiness::current_production_write_enablement_readiness;
use crate::write_flow::{apply_setting_change, write_flow_config_setting, write_flow_value_kind};
use crate::write_review_walkthrough::build_write_review_walkthrough;
use crate::write_target_candidate::write_target_candidates_for_layered_setting;
use crate::write_target_recommendation::recommend_write_targets;
use crate::write_verification_plan::planned_reread_verification;

const DASHBOARD_ID: &str = "dashboard";
const CONFIG_ID: &str = "config";

#[derive(Debug, Clone)]
struct SidebarItem {
    id: String,
    label: String,
    target_tab_id: Option<String>,
}

pub fn show_main_window(
    app: &adw::Application,
    bundle: ExportBundle,
    summary: ValidationSummary,
    config_discovery: ConfigDiscovery,
    current_config: CurrentConfigSnapshot,
) {
    let model = Rc::new(UiProjection::from_bundle(
        &bundle,
        &summary,
        config_discovery,
        current_config,
    ));
    let selected_tab_id = Rc::new(RefCell::new(String::from(DASHBOARD_ID)));
    let current_query = Rc::new(RefCell::new(String::new()));
    let displayed_row_ids = Rc::new(RefCell::new(Vec::new()));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Hyprland Settings")
        .default_width(1180)
        .default_height(760)
        .build();
    window.set_widget_name("hyprland-settings-main-window");

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.set_widget_name("hyprland-settings-root");

    let title = adw::WindowTitle::new("Hyprland Settings", "Hyprland config metadata and values");
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&title));
    root.append(&header);

    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    body.set_vexpand(true);
    body.set_hexpand(true);
    root.append(&body);

    let sidebar_items = Rc::new(sidebar_items(&model));
    let config_selection_state = Rc::new(RefCell::new(config_selection_state_for_discovery(
        &model.config_discovery,
    )));
    let sidebar = build_sidebar(&sidebar_items);
    let sidebar_scroll = gtk::ScrolledWindow::builder()
        .min_content_width(250)
        .child(&sidebar)
        .build();
    body.append(&sidebar_scroll);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_hexpand(true);
    content.set_vexpand(true);
    body.append(&content);

    let dashboard_view = build_dashboard_view(&model, &sidebar, &sidebar_items);
    dashboard_view.set_widget_name("hyprland-settings-dashboard-page");
    dashboard_view.set_tooltip_text(Some("Dashboard page"));
    content.append(&dashboard_view);

    let config_view = build_config_view(&model, &window, &config_selection_state);
    config_view.set_widget_name("hyprland-settings-config-page");
    config_view.set_tooltip_text(Some("Config page"));
    content.append(&config_view);

    let settings_view = gtk::Box::new(gtk::Orientation::Vertical, 12);
    settings_view.set_widget_name("hyprland-settings-category-page");
    settings_view.set_tooltip_text(Some("Settings category page"));
    settings_view.set_hexpand(true);
    settings_view.set_vexpand(true);
    content.append(&settings_view);

    let search_entry = gtk::SearchEntry::new();
    search_entry.set_widget_name("hyprland-settings-search");
    search_entry.set_placeholder_text(Some("Search settings"));
    search_entry.set_tooltip_text(Some("Search settings"));
    settings_view.append(&search_entry);

    let tab_title = title_label("");
    tab_title.set_widget_name("hyprland-settings-category-title");
    settings_view.append(&tab_title);

    let settings_list = gtk::ListBox::new();
    settings_list.set_widget_name("hyprland-settings-setting-list");
    settings_list.set_tooltip_text(Some("Setting row list"));
    settings_list.set_selection_mode(gtk::SelectionMode::Single);
    let settings_scroll = gtk::ScrolledWindow::builder()
        .min_content_width(420)
        .vexpand(true)
        .hexpand(true)
        .child(&settings_list)
        .build();

    let (detail_panel, detail_content) = build_detail_panel();

    let work_area = gtk::Paned::new(gtk::Orientation::Horizontal);
    work_area.set_vexpand(true);
    work_area.set_hexpand(true);
    work_area.set_wide_handle(true);
    work_area.set_position(520);
    work_area.set_start_child(Some(&settings_scroll));
    work_area.set_end_child(Some(&detail_panel));
    work_area.set_resize_start_child(true);
    work_area.set_resize_end_child(true);
    work_area.set_shrink_start_child(false);
    work_area.set_shrink_end_child(false);
    settings_view.append(&work_area);

    render_main_view(
        &model,
        &selected_tab_id.borrow(),
        &dashboard_view,
        &config_view,
        &settings_view,
        &current_query.borrow(),
        &tab_title,
        &settings_list,
        &displayed_row_ids,
        &detail_content,
        &config_selection_state,
    );

    {
        let model = Rc::clone(&model);
        let selected_tab_id = Rc::clone(&selected_tab_id);
        let current_query = Rc::clone(&current_query);
        let sidebar_items = Rc::clone(&sidebar_items);
        let dashboard_view = dashboard_view.clone();
        let config_view = config_view.clone();
        let settings_view = settings_view.clone();
        let tab_title = tab_title.clone();
        let settings_list = settings_list.clone();
        let displayed_row_ids = Rc::clone(&displayed_row_ids);
        let detail_content = detail_content.clone();
        let config_selection_state = Rc::clone(&config_selection_state);
        sidebar.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                if let Some(item) = sidebar_items.get(row.index() as usize) {
                    *selected_tab_id.borrow_mut() = item.id.clone();
                    render_main_view(
                        &model,
                        &selected_tab_id.borrow(),
                        &dashboard_view,
                        &config_view,
                        &settings_view,
                        &current_query.borrow(),
                        &tab_title,
                        &settings_list,
                        &displayed_row_ids,
                        &detail_content,
                        &config_selection_state,
                    );
                }
            }
        });
    }

    {
        let model = Rc::clone(&model);
        let selected_tab_id = Rc::clone(&selected_tab_id);
        let current_query = Rc::clone(&current_query);
        let dashboard_view = dashboard_view.clone();
        let config_view = config_view.clone();
        let settings_view = settings_view.clone();
        let tab_title = tab_title.clone();
        let settings_list = settings_list.clone();
        let displayed_row_ids = Rc::clone(&displayed_row_ids);
        let detail_content = detail_content.clone();
        let config_selection_state = Rc::clone(&config_selection_state);
        search_entry.connect_search_changed(move |entry| {
            *current_query.borrow_mut() = entry.text().to_string();
            render_main_view(
                &model,
                &selected_tab_id.borrow(),
                &dashboard_view,
                &config_view,
                &settings_view,
                &current_query.borrow(),
                &tab_title,
                &settings_list,
                &displayed_row_ids,
                &detail_content,
                &config_selection_state,
            );
        });
    }

    {
        let model = Rc::clone(&model);
        let displayed_row_ids = Rc::clone(&displayed_row_ids);
        let detail_content = detail_content.clone();
        let config_selection_state = Rc::clone(&config_selection_state);
        settings_list.connect_row_selected(move |_, row| {
            let Some(row) = row else {
                render_empty_detail(&detail_content);
                return;
            };
            let row_id = displayed_row_ids
                .borrow()
                .get(row.index() as usize)
                .cloned();
            if let Some(row_id) = row_id {
                render_detail(&model, &row_id, &detail_content, &config_selection_state);
            } else {
                render_empty_detail(&detail_content);
            }
        });
    }

    window.set_content(Some(&root));
    if let Some(row) = sidebar.row_at_index(0) {
        sidebar.select_row(Some(&row));
    }
    window.present();
}

pub fn show_error_window(app: &adw::Application, export_context: &str, error: &str) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Hyprland Settings")
        .default_width(760)
        .default_height(360)
        .build();

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let title = adw::WindowTitle::new("Hyprland Settings", "Startup error");
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&title));
    root.append(&header);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);
    root.append(&content);

    content.append(&title_label("Export metadata could not be loaded"));
    content.append(&body_label(&format!(
        "Export path attempted: {export_context}"
    )));
    content.append(&body_label(error));
    content.append(&body_label(
        "No live Hyprland config was read. No settings were changed.",
    ));

    window.set_content(Some(&root));
    window.present();
}

fn sidebar_items(model: &UiProjection) -> Vec<SidebarItem> {
    let mut items = vec![
        SidebarItem {
            id: DASHBOARD_ID.to_string(),
            label: "Dashboard".to_string(),
            target_tab_id: None,
        },
        SidebarItem {
            id: CONFIG_ID.to_string(),
            label: "Config".to_string(),
            target_tab_id: None,
        },
    ];
    let order = [
        "appearance",
        "windows-layout",
        "display",
        "input",
        "keybinds",
        "cursor",
        "permissions",
        "system",
        "animations",
    ];

    for tab_id in order {
        let Some(tab) = model
            .tabs
            .iter()
            .find(|tab| tab.id == tab_id && tab.row_count > 0)
        else {
            continue;
        };
        items.push(SidebarItem {
            id: tab.id.clone(),
            label: sidebar_tab_label(&tab.id, &tab.label),
            target_tab_id: Some(tab.id.clone()),
        });
    }
    items
}

fn sidebar_tab_label(tab_id: &str, label: &str) -> String {
    match tab_id {
        "keybinds" => "Keyboard".to_string(),
        _ => label.to_string(),
    }
}

fn build_sidebar(items: &[SidebarItem]) -> gtk::ListBox {
    let sidebar = gtk::ListBox::new();
    sidebar.set_widget_name("hyprland-settings-navigation-sidebar");
    sidebar.set_selection_mode(gtk::SelectionMode::Single);

    for item in items {
        let row = gtk::ListBoxRow::new();
        row.set_widget_name(&format!(
            "hyprland-settings-nav-{}",
            safe_widget_name(&item.id)
        ));
        row.set_tooltip_text(Some(&format!("Navigation: {}", item.label)));
        let row_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);

        row_box.append(&body_label(&item.label));

        row.set_child(Some(&row_box));
        sidebar.append(&row);
    }

    sidebar
}

fn build_dashboard_view(
    model: &UiProjection,
    sidebar: &gtk::ListBox,
    sidebar_items: &Rc<Vec<SidebarItem>>,
) -> gtk::ScrolledWindow {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 14);
    content.set_widget_name("hyprland-settings-dashboard-content");
    content.set_margin_top(4);
    content.set_margin_bottom(16);
    content.set_margin_start(4);
    content.set_margin_end(4);

    content.append(&title_label("Dashboard"));

    let cards = gtk::FlowBox::new();
    cards.set_selection_mode(gtk::SelectionMode::None);
    cards.set_homogeneous(true);
    cards.set_min_children_per_line(2);
    cards.set_max_children_per_line(3);
    cards.set_row_spacing(12);
    cards.set_column_spacing(12);
    for card in dashboard_cards() {
        cards.insert(
            &build_dashboard_card(
                card.title,
                card.description,
                card.target_tab_id,
                sidebar,
                sidebar_items,
            ),
            -1,
        );
    }
    content.append(&cards);

    if dashboard_needs_attention(model) {
        let attention = gtk::Frame::new(None);
        let box_content = gtk::Box::new(gtk::Orientation::Vertical, 6);
        box_content.set_margin_top(12);
        box_content.set_margin_bottom(12);
        box_content.set_margin_start(12);
        box_content.set_margin_end(12);
        box_content.append(&title_label("Needs attention"));
        box_content.append(&body_label(
            "Some settings appear more than once in your config.",
        ));
        box_content.append(&small_label("Review them before applying changes."));
        attention.set_child(Some(&box_content));
        content.append(&attention);
    }

    gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&content)
        .build()
}

struct DashboardCard {
    title: &'static str,
    description: &'static str,
    target_tab_id: &'static str,
}

fn dashboard_cards() -> [DashboardCard; 7] {
    [
        DashboardCard {
            title: "Config",
            description: "Choose which Hyprland config the app reviews and where future changes should be saved.",
            target_tab_id: CONFIG_ID,
        },
        DashboardCard {
            title: "Appearance",
            description: "Change blur, shadows, borders, gaps, and other visual settings.",
            target_tab_id: "appearance",
        },
        DashboardCard {
            title: "Windows & Layout",
            description: "Choose how windows open, move, resize, snap, and tile.",
            target_tab_id: "windows-layout",
        },
        DashboardCard {
            title: "Input",
            description: "Adjust keyboard, mouse, touchpad, gestures, and focus behavior.",
            target_tab_id: "input",
        },
        DashboardCard {
            title: "Displays",
            description: "Review monitor, rendering, color, and display-related options.",
            target_tab_id: "display",
        },
        DashboardCard {
            title: "Shortcuts",
            description: "Browse keybind-related settings and preserved shortcut entries.",
            target_tab_id: "keybinds",
        },
        DashboardCard {
            title: "Advanced",
            description: "Review settings that need extra care before changing.",
            target_tab_id: "system",
        },
    ]
}

fn build_dashboard_card(
    title: &str,
    description: &str,
    target_tab_id: &str,
    sidebar: &gtk::ListBox,
    sidebar_items: &Rc<Vec<SidebarItem>>,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&format!(
        "hyprland-settings-dashboard-card-{}",
        safe_widget_name(title)
    ));
    frame.set_tooltip_text(Some(&format!("Dashboard card: {title}")));
    let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
    card.set_margin_top(12);
    card.set_margin_bottom(12);
    card.set_margin_start(12);
    card.set_margin_end(12);
    card.append(&title_label(title));
    card.append(&body_label(description));

    if let Some(index) = sidebar_items.iter().position(|item| {
        item.id == target_tab_id || item.target_tab_id.as_deref() == Some(target_tab_id)
    }) {
        let button = gtk::Button::with_label("Open");
        button.set_widget_name(&format!(
            "hyprland-settings-open-{}",
            safe_widget_name(title)
        ));
        button.set_tooltip_text(Some(&format!("Open {title}")));
        let sidebar = sidebar.clone();
        button.connect_clicked(move |_| {
            if let Some(row) = sidebar.row_at_index(index as i32) {
                sidebar.select_row(Some(&row));
            }
        });
        card.append(&button);
    }

    frame.set_child(Some(&card));
    frame
}

fn dashboard_needs_attention(model: &UiProjection) -> bool {
    matches!(
        model.current_config.status,
        CurrentConfigLoadStatus::Loaded { .. }
    ) && (model.current_value_summary.duplicate_conflict_rows > 0
        || model.current_value_summary.parser_warning_rows > 0
        || model
            .structured_families
            .iter()
            .any(|family| family.warning_count > 0))
}

fn build_config_view(
    model: &UiProjection,
    window: &adw::ApplicationWindow,
    selection_state: &Rc<RefCell<ConfigSelectionState>>,
) -> gtk::ScrolledWindow {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 14);
    content.set_widget_name("hyprland-settings-config-content");
    content.set_margin_top(4);
    content.set_margin_bottom(16);
    content.set_margin_start(4);
    content.set_margin_end(4);

    content.append(&title_label("Config"));
    content.append(&body_label(
        "Review which Hyprland config the app is using before making changes.",
    ));

    content.append(&config_file_selection_section(
        &model.config_discovery,
        window,
        selection_state,
    ));

    content.append(&connected_files_review_section(&model.config_discovery));

    content.append(&profile_mode_detail_section());

    content.append(&config_section(
        "Future changes",
        vec![
            "When a setting is controlled in more than one place, the app will ask where to save the change before applying it.".to_string(),
            "The app will back up the file before saving changes.".to_string(),
        ],
        None,
    ));

    gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&content)
        .build()
}

fn config_path_summary(discovery: &ConfigDiscovery) -> String {
    match &discovery.status {
        crate::config_discovery::ConfigDiscoveryStatus::Found { path, .. } => {
            path.display().to_string()
        }
        crate::config_discovery::ConfigDiscoveryStatus::Missing => {
            "No Hyprland config file was detected.".to_string()
        }
        crate::config_discovery::ConfigDiscoveryStatus::Unreadable { path, .. } => {
            format!("{} could not be read.", path.display())
        }
        crate::config_discovery::ConfigDiscoveryStatus::NotAFile { path, .. } => {
            format!("{} is not a regular file.", path.display())
        }
    }
}

fn config_selection_state_for_discovery(discovery: &ConfigDiscovery) -> ConfigSelectionState {
    match &discovery.status {
        crate::config_discovery::ConfigDiscoveryStatus::Found { path, .. } => {
            ConfigSelectionState::auto_detected(path)
        }
        _ => ConfigSelectionState::no_detected_config(),
    }
}

fn config_file_selection_section(
    discovery: &ConfigDiscovery,
    window: &adw::ApplicationWindow,
    selection_state: &Rc<RefCell<ConfigSelectionState>>,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    let box_content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    box_content.set_margin_top(12);
    box_content.set_margin_bottom(12);
    box_content.set_margin_start(12);
    box_content.set_margin_end(12);

    box_content.append(&title_label("Config file"));
    for line in config_selection_scaffold_lines(discovery) {
        box_content.append(&body_label(&line));
    }

    let preview_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
    preview_box.set_margin_top(8);
    preview_box.set_visible(false);
    preview_box.append(&body_label("Selected file preview"));
    preview_box.append(&body_label("Selected for review:"));
    let selected_path_label = small_label("");
    preview_box.append(&selected_path_label);
    preview_box.append(&small_label("This file is only being reviewed."));
    preview_box.append(&small_label(
        "This has not changed what the app will write.",
    ));
    preview_box.append(&small_label("This selection is not saved yet."));
    preview_box.append(&small_label(
        "Choose how this preview should read connected files.",
    ));

    let follow_controls = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let review_all_button = gtk::Button::with_label("Review all connected files");
    let only_this_file_button = gtk::Button::with_label("Only this file");
    let cancel_preview_button = gtk::Button::with_label("Cancel");
    follow_controls.append(&review_all_button);
    follow_controls.append(&only_this_file_button);
    follow_controls.append(&cancel_preview_button);
    preview_box.append(&follow_controls);

    let preview_summary_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    preview_summary_box.set_margin_top(8);
    preview_box.append(&preview_summary_box);

    let session_button = gtk::Button::with_label("Use for this session preview");
    preview_box.append(&session_button);
    preview_box.append(&small_label(
        "Using this config for this app session only. This is not saved.",
    ));
    preview_box.append(&small_label(
        "This config is being reread for display only. Apply behavior has not changed.",
    ));

    let session_summary_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    session_summary_box.set_margin_top(8);
    session_summary_box.set_visible(false);
    preview_box.append(&session_summary_box);

    let clear_session_button = gtk::Button::with_label("Clear session preview");
    clear_session_button.set_visible(false);
    preview_box.append(&clear_session_button);

    let clear_button = gtk::Button::with_label("Clear selected file");
    {
        let selection_state = Rc::clone(&selection_state);
        let preview_box = preview_box.clone();
        let selected_path_label = selected_path_label.clone();
        let preview_summary_box = preview_summary_box.clone();
        let session_summary_box = session_summary_box.clone();
        let clear_session_button = clear_session_button.clone();
        clear_button.connect_clicked(move |_| {
            let next_state = selection_state.borrow().clone().cancel_preview();
            *selection_state.borrow_mut() = next_state;
            update_config_selection_preview(
                &selection_state.borrow(),
                &preview_box,
                &selected_path_label,
                &preview_summary_box,
                &session_summary_box,
                &clear_session_button,
            );
        });
    }
    preview_box.append(&clear_button);

    for (button, choice) in [
        (
            review_all_button,
            SourceFollowChoice::ReviewAllConnectedFiles,
        ),
        (only_this_file_button, SourceFollowChoice::OnlySelectedFile),
    ] {
        let selection_state = Rc::clone(&selection_state);
        let preview_box = preview_box.clone();
        let selected_path_label = selected_path_label.clone();
        let preview_summary_box = preview_summary_box.clone();
        let session_summary_box = session_summary_box.clone();
        let clear_session_button = clear_session_button.clone();
        button.connect_clicked(move |_| {
            let preview = selection_state.borrow().preview();
            let Some(path) = preview.selected_for_review else {
                return;
            };
            let mut next_state = selection_state
                .borrow()
                .clone()
                .preview_manual_config(path, choice);
            if preview.session_only {
                next_state = next_state.use_preview_for_session_read_only();
            }
            *selection_state.borrow_mut() = next_state;
            update_config_selection_preview(
                &selection_state.borrow(),
                &preview_box,
                &selected_path_label,
                &preview_summary_box,
                &session_summary_box,
                &clear_session_button,
            );
        });
    }

    {
        let selection_state = Rc::clone(&selection_state);
        let preview_box = preview_box.clone();
        let selected_path_label = selected_path_label.clone();
        let preview_summary_box = preview_summary_box.clone();
        let session_summary_box = session_summary_box.clone();
        let clear_session_button = clear_session_button.clone();
        cancel_preview_button.connect_clicked(move |_| {
            let next_state = selection_state.borrow().clone().cancel_preview();
            *selection_state.borrow_mut() = next_state;
            update_config_selection_preview(
                &selection_state.borrow(),
                &preview_box,
                &selected_path_label,
                &preview_summary_box,
                &session_summary_box,
                &clear_session_button,
            );
        });
    }

    {
        let selection_state = Rc::clone(&selection_state);
        let preview_box = preview_box.clone();
        let selected_path_label = selected_path_label.clone();
        let preview_summary_box = preview_summary_box.clone();
        let session_summary_box = session_summary_box.clone();
        let clear_session_button = clear_session_button.clone();
        session_button.connect_clicked(move |_| {
            let next_state = selection_state
                .borrow()
                .clone()
                .use_preview_for_session_read_only();
            *selection_state.borrow_mut() = next_state;
            update_config_selection_preview(
                &selection_state.borrow(),
                &preview_box,
                &selected_path_label,
                &preview_summary_box,
                &session_summary_box,
                &clear_session_button,
            );
        });
    }

    {
        let selection_state = Rc::clone(&selection_state);
        let preview_box = preview_box.clone();
        let selected_path_label = selected_path_label.clone();
        let preview_summary_box = preview_summary_box.clone();
        let session_summary_box = session_summary_box.clone();
        let clear_session_button = clear_session_button.clone();
        let clear_session_button_for_signal = clear_session_button.clone();
        clear_session_button_for_signal.connect_clicked(move |_| {
            let next_state = selection_state.borrow().clone().cancel_preview();
            *selection_state.borrow_mut() = next_state;
            update_config_selection_preview(
                &selection_state.borrow(),
                &preview_box,
                &selected_path_label,
                &preview_summary_box,
                &session_summary_box,
                &clear_session_button,
            );
        });
    }

    let choose_button = gtk::Button::with_label("Choose Config File...");
    {
        let window = window.clone();
        let selection_state = Rc::clone(&selection_state);
        let preview_box = preview_box.clone();
        let selected_path_label = selected_path_label.clone();
        let preview_summary_box = preview_summary_box.clone();
        let session_summary_box = session_summary_box.clone();
        let clear_session_button = clear_session_button.clone();
        choose_button.connect_clicked(move |_| {
            let dialog = gtk::FileChooserNative::new(
                Some("Choose Config File"),
                Some(&window),
                gtk::FileChooserAction::Open,
                Some("Choose"),
                Some("Cancel"),
            );
            let selection_state = Rc::clone(&selection_state);
            let preview_box = preview_box.clone();
            let selected_path_label = selected_path_label.clone();
            let preview_summary_box = preview_summary_box.clone();
            let session_summary_box = session_summary_box.clone();
            let clear_session_button = clear_session_button.clone();
            dialog.connect_response(move |dialog, response| {
                if response == gtk::ResponseType::Accept {
                    if let Some(path) = dialog.file().and_then(|file| file.path()) {
                        let next_state = selection_state.borrow().clone().preview_manual_config(
                            path,
                            SourceFollowChoice::ReviewAllConnectedFiles,
                        );
                        *selection_state.borrow_mut() = next_state;
                        update_config_selection_preview(
                            &selection_state.borrow(),
                            &preview_box,
                            &selected_path_label,
                            &preview_summary_box,
                            &session_summary_box,
                            &clear_session_button,
                        );
                    }
                }
                dialog.destroy();
            });
            dialog.show();
        });
    }

    box_content.append(&choose_button);
    box_content.append(&preview_box);
    frame.set_child(Some(&box_content));
    frame
}

fn update_config_selection_preview(
    state: &ConfigSelectionState,
    preview_box: &gtk::Box,
    selected_path_label: &gtk::Label,
    preview_summary_box: &gtk::Box,
    session_summary_box: &gtk::Box,
    clear_session_button: &gtk::Button,
) {
    let preview = state.preview();
    if let Some(path) = preview.selected_for_review {
        selected_path_label.set_label(&path.display().to_string());
        clear_box(preview_summary_box);
        for line in selected_file_preview_summary_lines(&path, preview.source_follow_choice) {
            preview_summary_box.append(&small_label(&line));
        }
        clear_box(session_summary_box);
        if preview.session_only {
            for line in session_config_preview_summary_lines(&path, preview.source_follow_choice) {
                session_summary_box.append(&small_label(&line));
            }
            session_summary_box.set_visible(true);
            clear_session_button.set_visible(true);
        } else {
            session_summary_box.set_visible(false);
            clear_session_button.set_visible(false);
        }
        preview_box.set_visible(true);
    } else {
        selected_path_label.set_label("");
        clear_box(preview_summary_box);
        clear_box(session_summary_box);
        session_summary_box.set_visible(false);
        clear_session_button.set_visible(false);
        preview_box.set_visible(false);
    }
}

fn selected_file_preview_summary_lines(
    path: &std::path::Path,
    source_follow_choice: SourceFollowChoice,
) -> Vec<String> {
    let source_follow_policy = match source_follow_choice {
        SourceFollowChoice::ReviewAllConnectedFiles => SourceFollowPolicy::ReviewAll,
        SourceFollowChoice::OnlySelectedFile | SourceFollowChoice::Cancel => {
            SourceFollowPolicy::OnlyRoot
        }
    };
    let graph = inspect_config_graph_with_options(
        path,
        ConfigGraphOptions {
            source_follow_policy,
            ..ConfigGraphOptions::from_env()
        },
    );

    if graph.files.first().is_none_or(|file| !file.readable) {
        return vec![
            "This file could not be read for preview.".to_string(),
            "No changes were made.".to_string(),
        ];
    }

    let mut lines = vec![
        format!("Connected files found: {}", graph.connected_file_count),
        format!("Unreadable files: {}", graph.unreadable_file_count),
        format!(
            "Profile-style files: {}",
            if graph.has_profile_hints || graph.has_mode_hints || graph.has_theme_hints {
                "detected"
            } else {
                "not detected"
            }
        ),
        format!(
            "Script-managed hints: {}",
            if graph.has_script_managed_hints {
                "detected"
            } else {
                "not detected"
            }
        ),
        format!(
            "Generated-file hints: {}",
            if graph.has_generated_hints {
                "detected"
            } else {
                "not detected"
            }
        ),
        format!(
            "Cycles: {}",
            if graph.cycles.is_empty() {
                "not detected"
            } else {
                "detected"
            }
        ),
        format!(
            "Unsupported patterns: {}",
            if graph.unsupported_sources.is_empty() {
                "not detected"
            } else {
                "detected"
            }
        ),
    ];

    if source_follow_choice == SourceFollowChoice::OnlySelectedFile {
        lines.push("Connected files are not included in this preview.".to_string());
    }
    lines
}

fn session_config_preview_summary_lines(
    path: &std::path::Path,
    source_follow_choice: SourceFollowChoice,
) -> Vec<String> {
    build_session_config_preview(path, source_follow_choice).user_facing_lines()
}

fn config_selection_scaffold_lines(discovery: &ConfigDiscovery) -> Vec<String> {
    vec![
        "Using:".to_string(),
        config_path_summary(discovery),
        "Auto-detection is a starting point.".to_string(),
        "Choose another config file to review.".to_string(),
        "This has not changed what the app will write.".to_string(),
        "The selected file is preview-only until a future review step.".to_string(),
    ]
}

fn config_graph_summary_lines(discovery: &ConfigDiscovery) -> Vec<String> {
    let crate::config_discovery::ConfigDiscoveryStatus::Found { path, .. } = &discovery.status
    else {
        return vec![
            "No connected config files were detected.".to_string(),
            "Connected-file review is read-only right now.".to_string(),
        ];
    };

    let graph = inspect_config_graph(path);
    friendly_config_graph_summary(&graph)
}

fn connected_files_review_section(discovery: &ConfigDiscovery) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-connected-files-section");
    frame.set_tooltip_text(Some(
        "Connected files section. This review is read-only and does not change files.",
    ));
    let box_content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    box_content.set_margin_top(12);
    box_content.set_margin_bottom(12);
    box_content.set_margin_start(12);
    box_content.set_margin_end(12);

    box_content.append(&title_label("Connected files"));
    let graph = config_graph_for_discovery(discovery);
    let summary_lines = graph
        .as_ref()
        .map(friendly_config_graph_summary)
        .unwrap_or_else(|| config_graph_summary_lines(discovery));
    for line in &summary_lines {
        box_content.append(&body_label(line));
    }

    if let Some(graph) = &graph {
        append_connected_files_review(&box_content, graph);
        append_connected_file_issue_warnings(&box_content, graph);
    }

    let action = gtk::Button::with_label("Choose review mode (planned)");
    action.set_sensitive(false);
    box_content.append(&action);

    frame.set_child(Some(&box_content));
    frame
}

fn config_graph_for_discovery(discovery: &ConfigDiscovery) -> Option<ConfigGraphSummary> {
    let crate::config_discovery::ConfigDiscoveryStatus::Found { path, .. } = &discovery.status
    else {
        return None;
    };
    Some(inspect_config_graph(path))
}

fn friendly_config_graph_summary(graph: &ConfigGraphSummary) -> Vec<String> {
    let mut lines = Vec::new();
    if graph.multi_file {
        lines.push(format!(
            "This setup uses {} config files.",
            graph.connected_file_count
        ));
        lines.push("Some files are connected through source/include lines.".to_string());
    } else {
        lines.push("No connected config files were detected.".to_string());
    }

    if graph.unreadable_file_count > 0 {
        lines.push(format!(
            "{} connected file(s) could not be read.",
            graph.unreadable_file_count
        ));
    }

    if graph.has_profile_hints || graph.has_mode_hints || graph.has_theme_hints {
        lines.push("Profile-style config files were detected.".to_string());
    }

    if graph.has_script_managed_hints {
        lines.push("Some files may be changed by scripts.".to_string());
    }

    if !graph.unsupported_sources.is_empty() {
        lines.push("Some connected files may not be shown yet.".to_string());
    }

    lines.push("Connected-file review is read-only right now.".to_string());
    lines.push("The app will review connected files before allowing changes.".to_string());
    lines
}

fn append_connected_files_review(parent: &gtk::Box, graph: &ConfigGraphSummary) {
    if graph.files.is_empty() {
        return;
    }

    for file in &graph.files {
        parent.append(&connected_file_card(file, graph));
    }
}

fn connected_file_card(file: &ConfigGraphFile, graph: &ConfigGraphSummary) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&format!(
        "hyprland-settings-connected-file-card-{}",
        connected_file_accessibility_suffix(file)
    ));
    frame.set_tooltip_text(Some(&connected_file_accessibility_summary(file)));
    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(10);
    content.set_margin_end(10);

    content.append(&body_label(&connected_file_title(file)));
    content.append(&small_label(&friendly_path(&file.path)));

    if let Some(target) = &file.symlink_target {
        content.append(&small_label(&format!(
            "Points to: {}",
            friendly_path(target)
        )));
    }

    if !file.readable {
        content.append(&small_label("Missing or unreadable"));
    }

    for line in connected_file_hint_lines(file) {
        content.append(&small_label(&line));
    }

    append_connected_file_blocker_detail_surfaces(&content, file);
    append_connected_file_details(&content, file, graph);

    frame.set_child(Some(&content));
    frame
}

fn append_connected_file_blocker_detail_surfaces(parent: &gtk::Box, file: &ConfigGraphFile) {
    if connected_file_has_hint(file, ConfigManagementHintKind::GeneratedFile) {
        parent.append(&connected_file_blocker_detail_surface(
            "hyprland-settings-connected-file-detail-generated",
            "Generated file detail",
            "This file may be generated. The app will not write it yet.",
            "Generated connected-file blocker detail. This file may be generated. The app will not write it yet.",
        ));
    }

    if connected_file_has_script_hint(file) {
        parent.append(&connected_file_blocker_detail_surface(
            "hyprland-settings-connected-file-detail-script-managed",
            "Script-managed file detail",
            "This file may be changed by a script. The app will not write it yet.",
            "Script-managed connected-file blocker detail. This file may be changed by a script. The app will not write it yet.",
        ));
    }

    if file.is_symlink || connected_file_has_hint(file, ConfigManagementHintKind::SymlinkManaged) {
        parent.append(&connected_file_blocker_detail_surface(
            "hyprland-settings-connected-file-detail-symlink-current-profile",
            "Symlink/current-profile detail",
            "This file may be a symlink or current-profile file. The app will not write it yet.",
            "Symlink current-profile connected-file blocker detail. This file may be a symlink or current-profile file. The app will not write it yet.",
        ));
    }
}

fn connected_file_blocker_detail_surface(
    widget_name: &str,
    title: &str,
    copy: &str,
    tooltip: &str,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(widget_name);
    frame.set_tooltip_text(Some(tooltip));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&body_label(title));
    content.append(&small_label(copy));
    content.append(&small_label(
        "This is a read-only safety detail. It does not edit files, reload Hyprland, run scripts, or switch profiles.",
    ));

    frame.set_child(Some(&content));
    frame
}

fn profile_mode_detail_section() -> gtk::Frame {
    let frame = config_section(
        "Profiles",
        vec![
            "Profile switching is not active yet.".to_string(),
            "Future versions may show desktop, gaming, theme, or host profiles here.".to_string(),
        ],
        Some(("Profile switching planned", false)),
    );
    frame.set_widget_name("hyprland-settings-profile-mode-detail");
    frame.set_tooltip_text(Some(
        "Profile mode detail. Profile switching is not active yet. The app will not change profile files or symlinks.",
    ));
    frame
}

fn append_connected_file_details(
    parent: &gtk::Box,
    file: &ConfigGraphFile,
    graph: &ConfigGraphSummary,
) {
    let expander = gtk::Expander::new(Some("Details"));
    expander.set_expanded(false);

    let details = gtk::Box::new(gtk::Orientation::Vertical, 5);
    details.set_margin_top(8);
    details.set_margin_bottom(8);
    details.set_margin_start(8);
    details.set_margin_end(8);

    append_detail_line(
        &details,
        "Why this file is listed",
        &connected_file_reason(file),
    );
    append_detail_line(&details, "Role", &connected_file_title(file));
    append_detail_line(&details, "Readable", &connected_file_readable_label(file));
    append_detail_line(
        &details,
        "Symlink",
        if file.is_symlink { "Yes" } else { "No" },
    );

    if let Some(target) = &file.symlink_target {
        append_detail_line(&details, "Points to", &friendly_path(target));
    }

    append_detail_line(
        &details,
        "Connected from",
        &connected_file_source_summary(file, graph),
    );

    let notes = connected_file_notes(file);
    if notes.is_empty() {
        append_detail_line(&details, "Notes", "No extra notes were detected.");
    } else {
        append_detail_line(&details, "Notes", &notes.join("; "));
    }

    expander.set_child(Some(&details));
    parent.append(&expander);
}

fn connected_file_reason(file: &ConfigGraphFile) -> String {
    if file.source_depth == 0 {
        "This is the config file the app is currently reviewing.".to_string()
    } else {
        "This file is connected from another config file.".to_string()
    }
}

fn connected_file_readable_label(file: &ConfigGraphFile) -> &'static str {
    if file.readable {
        "Yes"
    } else {
        "No"
    }
}

fn connected_file_source_summary(file: &ConfigGraphFile, graph: &ConfigGraphSummary) -> String {
    if file.source_depth == 0 {
        return "This is the selected config root.".to_string();
    }

    let Some(source) = source_reference_for_file(file, graph) else {
        return "The app found this file while reviewing connected configs.".to_string();
    };

    format!(
        "{}, line {}",
        friendly_path(&source.source_file),
        source.line_number
    )
}

fn source_reference_for_file<'a>(
    file: &ConfigGraphFile,
    graph: &'a ConfigGraphSummary,
) -> Option<&'a ConfigSourceReference> {
    graph.source_references.iter().find(|reference| {
        reference
            .resolved_target
            .as_ref()
            .is_some_and(|target| paths_match_file(target, file))
    })
}

fn paths_match_file(path: &std::path::Path, file: &ConfigGraphFile) -> bool {
    if path == file.path {
        return true;
    }
    if let Some(resolved) = &file.resolved_path {
        if path == resolved {
            return true;
        }
    }
    if let Some(target) = &file.symlink_target {
        if path == target {
            return true;
        }
    }
    false
}

fn connected_file_notes(file: &ConfigGraphFile) -> Vec<String> {
    let mut notes = Vec::new();
    for hint in &file.hints {
        let note = friendly_connected_file_note(&hint.kind);
        if !notes.iter().any(|existing| existing == note) {
            notes.push(note.to_string());
        }
    }
    notes
}

fn connected_file_has_hint(file: &ConfigGraphFile, kind: ConfigManagementHintKind) -> bool {
    file.hints.iter().any(|hint| hint.kind == kind)
}

fn connected_file_has_script_hint(file: &ConfigGraphFile) -> bool {
    file.hints.iter().any(|hint| {
        matches!(
            hint.kind,
            ConfigManagementHintKind::ScriptManaged | ConfigManagementHintKind::ScriptReferenced
        )
    })
}

fn connected_file_accessibility_suffix(file: &ConfigGraphFile) -> &'static str {
    if connected_file_has_hint(file, ConfigManagementHintKind::GeneratedFile) {
        "generated"
    } else if connected_file_has_script_hint(file) {
        "script-managed"
    } else if file.is_symlink
        || connected_file_has_hint(file, ConfigManagementHintKind::SymlinkManaged)
    {
        "symlink-current-profile"
    } else if file.source_depth == 0 {
        "root"
    } else {
        "connected"
    }
}

fn connected_file_accessibility_summary(file: &ConfigGraphFile) -> String {
    let mut parts = vec![format!(
        "Connected file card: {}",
        connected_file_title(file)
    )];
    if connected_file_has_hint(file, ConfigManagementHintKind::GeneratedFile) {
        parts.push("This file may be generated. The app will not write it yet.".to_string());
    }
    if connected_file_has_script_hint(file) {
        parts.push(
            "This file may be changed by a script. The app will not write it yet.".to_string(),
        );
    }
    if file.is_symlink || connected_file_has_hint(file, ConfigManagementHintKind::SymlinkManaged) {
        parts.push(
            "This file may be a symlink or current-profile file. The app will not write it yet."
                .to_string(),
        );
    }
    parts.join(" ")
}

fn friendly_connected_file_note(kind: &ConfigManagementHintKind) -> &'static str {
    match kind {
        ConfigManagementHintKind::GeneratedFile => "This file appears to be generated",
        ConfigManagementHintKind::ScriptReferenced | ConfigManagementHintKind::ScriptManaged => {
            "This file may be changed by scripts"
        }
        ConfigManagementHintKind::SymlinkManaged => "This file is symlinked",
        ConfigManagementHintKind::CurrentProfile
        | ConfigManagementHintKind::DesktopProfile
        | ConfigManagementHintKind::GamingProfile
        | ConfigManagementHintKind::LaptopProfile
        | ConfigManagementHintKind::PerformanceProfile
        | ConfigManagementHintKind::ModeProfile
        | ConfigManagementHintKind::ThemeProfile
        | ConfigManagementHintKind::HostProfile => "This file looks like a profile file",
    }
}

fn connected_file_title(file: &ConfigGraphFile) -> String {
    if file.source_depth == 0 {
        return "Main config".to_string();
    }

    for (kind, label) in [
        (ConfigManagementHintKind::CurrentProfile, "Current profile"),
        (ConfigManagementHintKind::DesktopProfile, "Desktop profile"),
        (ConfigManagementHintKind::GamingProfile, "Gaming profile"),
        (ConfigManagementHintKind::LaptopProfile, "Laptop profile"),
        (
            ConfigManagementHintKind::PerformanceProfile,
            "Performance profile",
        ),
        (ConfigManagementHintKind::ThemeProfile, "Theme profile"),
        (ConfigManagementHintKind::HostProfile, "Host profile"),
        (ConfigManagementHintKind::GeneratedFile, "Generated file"),
    ] {
        if file.hints.iter().any(|hint| hint.kind == kind) {
            return label.to_string();
        }
    }

    "Connected config".to_string()
}

fn connected_file_hint_lines(file: &ConfigGraphFile) -> Vec<String> {
    let mut lines = Vec::new();
    for hint in &file.hints {
        let label = friendly_config_hint_label(&hint.kind);
        if !lines.iter().any(|line| line == label) {
            lines.push(label.to_string());
        }
    }
    lines
}

fn friendly_config_hint_label(kind: &ConfigManagementHintKind) -> &'static str {
    match kind {
        ConfigManagementHintKind::CurrentProfile => "Current profile",
        ConfigManagementHintKind::DesktopProfile => "Desktop profile",
        ConfigManagementHintKind::GamingProfile => "Gaming profile",
        ConfigManagementHintKind::LaptopProfile => "Laptop profile",
        ConfigManagementHintKind::PerformanceProfile => "Performance profile",
        ConfigManagementHintKind::ModeProfile => "Profile file",
        ConfigManagementHintKind::ThemeProfile => "Theme profile",
        ConfigManagementHintKind::HostProfile => "Host profile",
        ConfigManagementHintKind::GeneratedFile => "Generated file",
        ConfigManagementHintKind::ScriptReferenced | ConfigManagementHintKind::ScriptManaged => {
            "May be changed by scripts"
        }
        ConfigManagementHintKind::SymlinkManaged => "Symlinked file",
    }
}

fn append_connected_file_issue_warnings(parent: &gtk::Box, graph: &ConfigGraphSummary) {
    if !graph.unreadable_files.is_empty() {
        parent.append(&small_label("Some connected files could not be read."));
        parent.append(&small_label("Missing or unreadable:"));
        for issue in &graph.unreadable_files {
            parent.append(&small_label(&format!("- {}", friendly_path(&issue.path))));
        }
    }

    if !graph.cycles.is_empty() {
        parent.append(&small_label(
            "Some connected files refer back to each other.",
        ));
        parent.append(&small_label(
            "The app stopped following them to avoid looping.",
        ));
    }

    if !graph.unsupported_sources.is_empty() {
        parent.append(&small_label(
            "Some connected file patterns are not shown yet.",
        ));
    }

    if graph.has_generated_hints || graph.has_script_managed_hints {
        parent.append(&small_label(
            "Review carefully before editing these files in a future version.",
        ));
    }
}

fn friendly_path(path: &std::path::Path) -> String {
    path.display().to_string()
}

fn config_section(title: &str, lines: Vec<String>, button: Option<(&str, bool)>) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    let box_content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    box_content.set_margin_top(12);
    box_content.set_margin_bottom(12);
    box_content.set_margin_start(12);
    box_content.set_margin_end(12);

    box_content.append(&title_label(title));
    for line in &lines {
        box_content.append(&body_label(line));
    }
    if let Some((label, active)) = button {
        let action = gtk::Button::with_label(label);
        action.set_sensitive(active);
        box_content.append(&action);
    }

    frame.set_child(Some(&box_content));
    frame
}

fn render_main_view(
    model: &UiProjection,
    selected_tab_id: &str,
    dashboard_view: &gtk::ScrolledWindow,
    config_view: &gtk::ScrolledWindow,
    settings_view: &gtk::Box,
    query: &str,
    tab_title: &gtk::Label,
    settings_list: &gtk::ListBox,
    displayed_row_ids: &Rc<RefCell<Vec<String>>>,
    detail_content: &gtk::Box,
    config_selection_state: &Rc<RefCell<ConfigSelectionState>>,
) {
    if selected_tab_id == DASHBOARD_ID {
        dashboard_view.set_visible(true);
        config_view.set_visible(false);
        settings_view.set_visible(false);
        settings_list.unselect_all();
        while let Some(child) = settings_list.first_child() {
            settings_list.remove(&child);
        }
        displayed_row_ids.borrow_mut().clear();
        render_empty_detail(detail_content);
        return;
    }

    if selected_tab_id == CONFIG_ID {
        dashboard_view.set_visible(false);
        config_view.set_visible(true);
        settings_view.set_visible(false);
        settings_list.unselect_all();
        while let Some(child) = settings_list.first_child() {
            settings_list.remove(&child);
        }
        displayed_row_ids.borrow_mut().clear();
        render_empty_detail(detail_content);
        return;
    }

    dashboard_view.set_visible(false);
    config_view.set_visible(false);
    settings_view.set_visible(true);
    render_settings_view(
        model,
        selected_tab_id,
        query,
        tab_title,
        settings_list,
        displayed_row_ids,
        detail_content,
        config_selection_state,
    );
}

fn render_settings_view(
    model: &UiProjection,
    selected_tab_id: &str,
    query: &str,
    tab_title: &gtk::Label,
    settings_list: &gtk::ListBox,
    displayed_row_ids: &Rc<RefCell<Vec<String>>>,
    detail_content: &gtk::Box,
    _config_selection_state: &Rc<RefCell<ConfigSelectionState>>,
) {
    settings_list.unselect_all();
    while let Some(child) = settings_list.first_child() {
        settings_list.remove(&child);
    }
    displayed_row_ids.borrow_mut().clear();
    render_empty_detail(detail_content);

    let view = search_projection(model, selected_tab_id, query);
    tab_title.set_label(&view.title);

    if view.results.is_empty() {
        let empty = gtk::ListBoxRow::new();
        empty.set_selectable(false);
        let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        if let Some(title) = view.empty_title {
            content.append(&body_label(&title));
        }
        if let Some(detail) = view.empty_detail {
            content.append(&small_label(&detail));
        }
        empty.set_child(Some(&content));
        settings_list.append(&empty);
        return;
    }

    for result in &view.results {
        displayed_row_ids
            .borrow_mut()
            .push(result.setting.row_id.clone());
        settings_list.append(&build_setting_row(result, view.is_searching));
    }
}

fn build_setting_row(result: &SearchResult, include_context: bool) -> gtk::ListBoxRow {
    let setting = &result.setting;
    let row = gtk::ListBoxRow::new();
    row.set_widget_name(&format!(
        "hyprland-settings-setting-row-{}",
        safe_widget_name(&setting.row_id)
    ));
    row.set_tooltip_text(Some(&setting_row_accessibility_text(setting)));
    row.set_activatable(true);
    row.set_selectable(true);

    let row_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    row_box.set_widget_name("hyprland-settings-setting-row-content");
    row_box.set_margin_top(10);
    row_box.set_margin_bottom(10);
    row_box.set_margin_start(12);
    row_box.set_margin_end(12);

    row_box.append(&body_label(&setting.label));
    if include_context {
        row_box.append(&small_label(&format!(
            "In {} / {} · {}",
            setting.tab_label,
            setting.subsection,
            search_rank_label(result.rank)
        )));
    }
    if !setting.description.is_empty() {
        row_box.append(&wrapped_small_label(&setting.description));
    }

    row_box.append(&small_label(&friendly_row_current_status(setting)));
    if let Some(status) = friendly_row_attention_status(setting) {
        row_box.append(&small_label(&status));
    }

    row.set_child(Some(&row_box));
    row
}

fn friendly_row_current_status(setting: &crate::ui::model::UiSetting) -> String {
    match setting.current_value.status {
        CurrentValueSourceStatus::Configured => {
            let value = setting
                .current_value
                .raw_value
                .as_deref()
                .map(friendly_current_value)
                .unwrap_or_else(|| "Set".to_string());
            format!("Current: {value}")
        }
        CurrentValueSourceStatus::NotConfigured => "Uses Hyprland default".to_string(),
        CurrentValueSourceStatus::DuplicateConflict => "Needs attention".to_string(),
        CurrentValueSourceStatus::ReadUnavailable => "Not available right now".to_string(),
    }
}

fn friendly_current_value(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => "On".to_string(),
        "0" | "false" | "no" | "off" => "Off".to_string(),
        "" => "Empty".to_string(),
        _ => value.to_string(),
    }
}

fn friendly_row_attention_status(setting: &crate::ui::model::UiSetting) -> Option<String> {
    if setting.current_value.status == CurrentValueSourceStatus::DuplicateConflict {
        return None;
    }

    if setting.current_value.warning.is_some() {
        return Some("Needs attention".to_string());
    }

    if high_risk_write_policy(&setting.row_id).is_some()
        || setting.row_id == "decoration.screen_shader"
    {
        return Some("Extra care needed".to_string());
    }

    None
}

fn setting_row_accessibility_text(setting: &crate::ui::model::UiSetting) -> String {
    let mut parts = vec![
        format!("Setting row: {}", setting.label),
        friendly_row_current_status(setting),
    ];

    match setting.current_value.status {
        CurrentValueSourceStatus::NotConfigured => parts.push(
            "Missing/default setting row. This setting is using Hyprland's default value."
                .to_string(),
        ),
        CurrentValueSourceStatus::DuplicateConflict => parts.push(
            "Duplicate conflict setting row. This setting appears more than once in your config."
                .to_string(),
        ),
        CurrentValueSourceStatus::ReadUnavailable => {
            parts.push("Current value unavailable setting row".to_string())
        }
        CurrentValueSourceStatus::Configured => {}
    }

    if setting.risk_class == "display_render_risk" || setting.row_id == "decoration.screen_shader" {
        parts.push("Display/render risk setting row. Extra care needed.".to_string());
    } else if high_risk_write_policy(&setting.row_id).is_some() {
        parts.push("High-risk setting row. Extra care needed.".to_string());
    } else if let Some(status) = friendly_row_attention_status(setting) {
        parts.push(status);
    } else {
        parts.push("Safe normal setting".to_string());
    }

    if let Some(warning) = &setting.current_value.warning {
        parts.push(format!("Blocked or warning detail: {warning}"));
    }

    parts.join(". ")
}

fn build_detail_panel() -> (gtk::ScrolledWindow, gtk::Box) {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-detail-pane");
    frame.set_tooltip_text(Some("Setting detail pane"));
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_widget_name("hyprland-settings-detail-pane-content");
    content.set_tooltip_text(Some("Setting detail pane content"));
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    render_empty_detail(&content);
    frame.set_child(Some(&content));

    let scroll = gtk::ScrolledWindow::builder()
        .min_content_width(420)
        .vexpand(true)
        .hexpand(true)
        .child(&frame)
        .build();
    scroll.set_widget_name("hyprland-settings-detail-pane-scroll");
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    (scroll, content)
}

fn render_empty_detail(detail_content: &gtk::Box) {
    detail_content.set_widget_name("hyprland-settings-detail-pane-empty");
    clear_box(detail_content);
    detail_content.append(&title_label("Setting details"));
    detail_content.append(&body_label("Select a setting to view metadata."));
    detail_content.append(&small_label(
        "No live value is read. This panel is read-only metadata.",
    ));
}

fn render_detail(
    model: &UiProjection,
    row_id: &str,
    detail_content: &gtk::Box,
    config_selection_state: &Rc<RefCell<ConfigSelectionState>>,
) {
    let Some(detail) = model.detail_for_row(row_id) else {
        render_empty_detail(detail_content);
        return;
    };

    detail_content.set_widget_name(&format!(
        "hyprland-settings-detail-pane-{}",
        safe_widget_name(row_id)
    ));
    detail_content.set_tooltip_text(Some(&detail_pane_accessibility_text(&detail)));
    clear_box(detail_content);
    append_detail_section(detail_content, "Setting", |section| {
        section.append(&title_label(&detail.label));
        append_detail_line(section, "Official setting", &detail.official_setting);
        append_detail_line(
            section,
            "Category",
            &format!("{} / {}", detail.tab_label, detail.subsection),
        );
        if !detail.description.is_empty() {
            append_detail_line(section, "Description", &detail.description);
        }
    });

    append_detail_section(detail_content, "Current value", |section| {
        append_current_value_summary(&detail, section);
        append_layered_value_summary(model, &detail, section);
        append_session_value_projection_summary(model, &detail, section, config_selection_state);
    });

    append_detail_section(detail_content, "Edit", |section| {
        append_user_facing_write_reason(&detail, section);
        append_source_include_insertion_target_review(model, &detail, section);
        append_pre_apply_review_scaffold(model, &detail, section);
        append_write_controls(model, &detail, section);
    });

    append_detail_section(detail_content, "Safety", |section| {
        append_safety_summary(&detail, section);
        append_screen_shader_advisory_controls(&detail, section);
    });

    append_advanced_detail_expander(&detail, detail_content);
}

fn append_current_value_summary(detail: &RowDetailProjection, section: &gtk::Box) {
    append_detail_line(
        section,
        "Status",
        match detail.current_value.status {
            CurrentValueSourceStatus::Configured => "Configured in your hyprland.conf",
            CurrentValueSourceStatus::NotConfigured => {
                "Not configured; Hyprland will use its default behavior"
            }
            CurrentValueSourceStatus::DuplicateConflict => {
                "Conflict: duplicate config entries found"
            }
            CurrentValueSourceStatus::ReadUnavailable => "Current config could not be read",
        },
    );
    append_detail_line(
        section,
        "Current value",
        detail
            .current_value
            .raw_value
            .as_deref()
            .unwrap_or("not configured"),
    );
    if let (Some(path), Some(line_number)) = (
        &detail.current_value.source_path,
        detail.current_value.line_number,
    ) {
        append_detail_line(
            section,
            "Source",
            &format!("{}:{line_number}", path.display()),
        );
    }
    if detail.current_value.status == CurrentValueSourceStatus::DuplicateConflict {
        let duplicate = body_label(
            "This setting appears more than once in your config. The app will not write this setting until the duplicate entries are resolved manually.",
        );
        duplicate.set_widget_name("hyprland-settings-blocked-duplicate-conflict");
        duplicate.set_tooltip_text(Some("Blocked reason: duplicate conflict"));
        section.append(&duplicate);
    }
    if let Some(warning) = &detail.current_value.warning {
        append_detail_line(section, "Warning", warning);
    }
}

fn detail_pane_accessibility_text(detail: &RowDetailProjection) -> String {
    let mut parts = vec![format!("Detail pane for {}", detail.label)];

    match detail.current_value.status {
        CurrentValueSourceStatus::NotConfigured => parts.push(
            "Missing/default detail. This setting is using Hyprland's default value.".to_string(),
        ),
        CurrentValueSourceStatus::DuplicateConflict => parts.push(
            "Duplicate conflict detail pane. This setting appears more than once in your config."
                .to_string(),
        ),
        CurrentValueSourceStatus::ReadUnavailable => {
            parts.push("Current value unavailable detail".to_string())
        }
        CurrentValueSourceStatus::Configured => {}
    }

    if detail.risk_class == "display_render_risk" || detail.row_id == "decoration.screen_shader" {
        parts.push("Display/render risk detail. Extra care needed.".to_string());
    } else if high_risk_write_policy(&detail.row_id).is_some() {
        parts.push("High-risk detail. Extra care needed.".to_string());
    }

    if let Some(warning) = &detail.current_value.warning {
        parts.push(format!("Blocked or warning detail: {warning}"));
    }

    parts.join(". ")
}

fn append_layered_value_summary(
    model: &UiProjection,
    detail: &RowDetailProjection,
    section: &gtk::Box,
) {
    let Some(graph) = config_graph_for_discovery(&model.config_discovery) else {
        return;
    };
    let setting_id = write_flow_config_setting(&detail.row_id)
        .unwrap_or(&detail.official_setting)
        .to_string();
    let layered = layered_values_for_setting(&graph, &setting_id);
    if !layered.controlled_in_more_than_one_place {
        return;
    }

    section.append(&body_label(
        "This setting is controlled in more than one place.",
    ));
    append_duplicate_occurrence_selector(&layered, section);
    if let Some(active) = &layered.currently_active_value {
        append_detail_line(section, "Currently active", active);
    }
    section.append(&small_label(
        "Choose where to save changes in a future version. This display is read-only.",
    ));
}

fn append_duplicate_occurrence_selector(
    layered: &crate::config_layered_values::LayeredSettingValues,
    section: &gtk::Box,
) {
    let selector = gtk::Box::new(gtk::Orientation::Vertical, 6);
    selector.set_widget_name("hyprland-settings-duplicate-occurrence-selector-disabled");
    selector.set_tooltip_text(Some(
        "Disabled duplicate occurrence selector. The app will not auto-choose a duplicate line.",
    ));

    selector.append(&body_label("Duplicate occurrences"));
    selector.append(&small_label(
        "The app will not auto-choose a duplicate line. Duplicate writes stay blocked until manual occurrence selection is reviewed.",
    ));
    selector.append(&body_label("Pre-Apply duplicate approval review"));
    selector.append(&small_label(
        "No duplicate target is confirmed for production. Production duplicate Apply remains disabled.",
    ));

    for (index, occurrence) in layered.occurrences.iter().enumerate() {
        let duplicate_occurrence = duplicate_occurrence_from_layered(occurrence);
        let gate = duplicate_production_approval_gate(Some(&duplicate_occurrence), None);
        let card = gtk::Box::new(gtk::Orientation::Vertical, 4);
        card.set_widget_name(&format!(
            "hyprland-settings-duplicate-occurrence-{}",
            index + 1
        ));
        card.set_tooltip_text(Some(
            "Read-only duplicate occurrence detail. Choosing this occurrence is planned but disabled.",
        ));
        card.append(&body_label(&format!(
            "Occurrence {} · {}",
            index + 1,
            occurrence.role_label
        )));
        card.append(&small_label(&format!(
            "File: {}",
            occurrence.file_path.display()
        )));
        card.append(&small_label(&format!("Line: {}", occurrence.line_number)));
        card.append(&small_label(&format!("Value: {}", occurrence.raw_value)));
        card.append(&small_label(&format!(
            "Source depth: {}",
            occurrence.source_depth
        )));
        card.append(&small_label(&format!("Raw line: {}", occurrence.raw_line)));
        card.append(&small_label(&format!(
            "Approval state: {}",
            duplicate_gate_status_label(gate.status)
        )));
        card.append(&small_label(&format!(
            "Precondition fingerprint: {}",
            gate.precondition
                .as_ref()
                .map(|precondition| precondition.fingerprint.as_str())
                .unwrap_or("not available")
        )));
        card.append(&small_label(&format!(
            "Block reason: {}",
            gate.block_reason
        )));
        for note in occurrence.friendly_notes() {
            card.append(&small_label(&note));
        }

        let confirm = gtk::Button::with_label("Confirm duplicate target (planned)");
        confirm.set_widget_name(&format!(
            "hyprland-settings-duplicate-production-confirm-disabled-{}",
            index + 1
        ));
        confirm.set_tooltip_text(Some(
            "Disabled future action. This does not confirm a target or unblock Apply.",
        ));
        confirm.set_sensitive(false);
        card.append(&confirm);

        let choose = gtk::Button::with_label("Choose this occurrence (planned)");
        choose.set_widget_name(&format!(
            "hyprland-settings-duplicate-occurrence-choice-disabled-{}",
            index + 1
        ));
        choose.set_tooltip_text(Some(
            "Disabled future action. This does not write config or unblock Apply.",
        ));
        choose.set_sensitive(false);
        card.append(&choose);
        selector.append(&card);
    }

    section.append(&selector);
}

fn duplicate_occurrence_from_layered(
    occurrence: &crate::config_layered_values::LayeredValueOccurrence,
) -> DuplicateOccurrence {
    DuplicateOccurrence {
        setting_id: occurrence.setting_id.clone(),
        path: occurrence.file_path.clone(),
        line_number: occurrence.line_number,
        raw_line: occurrence.raw_line.clone(),
        raw_value: occurrence.raw_value.clone(),
        source_depth: occurrence.source_depth,
    }
}

fn duplicate_gate_status_label(status: DuplicateProductionGateStatus) -> &'static str {
    match status {
        DuplicateProductionGateStatus::MissingConfirmation => "missing confirmation",
        DuplicateProductionGateStatus::PendingConfirmation => "pending confirmation",
        DuplicateProductionGateStatus::ConfirmedButProductionDisabled => {
            "confirmed but production disabled"
        }
        DuplicateProductionGateStatus::MissingCopiedProof => "missing copied proof",
        DuplicateProductionGateStatus::CopiedProofMismatch => "copied proof mismatch",
        DuplicateProductionGateStatus::ReadyButDefaultDisabled => "ready but default disabled",
        DuplicateProductionGateStatus::Rejected => "rejected",
        DuplicateProductionGateStatus::Expired => "expired",
        DuplicateProductionGateStatus::FingerprintMismatch => "fingerprint mismatch",
    }
}

fn append_session_value_projection_summary(
    _model: &UiProjection,
    detail: &RowDetailProjection,
    section: &gtk::Box,
    config_selection_state: &Rc<RefCell<ConfigSelectionState>>,
) {
    let preview = config_selection_state.borrow().preview();
    if !preview.session_only {
        return;
    }
    let Some(session_path) = preview.session_read_only_config else {
        return;
    };
    let graph = inspect_config_graph_with_options(
        &session_path,
        ConfigGraphOptions {
            source_follow_policy: source_follow_policy_for_choice(preview.source_follow_choice),
            ..ConfigGraphOptions::from_env()
        },
    );
    let setting_id = write_flow_config_setting(&detail.row_id)
        .unwrap_or(&detail.official_setting)
        .to_string();
    let layered = layered_values_for_setting(&graph, &setting_id);
    let projection = compare_active_and_session_values(
        detail.row_id.clone(),
        setting_id,
        &detail.current_value,
        &layered,
    );

    for line in projection.user_facing_lines() {
        section.append(&small_label(&line));
    }
    if let (Some(path), Some(line)) = (
        projection.session_source_path.as_ref(),
        projection.session_source_line,
    ) {
        append_detail_line(
            section,
            "Session source",
            &format!("{}:{line}", path.display()),
        );
    }
}

fn source_follow_policy_for_choice(choice: SourceFollowChoice) -> SourceFollowPolicy {
    match choice {
        SourceFollowChoice::ReviewAllConnectedFiles => SourceFollowPolicy::ReviewAll,
        SourceFollowChoice::OnlySelectedFile | SourceFollowChoice::Cancel => {
            SourceFollowPolicy::OnlyRoot
        }
    }
}

fn append_pre_apply_review_scaffold(
    model: &UiProjection,
    detail: &RowDetailProjection,
    section: &gtk::Box,
) {
    let Some(graph) = config_graph_for_discovery(&model.config_discovery) else {
        return;
    };
    let setting_id = write_flow_config_setting(&detail.row_id)
        .unwrap_or(&detail.official_setting)
        .to_string();
    let layered = layered_values_for_setting(&graph, &setting_id);
    if !layered.controlled_in_more_than_one_place {
        return;
    }

    let candidates = write_target_candidates_for_layered_setting(&layered, &graph.files);
    let recommendation = recommend_write_targets(&candidates);

    let frame = gtk::Frame::new(None);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label("Pre-apply review"));
    content.append(&small_label(
        "Before this setting can be changed, choose where the app should save it.",
    ));
    content.append(&small_label(
        "The app will back up the exact file before saving changes.",
    ));
    content.append(&small_label(
        "Generated or script-managed files may require advanced confirmation.",
    ));
    content.append(&small_label(
        "Safe batch writing is guarded by backup, verification, and recovery checks.",
    ));
    content.append(&small_label(
        "Apply writes only when every selected setting has a safe target.",
    ));

    content.append(&body_label("Save location"));
    for line in recommendation.user_facing_lines() {
        content.append(&small_label(&line));
    }

    for candidate in &candidates {
        let button = gtk::CheckButton::with_label(&candidate.label);
        button.set_sensitive(false);
        content.append(&button);
    }

    if let Some(candidate) = recommendation.recommended_target.clone() {
        let proposed_value = detail
            .edit
            .proposed_value
            .clone()
            .unwrap_or_else(|| "future value".to_string());
        let backup_plan = build_exact_backup_plan(&candidate);
        let advanced_confirmation = advanced_confirmation_for_candidate(&candidate);
        let verification_plan =
            planned_reread_verification(&candidate, &setting_id, proposed_value.clone());
        let guarded_review = build_guarded_write_target_review(
            detail.row_id.clone(),
            setting_id,
            proposed_value,
            detail.current_value.raw_value.clone(),
            None,
            &recommendation,
            Some(candidate),
            high_risk_write_policy(&detail.row_id).is_none(),
            FixtureProofStatus::NotRun,
        );
        let session_projection =
            inactive_session_value_projection(detail, &guarded_review.official_setting_id);
        let walkthrough = build_write_review_walkthrough(
            Some(&session_projection),
            Some(&layered),
            Some(&recommendation),
            Some(&guarded_review),
            Some(&backup_plan),
            Some(&advanced_confirmation),
            Some(&verification_plan),
        );

        content.append(&body_label("Write review"));
        for line in guarded_review.user_facing_lines() {
            content.append(&small_label(&line));
        }
        content.append(&body_label("Backup"));
        for line in backup_plan.user_facing_lines() {
            content.append(&small_label(&line));
        }
        content.append(&body_label("Verification"));
        for line in verification_plan.user_facing_lines() {
            content.append(&small_label(&line));
        }
        content.append(&body_label("Safety"));
        for line in advanced_confirmation.user_facing_lines() {
            content.append(&small_label(&line));
        }
        content.append(&small_label(if PRODUCTION_WRITE_TARGET_REVIEW_ENABLED {
            "Safe batch review is active."
        } else {
            "Real writing is not active yet."
        }));

        content.append(&body_label("Safe batch write review"));
        content.append(&small_label(
            "Shown when a setting is controlled in more than one place.",
        ));
        content.append(&small_label(
            "This review shows what the app checks before writing.",
        ));
        for line in walkthrough.user_facing_lines() {
            content.append(&small_label(&line));
        }
        let decision_button = gtk::Button::with_label("Target decisions are preview-only");
        decision_button.set_sensitive(false);
        content.append(&decision_button);

        let readiness = current_production_write_enablement_readiness();
        content.append(&body_label("Production write enablement"));
        for line in readiness.user_facing_lines() {
            content.append(&small_label(&line));
        }
        let enablement_button = gtk::Button::with_label("Production enablement is disabled");
        enablement_button.set_sensitive(false);
        content.append(&enablement_button);

        content.append(&body_label("Safe batch write"));
        for line in safe_batch_write_user_facing_lines() {
            content.append(&small_label(&line));
        }

        let pilot_readiness = current_one_target_pilot_readiness_mapping();
        content.append(&body_label("Production backup and verification"));
        for line in pilot_readiness.user_facing_lines() {
            content.append(&small_label(&line));
        }
        for line in disabled_advanced_confirmation_ui_lines() {
            content.append(&small_label(&line));
        }
        for line in disabled_high_risk_approval_ui_lines() {
            content.append(&small_label(&line));
        }
        for line in disabled_pre_enable_audit_ui_lines() {
            content.append(&small_label(&line));
        }
        for line in disabled_manual_smoke_review_ui_lines() {
            content.append(&small_label(&line));
        }
        for line in disabled_live_visual_smoke_review_ui_lines() {
            content.append(&small_label(&line));
        }

        let review_button = gtk::Button::with_label("Review save location");
        review_button.set_sensitive(false);
        content.append(&review_button);
    }

    frame.set_child(Some(&content));
    section.append(&frame);
}

fn append_source_include_insertion_target_review(
    model: &UiProjection,
    detail: &RowDetailProjection,
    section: &gtk::Box,
) {
    if detail.current_value.status != CurrentValueSourceStatus::NotConfigured {
        return;
    }
    let Some(graph) = config_graph_for_discovery(&model.config_discovery) else {
        return;
    };
    if graph.files.len() <= 1
        && graph.source_references.is_empty()
        && !graph.has_generated_hints
        && !graph.has_script_managed_hints
        && !graph.has_profile_hints
        && !graph.has_mode_hints
        && !graph.files.iter().any(|file| file.is_symlink)
    {
        return;
    }

    let candidate_targets: Vec<_> = graph.files.iter().map(|file| file.path.clone()).collect();
    let target_candidates: Vec<_> = graph
        .files
        .iter()
        .map(|file| SourceIncludeTargetCandidate {
            path: file.path.clone(),
            source_depth: file.source_depth,
            generated_or_script_managed: connected_file_has_hint(
                file,
                ConfigManagementHintKind::GeneratedFile,
            ) || connected_file_has_script_hint(file),
            symlink_or_profile_managed: file.is_symlink
                || connected_file_has_hint(file, ConfigManagementHintKind::CurrentProfile)
                || connected_file_has_hint(file, ConfigManagementHintKind::ModeProfile)
                || connected_file_has_hint(file, ConfigManagementHintKind::SymlinkManaged),
        })
        .collect::<Vec<_>>();
    let managed_or_ambiguous = graph.has_generated_hints
        || graph.has_script_managed_hints
        || graph.has_profile_hints
        || graph.has_mode_hints
        || graph.files.iter().any(|file| file.is_symlink)
        || !graph.unreadable_files.is_empty()
        || !graph.cycles.is_empty()
        || !graph.unsupported_sources.is_empty();
    let review = source_include_insertion_review(
        graph.root_path.clone(),
        candidate_targets,
        None,
        managed_or_ambiguous,
    );

    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-source-include-insertion-review-disabled");
    frame.set_tooltip_text(Some(
        "Disabled source/include insertion target-selection review. This does not write connected files.",
    ));
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label("Source/include insertion target review"));
    content.append(&small_label(
        "This setting uses Hyprland's default value, but this config uses connected files.",
    ));
    content.append(&small_label(
        "Source/include insertion is not active yet. The app will not pick a connected file automatically.",
    ));
    append_detail_line(&content, "Root config", &friendly_path(&review.root_path));
    append_detail_line(
        &content,
        "Readiness",
        source_include_readiness_label(review.readiness),
    );

    for line in &review.review_lines {
        content.append(&small_label(line));
    }

    if let Some(selected_target) = &review.selected_target {
        append_detail_line(&content, "Selected target", &friendly_path(selected_target));
    } else {
        append_detail_line(&content, "Selected target", "none");
    }

    let selected_target_proof = source_include_target_selection_fixture_proof(
        graph.root_path.clone(),
        target_candidates,
        review.selected_target.clone(),
        managed_or_ambiguous,
    );
    let preview_target = review
        .selected_target
        .clone()
        .or_else(|| review.candidate_targets.first().cloned())
        .unwrap_or_else(|| graph.root_path.clone());
    let proposed_value = detail.edit.proposed_value.clone().unwrap_or_else(|| {
        detail
            .current_value
            .raw_value
            .clone()
            .unwrap_or_else(|| "<proposed value required>".to_string())
    });
    let insertion_plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: detail.row_id.clone(),
        proposed_value: proposed_value.clone(),
        target_path: preview_target,
        backup_stamp: "ui-preview".to_string(),
    });
    let dry_run_preview =
        source_include_selected_target_dry_run_plan(&selected_target_proof, &insertion_plan);
    let preview = gtk::Box::new(gtk::Orientation::Vertical, 4);
    preview.set_widget_name(
        "hyprland-settings-source-include-selected-target-dry-run-preview-disabled",
    );
    preview.set_tooltip_text(Some(
        "Disabled selected-target insertion dry-run preview. This does not write connected files.",
    ));
    preview.append(&body_label("Selected-target insertion dry-run preview"));
    append_detail_line(&preview, "Root path", &friendly_path(&graph.root_path));
    append_detail_line(
        &preview,
        "Selected target path",
        dry_run_preview
            .selected_target
            .as_ref()
            .map(|path| friendly_path(path))
            .as_deref()
            .unwrap_or("none"),
    );
    append_detail_line(
        &preview,
        "Source depth",
        &dry_run_preview
            .source_depth
            .map(|depth| depth.to_string())
            .unwrap_or_else(|| "not selected".to_string()),
    );
    append_detail_line(&preview, "Proposed value", &proposed_value);
    append_detail_line(
        &preview,
        "Planned inserted line",
        dry_run_preview
            .insertion_line
            .as_deref()
            .unwrap_or(insertion_plan.insertion_line.as_str()),
    );
    append_detail_line(
        &preview,
        "Dry-run status",
        source_include_dry_run_status_label(dry_run_preview.status),
    );
    if let Some(line) = &dry_run_preview.dry_run_preview {
        preview.append(&small_label(line));
    }
    if !dry_run_preview.blocked_reasons.is_empty() {
        preview.append(&small_label(&format!(
            "Blocked: {}",
            dry_run_preview.blocked_reasons.join("; ")
        )));
    }
    preview.append(&small_label(
        "Production source/include insertion remains disabled. This preview does not write files.",
    ));
    let run_selected = gtk::Button::with_label("Run selected-target insertion (planned)");
    run_selected.set_widget_name("hyprland-settings-source-include-selected-target-run-disabled");
    run_selected.set_tooltip_text(Some(
        "Disabled future action. This does not run insertion or unblock Apply.",
    ));
    run_selected.set_sensitive(false);
    preview.append(&run_selected);
    content.append(&preview);

    content.append(&body_label("Candidate target files"));
    for (index, target) in review.candidate_targets.iter().enumerate() {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 4);
        card.set_widget_name(&format!(
            "hyprland-settings-source-include-target-candidate-disabled-{}",
            index + 1
        ));
        card.set_tooltip_text(Some(
            "Read-only source/include insertion target candidate. Choosing it is planned but disabled.",
        ));
        card.append(&small_label(&format!(
            "Candidate {}: {}",
            index + 1,
            friendly_path(target)
        )));
        let choose = gtk::Button::with_label("Use this target (planned)");
        choose.set_widget_name(&format!(
            "hyprland-settings-source-include-target-choice-disabled-{}",
            index + 1
        ));
        choose.set_tooltip_text(Some(
            "Disabled future action. This does not insert a config line or unblock Apply.",
        ));
        choose.set_sensitive(false);
        card.append(&choose);
        content.append(&card);
    }

    let choose_target = gtk::Button::with_label("Choose target file (planned)");
    choose_target.set_widget_name("hyprland-settings-source-include-target-selection-disabled");
    choose_target.set_tooltip_text(Some(
        "Disabled source/include target selection. Production insertion remains limited to single-root configs.",
    ));
    choose_target.set_sensitive(false);
    content.append(&choose_target);
    frame.set_child(Some(&content));
    section.append(&frame);
}

fn source_include_readiness_label(readiness: SourceIncludeInsertionReadiness) -> &'static str {
    match readiness {
        SourceIncludeInsertionReadiness::SingleRootEligible => "single-root eligible",
        SourceIncludeInsertionReadiness::SourceIncludeTargetSelectionRequired => {
            "target selection required"
        }
        SourceIncludeInsertionReadiness::ManagedTargetBlocked => "managed target blocked",
        SourceIncludeInsertionReadiness::DuplicateOrAmbiguousBlocked => {
            "duplicate or ambiguous target blocked"
        }
    }
}

fn source_include_dry_run_status_label(
    status: SourceIncludeSelectedTargetDryRunStatus,
) -> &'static str {
    match status {
        SourceIncludeSelectedTargetDryRunStatus::Planned => "planned for fixture dry-run",
        SourceIncludeSelectedTargetDryRunStatus::SelectionBlocked => "selection blocked",
        SourceIncludeSelectedTargetDryRunStatus::TargetMismatch => "target mismatch",
        SourceIncludeSelectedTargetDryRunStatus::InsertionPlanBlocked => "insertion plan blocked",
        SourceIncludeSelectedTargetDryRunStatus::NonFixtureTargetRefused => {
            "non-fixture target refused"
        }
    }
}

fn inactive_session_value_projection(
    detail: &RowDetailProjection,
    official_setting_id: &str,
) -> SessionValueProjection {
    SessionValueProjection {
        row_id: detail.row_id.clone(),
        official_setting_id: official_setting_id.to_string(),
        active_value: detail.current_value.raw_value.clone(),
        session_preview_value: None,
        comparison_status: if detail.current_value.raw_value.is_some() {
            SessionValueComparisonStatus::MissingInSessionPreview
        } else {
            SessionValueComparisonStatus::Unknown
        },
        active_source_path: detail.current_value.source_path.clone(),
        active_source_line: detail.current_value.line_number,
        session_source_path: None,
        session_source_line: None,
        read_only: true,
        affects_writes: false,
    }
}

fn append_user_facing_write_reason(detail: &RowDetailProjection, section: &gtk::Box) {
    append_detail_line(
        section,
        "Editable in app",
        if detail.edit.editable { "yes" } else { "no" },
    );
    if let Some(proposed_value) = &detail.edit.proposed_value {
        append_detail_line(section, "Proposed value", proposed_value);
    }
    section.append(&body_label(&apply_state_message(detail)));
}

fn apply_state_message(detail: &RowDetailProjection) -> String {
    if !detail.edit.editable {
        return format!(
            "Apply is blocked because this setting is not currently editable in the app: {}.",
            detail
                .edit
                .disabled_reason
                .as_deref()
                .unwrap_or("no edit path is available")
        );
    }

    let Some(pending) = &detail.edit.pending else {
        return "Apply is blocked because no pending value could be prepared.".to_string();
    };

    if pending.can_review {
        if high_risk_write_policy(&detail.row_id).is_some() {
            return "Apply can proceed only through the high-risk gated review path with recovery and confirmation proof."
                .to_string();
        }
        return "Apply is available after backup, config write, and reread verification."
            .to_string();
    }

    if let Some(reason) = pending
        .review_summary
        .iter()
        .find_map(|line| line.strip_prefix("blocked: "))
    {
        return format!("Apply is blocked because {reason}.");
    }

    if let Some(reason) = pending.validation_label.strip_prefix("invalid: ") {
        return format!("Apply is blocked because the proposed value is invalid: {reason}.");
    }

    if let Some(reason) = pending.validation_label.strip_prefix("not allowed: ") {
        return format!("Apply is blocked because {reason}.");
    }

    if high_risk_write_policy(&detail.row_id).is_some() {
        return "Apply is blocked until the high-risk gate has complete recovery, rollback, confirmation, and approval proof."
            .to_string();
    }

    "Apply is blocked until the current config is readable, conflict-free, and the proposed value passes validation."
        .to_string()
}

fn append_safety_summary(detail: &RowDetailProjection, section: &gtk::Box) {
    append_detail_line(section, "Risk", &risk_class_label(&detail.risk_class));
    append_detail_line(
        section,
        "Write path",
        if high_risk_write_policy(&detail.row_id).is_some() {
            "Gated high-risk config write"
        } else if detail.edit.editable {
            "Reviewed config write"
        } else {
            "Not currently writable in the app"
        },
    );

    if let Some(policy) = high_risk_write_policy(&detail.row_id) {
        section.append(&body_label(&policy.review_warning));
        append_detail_line(section, "Gate", policy.approval_gate);
        append_detail_line(section, "Recovery", policy.watchdog_requirement);
    }

    if detail.row_id == "cursor.default_monitor" {
        section.append(&body_label(
            "This is not a freeform string write. The proposed monitor name must be proven by the runtime monitor-name oracle before the high-risk gate can accept it.",
        ));
    }

    if detail.row_id == "debug.manual_crash" {
        section.append(&body_label(
            "This setting is crash/debug sensitive. It must not look or behave like a casual toggle; the production gate requires recovery and confirmation proof before any apply path can proceed.",
        ));
    }

    if let Some(advisory) = &detail.screen_shader_advisory {
        section.append(&body_label(&advisory.production_gate_disclaimer));
        section.append(&small_label(&advisory.runtime_safety_disclaimer));
    }
}

fn risk_class_label(risk_class: &str) -> String {
    match risk_class {
        "safe" => "Standard config setting".to_string(),
        "display_render_risk" => "Display/render high-risk setting".to_string(),
        "cursor_input_risk" => "Cursor/input high-risk setting".to_string(),
        "debug_crash_risk" => "Debug/crash high-risk setting".to_string(),
        other => other.replace('_', " "),
    }
}

fn append_advanced_detail_expander(detail: &RowDetailProjection, detail_content: &gtk::Box) {
    let expander = gtk::Expander::new(Some("Source / advanced metadata"));
    expander.set_expanded(false);

    let advanced = gtk::Box::new(gtk::Orientation::Vertical, 6);
    advanced.set_margin_top(10);
    advanced.set_margin_bottom(10);
    advanced.set_margin_start(10);
    advanced.set_margin_end(10);

    append_detail_line(&advanced, "Row ID", &detail.row_id);
    append_detail_line(&advanced, "Read support raw label", &detail.read_support);
    if let Some(status) = &detail.non_read_status {
        append_detail_line(&advanced, "Non-read status", status);
    }
    append_detail_line(&advanced, "Write support raw label", &detail.write_support);
    append_detail_line(&advanced, "Preview status", &detail.preview_status);
    append_detail_line(&advanced, "Report-only status", &detail.report_only_status);
    append_detail_line(
        &advanced,
        "Write candidate status",
        &detail.write_candidate_status,
    );
    append_detail_line(
        &advanced,
        "Default metadata",
        &detail.default_config_presence,
    );
    append_detail_line(&advanced, "Comparison", &detail.comparison.badge);
    append_detail_line(&advanced, "Comparison detail", &detail.comparison.detail);
    if let Some(raw_line) = &detail.current_value.raw_line {
        append_detail_line(&advanced, "Source line", raw_line);
    }
    if !detail.current_value.duplicate_lines.is_empty() {
        append_detail_line(
            &advanced,
            "Duplicate line numbers",
            &format!("{:?}", detail.current_value.duplicate_lines),
        );
    }
    if let Some(target_mode) = &detail.write_candidate_target_mode {
        append_detail_line(&advanced, "Target mode", target_mode);
    }
    if let Some(executable) = detail.write_candidate_executable {
        append_detail_line(&advanced, "Executable", &executable.to_string());
    }
    if let Some(command_generation_allowed) = detail.write_candidate_command_generation_allowed {
        append_detail_line(
            &advanced,
            "Command generation",
            &command_generation_allowed.to_string(),
        );
    }
    if let Some(pending) = &detail.edit.pending {
        append_detail_line(
            &advanced,
            "Pending change validation",
            &pending.validation_label,
        );
        append_detail_line(
            &advanced,
            "Pending review available",
            if pending.can_review { "yes" } else { "no" },
        );
        for line in &pending.review_summary {
            advanced.append(&small_label(line));
        }
    }
    for note in &detail.safety_notes {
        advanced.append(&small_label(note));
    }

    expander.set_child(Some(&advanced));
    detail_content.append(&expander);
}

fn append_screen_shader_advisory_controls(detail: &RowDetailProjection, detail_content: &gtk::Box) {
    let (Some(advisory), Some(widget)) = (
        detail.screen_shader_advisory.as_ref(),
        detail.screen_shader_advisory_widget.as_ref(),
    ) else {
        return;
    };

    let frame = gtk::Frame::new(Some("Advanced advisory shader check"));
    let controls = gtk::Box::new(gtk::Orientation::Vertical, 6);
    controls.set_margin_top(10);
    controls.set_margin_bottom(10);
    controls.set_margin_start(10);
    controls.set_margin_end(10);

    controls.append(&body_label("Optional standalone advisory check"));
    controls.append(&small_label(&advisory.consent_message));
    controls.append(&small_label(&advisory.temp_copy_message));
    controls.append(&small_label(&advisory.original_path_message));
    controls.append(&small_label(&advisory.runtime_safety_disclaimer));
    controls.append(&small_label(&advisory.production_gate_disclaimer));

    let button = gtk::Button::with_label(&widget.button_label);
    button.set_tooltip_text(Some(
        "This visible control uses the advisory action model. Direct GTK file chooser execution remains deferred.",
    ));
    controls.append(&button);

    let initial = initial_screen_shader_advisory_ui_action(&detail.row_id)
        .expect("screen shader widget should have initial advisory action state");
    let result_label = small_label(&format!("{}: {}", initial.state_label(), initial.message));
    controls.append(&result_label);

    let row_id = detail.row_id.clone();
    button.connect_clicked(move |_| {
        let render = run_screen_shader_advisory_ui_action(ScreenShaderAdvisoryUiActionRequest {
            row_id: row_id.clone(),
            explicit_user_trigger: true,
            helper_request: None,
        });
        result_label.set_label(&format!("{}: {}", render.state_label(), render.message));
    });

    frame.set_child(Some(&controls));
    detail_content.append(&frame);
}

fn append_detail_section(parent: &gtk::Box, title: &str, build: impl FnOnce(&gtk::Box)) {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&format!(
        "hyprland-settings-detail-section-{}",
        safe_widget_name(title)
    ));
    frame.set_tooltip_text(Some(&format!("Detail section: {title}")));
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(10);
    content.set_margin_end(10);
    content.append(&title_label(title));
    build(&content);
    frame.set_child(Some(&content));
    parent.append(&frame);
}

fn append_write_controls(
    model: &UiProjection,
    detail: &RowDetailProjection,
    detail_content: &gtk::Box,
) {
    if !detail.edit.editable {
        return;
    }

    let controls = gtk::Box::new(gtk::Orientation::Vertical, 6);
    controls.set_widget_name("hyprland-settings-review-apply-area");
    controls.set_tooltip_text(Some(
        "Review and Apply area. GTK automation must not click Apply.",
    ));
    controls.set_margin_top(8);

    controls.append(&body_label("Write review"));
    controls.append(&small_label(
        "Only allowlisted scalar settings can be applied. A backup is created first, the config is rewritten, and the value is reread for verification. Hyprland reload is not run.",
    ));

    let value_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    value_row.append(&body_label("Proposed value"));
    let value_kind = write_flow_value_kind(&detail.row_id);
    let value_choice = if value_kind == Some(ScalarWriteValueKind::Boolean) {
        let value_choice = gtk::ComboBoxText::new();
        value_choice.append(Some("true"), "true");
        value_choice.append(Some("false"), "false");
        if detail.edit.proposed_value.as_deref() == Some("false") {
            value_choice.set_active(Some(1));
        } else {
            value_choice.set_active(Some(0));
        }
        value_row.append(&value_choice);
        Some(value_choice)
    } else if value_kind == Some(ScalarWriteValueKind::FiniteChoice) {
        let value_choice = gtk::ComboBoxText::new();
        for choice in &detail.edit.choices {
            value_choice.append(Some(&choice.raw_value), &choice.label);
        }
        if let Some(proposed) = detail.edit.proposed_value.as_deref() {
            value_choice.set_active_id(Some(proposed));
        }
        if value_choice.active_id().is_none() && !detail.edit.choices.is_empty() {
            value_choice.set_active(Some(0));
        }
        value_row.append(&value_choice);
        Some(value_choice)
    } else {
        None
    };
    let value_entry = if value_choice.is_none() {
        let value_entry = gtk::Entry::new();
        value_entry.set_text(detail.edit.proposed_value.as_deref().unwrap_or_default());
        value_row.append(&value_entry);
        Some(value_entry)
    } else {
        None
    };
    controls.append(&value_row);

    let apply_button = gtk::Button::with_label("Apply reviewed change");
    apply_button.set_widget_name("hyprland-settings-apply-reviewed-change-button");
    apply_button.set_tooltip_text(Some(
        "Apply reviewed change. GTK automation must not activate this control.",
    ));
    let can_review = detail
        .edit
        .pending
        .as_ref()
        .map(|pending| pending.can_review)
        .unwrap_or(false);
    apply_button.set_sensitive(can_review);
    controls.append(&apply_button);

    let result_label = small_label(&apply_state_message(detail));
    controls.append(&result_label);

    let known_setting_ids = model.known_setting_ids.clone();
    let config_discovery = model.config_discovery.clone();
    let current_config = model.current_config.clone();
    let setting_id = detail.row_id.clone();
    apply_button.connect_clicked(move |button| {
        let proposed_value = value_choice
            .as_ref()
            .and_then(|choice| choice.active_id())
            .map(|value| value.to_string())
            .or_else(|| value_entry.as_ref().map(|entry| entry.text().to_string()))
            .unwrap_or_default();
        match apply_setting_change(
            known_setting_ids.clone(),
            &config_discovery,
            &current_config,
            &setting_id,
            &proposed_value,
        ) {
            Ok(outcome) => {
                button.set_sensitive(false);
                result_label.set_label(&format!(
                    "Applied and verified {} = {}. Backup: {}. Rollback source: {}. {}",
                    outcome.setting_id,
                    outcome
                        .verified_value
                        .unwrap_or_else(|| "unknown".to_string()),
                    outcome.backup_path.display(),
                    outcome.rollback_source_path.display(),
                    outcome.reload_note
                ));
            }
            Err(error) => {
                result_label.set_label(&format!(
                    "Apply blocked: {} ({})",
                    error.reason,
                    error.failures.join(", ")
                ));
            }
        }
    });

    detail_content.append(&controls);
}

fn append_detail_line(parent: &gtk::Box, label: &str, value: &str) {
    let line = body_label(&format!("{}: {}", label, value));
    line.set_widget_name(&format!(
        "hyprland-settings-detail-line-{}",
        safe_widget_name(label)
    ));
    if label.eq_ignore_ascii_case("status")
        || label.eq_ignore_ascii_case("warning")
        || label.eq_ignore_ascii_case("write target")
    {
        line.set_tooltip_text(Some(&format!("Blocked or status detail: {value}")));
    }
    parent.append(&line);
}

fn safe_widget_name(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
        } else if !output.ends_with('-') {
            output.push('-');
        }
    }
    output.trim_matches('-').to_string()
}

fn clear_box(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn search_rank_label(rank: Option<SearchRank>) -> &'static str {
    match rank {
        Some(SearchRank::ExactKey) => "exact key match",
        Some(SearchRank::PrefixKey) => "key prefix match",
        Some(SearchRank::Label) => "label match",
        Some(SearchRank::Context) => "tab/subsection match",
        Some(SearchRank::Description) => "description match",
        Some(SearchRank::Metadata) => "metadata match",
        None => "selected tab",
    }
}

fn title_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.add_css_class("title-3");
    label
}

fn body_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_wrap(true);
    label
}

fn small_label(text: &str) -> gtk::Label {
    let label = body_label(text);
    label.add_css_class("dim-label");
    label
}

fn wrapped_small_label(text: &str) -> gtk::Label {
    let label = small_label(text);
    label.set_wrap(true);
    label
}
