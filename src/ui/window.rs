use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk4 as gtk;

use crate::config_discovery::ConfigDiscovery;
use crate::current_config::{
    CurrentConfigLoadStatus, CurrentConfigSnapshot, CurrentValueSourceStatus,
};
use crate::export::ExportBundle;
use crate::search::{search_projection, SearchRank, SearchResult};
use crate::ui::model::{
    initial_screen_shader_advisory_ui_action, run_screen_shader_advisory_ui_action,
    RowDetailProjection, ScreenShaderAdvisoryUiActionRequest, UiProjection,
};
use crate::validation::ValidationSummary;
use crate::write_classification::{high_risk_write_policy, ScalarWriteValueKind};
use crate::write_flow::{apply_setting_change, write_flow_value_kind};

const DASHBOARD_ID: &str = "dashboard";

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

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let title = adw::WindowTitle::new("Hyprland Settings", "Hyprland config metadata and values");
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&title));
    root.append(&header);

    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    body.set_vexpand(true);
    body.set_hexpand(true);
    root.append(&body);

    let sidebar_items = Rc::new(sidebar_items(&model));
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
    content.append(&dashboard_view);

    let settings_view = gtk::Box::new(gtk::Orientation::Vertical, 12);
    settings_view.set_hexpand(true);
    settings_view.set_vexpand(true);
    content.append(&settings_view);

    let search_entry = gtk::SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search settings"));
    settings_view.append(&search_entry);

    let tab_title = title_label("");
    settings_view.append(&tab_title);

    let settings_list = gtk::ListBox::new();
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

    render_dashboard_view(
        &model,
        &selected_tab_id.borrow(),
        &dashboard_view,
        &settings_view,
        &current_query.borrow(),
        &tab_title,
        &settings_list,
        &displayed_row_ids,
        &detail_content,
    );

    {
        let model = Rc::clone(&model);
        let selected_tab_id = Rc::clone(&selected_tab_id);
        let current_query = Rc::clone(&current_query);
        let sidebar_items = Rc::clone(&sidebar_items);
        let dashboard_view = dashboard_view.clone();
        let settings_view = settings_view.clone();
        let tab_title = tab_title.clone();
        let settings_list = settings_list.clone();
        let displayed_row_ids = Rc::clone(&displayed_row_ids);
        let detail_content = detail_content.clone();
        sidebar.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                if let Some(item) = sidebar_items.get(row.index() as usize) {
                    *selected_tab_id.borrow_mut() = item.id.clone();
                    render_dashboard_view(
                        &model,
                        &selected_tab_id.borrow(),
                        &dashboard_view,
                        &settings_view,
                        &current_query.borrow(),
                        &tab_title,
                        &settings_list,
                        &displayed_row_ids,
                        &detail_content,
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
        let settings_view = settings_view.clone();
        let tab_title = tab_title.clone();
        let settings_list = settings_list.clone();
        let displayed_row_ids = Rc::clone(&displayed_row_ids);
        let detail_content = detail_content.clone();
        search_entry.connect_search_changed(move |entry| {
            *current_query.borrow_mut() = entry.text().to_string();
            render_dashboard_view(
                &model,
                &selected_tab_id.borrow(),
                &dashboard_view,
                &settings_view,
                &current_query.borrow(),
                &tab_title,
                &settings_list,
                &displayed_row_ids,
                &detail_content,
            );
        });
    }

    {
        let model = Rc::clone(&model);
        let displayed_row_ids = Rc::clone(&displayed_row_ids);
        let detail_content = detail_content.clone();
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
                render_detail(&model, &row_id, &detail_content);
            } else {
                render_empty_detail(&detail_content);
            }
        });
    }

    if let Some(row) = sidebar.row_at_index(0) {
        sidebar.select_row(Some(&row));
    }

    window.set_content(Some(&root));
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
    let mut items = vec![SidebarItem {
        id: DASHBOARD_ID.to_string(),
        label: "Dashboard".to_string(),
        target_tab_id: None,
    }];
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
    sidebar.set_selection_mode(gtk::SelectionMode::Single);

    for item in items {
        let row = gtk::ListBoxRow::new();
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

fn dashboard_cards() -> [DashboardCard; 6] {
    [
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
    let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
    card.set_margin_top(12);
    card.set_margin_bottom(12);
    card.set_margin_start(12);
    card.set_margin_end(12);
    card.append(&title_label(title));
    card.append(&body_label(description));

    if let Some(index) = sidebar_items
        .iter()
        .position(|item| item.target_tab_id.as_deref() == Some(target_tab_id))
    {
        let button = gtk::Button::with_label("Open");
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

fn render_dashboard_view(
    model: &UiProjection,
    selected_tab_id: &str,
    dashboard_view: &gtk::ScrolledWindow,
    settings_view: &gtk::Box,
    query: &str,
    tab_title: &gtk::Label,
    settings_list: &gtk::ListBox,
    displayed_row_ids: &Rc<RefCell<Vec<String>>>,
    detail_content: &gtk::Box,
) {
    if selected_tab_id == DASHBOARD_ID {
        dashboard_view.set_visible(true);
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
    settings_view.set_visible(true);
    render_settings_view(
        model,
        selected_tab_id,
        query,
        tab_title,
        settings_list,
        displayed_row_ids,
        detail_content,
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
    row.set_activatable(false);
    row.set_selectable(true);

    let row_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
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

fn build_detail_panel() -> (gtk::ScrolledWindow, gtk::Box) {
    let frame = gtk::Frame::new(None);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
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
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    (scroll, content)
}

fn render_empty_detail(detail_content: &gtk::Box) {
    clear_box(detail_content);
    detail_content.append(&title_label("Setting details"));
    detail_content.append(&body_label("Select a setting to view metadata."));
    detail_content.append(&small_label(
        "No live value is read. This panel is read-only metadata.",
    ));
}

fn render_detail(model: &UiProjection, row_id: &str, detail_content: &gtk::Box) {
    let Some(detail) = model.detail_for_row(row_id) else {
        render_empty_detail(detail_content);
        return;
    };

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
    });

    append_detail_section(detail_content, "Edit", |section| {
        append_user_facing_write_reason(&detail, section);
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
        section.append(&body_label(
            "This setting appears more than once in your config. The app will not write this setting until the duplicate entries are resolved manually.",
        ));
    }
    if let Some(warning) = &detail.current_value.warning {
        append_detail_line(section, "Warning", warning);
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
    parent.append(&body_label(&format!("{}: {}", label, value)));
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
