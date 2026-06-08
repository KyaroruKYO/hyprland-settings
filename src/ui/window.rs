use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk4 as gtk;

use crate::config_discovery::ConfigDiscovery;
use crate::current_config::CurrentConfigSnapshot;
use crate::export::ExportBundle;
use crate::search::{search_projection, SearchRank, SearchResult};
use crate::ui::model::{
    initial_screen_shader_advisory_ui_action, run_screen_shader_advisory_ui_action,
    RowDetailProjection, ScreenShaderAdvisoryUiActionRequest, UiProjection,
};
use crate::validation::ValidationSummary;
use crate::write_classification::ScalarWriteValueKind;
use crate::write_flow::{apply_setting_change, write_flow_value_kind};

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
    let selected_tab_id = Rc::new(RefCell::new(
        model
            .tabs
            .iter()
            .find(|tab| tab.id == "appearance")
            .or_else(|| model.tabs.first())
            .map(|tab| tab.id.clone())
            .unwrap_or_default(),
    ));
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

    let sidebar = build_sidebar(&model);
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

    content.append(&build_summary_card(&model));

    let search_entry = gtk::SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search export metadata"));
    content.append(&search_entry);

    let tab_title = title_label("");
    content.append(&tab_title);

    let settings_list = gtk::ListBox::new();
    settings_list.set_selection_mode(gtk::SelectionMode::Single);
    let settings_scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&settings_list)
        .build();
    content.append(&settings_scroll);

    let (detail_panel, detail_content) = build_detail_panel();
    content.append(&detail_panel);

    render_settings_view(
        &model,
        &selected_tab_id.borrow(),
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
        let tab_title = tab_title.clone();
        let settings_list = settings_list.clone();
        let displayed_row_ids = Rc::clone(&displayed_row_ids);
        let detail_content = detail_content.clone();
        sidebar.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                if let Some(tab) = model.tabs.get(row.index() as usize) {
                    *selected_tab_id.borrow_mut() = tab.id.clone();
                    render_settings_view(
                        &model,
                        &selected_tab_id.borrow(),
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
        let tab_title = tab_title.clone();
        let settings_list = settings_list.clone();
        let displayed_row_ids = Rc::clone(&displayed_row_ids);
        let detail_content = detail_content.clone();
        search_entry.connect_search_changed(move |entry| {
            *current_query.borrow_mut() = entry.text().to_string();
            render_settings_view(
                &model,
                &selected_tab_id.borrow(),
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

    if let Some(initial_index) = model
        .tabs
        .iter()
        .position(|tab| tab.id == *selected_tab_id.borrow())
    {
        if let Some(row) = sidebar.row_at_index(initial_index as i32) {
            sidebar.select_row(Some(&row));
        }
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

fn build_sidebar(model: &UiProjection) -> gtk::ListBox {
    let sidebar = gtk::ListBox::new();
    sidebar.set_selection_mode(gtk::SelectionMode::Single);

    for tab in &model.tabs {
        let row = gtk::ListBoxRow::new();
        let row_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);

        row_box.append(&body_label(&tab.label));
        let count = small_label(&format!("{} rows", tab.row_count));
        row_box.append(&count);

        row.set_child(Some(&row_box));
        sidebar.append(&row);
    }

    sidebar
}

fn build_summary_card(model: &UiProjection) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    content.append(&title_label("Export validation passed"));
    content.append(&body_label(&format!("Export path: {}", model.export_dir)));
    content.append(&body_label(&format!(
        "Hyprland {} · schema {}",
        model.hyprland_version, model.schema_version
    )));
    content.append(&body_label(&format!(
        "Inventory rows: {} · official scalar coverage: {} / {}",
        model.summary.inventory_rows,
        model.summary.official_scalar_covered,
        model.summary.official_scalar_total
    )));
    content.append(&body_label(&format!(
        "Read allowlist: {} · non-read: {} · preview/parser-needed: {} · report-only/high-risk: {}",
        model.summary.read_allowlist_rows,
        model.summary.non_read_rows,
        model.summary.preview_parser_needed_rows,
        model.summary.report_only_high_risk_rows
    )));
    content.append(&body_label(&format!(
        "Safe parsed-preview: {} · warning-preview: {} · deferred parser rows: {}",
        model.summary.safe_parsed_preview_candidates,
        model.summary.warning_preview_candidates,
        model.summary.deferred_parser_rows
    )));
    content.append(&body_label(&format!(
        "Active write candidate: {} · structured families: {}",
        model.summary.active_write_candidate_ids.join(", "),
        model.summary.structured_family_count
    )));
    content.append(&body_label(
        "Export metadata is bundled. Current values are parsed from hyprland.conf as plain text when available. AGS is not required at runtime.",
    ));
    content.append(&body_label(&model.config_discovery.summary()));
    content.append(&small_label(model.config_discovery.live_read_status()));
    content.append(&body_label(&model.current_config.summary()));
    content.append(&body_label(&format!(
        "Current-value rows: readable {} / {} · unreadable {} · configured {} · unconfigured {} · conflicts {} · parser warnings {}",
        model.current_value_summary.readable_rows,
        model.current_value_summary.total_rows,
        model.current_value_summary.unreadable_rows,
        model.current_value_summary.configured_rows,
        model.current_value_summary.unconfigured_rows,
        model.current_value_summary.duplicate_conflict_rows,
        model.current_value_summary.parser_warning_rows
    )));
    content.append(&body_label(&model.current_config.structured_summary()));
    append_structured_family_summary(model, &content);
    content.append(&body_label(&write_safety_text(model)));

    frame.set_child(Some(&content));
    frame
}

fn append_structured_family_summary(model: &UiProjection, content: &gtk::Box) {
    if model.structured_families.is_empty() {
        return;
    }

    content.append(&body_label("Structured config entries"));
    for family in &model.structured_families {
        content.append(&small_label(&format!(
            "{} ({}) · {} entries · warnings: {} · {}",
            family.label,
            family.family_id,
            family.entries.len(),
            family.warning_count,
            family.edit_status
        )));
        for entry in family.entries.iter().take(3) {
            content.append(&small_label(&format!(
                "{}:{} · {} · {}",
                entry.source_path, entry.line_number, entry.parser_status, entry.raw_line
            )));
            if let Some(warning) = &entry.warning {
                content.append(&small_label(&format!("warning: {warning}")));
            }
        }
        if family.entries.len() > 3 {
            content.append(&small_label(&format!(
                "{} more {} entries preserved read-only",
                family.entries.len() - 3,
                family.family_id
            )));
        }
    }
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
    row_box.append(&small_label(&format!(
        "{} · {} · {}",
        setting.official_setting, setting.row_id, setting.subsection
    )));
    if include_context {
        row_box.append(&small_label(&format!(
            "{} · {}",
            setting.tab_label,
            search_rank_label(result.rank)
        )));
    }
    if !setting.description.is_empty() {
        row_box.append(&wrapped_small_label(&setting.description));
    }

    let read_status = if setting
        .read_support
        .contains("current-value-read-allowlisted")
    {
        "read-allowlisted metadata"
    } else {
        "non-read classified metadata"
    };
    let report_status = if setting.report_only {
        "report-only"
    } else {
        "not report-only"
    };
    let write_status = if setting.edit.editable {
        "editable pilot"
    } else if setting.is_write_candidate {
        "write metadata present"
    } else {
        "not editable"
    };

    row_box.append(&small_label(&format!(
        "{} · current: {} · {} · {} · {} · preview: {} · risk: {} · write support: {}",
        read_status,
        setting.current_value.status_label(),
        setting.comparison.badge,
        report_status,
        write_status,
        setting.preview_status,
        setting.risk_class,
        setting.write_support
    )));

    row.set_child(Some(&row_box));
    row
}

fn build_detail_panel() -> (gtk::Frame, gtk::Box) {
    let frame = gtk::Frame::new(None);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    render_empty_detail(&content);
    frame.set_child(Some(&content));
    (frame, content)
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
    detail_content.append(&title_label(&detail.label));
    append_detail_line(detail_content, "Row ID", &detail.row_id);
    append_detail_line(detail_content, "Official setting", &detail.official_setting);
    append_detail_line(
        detail_content,
        "Tab",
        &format!("{} · {}", detail.tab_label, detail.subsection),
    );
    if !detail.description.is_empty() {
        append_detail_line(detail_content, "Description", &detail.description);
    }
    append_detail_line(
        detail_content,
        "Default metadata",
        &detail.default_config_presence,
    );
    append_detail_line(detail_content, "Comparison", &detail.comparison.badge);
    append_detail_line(
        detail_content,
        "Comparison detail",
        &detail.comparison.detail,
    );
    append_detail_line(detail_content, "Read support", &detail.read_support);
    append_detail_line(
        detail_content,
        "Current value status",
        detail.current_value.status_label(),
    );
    if let Some(raw_value) = &detail.current_value.raw_value {
        append_detail_line(detail_content, "Current value", raw_value);
    }
    if let (Some(path), Some(line_number)) = (
        &detail.current_value.source_path,
        detail.current_value.line_number,
    ) {
        append_detail_line(
            detail_content,
            "Current value source",
            &format!("{}:{line_number}", path.display()),
        );
    }
    if let Some(raw_line) = &detail.current_value.raw_line {
        append_detail_line(detail_content, "Source line", raw_line);
    }
    if !detail.current_value.duplicate_lines.is_empty() {
        append_detail_line(
            detail_content,
            "Duplicate lines",
            &format!("{:?}", detail.current_value.duplicate_lines),
        );
    }
    if let Some(warning) = &detail.current_value.warning {
        append_detail_line(detail_content, "Current value warning", warning);
    }
    if let Some(status) = &detail.non_read_status {
        append_detail_line(detail_content, "Non-read status", status);
    }
    append_detail_line(detail_content, "Preview status", &detail.preview_status);
    append_detail_line(detail_content, "Risk class", &detail.risk_class);
    append_detail_line(
        detail_content,
        "Report-only status",
        &detail.report_only_status,
    );
    append_detail_line(detail_content, "Write support", &detail.write_support);
    append_detail_line(
        detail_content,
        "Write candidate status",
        &detail.write_candidate_status,
    );
    if let Some(target_mode) = &detail.write_candidate_target_mode {
        append_detail_line(detail_content, "Target mode", target_mode);
    }
    if let Some(executable) = detail.write_candidate_executable {
        append_detail_line(detail_content, "Executable", &executable.to_string());
    }
    if let Some(command_generation_allowed) = detail.write_candidate_command_generation_allowed {
        append_detail_line(
            detail_content,
            "Command generation",
            &command_generation_allowed.to_string(),
        );
    }
    append_detail_line(
        detail_content,
        "Editable in app",
        if detail.edit.editable { "yes" } else { "no" },
    );
    if let Some(reason) = &detail.edit.disabled_reason {
        append_detail_line(detail_content, "Disabled reason", reason);
    }
    if let Some(proposed_value) = &detail.edit.proposed_value {
        append_detail_line(detail_content, "Suggested pending value", proposed_value);
    }
    if let Some(pending) = &detail.edit.pending {
        append_detail_line(
            detail_content,
            "Pending change validation",
            &pending.validation_label,
        );
        append_detail_line(
            detail_content,
            "Pending review available",
            if pending.can_review { "yes" } else { "no" },
        );
        for line in &pending.review_summary {
            detail_content.append(&small_label(line));
        }
    }
    append_screen_shader_advisory_controls(&detail, detail_content);
    append_write_controls(model, &detail, detail_content);
    for note in &detail.safety_notes {
        detail_content.append(&small_label(note));
    }
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

    let result_label = small_label(if can_review {
        "Ready for backup, write, and reread verification."
    } else {
        "Apply is blocked until current config is readable and conflict-free."
    });
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

fn write_safety_text(model: &UiProjection) -> String {
    let mut parts = vec!["Write controls gated".to_string()];
    for candidate in &model.active_write_candidates {
        parts.push(format!(
            "active candidate: {} · target mode: {} · executable: {} · command generation: {}",
            candidate.row_id,
            candidate.target_mode,
            candidate.executable,
            candidate.command_generation_allowed
        ));
    }
    parts.push("first pilot requires backup and reread verification".to_string());
    parts.push("no Hyprland reload command is run".to_string());
    parts.join(" · ")
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
