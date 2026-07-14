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
use crate::family_record_picker::{
    list_animation_records_live, list_curve_records_live, save_picked_record_live,
    AnimationRecordEntry, CurveRecordEntry, FamilyRecordPreviewController, PickedFamily,
    PickedRecordValues, RecordPickerPhase, ANIMATION_STYLE_BLOCKED_REASON,
};
use crate::future_capability::{
    apply_production_activation_draft_gtk_update, disabled_future_approval_card_projections,
    duplicate_activation_draft_gtk_review, duplicate_production_approval_gate,
    production_activation_approval_and_dry_run_reviews, production_activation_cap_reviews,
    production_activation_control_reviews, production_activation_decision_reviews,
    production_activation_draft_edit_reviews, production_activation_draft_persistence_boundaries,
    production_activation_draft_reviews, production_activation_final_decision_reviews,
    production_activation_form_reviews, production_activation_live_draft_gtk_reviews,
    production_activation_opt_in_requirement_reviews, production_activation_path_reviews,
    production_activation_safety_gate_proof_reviews, production_activation_safety_gate_reviews,
    proven_runtime_approval_evidence_summary, source_include_activation_draft_gtk_review,
    source_include_insertion_review, source_include_selected_target_dry_run_plan,
    source_include_target_selection_fixture_proof, DisabledApprovalCardProjection,
    DuplicateOccurrence, DuplicateProductionGateStatus,
    ProductionActivationApprovalAndDryRunReview, ProductionActivationCapReview,
    ProductionActivationControlReview, ProductionActivationDecisionReview,
    ProductionActivationDraftEditReview, ProductionActivationDraftGtkField,
    ProductionActivationDraftGtkReview, ProductionActivationDraftGtkState,
    ProductionActivationDraftGtkUpdate, ProductionActivationDraftPersistenceBoundary,
    ProductionActivationDraftReview, ProductionActivationFinalDecisionReview,
    ProductionActivationFormReview, ProductionActivationPathReview,
    ProductionActivationSafetyGateProofReview, ProductionActivationSafetyGateReview,
    ProductionExecutorWiringState, ProductionFlagAndExecutorOptInReview,
    SourceIncludeInsertionReadiness, SourceIncludeSelectedTargetDryRunStatus,
    SourceIncludeTargetCandidate,
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
use crate::runtime_preview_dead_man::{
    dead_man_ui_state, RuntimePreviewDeadManController, RuntimePreviewDeadManUiPhase,
};
use crate::runtime_preview_ui_projection::{
    runtime_preview_ui_row_state, RuntimePreviewUiControlKind, RuntimePreviewUiController,
    RuntimePreviewUiSessionState,
};
use crate::safe_batch_write::safe_batch_write_user_facing_lines;
use crate::safe_live_save_mode::{
    disable_safe_live_save_mode_live, enable_safe_live_save_mode_live,
    read_safe_live_save_mode_status_live, SafeLiveSaveModeState,
};
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
use crate::write_flow::{write_flow_config_setting, write_flow_value_kind};
use crate::write_review_walkthrough::build_write_review_walkthrough;
use crate::write_target_candidate::write_target_candidates_for_layered_setting;
use crate::write_target_recommendation::recommend_write_targets;
use crate::write_verification_plan::planned_reread_verification;

const DASHBOARD_ID: &str = "dashboard";
const CONFIG_ID: &str = "config";
const SAFETY_ID: &str = "safety-details";
const LAYOUTS_ID: &str = "layouts";
const PROFILES_ID: &str = "profiles";

thread_local! {
    /// Live preview controllers whose sessions may still be active. Drained
    /// on detail-pane re-render and on window close so an abandoned preview
    /// always restores the original runtime value.
    static ACTIVE_PREVIEW_CONTROLLERS: RefCell<Vec<std::rc::Weak<RefCell<RuntimePreviewUiController>>>> =
        const { RefCell::new(Vec::new()) };
}

fn register_preview_controller(controller: &Rc<RefCell<RuntimePreviewUiController>>) {
    ACTIVE_PREVIEW_CONTROLLERS.with(|controllers| {
        controllers.borrow_mut().push(Rc::downgrade(controller));
    });
}

/// Revert every still-active preview session (session-drop / app-close
/// recovery). Safe to call repeatedly; dead weak references are pruned.
fn revert_all_active_previews() {
    ACTIVE_PREVIEW_CONTROLLERS.with(|controllers| {
        let mut controllers = controllers.borrow_mut();
        for weak in controllers.iter() {
            if let Some(controller) = weak.upgrade() {
                if let Ok(mut controller) = controller.try_borrow_mut() {
                    let _ = controller.revert_if_active();
                }
            }
        }
        controllers.retain(|weak| weak.upgrade().is_some());
    });
}

fn preview_now_ms() -> u64 {
    (gtk::glib::monotonic_time() / 1000).max(0) as u64
}

thread_local! {
    /// Supervised (dead-man) controllers with possibly armed sessions.
    /// Unconfirmed previews revert on detail-pane re-render and window close;
    /// explicitly Kept previews are left in place.
    static ACTIVE_DEAD_MAN_CONTROLLERS: RefCell<Vec<std::rc::Weak<RefCell<RuntimePreviewDeadManController>>>> =
        const { RefCell::new(Vec::new()) };
}

fn register_dead_man_controller(controller: &Rc<RefCell<RuntimePreviewDeadManController>>) {
    ACTIVE_DEAD_MAN_CONTROLLERS.with(|controllers| {
        controllers.borrow_mut().push(Rc::downgrade(controller));
    });
}

thread_local! {
    /// Supervised family record picker controllers; unconfirmed previews
    /// revert on re-render and window close, like scalar dead-man sessions.
    static ACTIVE_RECORD_PICKER_CONTROLLERS: RefCell<Vec<std::rc::Weak<RefCell<FamilyRecordPreviewController>>>> =
        const { RefCell::new(Vec::new()) };
}

fn register_record_picker_controller(controller: &Rc<RefCell<FamilyRecordPreviewController>>) {
    ACTIVE_RECORD_PICKER_CONTROLLERS.with(|controllers| {
        controllers.borrow_mut().push(Rc::downgrade(controller));
    });
}

fn revert_all_unconfirmed_family_previews() {
    ACTIVE_RECORD_PICKER_CONTROLLERS.with(|controllers| {
        let mut controllers = controllers.borrow_mut();
        for weak in controllers.iter() {
            if let Some(controller) = weak.upgrade() {
                if let Ok(mut controller) = controller.try_borrow_mut() {
                    let _ = controller.revert_if_unconfirmed();
                }
            }
        }
        controllers.retain(|weak| weak.upgrade().is_some());
    });
}

fn revert_all_unconfirmed_dead_man_previews() {
    ACTIVE_DEAD_MAN_CONTROLLERS.with(|controllers| {
        let mut controllers = controllers.borrow_mut();
        for weak in controllers.iter() {
            if let Some(controller) = weak.upgrade() {
                if let Ok(mut controller) = controller.try_borrow_mut() {
                    let _ = controller.revert_if_unconfirmed();
                }
            }
        }
        controllers.retain(|weak| weak.upgrade().is_some());
    });
}

const PENDING_ID: &str = "pending-changes";

/// One row's live-preview session as the pending-changes surfaces see it:
/// the row identity plus a weak handle to its preview controller.
struct PendingLedgerEntry {
    row_id: String,
    official_setting: String,
    page_id: Option<&'static str>,
    /// Strong reference: a live preview session must outlive its widget
    /// (page re-renders destroy controls), or the pending state and the
    /// ability to revert it from the pending surfaces would vanish while
    /// the runtime change stayed applied.
    controller: Rc<RefCell<RuntimePreviewUiController>>,
}

/// A snapshot of one pending (previewed-live, not saved) change.
#[derive(Clone)]
struct PendingChangeSnapshot {
    row_id: String,
    official_setting: String,
    page_id: Option<&'static str>,
    current_value: String,
    controller: std::rc::Weak<RefCell<RuntimePreviewUiController>>,
}

thread_local! {
    /// Every scalar preview controller with its row identity: the source
    /// of truth for "unsaved changes" (previewing live, value differs from
    /// the session's original runtime value).
    static PENDING_LEDGER: RefCell<Vec<PendingLedgerEntry>> = const { RefCell::new(Vec::new()) };
    /// UI refreshers to run whenever the pending set may have changed
    /// (header chip, sidebar badges, bottom bar, review page, row accents).
    static PENDING_LISTENERS: RefCell<Vec<Box<dyn Fn()>>> = const { RefCell::new(Vec::new()) };
    /// Currently rendered setting rows by row id, for live accent updates.
    static PENDING_ROW_WIDGETS: RefCell<Vec<(String, gtk::glib::WeakRef<gtk::ListBoxRow>)>> =
        const { RefCell::new(Vec::new()) };
}

fn register_pending_controller(
    row_id: &str,
    official_setting: &str,
    page_id: Option<&'static str>,
    controller: &Rc<RefCell<RuntimePreviewUiController>>,
) {
    PENDING_LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        ledger.retain(|entry| entry.row_id != row_id);
        ledger.push(PendingLedgerEntry {
            row_id: row_id.to_string(),
            official_setting: official_setting.to_string(),
            page_id,
            controller: Rc::clone(controller),
        });
    });
}

/// The ledger's controller for a row, if one exists. Creation paths reuse
/// it so a row keeps ONE preview session across page re-renders — creating
/// a fresh controller mid-preview would re-capture the already-previewed
/// runtime value as the new "original" and silently lose the pending state.
fn pending_controller_for_row(row_id: &str) -> Option<Rc<RefCell<RuntimePreviewUiController>>> {
    PENDING_LEDGER.with(|ledger| {
        ledger
            .borrow()
            .iter()
            .find(|entry| entry.row_id == row_id)
            .map(|entry| Rc::clone(&entry.controller))
    })
}

/// Snapshot of every pending change: the session is still previewing live
/// and the applied value differs from the session's original.
fn pending_change_snapshots() -> Vec<PendingChangeSnapshot> {
    PENDING_LEDGER.with(|ledger| {
        ledger
            .borrow()
            .iter()
            .filter_map(|entry| {
                let controller_ref = entry.controller.try_borrow().ok()?;
                if controller_ref.session_state()
                    != crate::runtime_preview_ui_projection::RuntimePreviewUiSessionState::PreviewingLive
                {
                    return None;
                }
                let original = controller_ref.original_runtime_value()?;
                let current = controller_ref.last_applied_value()?;
                // Semantic comparison: "true" vs "1", "0.5" vs "0.500000",
                // rgba() vs bare-hex readback spellings are not changes.
                if crate::pending_changes_ui::values_semantically_equal(&current, &original) {
                    return None;
                }
                Some(PendingChangeSnapshot {
                    row_id: entry.row_id.clone(),
                    official_setting: entry.official_setting.clone(),
                    page_id: entry.page_id,
                    current_value: current,
                    controller: Rc::downgrade(&entry.controller),
                })
            })
            .collect()
    })
}

fn add_pending_listener(listener: Box<dyn Fn()>) {
    PENDING_LISTENERS.with(|listeners| listeners.borrow_mut().push(listener));
}

/// Run every pending-changes refresher (and refresh row accents). Called
/// after any operation that may change the pending set.
fn notify_pending_changed() {
    let pending_rows: std::collections::HashSet<String> = pending_change_snapshots()
        .into_iter()
        .map(|snapshot| snapshot.row_id)
        .collect();
    PENDING_ROW_WIDGETS.with(|rows| {
        let mut rows = rows.borrow_mut();
        rows.retain(|(_, weak)| weak.upgrade().is_some());
        for (row_id, weak) in rows.iter() {
            if let Some(row) = weak.upgrade() {
                if pending_rows.contains(row_id) {
                    row.add_css_class("hyprland-settings-row-pending");
                    row.update_property(&[gtk::accessible::Property::Description(
                        "Pending change",
                    )]);
                } else {
                    row.remove_css_class("hyprland-settings-row-pending");
                    row.update_property(&[gtk::accessible::Property::Description("")]);
                }
            }
        }
    });
    PENDING_LISTENERS.with(|listeners| {
        for listener in listeners.borrow().iter() {
            listener();
        }
    });
}

fn register_pending_row_widget(row_id: &str, row: &gtk::ListBoxRow) {
    let weak = gtk::glib::WeakRef::new();
    weak.set(Some(row));
    PENDING_ROW_WIDGETS.with(|rows| {
        let mut rows = rows.borrow_mut();
        rows.retain(|(_, existing)| existing.upgrade().is_some());
        rows.push((row_id.to_string(), weak));
    });
}

#[derive(Debug, Clone)]
struct SidebarItem {
    id: String,
    label: String,
    target_tab_id: Option<String>,
}

/// The standalone (non-settings-list) pages, keyed by page id. One bundle
/// instead of a dozen threaded parameters.
#[derive(Clone)]
struct StandalonePages {
    pages: Rc<Vec<(&'static str, gtk::ScrolledWindow)>>,
}

impl StandalonePages {
    /// Show the selected standalone page and hide the rest; false when the
    /// selected id is not a standalone page.
    fn show_only(&self, selected: &str) -> bool {
        let mut matched = false;
        for (page_id, view) in self.pages.iter() {
            let is_selected = *page_id == selected;
            view.set_visible(is_selected);
            matched |= is_selected;
        }
        matched
    }
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

    // App-level presentation CSS: sidebar legibility only (slightly larger
    // nav labels). Loaded once per display.
    if let Some(display) = gtk::gdk::Display::default() {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(
            ".hyprland-settings-nav-row-label { font-size: 1.08em; }\n\
             .hyprland-settings-swatch-button { padding: 0; min-width: 0; min-height: 0; }\n\
             .hyprland-settings-palette-tile { padding: 0; min-width: 0; min-height: 0; background: none; border: none; box-shadow: none; outline-offset: -2px; }\n\
             .hyprland-settings-row-pending { border-left: 3px solid @warning_bg_color; }\n\
             .hyprland-settings-pending-chip { background-color: alpha(@warning_bg_color, 0.18); color: @warning_color; border-radius: 9999px; padding: 0 10px; min-height: 26px; font-weight: 700; }\n\
             .hyprland-settings-pending-chip:hover { background-color: alpha(@warning_bg_color, 0.28); }\n\
             .hyprland-settings-pending-chip:active { background-color: alpha(@warning_bg_color, 0.36); }\n\
             .hyprland-settings-sidebar-badge { background-color: alpha(@warning_bg_color, 0.25); color: @warning_color; border-radius: 9999px; padding: 0 7px; font-size: 0.75em; font-weight: 700; min-height: 18px; min-width: 18px; }\n\
             .hyprland-settings-pending-badge { border-radius: 9999px; padding: 2px 10px; font-size: 0.75em; font-weight: 700; letter-spacing: 0.03em; }\n\
             .hyprland-settings-pending-badge-added { background-color: alpha(@success_bg_color, 0.25); color: @success_color; }\n\
             .hyprland-settings-pending-badge-modified { background-color: alpha(@warning_bg_color, 0.25); color: @warning_color; }\n\
             .hyprland-settings-diff-card { background-color: alpha(@view_fg_color, 0.04); border-radius: 12px; }\n\
             .hyprland-settings-diff-line { font-family: monospace; font-size: 0.85em; padding: 0 10px; }\n\
             .hyprland-settings-diff-added { background-color: alpha(@success_bg_color, 0.18); color: @success_color; }\n\
             .hyprland-settings-diff-removed { background-color: alpha(@error_bg_color, 0.16); color: @error_color; }\n\
             .hyprland-settings-diff-meta { opacity: 0.55; font-style: italic; }\n\
             .hyprland-settings-diff-context { opacity: 0.7; }\n\
             .hyprland-settings-diff-count-added { color: @success_color; font-weight: 700; }\n\
             .hyprland-settings-diff-count-removed { color: @error_color; font-weight: 700; }\n",
        );
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Hyprland Settings")
        .default_width(1180)
        .default_height(760)
        .build();
    window.set_widget_name("hyprland-settings-main-window");
    // App-close recovery: any preview still active restores its original
    // runtime value before the window closes.
    window.connect_close_request(|_| {
        revert_all_active_previews();
        revert_all_unconfirmed_dead_man_previews();
        revert_all_unconfirmed_family_previews();
        gtk::glib::Propagation::Proceed
    });

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.set_widget_name("hyprland-settings-root");

    // The header title follows the current page (the app name lives in
    // window metadata, not as a dominant page header).
    let header_title = adw::WindowTitle::new("Dashboard", "");
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&header_title));
    // Pending-changes chip: amber count button in the header, visible only
    // while unsaved (previewed-live) changes exist; opens the hidden
    // Pending Changes review page.
    let pending_chip = gtk::Button::new();
    pending_chip.add_css_class("flat");
    pending_chip.add_css_class("hyprland-settings-pending-chip");
    pending_chip.set_widget_name("hyprland-settings-pending-chip");
    pending_chip.update_property(&[gtk::accessible::Property::Label("View pending changes")]);
    let chip_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    chip_box.append(&gtk::Image::from_icon_name("view-list-symbolic"));
    let pending_chip_count = gtk::Label::new(None);
    chip_box.append(&pending_chip_count);
    pending_chip.set_child(Some(&chip_box));
    pending_chip.set_visible(false);
    header.pack_end(&pending_chip);
    root.append(&header);

    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    body.set_vexpand(true);
    body.set_hexpand(true);
    root.append(&body);

    let sidebar_items = Rc::new(sidebar_items(&model));
    let config_selection_state = Rc::new(RefCell::new(config_selection_state_for_discovery(
        &model.config_discovery,
    )));
    let (sidebar, sidebar_badges) = build_sidebar(&sidebar_items);
    let sidebar_scroll = gtk::ScrolledWindow::builder()
        .min_content_width(250)
        .max_content_width(300)
        .vexpand(true)
        .hexpand(false)
        .child(&sidebar)
        .build();
    // Sidebar column: compact identity header with the search icon, the
    // hidden search entry sliding in below it, then the navigation list.
    let sidebar_column = gtk::Box::new(gtk::Orientation::Vertical, 0);
    sidebar_column.set_widget_name("hyprland-settings-sidebar-column");
    sidebar_column.set_hexpand(false);
    sidebar_column.set_size_request(260, -1);
    let sidebar_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    sidebar_header.set_margin_top(8);
    sidebar_header.set_margin_bottom(4);
    sidebar_header.set_margin_start(12);
    sidebar_header.set_margin_end(6);
    let app_label = gtk::Label::new(Some("Hyprland Settings"));
    app_label.set_halign(gtk::Align::Start);
    app_label.set_hexpand(true);
    app_label.add_css_class("heading");
    sidebar_header.append(&app_label);
    sidebar_column.append(&sidebar_header);
    body.append(&sidebar_column);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_hexpand(true);
    content.set_vexpand(true);
    // The content column hosts the pages and, below them, the slide-up
    // unsaved-changes action bar (content-area width, like the reference).
    let content_column = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content_column.set_hexpand(true);
    content_column.set_vexpand(true);
    content_column.append(&content);
    body.append(&content_column);

    let dashboard_view = build_dashboard_view(&model, &sidebar, &sidebar_items);
    dashboard_view.set_widget_name("hyprland-settings-dashboard-page");
    let config_view = build_config_view(&model, &window, &config_selection_state);
    config_view.set_widget_name("hyprland-settings-config-page");
    let safety_view = build_safety_details_view(&model);
    safety_view.set_widget_name("hyprland-settings-safety-details-page");
    let layouts_view = build_layouts_view(&model);
    layouts_view.set_widget_name("hyprland-settings-layouts-page");
    let profiles_view = build_profiles_view();
    profiles_view.set_widget_name("hyprland-settings-profiles-page");
    let workspaces_view = empty_state_view(
        "hyprland-settings-workspaces-content",
        "view-grid-symbolic",
        "Workspaces",
        "Workspace rules are not editable here yet. Anything in your config stays exactly as written.",
    );
    workspaces_view.set_widget_name("hyprland-settings-workspaces-page");
    let window_rules_view = structured_locked_list_view(
        &model,
        "hl.windowrule",
        "hyprland-settings-window-rules-content",
        "Window Rules",
        "These rules come from your hyprland.conf or its sourced files. They are read-only here.",
        "No Window Rules",
        "Rules found in your config will show here, read-only.",
    );
    window_rules_view.set_widget_name("hyprland-settings-window-rules-page");
    let layer_rules_view = empty_state_view(
        "hyprland-settings-layer-rules-content",
        "text-x-generic-symbolic",
        "Layer Rules",
        "Layer rules are not shown yet: the read-only config view does not cover them. Your files stay untouched.",
    );
    layer_rules_view.set_widget_name("hyprland-settings-layer-rules-page");
    let autostart_view = empty_state_view(
        "hyprland-settings-autostart-content",
        "media-playback-start-symbolic",
        "Autostart",
        "Autostart entries come from exec lines in your hyprland.conf or its sourced files. A safe read-only view is not available yet, and nothing is changed.",
    );
    autostart_view.set_widget_name("hyprland-settings-autostart-page");
    let env_variables_view = empty_state_view(
        "hyprland-settings-env-variables-content",
        "utilities-terminal-symbolic",
        "Env Variables",
        "Environment variables come from env lines in your config files. A safe read-only view is not available yet, and nothing is changed.",
    );
    env_variables_view.set_widget_name("hyprland-settings-env-variables-page");

    let navigate_to_page: Rc<dyn Fn(&str)> = {
        let sidebar = sidebar.clone();
        let sidebar_items = Rc::clone(&sidebar_items);
        Rc::new(move |page_id: &str| {
            if let Some(index) = sidebar_items.iter().position(|item| item.id == page_id) {
                if let Some(target_row) = sidebar.row_at_index(index as i32) {
                    sidebar.select_row(Some(&target_row));
                }
            }
        })
    };
    let (pending_view, pending_refresh) =
        build_pending_changes_view(&model, Rc::clone(&navigate_to_page));
    pending_view.set_widget_name("hyprland-settings-pending-changes-page");
    let (pending_bar, pending_bar_refresh) = build_pending_bottom_bar(&model);
    content_column.append(&pending_bar);

    let standalone = StandalonePages {
        pages: Rc::new(vec![
            (PENDING_ID, pending_view.clone()),
            (DASHBOARD_ID, dashboard_view.clone()),
            (CONFIG_ID, config_view.clone()),
            (SAFETY_ID, safety_view.clone()),
            (LAYOUTS_ID, layouts_view.clone()),
            (PROFILES_ID, profiles_view.clone()),
            ("workspaces", workspaces_view.clone()),
            ("window-rules", window_rules_view.clone()),
            ("layer-rules", layer_rules_view.clone()),
            ("autostart", autostart_view.clone()),
            ("env-variables", env_variables_view.clone()),
        ]),
    };
    for (_, view) in standalone.pages.iter() {
        content.append(view);
    }

    let settings_view = gtk::Box::new(gtk::Orientation::Vertical, 12);
    settings_view.set_widget_name("hyprland-settings-category-page");
    settings_view.set_hexpand(true);
    settings_view.set_vexpand(true);
    content.append(&settings_view);

    // Search is the sometimes path: the entry hides by default behind a
    // small toggle and slides in on Ctrl+F or the toggle; Escape clears
    // and hides it again. Browsing the grouped categories stays the
    // everyday path.
    let search_toggle = gtk::ToggleButton::new();
    search_toggle.set_icon_name("system-search-symbolic");
    search_toggle.set_widget_name("hyprland-settings-search-toggle");
    search_toggle.add_css_class("flat");
    search_toggle.update_property(&[gtk::accessible::Property::Label("Search settings")]);

    let search_entry = gtk::SearchEntry::new();
    search_entry.set_widget_name("hyprland-settings-search");
    search_entry.set_placeholder_text(Some("Search settings…"));
    search_entry.update_property(&[gtk::accessible::Property::Label("Search settings")]);
    search_entry.set_visible(false);
    search_entry.set_margin_start(8);
    search_entry.set_margin_end(8);
    search_entry.set_margin_bottom(4);
    sidebar_header.append(&search_toggle);
    sidebar_column.append(&search_entry);
    sidebar_column.append(&sidebar_scroll);

    {
        // The toggle is the one owner of visibility: Ctrl+F and Escape
        // route through it so every path behaves identically.
        let entry = search_entry.clone();
        search_toggle.connect_toggled(move |toggle| {
            if toggle.is_active() {
                entry.set_visible(true);
                entry.grab_focus();
            } else {
                entry.set_text("");
                entry.set_visible(false);
            }
        });
    }

    {
        let search_toggle = search_toggle.clone();
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, keyval, _, state| {
            let control_held = state.contains(gtk::gdk::ModifierType::CONTROL_MASK);
            if control_held && (keyval == gtk::gdk::Key::f || keyval == gtk::gdk::Key::F) {
                search_toggle.set_active(true);
                return gtk::glib::Propagation::Stop;
            }
            if keyval == gtk::gdk::Key::Escape && search_toggle.is_active() {
                search_toggle.set_active(false);
                return gtk::glib::Propagation::Stop;
            }
            gtk::glib::Propagation::Proceed
        });
        window.add_controller(key_controller);
    }

    {
        // Esc inside the entry routes through the same clear-and-hide path.
        let search_toggle = search_toggle.clone();
        search_entry.connect_stop_search(move |_| {
            search_toggle.set_active(false);
        });
    }

    // The page name renders in the header; no duplicate giant content
    // title on settings pages. The label stays alive for internal state.
    let tab_title = title_label("");
    tab_title.set_widget_name("hyprland-settings-category-title");
    tab_title.set_visible(false);
    settings_view.append(&tab_title);

    // Settings pages are a single centered column of section cards:
    // a heading label above each rounded card, rows inside, controls on
    // the rows. No permanent detail pane — details open on demand
    // anchored to the opened row.
    let sections_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
    sections_box.set_widget_name("hyprland-settings-sections");

    let settings_clamp = adw::Clamp::new();
    settings_clamp.set_maximum_size(800);
    settings_clamp.set_tightening_threshold(600);
    settings_clamp.set_margin_top(12);
    settings_clamp.set_margin_bottom(24);
    settings_clamp.set_margin_start(12);
    settings_clamp.set_margin_end(12);
    settings_clamp.set_child(Some(&sections_box));

    let settings_scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&settings_clamp)
        .build();

    let (detail_panel, detail_content) = build_detail_panel();
    // The detail surface is a popover anchored to the opened row: it
    // appears only on demand and never occupies the page.
    let detail_popover = gtk::Popover::new();
    detail_popover.set_widget_name("hyprland-settings-detail-popover");
    detail_popover.set_child(Some(&detail_panel));
    detail_popover.set_parent(&sections_box);
    detail_popover.set_autohide(true);
    settings_view.append(&settings_scroll);

    render_main_view(
        &model,
        &selected_tab_id.borrow(),
        &standalone,
        &settings_view,
        &current_query.borrow(),
        &tab_title,
        &sections_box,
        &detail_content,
        &detail_popover,
        &config_selection_state,
    );

    {
        let model = Rc::clone(&model);
        let selected_tab_id = Rc::clone(&selected_tab_id);
        let current_query = Rc::clone(&current_query);
        let sidebar_items = Rc::clone(&sidebar_items);
        let standalone = standalone.clone();
        let settings_view = settings_view.clone();
        let tab_title = tab_title.clone();
        let sections_box = sections_box.clone();
        let detail_content = detail_content.clone();
        let detail_popover = detail_popover.clone();
        let config_selection_state = Rc::clone(&config_selection_state);
        let header_title = header_title.clone();
        sidebar.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                if let Some(item) = sidebar_items.get(row.index() as usize) {
                    *selected_tab_id.borrow_mut() = item.id.clone();
                    header_title.set_title(&item.label);
                    render_main_view(
                        &model,
                        &selected_tab_id.borrow(),
                        &standalone,
                        &settings_view,
                        &current_query.borrow(),
                        &tab_title,
                        &sections_box,
                        &detail_content,
                        &detail_popover,
                        &config_selection_state,
                    );
                    // Leaving/entering pages updates the pending chip
                    // visibility (it hides on the review page itself).
                    notify_pending_changed();
                }
            }
        });
    }

    {
        // Header chip opens the hidden Pending Changes review page.
        let model = Rc::clone(&model);
        let selected_tab_id = Rc::clone(&selected_tab_id);
        let current_query = Rc::clone(&current_query);
        let standalone = standalone.clone();
        let settings_view = settings_view.clone();
        let tab_title = tab_title.clone();
        let sections_box = sections_box.clone();
        let detail_content = detail_content.clone();
        let detail_popover = detail_popover.clone();
        let config_selection_state = Rc::clone(&config_selection_state);
        let header_title = header_title.clone();
        let sidebar = sidebar.clone();
        let pending_refresh = Rc::clone(&pending_refresh);
        pending_chip.connect_clicked(move |_| {
            *selected_tab_id.borrow_mut() = PENDING_ID.to_string();
            header_title.set_title("Pending Changes");
            sidebar.unselect_all();
            pending_refresh();
            render_main_view(
                &model,
                &selected_tab_id.borrow(),
                &standalone,
                &settings_view,
                &current_query.borrow(),
                &tab_title,
                &sections_box,
                &detail_content,
                &detail_popover,
                &config_selection_state,
            );
            notify_pending_changed();
        });
    }

    {
        // The one pending-changes refresher: chip count/visibility, sidebar
        // badges, bottom bar, and the review page, from the ledger.
        let pending_chip = pending_chip.clone();
        let pending_chip_count = pending_chip_count.clone();
        let sidebar_badges = sidebar_badges.clone();
        let selected_tab_id = Rc::clone(&selected_tab_id);
        let pending_bar_refresh = Rc::clone(&pending_bar_refresh);
        let pending_refresh = Rc::clone(&pending_refresh);
        add_pending_listener(Box::new(move || {
            let snapshots = pending_change_snapshots();
            let count = snapshots.len();
            pending_chip_count.set_label(&count.to_string());
            pending_chip.set_visible(count > 0 && selected_tab_id.borrow().as_str() != PENDING_ID);
            let mut per_page: std::collections::HashMap<&str, usize> =
                std::collections::HashMap::new();
            for snapshot in &snapshots {
                if let Some(page_id) = snapshot.page_id {
                    *per_page.entry(page_id).or_insert(0) += 1;
                }
            }
            for (page_id, badge) in sidebar_badges.iter() {
                let page_count = per_page.get(page_id.as_str()).copied().unwrap_or(0);
                badge.set_label(&page_count.to_string());
                badge.set_visible(page_count > 0);
            }
            pending_bar_refresh();
            pending_refresh();
        }));
    }

    {
        let model = Rc::clone(&model);
        let selected_tab_id = Rc::clone(&selected_tab_id);
        let current_query = Rc::clone(&current_query);
        let standalone = standalone.clone();
        let settings_view = settings_view.clone();
        let tab_title = tab_title.clone();
        let sections_box = sections_box.clone();
        let detail_content = detail_content.clone();
        let detail_popover = detail_popover.clone();
        let config_selection_state = Rc::clone(&config_selection_state);
        search_entry.connect_search_changed(move |entry| {
            *current_query.borrow_mut() = entry.text().to_string();
            render_main_view(
                &model,
                &selected_tab_id.borrow(),
                &standalone,
                &settings_view,
                &current_query.borrow(),
                &tab_title,
                &sections_box,
                &detail_content,
                &detail_popover,
                &config_selection_state,
            );
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
    let title = adw::WindowTitle::new("Startup Error", "Hyprland Settings");
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
    // Page-driven sidebar (see ux_presentation::SIDEBAR_PAGE_LAYOUT):
    // Dashboard pinned on top, then the categories in order. Model-backed
    // pages hide when no row of their tab lands on them (version-aware
    // hiding); standalone pages always show.
    let mut items = vec![SidebarItem {
        id: DASHBOARD_ID.to_string(),
        label: "Dashboard".to_string(),
        target_tab_id: None,
    }];

    for category in crate::ux_presentation::SIDEBAR_PAGE_LAYOUT {
        for page in category.pages {
            match page.source_tab {
                None => items.push(SidebarItem {
                    id: page.id.to_string(),
                    label: page.label.to_string(),
                    target_tab_id: None,
                }),
                Some(source_tab) => {
                    let has_rows =
                        crate::ux_presentation::page_source_tabs(page)
                            .iter()
                            .any(|tab| {
                                model.settings_for_tab(tab).iter().any(|setting| {
                                    crate::ux_presentation::page_claims_row_in_tab(
                                        page,
                                        tab,
                                        &setting.official_setting,
                                    )
                                })
                            });
                    if has_rows {
                        items.push(SidebarItem {
                            id: page.id.to_string(),
                            label: page.label.to_string(),
                            target_tab_id: Some(source_tab.to_string()),
                        });
                    }
                }
            }
        }
    }
    items
}

fn build_sidebar(items: &[SidebarItem]) -> (gtk::ListBox, Vec<(String, gtk::Label)>) {
    let mut badges: Vec<(String, gtk::Label)> = Vec::new();
    let sidebar = gtk::ListBox::new();
    sidebar.set_widget_name("hyprland-settings-navigation-sidebar");
    sidebar.set_selection_mode(gtk::SelectionMode::Single);
    sidebar.add_css_class("navigation-sidebar");

    // Category headers render via the ListBox header mechanism so they are
    // not rows: selection indices keep mapping 1:1 onto `items`.
    let categories: Vec<Option<&'static str>> = items
        .iter()
        .map(|item| crate::ux_presentation::category_for_tab(&item.id))
        .collect();
    sidebar.set_header_func(move |row, before| {
        let category = categories.get(row.index() as usize).copied().flatten();
        let previous = before
            .and_then(|previous_row| categories.get(previous_row.index() as usize))
            .copied()
            .flatten();
        if let Some(label) = category {
            if Some(label) != previous {
                // Category headers render uppercase in a small caption
                // style, hanging above the group like a section label.
                let header = gtk::Label::new(Some(&label.to_uppercase()));
                header.set_halign(gtk::Align::Start);
                header.set_margin_top(14);
                header.set_margin_start(14);
                header.set_margin_bottom(3);
                header.add_css_class("caption-heading");
                header.add_css_class("dim-label");
                header.set_widget_name(&format!(
                    "hyprland-settings-nav-category-{}",
                    safe_widget_name(label)
                ));
                row.set_header(Some(&header));
                return;
            }
        }
        row.set_header(None::<&gtk::Widget>);
    });

    for item in items {
        let row = gtk::ListBoxRow::new();
        row.set_widget_name(&format!(
            "hyprland-settings-nav-{}",
            safe_widget_name(&item.id)
        ));
        row.update_property(&[gtk::accessible::Property::Label(&format!(
            "Navigation: {}",
            item.label
        ))]);
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);

        let icon = gtk::Image::from_icon_name(crate::ux_presentation::page_icon(&item.id));
        icon.set_valign(gtk::Align::Center);
        icon.add_css_class("dim-label");
        row_box.append(&icon);
        let label = body_label(&item.label);
        label.set_halign(gtk::Align::Start);
        label.set_hexpand(true);
        label.add_css_class("hyprland-settings-nav-row-label");
        row_box.append(&label);

        // Pending-change count pill: amber, hidden while the page has no
        // unsaved (previewed-live) changes.
        let badge = gtk::Label::new(None);
        badge.set_halign(gtk::Align::End);
        badge.set_valign(gtk::Align::Center);
        badge.add_css_class("hyprland-settings-sidebar-badge");
        badge.set_visible(false);
        badge.set_widget_name(&format!(
            "hyprland-settings-nav-badge-{}",
            safe_widget_name(&item.id)
        ));
        row_box.append(&badge);
        badges.push((item.id.clone(), badge));

        row.set_child(Some(&row_box));
        sidebar.append(&row);
    }

    (sidebar, badges)
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

fn dashboard_cards() -> [DashboardCard; 9] {
    [
        DashboardCard {
            title: "Settings",
            description: "Choose which Hyprland config the app reviews and where future changes should be saved.",
            target_tab_id: CONFIG_ID,
        },
        DashboardCard {
            title: "General",
            description: "Change gaps, borders, border colors, layout, and snapping.",
            target_tab_id: "general",
        },
        DashboardCard {
            title: "Animations",
            description: "Tune animation settings; supervised live preview with automatic revert is available here.",
            target_tab_id: "animations",
        },
        DashboardCard {
            title: "Decoration",
            description: "Change rounding, opacity, blur, and shadows.",
            target_tab_id: "decoration",
        },
        DashboardCard {
            title: "Devices",
            description: "Adjust keyboard, mouse, touchpad, and focus behavior.",
            target_tab_id: "devices",
        },
        DashboardCard {
            title: "Monitors",
            description: "Review monitor, rendering, color, and display-related options.",
            target_tab_id: "monitors",
        },
        DashboardCard {
            title: "Keybinds",
            description: "Browse keybind-related settings and preserved shortcut entries.",
            target_tab_id: "keybinds",
        },
        DashboardCard {
            title: "System",
            description: "Review settings that need extra care before changing.",
            target_tab_id: "system",
        },
        DashboardCard {
            title: "Safety Details",
            description: "Proof receipts, safety evidence, and review-only activation cards.",
            target_tab_id: SAFETY_ID,
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

    content.append(&title_label("Settings"));
    content.append(&body_label(
        "Where the app reads your Hyprland config and how saving behaves.",
    ));

    content.append(&config_file_selection_section(
        &model.config_discovery,
        window,
        selection_state,
    ));

    content.append(&safe_live_save_mode_section(model));

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

/// The Safety Details page: every proof receipt, safety evidence, review,
/// and activation surface, moved out of the everyday path. Nothing here is
/// weakened or removed — the same widgets and assertions live one
/// navigation hop away so the normal pages stay settings-first.
fn build_safety_details_view(model: &UiProjection) -> gtk::ScrolledWindow {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 14);
    content.set_widget_name("hyprland-settings-safety-details-content");
    content.set_margin_top(4);
    content.set_margin_bottom(16);
    content.set_margin_start(4);
    content.set_margin_end(4);

    content.append(&title_label("Safety Details"));
    content.append(&body_label(
        "Proof receipts, safety evidence, and review-only activation cards — plus the supervised record workbench, which runs through the same gates as the Animations row menus.",
    ));

    content.append(&connected_files_review_section(&model.config_discovery));

    content.append(&structured_family_preview_controls_section(model));

    content.append(&profile_mode_detail_section());

    content.append(&structured_family_editor_section(
        &model.structured_families,
    ));

    content.append(&disabled_future_approval_cards_section());

    content.append(&controlled_write_and_active_pilot_status_section());

    content.append(&runtime_preview_readiness_section());

    content.append(&structured_family_runtime_preview_status_section());

    gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&content)
        .build()
}

/// The active config path the next save would write, when one is
/// detected. Read-only helper for the pending-changes review surfaces.
fn pending_target_config_path(discovery: &ConfigDiscovery) -> Option<std::path::PathBuf> {
    match &discovery.status {
        crate::config_discovery::ConfigDiscoveryStatus::Found { path, .. } => Some(path.clone()),
        _ => None,
    }
}

/// The staged next-save inputs for every pending change: proposed value
/// plus the config line the save would replace (None appends).
fn pending_next_save_changes(
    model: &UiProjection,
    snapshots: &[PendingChangeSnapshot],
    target_path: &std::path::Path,
) -> Vec<crate::pending_changes_ui::NextSaveChange> {
    snapshots
        .iter()
        .map(|snapshot| {
            let current = model
                .settings
                .iter()
                .find(|setting| setting.row_id == snapshot.row_id)
                .map(|setting| setting.current_value.clone());
            let line_in_target = current.as_ref().and_then(|value| {
                if value.source_path.as_deref() == Some(target_path) {
                    value.line_number
                } else {
                    None
                }
            });
            crate::pending_changes_ui::NextSaveChange {
                setting_id: snapshot.row_id.clone(),
                proposed_value: snapshot.current_value.clone(),
                line_in_target,
            }
        })
        .collect()
}

/// The bottom unsaved-changes action bar: a slide-up strip with the
/// warning icon and "Unsaved changes — applied live, not saved to disk"
/// status on the left and Discard / Save now (split button) on the right.
/// Save now routes every pending row through the existing per-row gated
/// scalar save (Safe Live Save Mode verified per write, fails closed);
/// Discard reverts every pending preview through its own controller. The
/// split menu's "Save as new profile" stays disabled: profile saving has
/// no enabled production behavior.
fn build_pending_bottom_bar(model: &Rc<UiProjection>) -> (gtk::Revealer, Rc<dyn Fn()>) {
    let revealer = gtk::Revealer::new();
    revealer.set_transition_type(gtk::RevealerTransitionType::SlideUp);
    revealer.set_reveal_child(false);
    revealer.set_widget_name("hyprland-settings-pending-bar");

    let bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    bar.add_css_class("toolbar");
    bar.set_margin_start(12);
    bar.set_margin_end(12);
    bar.set_margin_top(4);
    bar.set_margin_bottom(4);

    let icon = gtk::Image::from_icon_name("dialog-warning-symbolic");
    icon.add_css_class("warning");
    bar.append(&icon);

    let status_label = gtk::Label::new(Some("Unsaved changes — applied live, not saved to disk"));
    status_label.set_hexpand(true);
    status_label.set_xalign(0.0);
    status_label.set_widget_name("hyprland-settings-pending-bar-status");
    bar.append(&status_label);

    let discard_button = gtk::Button::with_label("Discard");
    discard_button.set_widget_name("hyprland-settings-pending-discard");
    bar.append(&discard_button);

    let save_split = adw::SplitButton::new();
    save_split.set_label("Save now");
    save_split.add_css_class("suggested-action");
    save_split.set_widget_name("hyprland-settings-pending-save-now");
    {
        // Split menu mirrors the reference affordance; the profile action
        // is registered but disabled — profile saving is not enabled.
        let menu = gtk::gio::Menu::new();
        menu.append(
            Some("Save as new profile"),
            Some("pendingbar.save-as-new-profile"),
        );
        let group = gtk::gio::SimpleActionGroup::new();
        let action = gtk::gio::SimpleAction::new("save-as-new-profile", None);
        action.set_enabled(false);
        group.add_action(&action);
        save_split.insert_action_group("pendingbar", Some(&group));
        save_split.set_menu_model(Some(&menu));
    }
    bar.append(&save_split);
    revealer.set_child(Some(&bar));

    let saved_state = Rc::new(std::cell::Cell::new(false));

    // Refresh: reveal while pending changes exist, restoring the default
    // "unsaved" presentation whenever the pending set changes.
    let refresh: Rc<dyn Fn()> = {
        let revealer = revealer.clone();
        let status_label = status_label.clone();
        let icon = icon.clone();
        let discard_button = discard_button.clone();
        let save_split = save_split.clone();
        let saved_state = Rc::clone(&saved_state);
        Rc::new(move || {
            let count = pending_change_snapshots().len();
            if count > 0 {
                saved_state.set(false);
                status_label.set_label("Unsaved changes — applied live, not saved to disk");
                icon.set_icon_name(Some("dialog-warning-symbolic"));
                icon.remove_css_class("accent");
                icon.add_css_class("warning");
                discard_button.set_visible(true);
                discard_button.set_sensitive(true);
                save_split.set_sensitive(true);
                revealer.set_reveal_child(true);
            } else if !saved_state.get() {
                revealer.set_reveal_child(false);
            }
        })
    };

    {
        let saved_state = Rc::clone(&saved_state);
        discard_button.connect_clicked(move |_| {
            saved_state.set(false);
            for snapshot in pending_change_snapshots() {
                if let Some(controller) = snapshot.controller.upgrade() {
                    if let Ok(mut controller) = controller.try_borrow_mut() {
                        let outcome = controller.revert();
                        if std::env::var("HYPRLAND_SETTINGS_DEBUG_PENDING").is_ok() {
                            eprintln!(
                                "pending-debug: discard revert {} -> {:?}",
                                snapshot.row_id,
                                outcome.as_ref().map(|_| "ok").map_err(|e| e.user_text())
                            );
                        }
                    } else if std::env::var("HYPRLAND_SETTINGS_DEBUG_PENDING").is_ok() {
                        eprintln!("pending-debug: discard borrow failed {}", snapshot.row_id);
                    }
                }
            }
            notify_pending_changed();
        });
    }

    {
        let model = Rc::clone(model);
        let status_label = status_label.clone();
        let icon = icon.clone();
        let discard_button = discard_button.clone();
        let revealer = revealer.clone();
        let saved_state = Rc::clone(&saved_state);
        save_split.connect_clicked(move |split| {
            let snapshots = pending_change_snapshots();
            if snapshots.is_empty() {
                return;
            }
            let total = snapshots.len();
            let mut saved = 0usize;
            let mut first_error: Option<String> = None;
            for snapshot in &snapshots {
                match crate::production_save::gated_scalar_save_live(
                    model.known_setting_ids.clone(),
                    &model.config_discovery,
                    &model.current_config,
                    &snapshot.row_id,
                    &snapshot.current_value,
                ) {
                    Ok(_) => {
                        saved += 1;
                        if let Some(controller) = snapshot.controller.upgrade() {
                            if let Ok(mut controller) = controller.try_borrow_mut() {
                                let _ = controller.mark_saved();
                            }
                        }
                    }
                    Err(reason) => {
                        first_error = Some(reason);
                        break;
                    }
                }
            }
            match first_error {
                None => {
                    // Saved state: brief confirmation, then slide away.
                    saved_state.set(true);
                    status_label.set_label("Changes saved to disk");
                    icon.set_icon_name(Some("emblem-ok-symbolic"));
                    icon.remove_css_class("warning");
                    icon.add_css_class("accent");
                    discard_button.set_visible(false);
                    split.set_sensitive(false);
                    let revealer = revealer.clone();
                    let saved_state = Rc::clone(&saved_state);
                    gtk::glib::timeout_add_local_once(
                        std::time::Duration::from_millis(1500),
                        move || {
                            saved_state.set(false);
                            revealer.set_reveal_child(false);
                        },
                    );
                }
                Some(reason) => {
                    status_label.set_label(&format!("Saved {saved} of {total} — {reason}"));
                }
            }
            notify_pending_changed();
        });
    }

    (revealer, refresh)
}

/// The hidden Pending Changes review page: a large unsaved-change count,
/// grouped change rows (icon, friendly label, key · value subtitle,
/// Added/Modified pill, revert button, navigation chevron), a config diff
/// preview of what the next gated save would write, and a calm
/// no-pending-changes empty state. Reached from the header chip, not the
/// sidebar. Review only: the page renders ledger state and never writes.
fn build_pending_changes_view(
    model: &Rc<UiProjection>,
    navigate_to_page: Rc<dyn Fn(&str)>,
) -> (gtk::ScrolledWindow, Rc<dyn Fn()>) {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 14);
    content.set_widget_name("hyprland-settings-pending-changes-content");
    content.set_margin_top(8);
    content.set_margin_bottom(24);

    let summary = gtk::Label::new(None);
    summary.set_xalign(0.0);
    summary.add_css_class("title-2");
    summary.set_widget_name("hyprland-settings-pending-summary");
    content.append(&summary);

    // Calm empty state, shown only when nothing is pending.
    let empty_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    empty_box.set_valign(gtk::Align::Center);
    empty_box.set_halign(gtk::Align::Center);
    empty_box.set_margin_top(90);
    empty_box.set_margin_bottom(40);
    empty_box.set_widget_name("hyprland-settings-pending-empty-state");
    let empty_icon = gtk::Image::from_icon_name("emblem-ok-symbolic");
    empty_icon.set_pixel_size(96);
    empty_icon.add_css_class("dim-label");
    empty_box.append(&empty_icon);
    let empty_title = title_label("No Pending Changes");
    empty_title.set_halign(gtk::Align::Center);
    empty_box.append(&empty_title);
    let empty_body =
        body_label("Changes previewed on any page appear here for review before saving.");
    empty_body.set_halign(gtk::Align::Center);
    empty_body.add_css_class("dim-label");
    empty_box.append(&empty_body);
    content.append(&empty_box);

    let groups_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
    groups_box.set_widget_name("hyprland-settings-pending-groups");
    content.append(&groups_box);

    // Config diff preview section (hidden while the next save would write
    // nothing new).
    let diff_section = gtk::Box::new(gtk::Orientation::Vertical, 4);
    diff_section.set_widget_name("hyprland-settings-pending-diff-section");
    let diff_heading = body_label("Config diff preview");
    diff_heading.set_halign(gtk::Align::Start);
    diff_heading.set_margin_top(14);
    diff_heading.add_css_class("heading");
    diff_section.append(&diff_heading);
    let diff_caption =
        small_label("What the next gated save would write, compared with the saved config.");
    diff_caption.set_halign(gtk::Align::Start);
    diff_caption.add_css_class("dim-label");
    diff_section.append(&diff_caption);
    let diff_card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    diff_card.add_css_class("hyprland-settings-diff-card");
    diff_card.set_margin_top(8);
    diff_section.append(&diff_card);
    content.append(&diff_section);

    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(800);
    clamp.set_tightening_threshold(600);
    clamp.set_margin_top(12);
    clamp.set_margin_bottom(12);
    clamp.set_margin_start(12);
    clamp.set_margin_end(12);
    clamp.set_child(Some(&content));

    let refresh: Rc<dyn Fn()> = {
        let model = Rc::clone(model);
        let summary = summary.clone();
        let empty_box = empty_box.clone();
        let groups_box = groups_box.clone();
        let diff_section = diff_section.clone();
        let diff_card = diff_card.clone();
        let navigate_to_page = Rc::clone(&navigate_to_page);
        Rc::new(move || {
            let snapshots = pending_change_snapshots();
            let count = snapshots.len();
            summary.set_visible(count > 0);
            summary.set_label(&crate::pending_changes_ui::pending_summary_title(count));
            empty_box.set_visible(count == 0);

            while let Some(child) = groups_box.first_child() {
                groups_box.remove(&child);
            }

            // Group change rows by their page, in sidebar order.
            let mut grouped: Vec<(&'static str, String, Vec<&PendingChangeSnapshot>)> = Vec::new();
            for snapshot in &snapshots {
                let page_id = snapshot.page_id.unwrap_or("general");
                let label = crate::ux_presentation::page_spec(page_id)
                    .map(|page| page.label.to_string())
                    .unwrap_or_else(|| "Options".to_string());
                match grouped.iter_mut().find(|(id, _, _)| *id == page_id) {
                    Some((_, _, rows)) => rows.push(snapshot),
                    None => grouped.push((page_id, label, vec![snapshot])),
                }
            }

            for (page_id, page_label, rows) in &grouped {
                let heading = body_label(page_label);
                heading.set_halign(gtk::Align::Start);
                heading.set_margin_top(14);
                heading.set_margin_start(6);
                heading.add_css_class("heading");
                groups_box.append(&heading);
                let caption = small_label(&crate::pending_changes_ui::pending_group_caption(
                    rows.len(),
                ));
                caption.set_halign(gtk::Align::Start);
                caption.set_margin_start(6);
                caption.add_css_class("dim-label");
                groups_box.append(&caption);

                let list = gtk::ListBox::new();
                list.set_selection_mode(gtk::SelectionMode::None);
                list.add_css_class("boxed-list");
                list.set_margin_top(4);
                for snapshot in rows {
                    let row = gtk::ListBoxRow::new();
                    row.set_activatable(false);
                    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                    row_box.set_margin_top(8);
                    row_box.set_margin_bottom(8);
                    row_box.set_margin_start(12);
                    row_box.set_margin_end(12);

                    let icon =
                        gtk::Image::from_icon_name(crate::ux_presentation::page_icon(page_id));
                    icon.set_valign(gtk::Align::Center);
                    icon.add_css_class("dim-label");
                    row_box.append(&icon);

                    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
                    text_box.set_hexpand(true);
                    // Same friendly title resolution the setting rows use.
                    let friendly =
                        match crate::presentation_labels::display_label_for_row(&snapshot.row_id) {
                            Some(matched) => matched.to_string(),
                            None => model
                                .settings
                                .iter()
                                .find(|setting| setting.row_id == snapshot.row_id)
                                .map(|setting| {
                                    crate::ux_presentation::row_display_title(
                                        &setting.label,
                                        &setting.tab_label,
                                        &setting.official_setting,
                                    )
                                })
                                .unwrap_or_else(|| snapshot.row_id.clone()),
                        };
                    let title = body_label(&friendly);
                    title.set_halign(gtk::Align::Start);
                    text_box.append(&title);
                    let subtitle =
                        small_label(&crate::pending_changes_ui::pending_change_subtitle(
                            &snapshot.official_setting,
                            &snapshot.current_value,
                        ));
                    subtitle.set_halign(gtk::Align::Start);
                    subtitle.add_css_class("dim-label");
                    text_box.append(&subtitle);
                    row_box.append(&text_box);

                    // Added / Modified pill from what the save would do.
                    let config_has_line = model
                        .settings
                        .iter()
                        .find(|setting| setting.row_id == snapshot.row_id)
                        .map(|setting| setting.current_value.raw_value.is_some())
                        .unwrap_or(false);
                    let kind = crate::pending_changes_ui::pending_change_kind(config_has_line);
                    let badge = gtk::Label::new(Some(kind));
                    badge.set_valign(gtk::Align::Center);
                    badge.add_css_class("hyprland-settings-pending-badge");
                    badge.add_css_class(if kind == "Added" {
                        "hyprland-settings-pending-badge-added"
                    } else {
                        "hyprland-settings-pending-badge-modified"
                    });
                    row_box.append(&badge);

                    let revert = gtk::Button::from_icon_name("edit-undo-symbolic");
                    revert.add_css_class("flat");
                    revert.set_valign(gtk::Align::Center);
                    revert.set_widget_name(&format!(
                        "hyprland-settings-pending-revert-{}",
                        safe_widget_name(&snapshot.row_id)
                    ));
                    revert
                        .update_property(&[gtk::accessible::Property::Label("Revert this change")]);
                    {
                        let controller = snapshot.controller.clone();
                        revert.connect_clicked(move |_| {
                            if let Some(controller) = controller.upgrade() {
                                if let Ok(mut controller) = controller.try_borrow_mut() {
                                    let _ = controller.revert();
                                }
                            }
                            notify_pending_changed();
                        });
                    }
                    row_box.append(&revert);

                    let open = gtk::Button::from_icon_name("go-next-symbolic");
                    open.add_css_class("flat");
                    open.set_valign(gtk::Align::Center);
                    open.update_property(&[gtk::accessible::Property::Label(&format!(
                        "Open {page_label}"
                    ))]);
                    {
                        let navigate_to_page = Rc::clone(&navigate_to_page);
                        let page_id = (*page_id).to_string();
                        open.connect_clicked(move |_| navigate_to_page(&page_id));
                    }
                    row_box.append(&open);

                    row.set_child(Some(&row_box));
                    list.append(&row);
                }
                groups_box.append(&list);
            }

            // Diff preview: saved config vs what the next save would write.
            while let Some(child) = diff_card.first_child() {
                diff_card.remove(&child);
            }
            let target_path = pending_target_config_path(&model.config_discovery);
            let preview = target_path.as_ref().and_then(|path| {
                let original = std::fs::read_to_string(path).ok()?;
                let changes = pending_next_save_changes(&model, &snapshots, path);
                let next =
                    crate::pending_changes_ui::next_save_config_text(&original, &changes).ok()?;
                Some((path.clone(), original, next))
            });
            match preview {
                Some((path, original, next)) => {
                    let diff = crate::pending_changes_ui::unified_diff(
                        &original,
                        &next,
                        &path.display().to_string(),
                        &format!("{} (next save)", path.display()),
                    );
                    if diff.is_empty() {
                        diff_section.set_visible(false);
                    } else {
                        diff_section.set_visible(true);
                        // Header strip: file name + added/removed counts.
                        let header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                        header.set_margin_top(10);
                        header.set_margin_bottom(10);
                        header.set_margin_start(12);
                        header.set_margin_end(12);
                        let name = path
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.display().to_string());
                        let file_label = gtk::Label::new(Some(&name));
                        file_label.set_xalign(0.0);
                        file_label.set_hexpand(true);
                        file_label.add_css_class("heading");
                        header.append(&file_label);
                        let added = gtk::Label::new(Some(&format!("+{}", diff.added)));
                        added.add_css_class("hyprland-settings-diff-count-added");
                        header.append(&added);
                        let removed = gtk::Label::new(Some(&format!("−{}", diff.removed)));
                        removed.add_css_class("hyprland-settings-diff-count-removed");
                        header.append(&removed);
                        diff_card.append(&header);
                        diff_card.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

                        const MAX_DIFF_LINES: usize = 400;
                        for line in diff.lines.iter().take(MAX_DIFF_LINES) {
                            let label = gtk::Label::new(Some(&line.text));
                            label.set_xalign(0.0);
                            label.set_wrap(false);
                            label.add_css_class("hyprland-settings-diff-line");
                            label.add_css_class(match line.kind {
                                crate::pending_changes_ui::DiffLineKind::Added => {
                                    "hyprland-settings-diff-added"
                                }
                                crate::pending_changes_ui::DiffLineKind::Removed => {
                                    "hyprland-settings-diff-removed"
                                }
                                crate::pending_changes_ui::DiffLineKind::Meta
                                | crate::pending_changes_ui::DiffLineKind::Hunk => {
                                    "hyprland-settings-diff-meta"
                                }
                                crate::pending_changes_ui::DiffLineKind::Context => {
                                    "hyprland-settings-diff-context"
                                }
                            });
                            diff_card.append(&label);
                        }
                        if diff.lines.len() > MAX_DIFF_LINES {
                            let truncated = small_label("… diff truncated");
                            truncated.set_margin_start(12);
                            truncated.set_margin_bottom(8);
                            diff_card.append(&truncated);
                        }
                        let bottom_pad = gtk::Box::new(gtk::Orientation::Vertical, 0);
                        bottom_pad.set_margin_bottom(8);
                        diff_card.append(&bottom_pad);
                    }
                }
                None => diff_section.set_visible(false),
            }
        })
    };
    refresh();

    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&clamp)
        .build();
    (scroll, refresh)
}

/// Profiles page: a friendly centered empty state. Profile switching has
/// no enabled production behavior, so the action stays inert — the page is
/// presentation only and introduces no symlink/config path.
fn build_profiles_view() -> gtk::ScrolledWindow {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 10);
    content.set_widget_name("hyprland-settings-profiles-content");
    content.set_valign(gtk::Align::Center);
    content.set_halign(gtk::Align::Center);
    content.set_vexpand(true);

    let icon = gtk::Image::from_icon_name("folder-symbolic");
    icon.set_pixel_size(96);
    content.append(&icon);

    let title = title_label("No Profiles");
    title.set_halign(gtk::Align::Center);
    content.append(&title);

    let body = body_label(
        "Profiles will let you save and switch between complete setups. Switching is not enabled yet.",
    );
    body.set_halign(gtk::Align::Center);
    content.append(&body);

    let action = gtk::Button::with_label("Save Current as Profile");
    action.add_css_class("pill");
    action.set_halign(gtk::Align::Center);
    action.set_sensitive(false);
    action.set_widget_name("hyprland-settings-profiles-save-disabled");
    content.append(&action);

    gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&content)
        .build()
}

/// Layouts page: one page for the dwindle/master/scrolling layout rows
/// with top tabs, presentation-only — the rows are the same model rows
/// with the same classifications, raw keys, and save/preview behavior.
fn build_layouts_view(model: &Rc<UiProjection>) -> gtk::ScrolledWindow {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_widget_name("hyprland-settings-layouts-content");
    content.append(&title_label("Layouts"));

    let tabs = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    tabs.set_halign(gtk::Align::Center);
    tabs.add_css_class("linked");
    let list = gtk::ListBox::new();
    list.set_widget_name("hyprland-settings-layouts-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    list.add_css_class("boxed-list");
    list.set_valign(gtk::Align::Start);

    let render_layout = {
        let model = Rc::clone(model);
        let list = list.clone();
        Rc::new(move |prefix: &str| {
            while let Some(row) = list.row_at_index(0) {
                list.remove(&row);
            }
            let mut results: Vec<SearchResult> = model
                .settings_for_tab("windows-layout")
                .into_iter()
                .filter(|setting| setting.row_id.starts_with(prefix))
                .map(|setting| SearchResult {
                    setting,
                    rank: None,
                })
                .collect();
            results.sort_by_key(|result| result.setting.row_order);
            if results.is_empty() {
                let empty = gtk::ListBoxRow::new();
                empty.set_selectable(false);
                empty.set_child(Some(&small_label(
                    "No settings for this layout in the current model.",
                )));
                list.append(&empty);
                return;
            }
            for result in &results {
                list.append(&build_setting_row(result, false));
            }
        })
    };

    let mut first_button: Option<gtk::ToggleButton> = None;
    for (label, prefix) in [
        ("Dwindle", "dwindle."),
        ("Master", "master."),
        ("Scrolling", "scrolling."),
    ] {
        let button = gtk::ToggleButton::with_label(label);
        button.set_widget_name(&format!(
            "hyprland-settings-layouts-tab-{}",
            label.to_ascii_lowercase()
        ));
        if let Some(first) = &first_button {
            button.set_group(Some(first));
        } else {
            first_button = Some(button.clone());
        }
        let render_layout = render_layout.clone();
        button.connect_toggled(move |button| {
            if button.is_active() {
                render_layout(prefix);
            }
        });
        tabs.append(&button);
    }
    content.append(&tabs);
    if let Some(first) = &first_button {
        first.set_active(true);
    }

    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(800);
    clamp.set_tightening_threshold(600);
    clamp.set_margin_top(6);
    clamp.set_margin_bottom(24);
    clamp.set_child(Some(&list));
    content.append(&clamp);

    gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&content)
        .build()
}

/// A centered empty state used by pages whose editing surface is not
/// safely available yet. Honest: it explains where entries come from and
/// adds no write path.
fn empty_state_view(
    widget_name: &str,
    icon_name: &str,
    heading: &str,
    text: &str,
) -> gtk::ScrolledWindow {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 10);
    content.set_widget_name(widget_name);
    content.set_valign(gtk::Align::Center);
    content.set_halign(gtk::Align::Center);
    content.set_vexpand(true);
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_pixel_size(96);
    content.append(&icon);
    let title = title_label(heading);
    title.set_halign(gtk::Align::Center);
    content.append(&title);
    let body = body_label(text);
    body.set_halign(gtk::Align::Center);
    body.set_wrap(true);
    content.append(&body);
    gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&content)
        .build()
}

/// Source-file-grouped, read-only list of preserved structured config
/// entries for one family. Locked rows: raw line, source path and line
/// number, a lock icon, and no edit affordance — the entries stay exactly
/// as written in the user's files.
fn structured_locked_list_view(
    model: &UiProjection,
    family_id: &str,
    widget_name: &str,
    heading: &str,
    explanation: &str,
    empty_heading: &str,
    empty_text: &str,
) -> gtk::ScrolledWindow {
    let entries: Vec<crate::ui::model::UiStructuredEntry> = model
        .structured_families
        .iter()
        .filter(|family| family.family_id == family_id)
        .flat_map(|family| family.entries.iter().cloned())
        .collect();
    if entries.is_empty() {
        return empty_state_view(
            widget_name,
            "text-x-generic-symbolic",
            empty_heading,
            empty_text,
        );
    }

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_widget_name(widget_name);
    content.append(&title_label(heading));
    let explain = small_label(explanation);
    explain.set_halign(gtk::Align::Start);
    content.append(&explain);

    // Group by source file, preserving order.
    let mut groups: Vec<(String, Vec<crate::ui::model::UiStructuredEntry>)> = Vec::new();
    for entry in entries {
        match groups
            .iter_mut()
            .find(|(path, _)| *path == entry.source_path)
        {
            Some((_, list)) => list.push(entry),
            None => groups.push((entry.source_path.clone(), vec![entry])),
        }
    }
    for (path, list_entries) in groups {
        let heading = small_label(&path);
        heading.set_halign(gtk::Align::Start);
        heading.set_margin_top(10);
        heading.add_css_class("heading");
        content.append(&heading);
        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::None);
        list.add_css_class("boxed-list");
        for entry in list_entries {
            let row = gtk::ListBoxRow::new();
            row.set_activatable(false);
            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            row_box.set_margin_top(6);
            row_box.set_margin_bottom(6);
            row_box.set_margin_start(12);
            row_box.set_margin_end(12);
            let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
            text_box.set_hexpand(true);
            let line = body_label(entry.raw_line.trim());
            line.set_halign(gtk::Align::Start);
            text_box.append(&line);
            let origin = small_label(&format!("line {}", entry.line_number));
            origin.set_halign(gtk::Align::Start);
            text_box.append(&origin);
            row_box.append(&text_box);
            let lock = gtk::Image::from_icon_name("system-lock-screen-symbolic");
            lock.set_valign(gtk::Align::Center);
            row_box.append(&lock);
            row.set_child(Some(&row_box));
            list.append(&row);
        }
        content.append(&list);
    }

    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(800);
    clamp.set_tightening_threshold(600);
    clamp.set_margin_top(12);
    clamp.set_margin_bottom(24);
    clamp.set_margin_start(12);
    clamp.set_margin_end(12);
    clamp.set_child(Some(&content));
    gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&clamp)
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

fn structured_family_editor_section(
    families: &[crate::ui::model::UiStructuredFamily],
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-structured-family-section");
    frame.set_tooltip_text(Some(
        "Structured family editors. These projections are review-only and cannot write config or reload Hyprland.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    content.append(&title_label("Structured family editors"));
    content.append(&body_label(
        "These editors are available as review-only projections.",
    ));
    content.append(&small_label("Writes are blocked by default."));
    content.append(&small_label(
        "Fixture parse/render proof is available where shown.",
    ));
    content.append(&small_label("Real config writes are not active."));
    content.append(&small_label(
        "Family-specific validator and temp-fixture write plan proof remain review-only.",
    ));

    for family in families {
        content.append(&structured_family_card(family));
    }
    content.append(&structured_family_record_editor_section(families));
    content.append(&structured_family_record_draft_section(families));
    content.append(&structured_family_record_draft_binding_section(families));

    frame.set_child(Some(&content));
    frame
}

fn structured_family_card(family: &crate::ui::model::UiStructuredFamily) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&family.widget_name);
    frame.set_tooltip_text(Some(
        "Structured family review card. This is review-only and cannot write config, mutate runtime, or reload Hyprland.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&family.label));
    append_detail_line(&content, "Syntax", &family.syntax_description);
    append_detail_line(&content, "Projection status", &family.projection_status);
    append_detail_line(
        &content,
        "Fixture parse proof",
        &family.fixture_parse_proof_status,
    );
    append_detail_line(
        &content,
        "Fixture render proof",
        &family.fixture_render_proof_status,
    );
    append_detail_line(
        &content,
        "Family-specific validator",
        &family.family_specific_validation_status,
    );
    append_detail_line(
        &content,
        "Temp-fixture write plan",
        &family.temp_write_plan_status,
    );
    content.append(&small_label("Temp-fixture write plan validated"));
    append_detail_line(
        &content,
        "Temp-fixture render/reread proof",
        &family.temp_fixture_render_reread_status,
    );
    content.append(&small_label("Temp-fixture render/reread verified"));
    append_detail_line(&content, "Path guard", &family.path_guard_status);
    append_detail_line(
        &content,
        "Write status",
        &format!(
            "{}; Production writes blocked by default",
            family.write_status
        ),
    );
    append_detail_line(
        &content,
        "Records in current config",
        &family.entries.len().to_string(),
    );
    append_detail_line(
        &content,
        "Not proven yet",
        &family.unproven_record_count.to_string(),
    );
    content.append(&small_label(&format!(
        "Field schema: {}",
        family.field_schema.join(", ")
    )));
    content.append(&small_label(
        "Structured family editor projection cannot write real config.",
    ));
    content.append(&small_label(&family.real_config_target_status));
    content.append(&small_label(
        "Structured family editor projection cannot reload Hyprland.",
    ));
    content.append(&small_label(&family.reload_status));
    content.append(&small_label(
        "Structured family editor projection cannot mutate runtime.",
    ));
    content.append(&small_label(&family.runtime_mutation_status));

    let action = gtk::Button::with_label(&family.review_button_label);
    action.set_sensitive(false);
    action.set_tooltip_text(Some(
        "Review-only control. No config write, runtime mutation, reload, or executor callback is connected.",
    ));
    content.append(&action);

    frame.set_child(Some(&content));
    frame
}

fn structured_family_record_editor_section(
    families: &[crate::ui::model::UiStructuredFamily],
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-structured-family-record-editor-section");
    frame.set_tooltip_text(Some(
        "Review-only per-record structured family editor forms. Real writes remain blocked.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(10);
    content.set_margin_end(10);

    content.append(&title_label("Review-only per-record editor forms"));
    content.append(&small_label("Record editor projection ready"));
    content.append(&small_label("Family-specific fields projected"));
    content.append(&small_label("Raw fallback required where not proven"));
    content.append(&small_label("Editor actions disabled"));
    content.append(&small_label("Real writes blocked by default"));
    content.append(&small_label("Production writes blocked by default"));
    content.append(&small_label("Real config target not allowed"));
    content.append(&small_label("Runtime mutation not allowed"));
    content.append(&small_label("Hyprland reload not allowed"));

    for family in families {
        content.append(&structured_family_record_editor_family_card(family));
    }

    frame.set_child(Some(&content));
    frame
}

fn structured_family_record_editor_family_card(
    family: &crate::ui::model::UiStructuredFamily,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&family.record_editor_widget_name);
    frame.set_tooltip_text(Some(
        "Family record editor projection. This is review-only and has no write, reload, runtime, persistence, or executor callback.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&format!("{} record editor", family.label)));
    append_detail_line(&content, "Status", &family.record_editor_status);
    append_detail_line(
        &content,
        "Action policy",
        &family.record_editor_action_policy_status,
    );
    append_detail_line(
        &content,
        "Form count",
        &family.record_editor_form_count.to_string(),
    );

    if family.record_editor_forms.is_empty() {
        content.append(&small_label("No records available for review-only forms."));
    }

    for form in &family.record_editor_forms {
        content.append(&small_label(&format!("Record {}", form.record_index + 1)));
        append_detail_line(&content, "source path", &form.source_path);
        append_detail_line(&content, "line number", &form.line_number.to_string());
        append_detail_line(&content, "raw line", &form.raw_line);
        append_detail_line(&content, "validation status", &form.validation_status);
        append_detail_line(&content, "field count", &form.field_count.to_string());
        append_detail_line(&content, "raw fallback status", &form.raw_fallback_status);
        append_detail_line(&content, "write policy", &form.write_policy);
        append_detail_line(
            &content,
            "temp-fixture plan status",
            &form.temp_fixture_plan_status,
        );
        if let Some(reason) = &form.unsupported_reason {
            append_detail_line(&content, "not proven reason", reason);
        }
    }

    let edit = gtk::Button::with_label(&family.disabled_record_edit_label);
    edit.set_sensitive(false);
    edit.set_tooltip_text(Some("Review-only placeholder. Editing is not available."));
    content.append(&edit);

    let apply = gtk::Button::with_label("Apply structured-family record change (not available)");
    apply.set_sensitive(false);
    apply.set_tooltip_text(Some("Disabled. This button has no write callback."));
    content.append(&apply);

    let render =
        gtk::Button::with_label("Render structured-family record to real config (not available)");
    render.set_sensitive(false);
    render.set_tooltip_text(Some(
        "Disabled. Real config render targets are not allowed.",
    ));
    content.append(&render);

    frame.set_child(Some(&content));
    frame
}

fn structured_family_record_draft_section(
    families: &[crate::ui::model::UiStructuredFamily],
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-structured-family-record-draft-section");
    frame.set_tooltip_text(Some(
        "Review-only in-memory structured family record drafts. Persistence and real writes remain blocked.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(10);
    content.set_margin_end(10);

    content.append(&title_label("Review-only structured-family record drafts"));
    content.append(&small_label("Draft projection ready"));
    content.append(&small_label("Draft created in memory only"));
    content.append(&small_label("Draft starts clean"));
    content.append(&small_label("Draft dirty state tracked"));
    content.append(&small_label("Draft reset proof available"));
    content.append(&small_label("Draft validation ready"));
    content.append(&small_label("Raw fallback required where not proven"));
    content.append(&small_label("Draft actions disabled"));
    content.append(&small_label("Draft persistence forbidden"));
    content.append(&small_label("Real writes blocked by default"));
    content.append(&small_label("Production writes blocked by default"));
    content.append(&small_label("Real config target not allowed"));
    content.append(&small_label("Runtime mutation not allowed"));
    content.append(&small_label("Hyprland reload not allowed"));

    for family in families {
        content.append(&structured_family_record_draft_family_card(family));
    }

    frame.set_child(Some(&content));
    frame
}

fn structured_family_record_draft_family_card(
    family: &crate::ui::model::UiStructuredFamily,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&family.record_draft_widget_name);
    frame.set_tooltip_text(Some(
        "Family record draft projection. This is in-memory and review-only with no persistence, write, reload, runtime, or executor callback.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&format!("{} record drafts", family.label)));
    append_detail_line(&content, "family", &family.family_id);
    append_detail_line(&content, "record count", &family.entries.len().to_string());
    append_detail_line(
        &content,
        "draft count",
        &family.record_draft_count.to_string(),
    );
    append_detail_line(
        &content,
        "dirty draft count",
        &family.dirty_draft_count.to_string(),
    );
    append_detail_line(
        &content,
        "raw fallback draft count",
        &family.raw_fallback_draft_count.to_string(),
    );
    append_detail_line(&content, "write policy", &family.write_status);
    append_detail_line(
        &content,
        "persistence policy",
        &family.record_draft_persistence_policy_status,
    );
    append_detail_line(
        &content,
        "action policy",
        &family.record_draft_action_policy_status,
    );

    for draft in &family.record_drafts {
        content.append(&small_label(&format!("Draft {}", draft.record_index + 1)));
        append_detail_line(&content, "dirty state", &draft.dirty_state);
        append_detail_line(&content, "validation status", &draft.validation_status);
        append_detail_line(&content, "raw fallback status", &draft.raw_fallback_status);
        append_detail_line(&content, "reset status", &draft.reset_status);
        append_detail_line(&content, "draft written to disk", "false");
        if let Some(reason) = &draft.unsupported_reason {
            append_detail_line(&content, "not proven reason", reason);
        }
    }

    let update = gtk::Button::with_label(&family.disabled_record_draft_update_label);
    update.set_sensitive(false);
    update.set_tooltip_text(Some(
        "Review-only placeholder. Draft field updates are model-only and not available in GTK.",
    ));
    content.append(&update);

    let reset = gtk::Button::with_label("Reset structured-family draft (not available)");
    reset.set_sensitive(false);
    reset.set_tooltip_text(Some("Disabled. This button has no reset callback."));
    content.append(&reset);

    let persist = gtk::Button::with_label("Persist structured-family draft (not available)");
    persist.set_sensitive(false);
    persist.set_tooltip_text(Some(
        "Disabled. Structured-family draft persistence is forbidden.",
    ));
    content.append(&persist);

    let apply =
        gtk::Button::with_label("Apply structured-family draft to real config (not available)");
    apply.set_sensitive(false);
    content.append(&apply);

    frame.set_child(Some(&content));
    frame
}

fn structured_family_record_draft_binding_section(
    families: &[crate::ui::model::UiStructuredFamily],
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-structured-family-record-draft-binding-section");
    frame.set_tooltip_text(Some(
        "Disabled live GTK draft-field binding projections. Draft updates are memory-only in model tests and real writes remain blocked.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(10);
    content.set_margin_end(10);

    content.append(&title_label("Disabled live GTK draft-field binding"));
    content.append(&small_label("Draft-field binding projection ready"));
    content.append(&small_label("Draft-field widgets insensitive"));
    content.append(&small_label("Draft-field update is memory-only"));
    content.append(&small_label("Draft dirty state recomputed"));
    content.append(&small_label("Draft validation recomputed"));
    content.append(&small_label("Raw fallback preserved"));
    content.append(&small_label("Draft binding actions disabled"));
    content.append(&small_label("Draft binding persistence forbidden"));
    content.append(&small_label("Real writes blocked by default"));
    content.append(&small_label("Production writes blocked by default"));
    content.append(&small_label("Real config target not allowed"));
    content.append(&small_label("Runtime mutation not allowed"));
    content.append(&small_label("Hyprland reload not allowed"));

    for family in families {
        content.append(&structured_family_record_draft_binding_family_card(family));
    }

    frame.set_child(Some(&content));
    frame
}

fn structured_family_record_draft_binding_family_card(
    family: &crate::ui::model::UiStructuredFamily,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&family.record_draft_binding_widget_name);
    frame.set_tooltip_text(Some(
        "Family draft-field binding projection. All widgets are insensitive and no write, reload, runtime, persistence, or executor callback is connected.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&format!(
        "{} draft-field binding",
        family.label
    )));
    append_detail_line(&content, "family", &family.family_id);
    append_detail_line(&content, "record count", &family.entries.len().to_string());
    append_detail_line(
        &content,
        "draft count",
        &family.record_draft_count.to_string(),
    );
    append_detail_line(
        &content,
        "bound field count",
        &family.bound_field_count.to_string(),
    );
    append_detail_line(
        &content,
        "insensitive widget count",
        &family.insensitive_widget_count.to_string(),
    );
    append_detail_line(
        &content,
        "dirty state",
        "StructuredFamilyRecordDraftGtkBindingDirtyStateRecomputed",
    );
    append_detail_line(
        &content,
        "validation status",
        "StructuredFamilyRecordDraftGtkBindingValidationRecomputed",
    );
    append_detail_line(
        &content,
        "raw fallback status",
        "StructuredFamilyRecordDraftGtkBindingRawFallbackPreserved",
    );
    append_detail_line(&content, "write policy", "Real writes blocked by default");
    append_detail_line(
        &content,
        "persistence policy",
        &family.record_draft_binding_persistence_policy_status,
    );
    append_detail_line(
        &content,
        "action policy",
        &family.record_draft_binding_action_policy_status,
    );

    for binding in &family.record_draft_bindings {
        content.append(&small_label(&format!(
            "Draft binding {}",
            binding.record_index + 1
        )));
        append_detail_line(&content, "binding status", &binding.binding_status);
        append_detail_line(&content, "dirty state", &binding.dirty_state);
        append_detail_line(&content, "validation status", &binding.validation_status);
        append_detail_line(
            &content,
            "raw fallback status",
            &binding.raw_fallback_status,
        );
        for field in binding.fields.iter().take(2) {
            let field_label = format!(
                "{} draft field",
                family
                    .family_id
                    .strip_prefix("hl.")
                    .unwrap_or(&family.family_id)
            );
            content.append(&small_label(&field_label));
            let entry = gtk::Entry::new();
            entry.set_text(&field.display_value);
            entry.set_sensitive(false);
            entry.set_tooltip_text(Some(
                "Insensitive draft-field widget. Model-only tests prove memory-only updates.",
            ));
            content.append(&entry);
            append_detail_line(&content, "field", &field.field_name);
            append_detail_line(&content, "widget kind", &field.widget_kind);
            append_detail_line(&content, "widget sensitive", "false");
        }
    }

    let update = gtk::Button::with_label(&family.disabled_record_draft_binding_update_label);
    update.set_sensitive(false);
    update.set_tooltip_text(Some(
        "Disabled. Draft-field updates are memory-only in model tests.",
    ));
    content.append(&update);

    let reset =
        gtk::Button::with_label("Reset structured-family GTK draft binding (not available)");
    reset.set_sensitive(false);
    content.append(&reset);

    let persist =
        gtk::Button::with_label("Persist structured-family GTK draft binding (not available)");
    persist.set_sensitive(false);
    persist.set_tooltip_text(Some(
        "Disabled. Structured-family draft binding persistence is forbidden.",
    ));
    content.append(&persist);

    let apply = gtk::Button::with_label(
        "Apply structured-family GTK draft binding to real config (not available)",
    );
    apply.set_sensitive(false);
    content.append(&apply);

    frame.set_child(Some(&content));
    frame
}

fn disabled_future_approval_cards_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-disabled-approval-cards-section");
    frame.set_tooltip_text(Some(
        "Disabled future approval reviews. These cards show proof and blockers without enabling production behavior.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.append(&title_label("Future approval reviews"));
    content.append(&body_label(
        "These review cards show proof and blockers for future capabilities.",
    ));
    content.append(&small_label(
        "All planned enable controls are disabled. No production behavior is enabled here.",
    ));

    for card in disabled_future_approval_card_projections() {
        content.append(&disabled_future_approval_card(&card));
    }
    content.append(&production_activation_decision_reviews_section());

    frame.set_child(Some(&content));
    frame
}

fn disabled_future_approval_card(card: &DisabledApprovalCardProjection) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&card.widget_name);
    frame.set_tooltip_text(Some(
        "Disabled approval review card. This is review-only and cannot enable production behavior.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&card.evidence_widget_name);
    content.set_tooltip_text(Some(
        "Approval evidence summary. This card is disabled and has no mutation handler.",
    ));
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&card.heading));
    for line in &card.summary_lines {
        content.append(&small_label(line));
    }
    append_detail_line(&content, "Proof source", &card.proof_record.source);
    append_detail_line(&content, "Proof status", &card.proof_record.status);
    for (label, value) in &card.proof_record.fields {
        append_detail_line(&content, label, value);
    }
    if !card.preconditions.is_empty() {
        content.append(&body_label("Preconditions"));
        for precondition in &card.preconditions {
            append_detail_line(
                &content,
                &precondition.label,
                &format!("{} ({})", precondition.value, precondition.status),
            );
        }
    }
    if !card.restore_evidence.is_empty() {
        content.append(&body_label("Restore and unchanged evidence"));
        for evidence in &card.restore_evidence {
            append_detail_line(&content, &evidence.label, &evidence.status);
        }
    }
    for (label, value) in &card.evidence_lines {
        append_detail_line(&content, label, value);
    }
    append_detail_line(&content, "Production status", &card.production_status);
    content.append(&body_label("Blockers"));
    for blocker in &card.blockers {
        content.append(&small_label(blocker));
    }

    let enable = gtk::Button::with_label(&card.disabled_action_label);
    enable.set_widget_name(&card.disabled_action_widget_name);
    enable.set_tooltip_text(Some(
        "Disabled future action. This does not enable production behavior or run any executor.",
    ));
    enable.set_sensitive(false);
    content.append(&enable);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_decision_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-decision-section");
    frame.set_tooltip_text(Some(
        "Default-disabled future production activation decision reviews. These consume report-backed card data but cannot enable production behavior.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label(
        "Future production activation decision reviews",
    ));
    content.append(&small_label(
        "These reviews use report-backed approval card data as input. Production flags stay disabled.",
    ));

    for review in production_activation_decision_reviews() {
        content.append(&production_activation_decision_review_card(&review));
    }
    content.append(&production_activation_path_reviews_section());

    frame.set_child(Some(&content));
    frame
}

fn production_activation_decision_review_card(
    review: &ProductionActivationDecisionReview,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&review.widget_name);
    frame.set_tooltip_text(Some(
        "Disabled production activation decision review. This is review-only and cannot run production executors.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&review.evidence_widget_name);
    content.set_tooltip_text(Some(
        "Report-backed decision evidence. This review has no mutation handler.",
    ));
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&review.heading));
    append_detail_line(
        &content,
        "Decision status",
        review.status.user_facing_label(),
    );
    append_detail_line(&content, "Decision input source", &review.input_source);
    if !review.required_proof_summary.is_empty() {
        content.append(&body_label("Required proof summary"));
        for proof in &review.required_proof_summary {
            content.append(&small_label(proof));
        }
    }
    content.append(&body_label("Decision blockers"));
    if review.blockers.is_empty() {
        content.append(&small_label(
            "No missing proof blockers; production remains disabled.",
        ));
    } else {
        for blocker in &review.blockers {
            content.append(&small_label(blocker));
        }
    }
    append_detail_line(
        &content,
        &review.production_label,
        &review.production_status,
    );

    let enable = gtk::Button::with_label(&review.disabled_action_label);
    enable.set_widget_name(&review.disabled_action_widget_name);
    enable.set_tooltip_text(Some(
        "Disabled planned activation. This does not enable production behavior.",
    ));
    enable.set_sensitive(false);
    content.append(&enable);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_path_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-path-section");
    frame.set_tooltip_text(Some(
        "Default-disabled production activation path reviews. These show the future steps required before production activation could ever be considered.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label("Future production activation paths"));
    content.append(&small_label(
        "These paths consume approved decisions but do not enable production flags or run executors.",
    ));

    for review in production_activation_path_reviews() {
        content.append(&production_activation_path_review_card(&review));
    }
    content.append(&production_activation_control_reviews_section());
    content.append(&production_activation_form_reviews_section());
    content.append(&production_activation_draft_reviews_section());
    content.append(&production_activation_draft_edit_reviews_section());
    content.append(&production_activation_live_draft_edit_reviews_section());
    content.append(&production_activation_draft_persistence_boundary_section());
    content.append(&production_activation_safety_gate_reviews_section());

    frame.set_child(Some(&content));
    frame
}

fn production_activation_path_review_card(review: &ProductionActivationPathReview) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&review.widget_name);
    frame.set_tooltip_text(Some(
        "Disabled production activation path. This card cannot enable writes or run production executors.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&review.evidence_widget_name);
    content.set_tooltip_text(Some(
        "Activation path evidence. This path is review-only and has no mutation handler.",
    ));
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&review.heading));
    append_detail_line(
        &content,
        "Input decision",
        review.input_decision_status.user_facing_label(),
    );
    append_detail_line(&content, "Proof source", &review.input_proof_source);
    append_detail_line(
        &content,
        "Activation path status",
        review.status.user_facing_label(),
    );
    content.append(&body_label("Required before enabling"));
    for requirement in &review.required_before_enabling {
        content.append(&small_label(requirement));
    }
    content.append(&body_label("Activation path blockers"));
    for blocker in &review.blockers {
        content.append(&small_label(blocker));
    }
    append_detail_line(
        &content,
        &review.production_label,
        &review.production_status,
    );

    let start = gtk::Button::with_label(&review.disabled_action_label);
    start.set_widget_name(&review.disabled_action_widget_name);
    start.set_tooltip_text(Some(
        "Disabled planned activation path. This does not enable production behavior.",
    ));
    start.set_sensitive(false);
    content.append(&start);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_control_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-control-section");
    frame.set_tooltip_text(Some(
        "Default-disabled production activation controls. These validate request and safety-plan inputs but keep executors unwired.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label("Final production activation controls"));
    content.append(&small_label(
        "These controls validate complete inputs for review only. Production executors remain unwired.",
    ));

    for review in production_activation_control_reviews() {
        content.append(&production_activation_control_review_card(&review));
    }

    frame.set_child(Some(&content));
    frame
}

fn production_activation_control_review_card(
    review: &ProductionActivationControlReview,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&review.widget_name);
    frame.set_tooltip_text(Some(
        "Disabled production activation control. This card validates review inputs only and cannot run an executor.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&review.evidence_widget_name);
    content.set_tooltip_text(Some(
        "Activation control evidence. Request and safety-plan validation are review-only.",
    ));
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&review.heading));
    append_detail_line(
        &content,
        "Input path status",
        review.input_path_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Control status",
        review.status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Request validation",
        &review.request_validation_status,
    );
    append_detail_line(
        &content,
        "Safety plan validation",
        &review.safety_plan_validation_status,
    );
    append_detail_line(
        &content,
        "Executor wiring",
        review.executor_wiring_status.user_facing_label(),
    );
    content.append(&body_label("Activation control blockers"));
    if review.blockers.is_empty() {
        content.append(&small_label(
            "No review blockers; executor remains unwired and production remains disabled.",
        ));
    } else {
        for blocker in &review.blockers {
            content.append(&small_label(blocker));
        }
    }
    append_detail_line(
        &content,
        &review.production_label,
        &review.production_status,
    );

    let validate = gtk::Button::with_label(&review.disabled_action_label);
    validate.set_widget_name(&review.disabled_action_widget_name);
    validate.set_tooltip_text(Some(
        "Disabled planned validation control. This has no mutation handler.",
    ));
    validate.set_sensitive(false);
    content.append(&validate);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_form_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-form-section");
    frame.set_tooltip_text(Some(
        "Review-only production activation forms. These collect request and safety-plan fields without wiring executors.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label("Review-only activation request forms"));
    content.append(&small_label(
        "These form projections collect activation request data for validation only. Production writes remain disabled.",
    ));

    for review in production_activation_form_reviews() {
        content.append(&production_activation_form_review_card(&review));
    }

    frame.set_child(Some(&content));
    frame
}

fn production_activation_draft_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-draft-section");
    frame.set_tooltip_text(Some(
        "In-memory activation drafts. These drafts are review-only, non-persistent, and cannot run executors.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label("In-memory activation drafts"));
    content.append(&small_label(
        "These draft states model future form edits in memory only. The visible form fields remain disabled.",
    ));

    for review in production_activation_draft_reviews() {
        content.append(&production_activation_draft_review_card(&review));
    }

    frame.set_child(Some(&content));
    frame
}

fn production_activation_draft_edit_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-draft-edit-section");
    frame.set_tooltip_text(Some(
        "Still-disabled activation draft editing. Edit updates are modeled in memory only and cannot persist or run executors.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label("Activation draft editing"));
    content.append(&small_label(
        "Editable draft mode is modeled for memory-only validation. Live fields and controls remain disabled.",
    ));

    for review in production_activation_draft_edit_reviews() {
        content.append(&production_activation_draft_edit_review_card(&review));
    }

    frame.set_child(Some(&content));
    frame
}

fn production_activation_draft_edit_review_card(
    review: &ProductionActivationDraftEditReview,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&review.widget_name);
    frame.set_tooltip_text(Some(
        "Disabled activation draft edit surface. This does not persist data or wire production executors.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&review.evidence_widget_name);
    content.set_tooltip_text(Some(
        "Draft edit evidence. Editable updates are modeled in memory only.",
    ));
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&review.heading));
    append_detail_line(&content, "Editing mode", review.mode.user_facing_label());
    append_detail_line(&content, "Draft dirty state", &review.dirty_state);
    append_detail_line(
        &content,
        "Draft validation",
        review.draft_status.user_facing_label(),
    );
    append_detail_line(&content, "In-memory only", &review.persistence_status);
    append_detail_line(
        &content,
        "Form validation",
        review.form_validation_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Control validation",
        review.control_validation_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Executor wiring",
        review.executor_wiring_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        &review.production_label,
        &review.production_status,
    );

    let mode = gtk::Button::with_label(review.mode.user_facing_label());
    mode.set_widget_name(&review.mode_widget_name);
    mode.set_tooltip_text(Some(
        "Disabled draft editing mode control. This has no live edit callback.",
    ));
    mode.set_sensitive(false);
    content.append(&mode);

    let update = gtk::Button::with_label(&review.disabled_update_label);
    update.set_widget_name(&review.disabled_update_widget_name);
    update.set_tooltip_text(Some(
        "Disabled planned draft edit update. This has no persistence, mutation, or executor handler.",
    ));
    update.set_sensitive(false);
    content.append(&update);

    let reset = gtk::Button::with_label(&review.disabled_reset_label);
    reset.set_widget_name(&review.disabled_reset_widget_name);
    reset.set_tooltip_text(Some(
        "Disabled planned draft edit reset. This has no persistence, mutation, or executor handler.",
    ));
    reset.set_sensitive(false);
    content.append(&reset);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_live_draft_edit_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-live-draft-edit-section");
    frame.set_tooltip_text(Some(
        "Live activation draft field edits update memory only. No draft persistence or production executor is available.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label("Live memory-only activation draft editing"));
    content.append(&small_label(
        "These editable draft fields update in-memory review state only. They are not saved to disk and cannot run production writes.",
    ));

    for review in production_activation_live_draft_gtk_reviews() {
        content.append(&production_activation_live_draft_edit_review_card(&review));
    }

    frame.set_child(Some(&content));
    frame
}

fn production_activation_draft_persistence_boundary_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(
        "hyprland-settings-production-activation-draft-persistence-boundary-section",
    );

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label("Activation draft persistence boundary"));
    content.append(&small_label(
        "Draft persistence is not available. No draft data is saved, no storage path exists, and production executors remain unwired.",
    ));

    for boundary in production_activation_draft_persistence_boundaries() {
        content.append(&production_activation_draft_persistence_boundary_card(
            &boundary,
        ));
    }

    frame.set_child(Some(&content));
    frame
}

fn production_activation_draft_persistence_boundary_card(
    boundary: &ProductionActivationDraftPersistenceBoundary,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&boundary.widget_name);
    frame.set_tooltip_text(Some(
        "Disabled activation draft persistence boundary. This has no storage, serializer, or executor handler.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&boundary.evidence_widget_name);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&boundary.heading));
    append_detail_line(
        &content,
        "Persistence status",
        boundary.status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Persistence enabled",
        &boundary.persistence_enabled.to_string(),
    );
    append_detail_line(
        &content,
        "Draft written to disk",
        &boundary.draft_written_to_disk.to_string(),
    );
    append_detail_line(
        &content,
        "Storage path",
        boundary.storage_path.as_deref().unwrap_or("none"),
    );
    append_detail_line(
        &content,
        "Serializer called",
        &boundary.serializer_called.to_string(),
    );
    append_detail_line(
        &content,
        "Storage directory created",
        &boundary.storage_directory_created.to_string(),
    );
    append_detail_line(
        &content,
        "Executor wiring",
        boundary.executor_wiring_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        &boundary.production_label,
        &boundary.production_status,
    );

    content.append(&small_label("Required before persistence"));
    for requirement in &boundary.required_before_persistence {
        content.append(&small_label(&format!(
            "Required before persistence: {requirement}"
        )));
    }

    let enable = gtk::Button::with_label(&boundary.disabled_enable_label);
    enable.set_widget_name(&format!("{}-enable-disabled", boundary.widget_name));
    enable.set_tooltip_text(Some(
        "Draft persistence is not available. This button has no persistence or executor callback.",
    ));
    enable.set_sensitive(false);
    content.append(&enable);

    let clear = gtk::Button::with_label(&boundary.disabled_clear_label);
    clear.set_widget_name(&format!("{}-clear-disabled", boundary.widget_name));
    clear.set_tooltip_text(Some(
        "There is no persisted draft to clear. This button has no storage callback.",
    ));
    clear.set_sensitive(false);
    content.append(&clear);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_safety_gate_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-safety-gates-section");
    frame.set_tooltip_text(Some(
        "Default-disabled production activation safety gates. These show the missing proof required before production activation could ever be considered.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label("Production activation safety gates"));
    content.append(&small_label(
        "Production activation is blocked by default. Executors remain unwired, draft persistence remains forbidden, and no write path is available.",
    ));
    content.append(&small_label(
        "Production activation blocked by default until every required proof item is satisfied.",
    ));

    for gate in production_activation_safety_gate_reviews() {
        content.append(&production_activation_safety_gate_review_card(&gate));
    }
    content.append(&production_activation_safety_gate_proof_reviews_section());
    content.append(&production_activation_final_decision_reviews_section());
    content.append(&production_activation_approval_and_dry_run_reviews_section());
    content.append(&production_activation_opt_in_requirement_reviews_section());
    content.append(&production_activation_cap_reviews_section());

    frame.set_child(Some(&content));
    frame
}

fn production_activation_safety_gate_review_card(
    gate: &ProductionActivationSafetyGateReview,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&gate.widget_name);
    frame.set_tooltip_text(Some(
        "Disabled production activation safety gate. This has no persistence, mutation, production, compositor refresh, or executor handler.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&gate.evidence_widget_name);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&gate.heading));
    append_detail_line(&content, "Gate status", gate.status.user_facing_label());
    append_detail_line(
        &content,
        "Report-backed proof",
        &gate.report_backed_proof_status,
    );
    append_detail_line(
        &content,
        "Executor wiring",
        gate.executor_wiring_status.user_facing_label(),
    );
    append_detail_line(&content, &gate.production_label, &gate.production_status);
    append_detail_line(
        &content,
        "Draft persistence boundary",
        gate.draft_persistence_status.user_facing_label(),
    );

    content.append(&small_label("Required before production activation"));
    for requirement in &gate.requirements {
        append_detail_line(&content, &requirement.label, &requirement.status);
    }

    content.append(&small_label("Safety gate blockers"));
    for blocker in &gate.blockers {
        content.append(&small_label(blocker));
    }

    let review = gtk::Button::with_label(&gate.disabled_review_label);
    review.set_widget_name(&gate.disabled_review_widget_name);
    review.set_tooltip_text(Some(
        "Gate review is not available. This button has no persistence, mutation, production, compositor refresh, or executor callback.",
    ));
    review.set_sensitive(false);
    content.append(&review);

    let enable = gtk::Button::with_label(&gate.disabled_enable_label);
    enable.set_widget_name(&gate.disabled_enable_widget_name);
    enable.set_tooltip_text(Some(
        "Production activation is not available. This button has no write or executor callback.",
    ));
    enable.set_sensitive(false);
    content.append(&enable);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_safety_gate_proof_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-safety-proof-section");
    frame.set_tooltip_text(Some(
        "Copied-fixture production activation safety proof. This is review-only and has no production executor callback.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label("Production activation safety proof"));
    content.append(&small_label(
        "Copied-fixture proof can satisfy backup, dry-run, reread, restore, and post-restore checks without touching real config.",
    ));
    content.append(&small_label(
        "Production activation proof partially satisfied but default-disabled.",
    ));
    content.append(&small_label(
        "Final approval, production flag, executor wiring, and live production dry-run decisions remain unresolved.",
    ));

    for proof in production_activation_safety_gate_proof_reviews() {
        content.append(&production_activation_safety_gate_proof_review_card(&proof));
    }

    frame.set_child(Some(&content));
    frame
}

fn production_activation_safety_gate_proof_review_card(
    proof: &ProductionActivationSafetyGateProofReview,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&proof.widget_name);
    frame.set_tooltip_text(Some(
        "Review-only copied-fixture safety proof. This card has no persistence, mutation, production, compositor refresh, or executor handler.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&proof.evidence_widget_name);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&proof.heading));
    append_detail_line(&content, "Proof status", proof.status.user_facing_label());
    append_detail_line(
        &content,
        "Executor wiring",
        proof.executor_wiring_status.user_facing_label(),
    );
    append_detail_line(&content, &proof.production_label, &proof.production_status);
    append_detail_line(
        &content,
        "Draft persistence boundary",
        proof.draft_persistence_status.user_facing_label(),
    );

    content.append(&small_label("Proof requirements"));
    for item in &proof.proof_items {
        append_detail_line(&content, &item.label, item.status.user_facing_label());
    }

    content.append(&small_label("Copied fixture evidence"));
    append_detail_line(
        &content,
        "Byte-exact backup",
        if proof.fixture_proof.backup_bytes_equal {
            "satisfied in copied fixture"
        } else {
            "missing/proof-required"
        },
    );
    append_detail_line(
        &content,
        "Dry-run write plan",
        &proof.fixture_proof.planned_write,
    );
    append_detail_line(&content, "Diff preview", &proof.fixture_proof.diff_preview);
    append_detail_line(
        &content,
        "Post-write reread",
        if proof.fixture_proof.reread_verified {
            "satisfied in copied fixture"
        } else {
            "missing/proof-required"
        },
    );
    append_detail_line(
        &content,
        "Restore plan",
        if proof.fixture_proof.restore_verified {
            "satisfied in copied fixture"
        } else {
            "missing/proof-required"
        },
    );
    append_detail_line(
        &content,
        "Post-restore verification",
        if proof.fixture_proof.restore_verified {
            "satisfied in copied fixture"
        } else {
            "missing/proof-required"
        },
    );
    append_detail_line(
        &content,
        "Final approval still required",
        "still requires explicit user approval",
    );

    content.append(&small_label("Safety proof blockers"));
    for blocker in &proof.blockers {
        content.append(&small_label(blocker));
    }

    let fixture = gtk::Button::with_label(&proof.disabled_fixture_proof_label);
    fixture.set_widget_name(&proof.disabled_fixture_proof_widget_name);
    fixture.set_tooltip_text(Some(
        "Fixture proof running is not available from this UI. This button has no persistence, mutation, production, compositor refresh, or executor callback.",
    ));
    fixture.set_sensitive(false);
    content.append(&fixture);

    let enable = gtk::Button::with_label(&proof.disabled_enable_label);
    enable.set_widget_name(&proof.disabled_enable_widget_name);
    enable.set_sensitive(false);
    content.append(&enable);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_final_decision_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-final-decision-section");
    frame.set_tooltip_text(Some(
        "Default-disabled final activation decisions. These cannot approve production or wire executors.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label("Production activation final decisions"));
    content.append(&small_label(
        "Copied-fixture proof does not imply final approval, production flag opt-in, executor wiring, or live production dry-run permission.",
    ));
    content.append(&small_label(
        "Final approval, production flag, executor wiring, and live production dry-run decisions remain missing/required.",
    ));
    content.append(&small_label(
        "Final decision proof satisfied but decisions missing.",
    ));

    for decision in production_activation_final_decision_reviews() {
        content.append(&production_activation_final_decision_review_card(&decision));
    }

    frame.set_child(Some(&content));
    frame
}

fn production_activation_final_decision_review_card(
    decision: &ProductionActivationFinalDecisionReview,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&decision.widget_name);
    frame.set_tooltip_text(Some(
        "Review-only final activation decision. This card has no persistence, mutation, production, compositor refresh, or executor handler.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&decision.evidence_widget_name);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&decision.heading));
    append_detail_line(
        &content,
        "Final decision status",
        decision.status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Final approval",
        decision.final_approval_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Production flag decision",
        decision.production_flag_decision_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Executor wiring decision",
        decision.executor_wiring_decision_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Live production dry-run policy",
        decision.live_dry_run_policy_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Copied-fixture proof",
        decision.copied_fixture_proof_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Form/control/draft proof",
        &decision.form_control_draft_proof_status,
    );
    append_detail_line(
        &content,
        "Draft persistence",
        decision.draft_persistence_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Executor wiring",
        decision.executor_wiring_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        &decision.production_label,
        &decision.production_status,
    );

    content.append(&small_label("Final decision blockers"));
    for blocker in &decision.blockers {
        content.append(&small_label(blocker));
    }

    let approval = gtk::Button::with_label(&decision.disabled_final_approval_label);
    approval.set_widget_name(&decision.disabled_final_approval_widget_name);
    approval.set_tooltip_text(Some(
        "Final approval is not available. This button has no production callback.",
    ));
    approval.set_sensitive(false);
    content.append(&approval);

    let flag = gtk::Button::with_label(&decision.disabled_production_flag_label);
    flag.set_widget_name(&decision.disabled_production_flag_widget_name);
    flag.set_tooltip_text(Some(
        "Production flag opt-in is not available. This button does not change flags.",
    ));
    flag.set_sensitive(false);
    content.append(&flag);

    let wiring = gtk::Button::with_label(&decision.disabled_executor_wiring_label);
    wiring.set_widget_name(&decision.disabled_executor_wiring_widget_name);
    wiring.set_tooltip_text(Some(
        "Executor wiring is not available. This button has no executor callback.",
    ));
    wiring.set_sensitive(false);
    content.append(&wiring);

    let dry_run = gtk::Button::with_label(&decision.disabled_live_dry_run_label);
    dry_run.set_widget_name(&decision.disabled_live_dry_run_widget_name);
    dry_run.set_tooltip_text(Some(
        "Live production dry-run is not available. This button has no config, runtime, compositor refresh, or IPC callback.",
    ));
    dry_run.set_sensitive(false);
    content.append(&dry_run);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_approval_and_dry_run_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(
        "hyprland-settings-production-activation-approval-ux-and-dry-run-policy-section",
    );

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label(
        "Production activation approval UX and dry-run policy",
    ));
    content.append(&small_label(
        "Approval UX and live production dry-run policy are designed but disabled.",
    ));
    content.append(&small_label(
        "Approval controls and live dry-run controls are not available, executors remain Unwired, and production flags remain false.",
    ));

    for review in production_activation_approval_and_dry_run_reviews() {
        content.append(&production_activation_approval_ux_review_card(&review));
        content.append(&production_activation_live_dry_run_policy_review_card(
            &review,
        ));
    }

    frame.set_child(Some(&content));
    frame
}

fn production_activation_opt_in_requirement_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-opt-in-requirements-section");
    frame.set_tooltip_text(Some(
        "Default-disabled production flag and executor-wiring opt-in requirements. These cards cannot set flags or wire executors.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label(
        "Production flag and executor-wiring opt-in requirements",
    ));
    content.append(&small_label(
        "Production flag opt-in and executor wiring are separate future gates. Neither gate can auto-enable the other, run writes, reload Hyprland, mutate runtime, or touch real config.",
    ));
    content.append(&small_label(
        "Opt-in requirements are designed but disabled; production flags remain false and executors remain Unwired.",
    ));

    for review in production_activation_opt_in_requirement_reviews() {
        content.append(&production_activation_opt_in_requirement_review_card(
            &review,
        ));
    }

    frame.set_child(Some(&content));
    frame
}

fn production_activation_opt_in_requirement_review_card(
    review: &ProductionFlagAndExecutorOptInReview,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&review.widget_name);
    frame.set_tooltip_text(Some(
        "Review-only opt-in requirements. This card has no persistence, mutation, production, compositor refresh, or executor handler.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&review.evidence_widget_name);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&review.heading));
    content.append(&small_label("Opt-in requirements designed but disabled"));
    append_detail_line(
        &content,
        "Opt-in requirements status",
        review.status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Production flag opt-in",
        review.production_flag_opt_in_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Executor wiring opt-in",
        review.executor_wiring_opt_in_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Flag and executor wiring must be separate future steps",
        review.separate_steps_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Explicit user action",
        review.explicit_user_action_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Typed confirmation",
        review.typed_confirmation_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Report-backed proof",
        review.report_backed_proof_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Rollback-ready state",
        review.rollback_ready_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "No auto-apply proof",
        review.no_auto_apply_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Production flag",
        &review.production_flag.to_string(),
    );
    append_detail_line(
        &content,
        "Executor wiring",
        review.executor_wiring_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        &review.production_label,
        &review.production_status,
    );

    content.append(&small_label("Opt-in requirements"));
    for requirement in &review.requirements {
        content.append(&small_label(requirement));
    }
    content.append(&small_label("Opt-in negative proofs"));
    for proof in &review.negative_proofs {
        content.append(&small_label(proof));
    }

    let flag = gtk::Button::with_label(&review.disabled_flag_label);
    flag.set_widget_name(&review.disabled_flag_widget_name);
    flag.set_sensitive(false);
    content.append(&flag);

    let executor = gtk::Button::with_label(&review.disabled_executor_label);
    executor.set_widget_name(&review.disabled_executor_widget_name);
    executor.set_tooltip_text(Some(
        "Executor wiring is not available. This button has no executor callback.",
    ));
    executor.set_sensitive(false);
    content.append(&executor);

    let confirm = gtk::Button::with_label(&review.disabled_confirm_label);
    confirm.set_widget_name(&review.disabled_confirm_widget_name);
    confirm.set_tooltip_text(Some(
        "Opt-in confirmation is not available. This button has no production callback.",
    ));
    confirm.set_sensitive(false);
    content.append(&confirm);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_approval_ux_review_card(
    review: &ProductionActivationApprovalAndDryRunReview,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&review.approval_widget_name);
    frame.set_tooltip_text(Some(
        "Review-only approval UX design. This card has no persistence, mutation, production, compositor refresh, or executor handler.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&review.approval_evidence_widget_name);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&review.approval_heading));
    content.append(&small_label("Approval UX designed but disabled"));
    append_detail_line(
        &content,
        "Approval UX status",
        review.approval_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Explicit final approval",
        review.approval_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Typed confirmation phrase",
        review.typed_confirmation_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Backup/restore acknowledgement",
        review
            .backup_restore_acknowledgement_status
            .user_facing_label(),
    );
    append_detail_line(
        &content,
        "Production flag opt-in",
        review.production_flag_opt_in_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Executor wiring opt-in",
        review.executor_wiring_opt_in_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Copied-fixture proof cannot approve production",
        review.proof_inference_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Draft edit cannot approve production",
        review.proof_inference_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Persistence boundary cannot approve production",
        review.proof_inference_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Draft persistence",
        review.draft_persistence_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Executor wiring",
        review.executor_wiring_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        &review.production_label,
        &review.production_status,
    );

    content.append(&small_label("Approval UX requirements"));
    for requirement in &review.approval_requirements {
        content.append(&small_label(requirement));
    }
    content.append(&small_label("Approval UX negative proofs"));
    for proof in &review.negative_proofs {
        content.append(&small_label(proof));
    }

    let approval = gtk::Button::with_label(&review.disabled_approval_label);
    approval.set_widget_name(&review.disabled_approval_widget_name);
    approval.set_tooltip_text(Some(
        "Approval is not available. This button has no production callback.",
    ));
    approval.set_sensitive(false);
    content.append(&approval);

    let confirmation = gtk::Button::with_label(&review.disabled_confirmation_label);
    confirmation.set_widget_name(&review.disabled_confirmation_widget_name);
    confirmation.set_tooltip_text(Some(
        "Typed confirmation is not available. This button only documents a future requirement.",
    ));
    confirmation.set_sensitive(false);
    content.append(&confirmation);

    let flag = gtk::Button::with_label(&review.disabled_flag_label);
    flag.set_widget_name(&review.disabled_flag_widget_name);
    flag.set_sensitive(false);
    content.append(&flag);

    let wiring = gtk::Button::with_label(&review.disabled_wiring_label);
    wiring.set_widget_name(&review.disabled_wiring_widget_name);
    wiring.set_tooltip_text(Some(
        "Executor wiring opt-in is not available. This button has no executor callback.",
    ));
    wiring.set_sensitive(false);
    content.append(&wiring);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_live_dry_run_policy_review_card(
    review: &ProductionActivationApprovalAndDryRunReview,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&review.dry_run_widget_name);
    frame.set_tooltip_text(Some(
        "Review-only live dry-run policy. This card has no config, runtime, compositor refresh, IPC, production, or executor handler.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&review.dry_run_evidence_widget_name);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&review.dry_run_heading));
    content.append(&small_label("Dry-run policy designed but disabled"));
    append_detail_line(
        &content,
        "Dry-run policy status",
        review.dry_run_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Live dry-run requires explicit user action",
        review.dry_run_explicit_action_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Live dry-run cannot run by default",
        review.dry_run_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Live dry-run cannot touch real config by default",
        review
            .dry_run_real_config_boundary_status
            .user_facing_label(),
    );
    append_detail_line(
        &content,
        "Live dry-run cannot reload Hyprland by default",
        review.dry_run_no_reload_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Live dry-run cannot mutate runtime by default",
        review
            .dry_run_no_runtime_mutation_status
            .user_facing_label(),
    );
    append_detail_line(
        &content,
        "Live dry-run requires rollback-ready state",
        review.dry_run_rollback_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Copied-fixture proof is not live production dry-run",
        review.copied_fixture_proof_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Executor wiring",
        review.executor_wiring_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        &review.production_label,
        &review.production_status,
    );

    content.append(&small_label("Live dry-run policy requirements"));
    for requirement in &review.dry_run_requirements {
        content.append(&small_label(requirement));
    }

    let dry_run = gtk::Button::with_label(&review.disabled_dry_run_label);
    dry_run.set_widget_name(&review.disabled_dry_run_widget_name);
    dry_run.set_tooltip_text(Some(
        "Live production dry-run is not available. This button has no config, runtime, compositor refresh, IPC, production, or executor callback.",
    ));
    dry_run.set_sensitive(false);
    content.append(&dry_run);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_cap_reviews_section() -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-production-activation-cap-section");
    frame.set_tooltip_text(Some(
        "Final non-production cap for source/include and duplicate activation runway. These cards cannot start a production phase.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.append(&title_label("Production activation cap"));
    content.append(&small_label(
        "Branch capped for non-production runway. Future production activation requires a separate approved phase.",
    ));
    content.append(&small_label(
        "The cap does not set production flags, wire executors, persist drafts, run writes, reload Hyprland, mutate runtime, or touch real config.",
    ));

    for review in production_activation_cap_reviews() {
        content.append(&production_activation_cap_review_card(&review));
    }

    frame.set_child(Some(&content));
    frame
}

fn production_activation_cap_review_card(review: &ProductionActivationCapReview) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&review.widget_name);
    frame.set_tooltip_text(Some(
        "Review-only production activation cap. This card has no persistence, mutation, production, compositor refresh, or executor handler.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&review.evidence_widget_name);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&review.heading));
    append_detail_line(&content, "Cap status", review.status.user_facing_label());
    append_detail_line(
        &content,
        "Future production activation",
        review.future_phase_requirement_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        &review.production_label,
        &review.production_status,
    );
    append_detail_line(
        &content,
        "Production flag",
        &review.production_flag.to_string(),
    );
    append_detail_line(
        &content,
        "Executor wiring",
        review.executor_wiring_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Draft persistence",
        review.draft_persistence_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Real config touched",
        &review.real_config_touched.to_string(),
    );
    append_detail_line(
        &content,
        "Runtime mutated",
        &review.runtime_mutated.to_string(),
    );
    append_detail_line(
        &content,
        "Production write executed",
        &review.production_write_executed.to_string(),
    );

    content.append(&small_label("Cap reasons"));
    for reason in &review.cap_reasons {
        content.append(&small_label(reason));
    }
    content.append(&small_label("Future phase requirements"));
    for requirement in &review.future_phase_requirements {
        content.append(&small_label(requirement));
    }
    content.append(&small_label("Cap negative proofs"));
    for proof in &review.negative_proofs {
        content.append(&small_label(proof));
    }

    let start = gtk::Button::with_label(&review.disabled_start_label);
    start.set_widget_name(&review.disabled_start_widget_name);
    start.set_tooltip_text(Some(
        "Starting a production activation phase is not available from this branch.",
    ));
    start.set_sensitive(false);
    content.append(&start);

    let confirm = gtk::Button::with_label(&review.disabled_confirm_label);
    confirm.set_widget_name(&review.disabled_confirm_widget_name);
    confirm.set_tooltip_text(Some(
        "Branch cap confirmation is review-only and has no production callback.",
    ));
    confirm.set_sensitive(false);
    content.append(&confirm);

    frame.set_child(Some(&content));
    frame
}

#[derive(Clone)]
struct LiveDraftStatusLabels {
    bridge: gtk::Label,
    dirty: gtk::Label,
    draft: gtk::Label,
    form: gtk::Label,
    control: gtk::Label,
}

fn production_activation_live_draft_edit_review_card(
    review: &ProductionActivationDraftGtkReview,
) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&review.widget_name);
    frame.set_tooltip_text(Some(
        "Memory-only activation draft edit bridge. This does not persist data or wire production executors.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&review.evidence_widget_name);
    content.set_tooltip_text(Some(
        "Live draft edit bridge evidence. GTK field changes update only in-memory draft state.",
    ));
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&review.heading));
    content.append(&small_label("Draft editing mode: memory-only"));
    content.append(&small_label(&review.not_saved_status));

    let labels = LiveDraftStatusLabels {
        bridge: body_label(""),
        dirty: body_label(""),
        draft: body_label(""),
        form: body_label(""),
        control: body_label(""),
    };
    labels
        .bridge
        .set_widget_name("hyprland-settings-live-draft-bridge-status");
    labels
        .dirty
        .set_widget_name("hyprland-settings-live-draft-dirty-state");
    labels
        .draft
        .set_widget_name("hyprland-settings-live-draft-validation");
    labels
        .form
        .set_widget_name("hyprland-settings-live-draft-form-validation");
    labels
        .control
        .set_widget_name("hyprland-settings-live-draft-control-validation");
    refresh_live_draft_status_labels(&labels, review);
    content.append(&labels.bridge);
    content.append(&labels.dirty);
    content.append(&labels.draft);
    content.append(&labels.form);
    content.append(&labels.control);

    append_detail_line(&content, "In-memory only", &review.persistence_status);
    append_detail_line(&content, "Not saved to disk", &review.not_saved_status);
    append_detail_line(
        &content,
        "Executor wiring",
        review.executor_wiring_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        &review.production_label,
        &review.production_status,
    );

    let state = Rc::new(RefCell::new(review.state.clone()));
    let refresh = live_draft_refresh(review);
    append_live_draft_text_field(
        &content,
        "User-facing reason",
        &review.state.edit_state.draft.form_state.user_facing_reason,
        &live_draft_widget_name(review, "reason-field"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::UserFacingReason,
    );
    append_live_draft_text_field(
        &content,
        "Explicit activation phrase/token",
        &review
            .state
            .edit_state
            .draft
            .form_state
            .explicit_activation_token,
        &live_draft_widget_name(review, "token-field"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::ExplicitActivationToken,
    );
    append_live_draft_check_field(
        &content,
        "Backup-before-write acknowledgement",
        review
            .state
            .edit_state
            .draft
            .form_state
            .backup_plan_acknowledged,
        &live_draft_widget_name(review, "backup-check"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::BackupPlanAcknowledged,
    );
    append_live_draft_check_field(
        &content,
        "Restore-plan acknowledgement",
        review
            .state
            .edit_state
            .draft
            .form_state
            .restore_plan_acknowledged,
        &live_draft_widget_name(review, "restore-check"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::RestorePlanAcknowledged,
    );
    append_live_draft_check_field(
        &content,
        "Post-write reread acknowledgement",
        review
            .state
            .edit_state
            .draft
            .form_state
            .reread_plan_acknowledged,
        &live_draft_widget_name(review, "reread-check"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::RereadPlanAcknowledged,
    );
    append_live_draft_check_field(
        &content,
        "Post-restore verification acknowledgement",
        review
            .state
            .edit_state
            .draft
            .form_state
            .post_restore_verification_acknowledged,
        &live_draft_widget_name(review, "post-restore-check"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::PostRestoreVerificationAcknowledged,
    );
    append_live_draft_check_field(
        &content,
        "Final confirmation acknowledgement",
        review
            .state
            .edit_state
            .draft
            .form_state
            .final_confirmation_acknowledged,
        &live_draft_widget_name(review, "final-check"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::FinalConfirmationAcknowledged,
    );
    append_live_draft_multiline_field(
        &content,
        "Backup-before-write plan",
        &review
            .state
            .edit_state
            .draft
            .form_state
            .backup_before_write_plan,
        &live_draft_widget_name(review, "backup-plan-field"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::BackupBeforeWritePlan,
    );
    append_live_draft_multiline_field(
        &content,
        "Restore plan",
        &review.state.edit_state.draft.form_state.restore_plan,
        &live_draft_widget_name(review, "restore-plan-field"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::RestorePlan,
    );
    append_live_draft_multiline_field(
        &content,
        "Post-write reread plan",
        &review
            .state
            .edit_state
            .draft
            .form_state
            .post_write_reread_plan,
        &live_draft_widget_name(review, "reread-plan-field"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::PostWriteRereadPlan,
    );
    append_live_draft_multiline_field(
        &content,
        "Post-restore verification plan",
        &review
            .state
            .edit_state
            .draft
            .form_state
            .post_restore_verification_plan,
        &live_draft_widget_name(review, "post-restore-plan-field"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::PostRestoreVerificationPlan,
    );
    append_live_draft_multiline_field(
        &content,
        "Dry-run summary",
        &review.state.edit_state.draft.form_state.dry_run_summary,
        &live_draft_widget_name(review, "dry-run-summary-field"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::DryRunSummary,
    );
    append_live_draft_multiline_field(
        &content,
        "Files that would be touched",
        &review
            .state
            .edit_state
            .draft
            .form_state
            .files_that_would_be_touched
            .join("\n"),
        &live_draft_widget_name(review, "touched-files-field"),
        state.clone(),
        labels.clone(),
        refresh.clone(),
        ProductionActivationDraftGtkField::FilesThatWouldBeTouched,
    );

    let update = gtk::Button::with_label(&review.update_label);
    update.set_widget_name(&live_draft_widget_name(review, "update-memory-only"));
    update.set_tooltip_text(Some(
        "Recomputes in-memory draft review state only. This does not persist or run executors.",
    ));
    update.connect_clicked({
        let state = state.clone();
        let labels = labels.clone();
        let refresh = refresh.clone();
        move |_| {
            let review = refresh(&state.borrow());
            refresh_live_draft_status_labels(&labels, &review);
        }
    });
    content.append(&update);

    let reset = gtk::Button::with_label(&review.reset_label);
    reset.set_widget_name(&live_draft_widget_name(review, "reset-memory-only"));
    reset.set_tooltip_text(Some(
        "Resets only the in-memory draft. This does not persist or run executors.",
    ));
    reset.connect_clicked({
        let state = state.clone();
        let labels = labels.clone();
        let refresh = refresh.clone();
        move |_| {
            apply_production_activation_draft_gtk_update(
                &mut state.borrow_mut(),
                ProductionActivationDraftGtkUpdate::ResetToDefault,
            );
            let review = refresh(&state.borrow());
            refresh_live_draft_status_labels(&labels, &review);
        }
    });
    content.append(&reset);

    frame.set_child(Some(&content));
    frame
}

fn live_draft_widget_name(review: &ProductionActivationDraftGtkReview, suffix: &str) -> String {
    let prefix = if review.widget_name.contains("source-include") {
        "hyprland-settings-source-include-activation-live-draft-edit"
    } else {
        "hyprland-settings-duplicate-activation-live-draft-edit"
    };
    format!("{prefix}-{suffix}-disabled")
}

fn live_draft_refresh(
    review: &ProductionActivationDraftGtkReview,
) -> Rc<dyn Fn(&ProductionActivationDraftGtkState) -> ProductionActivationDraftGtkReview> {
    let is_source = review.widget_name.contains("source-include");
    Rc::new(move |state| {
        if is_source {
            source_include_activation_draft_gtk_review(
                None,
                state.clone(),
                ProductionExecutorWiringState::Unwired,
                false,
            )
        } else {
            duplicate_activation_draft_gtk_review(
                None,
                state.clone(),
                ProductionExecutorWiringState::Unwired,
                false,
            )
        }
    })
}

fn refresh_live_draft_status_labels(
    labels: &LiveDraftStatusLabels,
    review: &ProductionActivationDraftGtkReview,
) {
    labels.bridge.set_text(&format!(
        "GTK bridge status: {}",
        review.status.user_facing_label()
    ));
    labels
        .dirty
        .set_text(&format!("Draft dirty state: {}", review.dirty_state));
    labels.draft.set_text(&format!(
        "Draft validation: {}",
        review.draft_status.user_facing_label()
    ));
    labels.form.set_text(&format!(
        "Form validation: {}",
        review.form_validation_status.user_facing_label()
    ));
    labels.control.set_text(&format!(
        "Control validation: {}",
        review.control_validation_status.user_facing_label()
    ));
}

fn append_live_draft_text_field(
    parent: &gtk::Box,
    label: &str,
    value: &str,
    widget_name: &str,
    state: Rc<RefCell<ProductionActivationDraftGtkState>>,
    labels: LiveDraftStatusLabels,
    refresh: Rc<dyn Fn(&ProductionActivationDraftGtkState) -> ProductionActivationDraftGtkReview>,
    field: ProductionActivationDraftGtkField,
) {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 3);
    row.append(&small_label(label));
    let entry = gtk::Entry::new();
    entry.set_widget_name(widget_name);
    entry.set_tooltip_text(Some(
        "Memory-only activation draft field. This is not saved to disk.",
    ));
    entry.set_text(value);
    entry.connect_changed(move |entry| {
        apply_production_activation_draft_gtk_update(
            &mut state.borrow_mut(),
            ProductionActivationDraftGtkUpdate::Text {
                field,
                value: entry.text().to_string(),
            },
        );
        let review = refresh(&state.borrow());
        refresh_live_draft_status_labels(&labels, &review);
    });
    row.append(&entry);
    parent.append(&row);
}

fn append_live_draft_multiline_field(
    parent: &gtk::Box,
    label: &str,
    value: &str,
    widget_name: &str,
    state: Rc<RefCell<ProductionActivationDraftGtkState>>,
    labels: LiveDraftStatusLabels,
    refresh: Rc<dyn Fn(&ProductionActivationDraftGtkState) -> ProductionActivationDraftGtkReview>,
    field: ProductionActivationDraftGtkField,
) {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 3);
    row.append(&small_label(label));
    let text = gtk::TextView::new();
    text.set_widget_name(widget_name);
    text.set_tooltip_text(Some(
        "Memory-only activation draft text field. This is not saved to disk.",
    ));
    text.set_wrap_mode(gtk::WrapMode::WordChar);
    text.buffer().set_text(value);
    text.buffer().connect_changed(move |buffer| {
        let value = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
        apply_production_activation_draft_gtk_update(
            &mut state.borrow_mut(),
            ProductionActivationDraftGtkUpdate::Text {
                field,
                value: value.to_string(),
            },
        );
        let review = refresh(&state.borrow());
        refresh_live_draft_status_labels(&labels, &review);
    });
    row.append(&text);
    parent.append(&row);
}

fn append_live_draft_check_field(
    parent: &gtk::Box,
    label: &str,
    checked: bool,
    widget_name: &str,
    state: Rc<RefCell<ProductionActivationDraftGtkState>>,
    labels: LiveDraftStatusLabels,
    refresh: Rc<dyn Fn(&ProductionActivationDraftGtkState) -> ProductionActivationDraftGtkReview>,
    field: ProductionActivationDraftGtkField,
) {
    let check = gtk::CheckButton::with_label(label);
    check.set_widget_name(widget_name);
    check.set_tooltip_text(Some(
        "Memory-only activation draft acknowledgement. This is not saved to disk.",
    ));
    check.set_active(checked);
    check.connect_toggled(move |check| {
        apply_production_activation_draft_gtk_update(
            &mut state.borrow_mut(),
            ProductionActivationDraftGtkUpdate::Acknowledgement {
                field,
                value: check.is_active(),
            },
        );
        let review = refresh(&state.borrow());
        refresh_live_draft_status_labels(&labels, &review);
    });
    parent.append(&check);
}

fn production_activation_draft_review_card(review: &ProductionActivationDraftReview) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&review.widget_name);
    frame.set_tooltip_text(Some(
        "Disabled activation draft. This card does not persist draft data or wire production executors.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&review.evidence_widget_name);
    content.set_tooltip_text(Some(
        "In-memory draft evidence. Draft validation is review-only.",
    ));
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&review.heading));
    append_detail_line(&content, "Draft status", review.status.user_facing_label());
    append_detail_line(
        &content,
        "Draft validation",
        review.form_validation_status.user_facing_label(),
    );
    append_detail_line(&content, "Dirty state", &review.dirty_state);
    append_detail_line(&content, "In-memory only", &review.persistence_status);
    content.append(&small_label(&review.persistence_status));
    append_detail_line(
        &content,
        "Control validation",
        review.control_validation_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Executor wiring",
        review.executor_wiring_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        &review.production_label,
        &review.production_status,
    );

    let update = gtk::Button::with_label(&review.disabled_update_label);
    update.set_widget_name(&review.disabled_update_widget_name);
    update.set_tooltip_text(Some(
        "Disabled planned draft update. This has no persistence, mutation, or executor handler.",
    ));
    update.set_sensitive(false);
    content.append(&update);

    let reset = gtk::Button::with_label(&review.disabled_reset_label);
    reset.set_widget_name(&review.disabled_reset_widget_name);
    reset.set_tooltip_text(Some(
        "Disabled planned draft reset. This has no persistence, mutation, or executor handler.",
    ));
    reset.set_sensitive(false);
    content.append(&reset);

    frame.set_child(Some(&content));
    frame
}

fn production_activation_form_review_card(review: &ProductionActivationFormReview) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name(&review.widget_name);
    frame.set_tooltip_text(Some(
        "Disabled activation form. This review surface cannot run production executors.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_widget_name(&review.evidence_widget_name);
    content.set_tooltip_text(Some(
        "Activation form evidence. Request and safety-plan values are review-only.",
    ));
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label(&review.heading));
    append_detail_line(&content, "Form status", review.status.user_facing_label());
    append_detail_line(
        &content,
        "Request generation",
        &review.request_generation_status,
    );
    append_detail_line(
        &content,
        "Safety plan generation",
        &review.safety_plan_generation_status,
    );
    append_detail_line(
        &content,
        "Control validation",
        review.control_validation_status.user_facing_label(),
    );
    append_detail_line(
        &content,
        "Executor wiring",
        review.executor_wiring_status.user_facing_label(),
    );
    append_disabled_activation_form_fields(&content, review);
    content.append(&body_label("Required fields"));
    if review.missing_fields.is_empty() {
        content.append(&small_label("All required form fields are present."));
    } else {
        for field in &review.missing_fields {
            content.append(&small_label(field));
        }
    }
    content.append(&body_label("Request preview"));
    for field in &review.request_preview {
        content.append(&small_label(field));
    }
    content.append(&body_label("Safety plan preview"));
    for field in &review.safety_plan_preview {
        content.append(&small_label(field));
    }
    append_detail_line(
        &content,
        &review.production_label,
        &review.production_status,
    );

    let validate = gtk::Button::with_label(&review.disabled_action_label);
    validate.set_widget_name(&review.disabled_action_widget_name);
    validate.set_tooltip_text(Some(
        "Disabled planned form validation. This has no mutation handler.",
    ));
    validate.set_sensitive(false);
    content.append(&validate);

    frame.set_child(Some(&content));
    frame
}

fn append_disabled_activation_form_fields(
    parent: &gtk::Box,
    review: &ProductionActivationFormReview,
) {
    let prefix = activation_form_widget_prefix(review);
    parent.append(&body_label("Disabled activation form fields"));
    parent.append(&small_label(
        "These fields show the future activation request shape. They are read-only and cannot run production writes.",
    ));
    append_disabled_activation_text_field(
        parent,
        &prefix,
        "scope-field",
        "Scope/category",
        review
            .form_state
            .scope
            .as_ref()
            .map(|scope| match scope {
                crate::future_capability::ProductionActivationRequestScope::SourceIncludeInsertion => {
                    "source/include"
                }
                crate::future_capability::ProductionActivationRequestScope::DuplicateReplacement => {
                    "duplicate"
                }
            })
            .unwrap_or("Missing from form"),
    );
    append_disabled_activation_text_field(
        parent,
        &prefix,
        "reason-field",
        "User-facing reason",
        &review.form_state.user_facing_reason,
    );
    append_disabled_activation_text_field(
        parent,
        &prefix,
        "token-field",
        "Explicit activation phrase/token",
        &review.form_state.explicit_activation_token,
    );
    append_disabled_activation_text_field(
        parent,
        &prefix,
        "decision-category-field",
        "Decision category",
        &review.form_state.decision_category,
    );
    append_disabled_activation_check_field(
        parent,
        &prefix,
        "backup-check",
        "Backup-before-write acknowledgement",
        review.form_state.backup_plan_acknowledged,
    );
    append_disabled_activation_check_field(
        parent,
        &prefix,
        "restore-check",
        "Restore-plan acknowledgement",
        review.form_state.restore_plan_acknowledged,
    );
    append_disabled_activation_check_field(
        parent,
        &prefix,
        "reread-check",
        "Post-write reread acknowledgement",
        review.form_state.reread_plan_acknowledged,
    );
    append_disabled_activation_check_field(
        parent,
        &prefix,
        "post-restore-check",
        "Post-restore verification acknowledgement",
        review.form_state.post_restore_verification_acknowledged,
    );
    append_disabled_activation_check_field(
        parent,
        &prefix,
        "final-check",
        "Final confirmation acknowledgement",
        review.form_state.final_confirmation_acknowledged,
    );
    append_disabled_activation_multiline_field(
        parent,
        &prefix,
        "backup-plan-field",
        "Backup-before-write plan",
        &review.form_state.backup_before_write_plan,
    );
    append_disabled_activation_multiline_field(
        parent,
        &prefix,
        "restore-plan-field",
        "Restore plan",
        &review.form_state.restore_plan,
    );
    append_disabled_activation_multiline_field(
        parent,
        &prefix,
        "reread-plan-field",
        "Post-write reread plan",
        &review.form_state.post_write_reread_plan,
    );
    append_disabled_activation_multiline_field(
        parent,
        &prefix,
        "post-restore-plan-field",
        "Post-restore verification plan",
        &review.form_state.post_restore_verification_plan,
    );
    append_disabled_activation_multiline_field(
        parent,
        &prefix,
        "dry-run-summary-field",
        "Dry-run summary",
        &review.form_state.dry_run_summary,
    );
    append_disabled_activation_multiline_field(
        parent,
        &prefix,
        "touched-files-field",
        "Files that would be touched",
        &review.form_state.files_that_would_be_touched.join(", "),
    );
}

fn activation_form_widget_prefix(review: &ProductionActivationFormReview) -> String {
    review
        .widget_name
        .strip_suffix("-disabled")
        .unwrap_or(&review.widget_name)
        .to_string()
}

fn append_disabled_activation_text_field(
    parent: &gtk::Box,
    prefix: &str,
    suffix: &str,
    label: &str,
    value: &str,
) {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 3);
    row.append(&small_label(label));
    let entry = gtk::Entry::new();
    entry.set_widget_name(&format!("{prefix}-{suffix}-disabled"));
    entry.set_tooltip_text(Some("Read-only future activation form field."));
    entry.set_text(value);
    entry.set_editable(false);
    entry.set_sensitive(false);
    row.append(&entry);
    parent.append(&row);
}

fn append_disabled_activation_multiline_field(
    parent: &gtk::Box,
    prefix: &str,
    suffix: &str,
    label: &str,
    value: &str,
) {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 3);
    row.append(&small_label(label));
    let text = gtk::TextView::new();
    text.set_widget_name(&format!("{prefix}-{suffix}-disabled"));
    text.set_tooltip_text(Some("Read-only future activation safety-plan field."));
    text.set_wrap_mode(gtk::WrapMode::WordChar);
    text.set_editable(false);
    text.set_cursor_visible(false);
    text.set_sensitive(false);
    text.buffer().set_text(value);
    row.append(&text);
    parent.append(&row);
}

fn append_disabled_activation_check_field(
    parent: &gtk::Box,
    prefix: &str,
    suffix: &str,
    label: &str,
    checked: bool,
) {
    let check = gtk::CheckButton::with_label(label);
    check.set_widget_name(&format!("{prefix}-{suffix}-disabled"));
    check.set_tooltip_text(Some(
        "Read-only acknowledgement for future activation review.",
    ));
    check.set_active(checked);
    check.set_sensitive(false);
    parent.append(&check);
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

/// Review-only live runtime preview readiness card. Displays the real
/// capability matrix counts; the preview badge control is insensitive — no
/// runtime mutation can be triggered from this card.
fn runtime_preview_readiness_section() -> gtk::Frame {
    let summary = crate::runtime_preview::runtime_preview_matrix_summary();
    config_section(
        "Live runtime preview readiness",
        vec![
            format!(
                "{} of {} scalar settings are live-previewable today ({} direct, {} throttled): visual and layout values apply to the running session instantly and revert on Cancel.",
                summary.live_preview_supported + summary.live_preview_supported_with_throttle,
                summary.scalar_rows_classified,
                summary.live_preview_supported,
                summary.live_preview_supported_with_throttle,
            ),
            {
                let dead_man =
                    crate::runtime_preview_dead_man::dead_man_classification_summary();
                format!(
                    "{} settings support supervised preview with a recovery countdown; {} of them are armed today after passed per-row live proofs, and {} stay disarmed pending hardware or further proof.",
                    summary.dead_man_required,
                    dead_man.candidates,
                    summary.dead_man_required - dead_man.candidates,
                )
            },
            format!(
                "{} settings persist through config writes only, {} are blocked as high-risk, and {} are not proven yet.",
                summary.requires_config_write,
                summary.blocked_high_risk,
                summary.not_proven_yet,
            ),
            "Structured families do not live-preview in this phase: monitor/bind/device/permission are blocked as high-risk, gestures are blocked by record semantics, and animation/curve records are not proven yet.".to_string(),
            "Preview never writes the config file and never reloads Hyprland; saving persists once through the existing backup/write/reread path.".to_string(),
        ],
        Some((
            "Live preview controls are on each setting's detail pane",
            false,
        )),
    )
}

/// Safe Live Save Mode card: shows the live autoreload state and the
/// persisted-in-config state, offers the proven runtime-only enable/disable
/// transitions, and lets the user explicitly save the mode as the default
/// through the gated scalar Save. Enabling touches no file, runs no reload,
/// and is verified through read-only readback; the buttons route through
/// the safe_live_save_mode / persist_safe_live_save_mode modules (fixed
/// constant commands and a fixed setting/value).
fn safe_live_save_mode_section(model: &UiProjection) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-safe-live-save-mode");
    frame.set_tooltip_text(Some(
        "Safe Live Save Mode. GTK automation must not activate these controls.",
    ));
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(10);
    content.set_margin_end(10);
    content.append(&title_label("Safe Live Save Mode (recommended)"));

    let status = read_safe_live_save_mode_status_live();

    content.append(&body_label(status.explanation));
    let badge = body_label(&format!(
        "Status: {} (misc:disable_autoreload = {})",
        match status.state {
            SafeLiveSaveModeState::ActiveViaRuntime => "Active - saves cannot trigger a reload",
            SafeLiveSaveModeState::Inactive =>
                "Inactive - a config write now would reload Hyprland",
            SafeLiveSaveModeState::Unknown => "Unknown - failing closed",
        },
        status
            .runtime_disable_autoreload
            .as_deref()
            .unwrap_or("unreadable"),
    ));
    badge.set_widget_name("hyprland-settings-safe-live-save-mode-status");
    content.append(&badge);
    let persisted_state = crate::persist_safe_live_save_mode::read_persisted_safe_live_save_mode(
        &model.current_config,
    );
    let persisted_badge = body_label(&format!(
        "Persisted in config: {}",
        persisted_state.user_text()
    ));
    persisted_badge.set_widget_name("hyprland-settings-safe-live-save-mode-persisted");
    content.append(&persisted_badge);
    if let Some(reason) = status.blocked_reason {
        let blocked = small_label(&format!("Active-config save blocked: {reason}"));
        blocked.set_widget_name("hyprland-settings-safe-live-save-mode-blocked");
        content.append(&blocked);
    }
    content.append(&small_label(
        "Enabling this mode changes only the runtime value: no file is written and no reload runs (live-proven, instantly reversible). Saving it as the default writes misc:disable_autoreload = true to your config once through the normal gated Save (backup first, reread-verified) - your choice, never automatic.",
    ));

    let status_line = small_label("");
    status_line.set_widget_name("hyprland-settings-safe-live-save-mode-result");

    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let enable_button = gtk::Button::with_label("Enable Safe Live Save Mode");
    enable_button.set_widget_name("hyprland-settings-safe-live-save-enable");
    enable_button.set_tooltip_text(Some(
        "Disable autoreload at runtime (no file write, no reload). GTK automation must not activate this control.",
    ));
    enable_button.set_sensitive(status.state == SafeLiveSaveModeState::Inactive);
    let disable_button = gtk::Button::with_label("Disable Safe Live Save Mode");
    disable_button.set_widget_name("hyprland-settings-safe-live-save-disable");
    disable_button.set_tooltip_text(Some(
        "Restore Hyprland's default autoreload behavior at runtime. GTK automation must not activate this control.",
    ));
    disable_button.set_sensitive(status.state == SafeLiveSaveModeState::ActiveViaRuntime);
    {
        let status_line = status_line.clone();
        let disable_button = disable_button.clone();
        enable_button.connect_clicked(move |enable_button| {
            match enable_safe_live_save_mode_live() {
                Ok(receipt) => {
                    status_line.set_label(&receipt.status_text);
                    enable_button.set_sensitive(false);
                    disable_button.set_sensitive(true);
                }
                Err(error) => status_line.set_label(&error),
            }
        });
    }
    {
        let status_line = status_line.clone();
        let enable_button = enable_button.clone();
        disable_button.connect_clicked(
            move |disable_button| match disable_safe_live_save_mode_live() {
                Ok(receipt) => {
                    status_line.set_label(&receipt.status_text);
                    disable_button.set_sensitive(false);
                    enable_button.set_sensitive(true);
                }
                Err(error) => status_line.set_label(&error),
            },
        );
    }
    buttons.append(&enable_button);
    buttons.append(&disable_button);

    // Save as default: persist misc:disable_autoreload = true once through
    // the gated scalar Save. Enabled only while the runtime mode is active
    // (the write itself must not be able to reload the compositor) and not
    // already persisted.
    let persist_button = gtk::Button::with_label("Save as default");
    persist_button.set_widget_name("hyprland-settings-safe-live-save-persist");
    persist_button.set_tooltip_text(Some(
        "Persist misc:disable_autoreload = true to your config once through the gated Save (backup first, reread-verified, no reload). GTK automation must not activate this control.",
    ));
    persist_button.set_sensitive(
        status.state == SafeLiveSaveModeState::ActiveViaRuntime
            && persisted_state
                != crate::persist_safe_live_save_mode::PersistedSafeLiveSaveModeState::PersistedTrue,
    );
    {
        let status_line = status_line.clone();
        let persisted_badge = persisted_badge.clone();
        let known_setting_ids = model.known_setting_ids.clone();
        let discovery = model.config_discovery.clone();
        let current_config = model.current_config.clone();
        persist_button.connect_clicked(move |persist_button| {
            match crate::persist_safe_live_save_mode::persist_safe_live_save_mode_live(
                known_setting_ids.clone(),
                &discovery,
                &current_config,
            ) {
                Ok(receipt) => {
                    status_line.set_label(&receipt.status_text);
                    persisted_badge.set_label(
                        "Persisted in config: yes - Safe Live Save Mode is active from config after restarts",
                    );
                    persist_button.set_sensitive(false);
                }
                Err(error) => status_line.set_label(&error),
            }
        });
    }
    buttons.append(&persist_button);
    content.append(&buttons);
    content.append(&status_line);
    frame.set_child(Some(&content));
    frame
}

/// Shared picker plumbing: preview/keep/revert/cancel/save buttons wired to
/// a lazily created FamilyRecordPreviewController for the currently selected
/// record, with the same countdown/recovery semantics as before. All preview
/// actions route through the controller and every Save routes through the
/// gated persistence path; no commands are built here.
#[allow(clippy::too_many_arguments)]
fn record_picker_action_row(
    family: PickedFamily,
    family_slug: &str,
    combo: &gtk::ComboBoxText,
    values_for_save: Rc<dyn Fn() -> PickedRecordValues>,
    controller_slot: Rc<RefCell<Option<Rc<RefCell<FamilyRecordPreviewController>>>>>,
    discovery: &ConfigDiscovery,
    preview_button: &gtk::Button,
    parent: &gtk::Box,
) -> gtk::Label {
    let keep_button = gtk::Button::with_label("Keep changes");
    keep_button.set_sensitive(false);
    let revert_button = gtk::Button::with_label("Revert now");
    revert_button.set_sensitive(false);
    let cancel_button = gtk::Button::with_label("Cancel");
    cancel_button.set_sensitive(false);

    let status = small_label(
        "Supervised preview: modify-existing only, readback-verified, auto-revert on timeout.",
    );
    status.set_widget_name(&format!(
        "hyprland-settings-record-picker-status-{family_slug}"
    ));

    {
        let combo = combo.clone();
        let values_for_save = values_for_save.clone();
        let controller_slot = controller_slot.clone();
        let status = status.clone();
        let keep_button = keep_button.clone();
        let revert_button = revert_button.clone();
        let cancel_button = cancel_button.clone();
        preview_button.connect_clicked(move |_| {
            let Some(record) = combo.active_id() else {
                status.set_label("Select a record first.");
                return;
            };
            let need_new = controller_slot
                .borrow()
                .as_ref()
                .map(|controller| controller.borrow().record_name() != record.as_str())
                .unwrap_or(true);
            if need_new {
                if let Some(previous) = controller_slot.borrow_mut().take() {
                    let _ = previous.borrow_mut().revert_if_unconfirmed();
                }
                match FamilyRecordPreviewController::new_live(family, record.as_str()) {
                    Ok(controller) => {
                        let controller = Rc::new(RefCell::new(controller));
                        register_record_picker_controller(&controller);
                        *controller_slot.borrow_mut() = Some(controller);
                    }
                    Err(error) => {
                        status.set_label(&error.user_text());
                        return;
                    }
                }
            }
            let Some(controller) = controller_slot.borrow().as_ref().cloned() else {
                return;
            };
            let outcome = controller.borrow_mut().preview(values_for_save());
            match outcome {
                Ok(receipt) => {
                    status.set_label(&receipt.status_text);
                    keep_button.set_sensitive(true);
                    revert_button.set_sensitive(true);
                    cancel_button.set_sensitive(true);
                    let controller = controller.clone();
                    let status = status.clone();
                    let keep_button = keep_button.clone();
                    let revert_button = revert_button.clone();
                    let cancel_button = cancel_button.clone();
                    gtk::glib::timeout_add_local(
                        std::time::Duration::from_millis(1000),
                        move || {
                            let outcome = match controller.try_borrow_mut() {
                                Ok(mut controller) => controller.tick(1000),
                                Err(_) => return gtk::glib::ControlFlow::Continue,
                            };
                            match outcome {
                                Ok(Some(receipt)) => {
                                    status.set_label(&receipt.status_text);
                                    keep_button.set_sensitive(false);
                                    revert_button.set_sensitive(false);
                                    cancel_button.set_sensitive(false);
                                    gtk::glib::ControlFlow::Break
                                }
                                Ok(None) => {
                                    let phase = controller.borrow().phase();
                                    if phase == RecordPickerPhase::CountingDown {
                                        status.set_label(&format!(
                                            "Previewing live: auto-revert in {} seconds unless you Keep changes.",
                                            controller.borrow().remaining_seconds()
                                        ));
                                        gtk::glib::ControlFlow::Continue
                                    } else {
                                        gtk::glib::ControlFlow::Break
                                    }
                                }
                                Err(error) => {
                                    status.set_label(&error.user_text());
                                    gtk::glib::ControlFlow::Break
                                }
                            }
                        },
                    );
                }
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }
    {
        let controller_slot = controller_slot.clone();
        let status = status.clone();
        let revert_button = revert_button.clone();
        let cancel_button = cancel_button.clone();
        keep_button.connect_clicked(move |keep_button| {
            let Some(controller) = controller_slot.borrow().as_ref().cloned() else {
                return;
            };
            let outcome = controller.borrow_mut().keep();
            match outcome {
                Ok(receipt) => {
                    status.set_label(&receipt.status_text);
                    keep_button.set_sensitive(false);
                    revert_button.set_sensitive(true);
                    cancel_button.set_sensitive(false);
                }
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }
    {
        let controller_slot = controller_slot.clone();
        let status = status.clone();
        let keep_button = keep_button.clone();
        let cancel_button = cancel_button.clone();
        revert_button.connect_clicked(move |revert_button| {
            let Some(controller) = controller_slot.borrow().as_ref().cloned() else {
                return;
            };
            let outcome = controller.borrow_mut().revert_now();
            match outcome {
                Ok(receipt) => {
                    status.set_label(&receipt.status_text);
                    keep_button.set_sensitive(false);
                    revert_button.set_sensitive(false);
                    cancel_button.set_sensitive(false);
                }
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }
    {
        let controller_slot = controller_slot.clone();
        let status = status.clone();
        let keep_button = keep_button.clone();
        let revert_button = revert_button.clone();
        cancel_button.connect_clicked(move |cancel_button| {
            let Some(controller) = controller_slot.borrow().as_ref().cloned() else {
                return;
            };
            let outcome = controller.borrow_mut().cancel();
            match outcome {
                Ok(receipt) => {
                    status.set_label(&receipt.status_text);
                    keep_button.set_sensitive(false);
                    revert_button.set_sensitive(false);
                    cancel_button.set_sensitive(false);
                }
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }

    // Production Save: persist the selected record's values once through the
    // gated persistence flow (Safe Live Save Mode verified live inside).
    let save_button = gtk::Button::with_label("Save previewed value");
    save_button.set_widget_name(&format!(
        "hyprland-settings-record-picker-save-{family_slug}"
    ));
    {
        let combo = combo.clone();
        let status = status.clone();
        let discovery = discovery.clone();
        save_button.connect_clicked(move |_| {
            let Some(record) = combo.active_id() else {
                status.set_label("Select a record first.");
                return;
            };
            let outcome =
                save_picked_record_live(&discovery, family, record.as_str(), values_for_save());
            match outcome {
                Ok(receipt) => status.set_label(&receipt.status_text),
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }

    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    buttons.append(preview_button);
    buttons.append(&keep_button);
    buttons.append(&revert_button);
    buttons.append(&cancel_button);
    buttons.append(&save_button);
    parent.append(&buttons);
    parent.append(&status);
    status
}

/// Records that exist in the readback but cannot be picked, with the honest
/// reason, kept out of the main flow under an expander.
fn record_picker_blocked_expander(
    title: &str,
    family_slug: &str,
    blocked: Vec<(String, String)>,
    parent: &gtk::Box,
) {
    if blocked.is_empty() {
        return;
    }
    let expander = gtk::Expander::new(Some(title));
    expander.set_widget_name(&format!(
        "hyprland-settings-record-picker-blocked-{family_slug}"
    ));
    let blocked_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    for (name, reason) in blocked {
        blocked_box.append(&small_label(&format!("{name}: {reason}")));
    }
    expander.set_child(Some(&blocked_box));
    parent.append(&expander);
}

/// Animation record picker: existing overridden records from the readback,
/// editing of the proven fields (enabled, speed, bezier — existing curves
/// only), supervised preview, gated Save. The style field is shown as a
/// disabled row with the reason it is not editable.
fn animation_record_picker_group(discovery: &ConfigDiscovery, parent: &gtk::Box) {
    parent.append(&body_label("Animation records"));
    let entries = match list_animation_records_live() {
        Ok(entries) => entries,
        Err(error) => {
            parent.append(&small_label(&format!(
                "Animation records unavailable: {error}"
            )));
            return;
        }
    };
    let existing_curves: Vec<String> = list_curve_records_live()
        .map(|curves| curves.into_iter().map(|entry| entry.record.name).collect())
        .unwrap_or_default();
    let selectable: Vec<AnimationRecordEntry> = entries
        .iter()
        .filter(|entry| entry.save_supported)
        .cloned()
        .collect();
    let blocked: Vec<(String, String)> = entries
        .iter()
        .filter(|entry| !entry.save_supported)
        .map(|entry| {
            (
                entry.record.name.clone(),
                entry
                    .blocked_reason
                    .clone()
                    .unwrap_or_else(|| "not supported".to_string()),
            )
        })
        .collect();
    if selectable.is_empty() {
        parent.append(&small_label(
            "No editable animation records yet. Records appear here once your session exposes them.",
        ));
        record_picker_blocked_expander(
            "Animation records not yet supported",
            "animation",
            blocked,
            parent,
        );
        return;
    }

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let combo = gtk::ComboBoxText::new();
    combo.set_widget_name("hyprland-settings-record-picker-animation-records");
    for entry in &selectable {
        combo.append(
            Some(&entry.record.name),
            &format!("{} — {}", entry.record.name, entry.current_value_text),
        );
    }
    combo.set_active(Some(0));
    row.append(&combo);
    row.append(&body_label("Speed"));
    let spin = gtk::SpinButton::with_range(0.1, 20.0, 0.1);
    spin.set_digits(2);
    spin.set_widget_name("hyprland-settings-record-picker-animation-speed");
    row.append(&spin);
    row.append(&body_label("Enabled"));
    let enabled_switch = gtk::Switch::new();
    enabled_switch.set_valign(gtk::Align::Center);
    enabled_switch.set_widget_name("hyprland-settings-record-picker-animation-enabled");
    row.append(&enabled_switch);
    row.append(&body_label("Curve"));
    let bezier_combo = gtk::ComboBoxText::new();
    bezier_combo.set_widget_name("hyprland-settings-record-picker-animation-bezier");
    for curve in &existing_curves {
        bezier_combo.append(Some(curve), curve);
    }
    row.append(&bezier_combo);
    parent.append(&row);

    // The style field is deliberately not editable: no trusted evidence
    // enumerates the valid values and no live proof exists. Shown disabled
    // with the reason, per the not-proven UI rule.
    let style_blocked = small_label(ANIMATION_STYLE_BLOCKED_REASON);
    style_blocked.set_sensitive(false);
    style_blocked.set_widget_name("hyprland-settings-record-picker-animation-style-blocked");
    parent.append(&style_blocked);

    let value_label = small_label("");
    value_label.set_widget_name("hyprland-settings-record-picker-animation-value");
    parent.append(&value_label);
    let reason_label = small_label("");
    reason_label.set_widget_name("hyprland-settings-record-picker-animation-reason");
    parent.append(&reason_label);

    let preview_button = gtk::Button::with_label("Preview with recovery");
    preview_button.set_widget_name("hyprland-settings-record-picker-preview-animation");
    let controller_slot: Rc<RefCell<Option<Rc<RefCell<FamilyRecordPreviewController>>>>> =
        Rc::new(RefCell::new(None));

    {
        let selectable = selectable.clone();
        let value_label = value_label.clone();
        let reason_label = reason_label.clone();
        let spin = spin.clone();
        let enabled_switch = enabled_switch.clone();
        let bezier_combo = bezier_combo.clone();
        let preview_button = preview_button.clone();
        let controller_slot = controller_slot.clone();
        let update = move |combo: &gtk::ComboBoxText| {
            let Some(active) = combo.active_id() else {
                return;
            };
            let Some(entry) = selectable
                .iter()
                .find(|entry| entry.record.name == active.as_str())
            else {
                return;
            };
            value_label.set_label(&format!("Current: {}", entry.current_value_text));
            if let Ok(speed) = entry.record.speed.parse::<f64>() {
                spin.set_value(speed);
            }
            enabled_switch.set_active(entry.record.enabled == "1");
            if entry.record.bezier.is_empty() {
                bezier_combo.set_active_id(Some("default"));
            } else {
                bezier_combo.set_active_id(Some(&entry.record.bezier));
            }
            preview_button.set_sensitive(entry.preview_supported);
            reason_label.set_label(
                &entry
                    .blocked_reason
                    .clone()
                    .map(|reason| format!("Preview limited: {reason}"))
                    .unwrap_or_default(),
            );
            if let Some(previous) = controller_slot.borrow_mut().take() {
                let _ = previous.borrow_mut().revert_if_unconfirmed();
            }
        };
        update(&combo);
        combo.connect_changed(update);
    }

    let values_for_save: Rc<dyn Fn() -> PickedRecordValues> = {
        let spin = spin.clone();
        let enabled_switch = enabled_switch.clone();
        let bezier_combo = bezier_combo.clone();
        Rc::new(move || PickedRecordValues::AnimationRecord {
            enabled: enabled_switch.is_active(),
            speed: spin.value(),
            bezier: bezier_combo
                .active_id()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "default".to_string()),
        })
    };
    record_picker_action_row(
        PickedFamily::Animation,
        "animation",
        &combo,
        values_for_save,
        controller_slot,
        discovery,
        &preview_button,
        parent,
    );
    record_picker_blocked_expander(
        "Animation records not yet supported",
        "animation",
        blocked,
        parent,
    );
}

/// Bezier curve record picker: existing curves from the readback, control
/// point editing, supervised preview, gated Save.
fn curve_record_picker_group(discovery: &ConfigDiscovery, parent: &gtk::Box) {
    parent.append(&body_label("Bezier curves"));
    let entries = match list_curve_records_live() {
        Ok(entries) => entries,
        Err(error) => {
            parent.append(&small_label(&format!("Bezier curves unavailable: {error}")));
            return;
        }
    };
    let selectable: Vec<CurveRecordEntry> = entries
        .iter()
        .filter(|entry| entry.save_supported)
        .cloned()
        .collect();
    let blocked: Vec<(String, String)> = entries
        .iter()
        .filter(|entry| !entry.save_supported)
        .map(|entry| {
            (
                entry.record.name.clone(),
                entry
                    .blocked_reason
                    .clone()
                    .unwrap_or_else(|| "not supported".to_string()),
            )
        })
        .collect();
    if selectable.is_empty() {
        parent.append(&small_label(
            "No bezier curve in the runtime readback is currently supported for gated Save.",
        ));
        record_picker_blocked_expander("Curves not yet supported", "curve", blocked, parent);
        return;
    }

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let combo = gtk::ComboBoxText::new();
    combo.set_widget_name("hyprland-settings-record-picker-curve-records");
    for entry in &selectable {
        combo.append(
            Some(&entry.record.name),
            &format!("{} — {}", entry.record.name, entry.current_value_text),
        );
    }
    combo.set_active(Some(0));
    row.append(&combo);
    parent.append(&row);

    let points_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let mut spins = Vec::new();
    for (label, is_x) in [("X0", true), ("Y0", false), ("X1", true), ("Y1", false)] {
        points_row.append(&body_label(label));
        let spin = if is_x {
            gtk::SpinButton::with_range(0.0, 1.0, 0.01)
        } else {
            gtk::SpinButton::with_range(-1.0, 2.0, 0.01)
        };
        spin.set_digits(2);
        spin.set_widget_name(&format!(
            "hyprland-settings-record-picker-curve-{}",
            label.to_lowercase()
        ));
        points_row.append(&spin);
        spins.push(spin);
    }
    parent.append(&points_row);

    let value_label = small_label("");
    value_label.set_widget_name("hyprland-settings-record-picker-curve-value");
    parent.append(&value_label);

    let preview_button = gtk::Button::with_label("Preview with recovery");
    preview_button.set_widget_name("hyprland-settings-record-picker-preview-curve");
    let controller_slot: Rc<RefCell<Option<Rc<RefCell<FamilyRecordPreviewController>>>>> =
        Rc::new(RefCell::new(None));

    {
        let selectable = selectable.clone();
        let value_label = value_label.clone();
        let spins = spins.clone();
        let controller_slot = controller_slot.clone();
        let update = move |combo: &gtk::ComboBoxText| {
            let Some(active) = combo.active_id() else {
                return;
            };
            let Some(entry) = selectable
                .iter()
                .find(|entry| entry.record.name == active.as_str())
            else {
                return;
            };
            value_label.set_label(&format!(
                "Current control points: {}",
                entry.current_value_text
            ));
            for (spin, value) in spins.iter().zip([
                entry.record.x0,
                entry.record.y0,
                entry.record.x1,
                entry.record.y1,
            ]) {
                spin.set_value(value);
            }
            if let Some(previous) = controller_slot.borrow_mut().take() {
                let _ = previous.borrow_mut().revert_if_unconfirmed();
            }
        };
        update(&combo);
        combo.connect_changed(update);
    }

    let values_for_save: Rc<dyn Fn() -> PickedRecordValues> = {
        let spins = spins.clone();
        Rc::new(move || PickedRecordValues::CurvePoints {
            x0: spins[0].value(),
            y0: spins[1].value(),
            x1: spins[2].value(),
            y1: spins[3].value(),
        })
    };
    record_picker_action_row(
        PickedFamily::Curve,
        "curve",
        &combo,
        values_for_save,
        controller_slot,
        discovery,
        &preview_button,
        parent,
    );
    record_picker_blocked_expander("Curves not yet supported", "curve", blocked, parent);
}

/// Live supervised record picker for the two proven structured families.
fn structured_family_preview_controls_section(model: &UiProjection) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-family-preview-controls");
    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(10);
    content.set_margin_end(10);
    content.append(&title_label("Animations & curves"));
    content.append(&body_label(
        "Pick a record, preview it live with automatic recovery, and save it when you are happy.",
    ));
    animation_record_picker_group(&model.config_discovery, &content);
    curve_record_picker_group(&model.config_discovery, &content);
    content.append(&small_label(&format!(
        "Save writes once with a backup · {} · Full safety detail: Safety Details page",
        crate::ux_presentation::SAVE_GATE_CHIP
    )));
    frame.set_child(Some(&content));
    frame
}

/// Review-only structured-family live preview and persistence status card,
/// built from the evidence-backed family profiles. Display only.
fn structured_family_runtime_preview_status_section() -> gtk::Frame {
    let mut lines: Vec<String> = vec![
        "Structured-family records (monitors, binds, animations, curves, gestures, devices, permissions) have evidence-based live preview classifications:".to_string(),
    ];
    for profile in
        crate::structured_family_runtime_preview::structured_family_runtime_preview_profiles()
    {
        lines.push(format!(
            "{}: {} — {}{}",
            profile.family_id,
            profile.capability.as_str(),
            profile.ui_status,
            profile
                .blocked_reason
                .map(|reason| format!(" ({reason})"))
                .unwrap_or_default(),
        ));
    }
    lines.push(
        "Persistence: saving structured-family records to your config goes through the active-config pilot, which is currently blocked by autoreload (see the pilot card above). Copied-config rehearsal has passed; backups and restore proof exist for controlled targets.".to_string(),
    );
    lines.push(
        "Your config currently has misc:disable_autoreload = false, so writing hyprland.conf would reload Hyprland immediately. To run the pilot without a live reload, set misc:disable_autoreload = true first — or explicitly approve the one reload the write-and-restore pilot would cause.".to_string(),
    );
    let frame = config_section(
        "Structured-family live preview & persistence",
        lines,
        Some((
            "Supervised family preview (proven for existing animation/curve records)",
            false,
        )),
    );
    frame.set_widget_name("hyprland-settings-structured-family-preview-status");
    frame
}

/// Review-only, report-backed status card for the structured-family
/// controlled write executor and the active-config write pilot. Display only:
/// this card reads no executor state and can trigger nothing. The Apply
/// control is permanently insensitive until a future explicitly approved
/// pilot flow exists.
fn controlled_write_and_active_pilot_status_section() -> gtk::Frame {
    config_section(
        "Structured-family write status (review only)",
        vec![
            "Controlled-target writes: implemented and proven for test-owned, copied, and temporary configs with byte-exact backup, restore, rollback, and reread verification for all seven families.".to_string(),
            "Copied active-config rehearsal: proven — the active config content round-trips (backup, write, verify, restore, verify) in a temp copy without touching the real file.".to_string(),
            "First active config write pilot: blocked — compositor autoreload (misc:disable_autoreload=false) means a config write would reload Hyprland live, and runtime mutation is not approved.".to_string(),
            "Active real config writes, Hyprland reload, and runtime mutation remain disabled and require explicit approval.".to_string(),
        ],
        Some((
            "Apply to active config (blocked: requires pilot approval)",
            false,
        )),
    )
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
    model: &Rc<UiProjection>,
    selected_page_id: &str,
    standalone: &StandalonePages,
    settings_view: &gtk::Box,
    query: &str,
    tab_title: &gtk::Label,
    sections_box: &gtk::Box,
    detail_content: &gtk::Box,
    detail_popover: &gtk::Popover,
    config_selection_state: &Rc<RefCell<ConfigSelectionState>>,
) {
    if standalone.show_only(selected_page_id) {
        settings_view.set_visible(false);
        clear_sections_box(sections_box);
        render_empty_detail(detail_content);
        detail_popover.popdown();
        return;
    }
    settings_view.set_visible(true);
    render_settings_view(
        model,
        selected_page_id,
        query,
        tab_title,
        sections_box,
        detail_content,
        detail_popover,
        config_selection_state,
    );
}

/// Remove every child of the sections container except the detail popover
/// (which is parented here so its anchor coordinates match the rows).
fn clear_sections_box(sections_box: &gtk::Box) {
    let mut child = sections_box.first_child();
    while let Some(current) = child {
        child = current.next_sibling();
        if current.widget_name() != "hyprland-settings-detail-popover" {
            sections_box.remove(&current);
        }
    }
}

fn render_settings_view(
    model: &Rc<UiProjection>,
    selected_page_id: &str,
    query: &str,
    tab_title: &gtk::Label,
    sections_box: &gtk::Box,
    detail_content: &gtk::Box,
    detail_popover: &gtk::Popover,
    config_selection_state: &Rc<RefCell<ConfigSelectionState>>,
) {
    clear_sections_box(sections_box);
    render_empty_detail(detail_content);
    detail_popover.popdown();

    let Some(page) = crate::ux_presentation::page_spec(selected_page_id) else {
        tab_title.set_label("");
        return;
    };
    let Some(source_tab) = page.source_tab else {
        tab_title.set_label(page.label);
        return;
    };
    tab_title.set_label(page.label);

    let _ = source_tab;
    // Collect claimed rows from every model tab the page draws from
    // (its own tab plus any cross-tab extra sources).
    let mut views = Vec::new();
    for tab in crate::ux_presentation::page_source_tabs(page) {
        views.push((tab, search_projection(model, tab, query)));
    }
    let results: Vec<&SearchResult> = views
        .iter()
        .flat_map(|(tab, view)| {
            view.results.iter().filter(move |result| {
                crate::ux_presentation::page_claims_row_in_tab(
                    page,
                    tab,
                    &result.setting.official_setting,
                )
            })
        })
        .collect();
    let view = &views
        .first()
        .map(|(_, view)| view)
        .expect("page has at least one source tab");

    // Page-specific content above the sections. The record groups render
    // after the scalar sections (reference order: Bezier row, General,
    // then the record groups); the supervised-preview workbench lives on
    // the Safety Details page, not here.
    if page.id == "animations" {
        append_animations_bezier_row(model, sections_box);
    }
    if let Some(family_id) = page_structured_family(page.id) {
        append_structured_entries_card(model, family_id, sections_box);
    }

    if results.is_empty() {
        let empty = body_label(
            view.empty_title
                .as_deref()
                .unwrap_or("No settings match here."),
        );
        empty.set_margin_top(18);
        empty.set_halign(gtk::Align::Center);
        sections_box.append(&empty);
        return;
    }

    // Group rows by their curated section (falling back to the generated
    // subsection), preserving model order. Each group renders as a heading
    // OUTSIDE the card and a rounded card of rows below it.
    let mut groups: Vec<(String, Vec<&SearchResult>)> = Vec::new();
    for result in results {
        let section = crate::ux_presentation::section_for_row(
            &result.setting.official_setting,
            &result.setting.subsection,
            page.label,
        );
        match groups.iter_mut().find(|(name, _)| *name == section) {
            Some((_, rows)) => rows.push(result),
            None => groups.push((section, vec![result])),
        }
    }

    let page_lists: Rc<RefCell<Vec<gtk::ListBox>>> = Rc::new(RefCell::new(Vec::new()));
    for (section, rows) in groups {
        let heading_text = section.clone();
        let heading = body_label(&heading_text);
        heading.set_halign(gtk::Align::Start);
        heading.set_margin_top(14);
        heading.set_margin_start(6);
        heading.add_css_class("heading");
        heading.set_widget_name(&format!(
            "hyprland-settings-section-heading-{}",
            safe_widget_name(&heading_text)
        ));
        sections_box.append(&heading);

        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.add_css_class("boxed-list");
        list.set_widget_name(&format!(
            "hyprland-settings-section-card-{}",
            safe_widget_name(&heading_text)
        ));
        let mut row_ids = Vec::new();
        for result in &rows {
            row_ids.push(result.setting.row_id.clone());
            list.append(&build_setting_row(result, view.is_searching));
        }
        let row_ids = Rc::new(row_ids);
        {
            let model = Rc::clone(model);
            let detail_content = detail_content.clone();
            let detail_popover = detail_popover.clone();
            let config_selection_state = Rc::clone(config_selection_state);
            let sections_box = sections_box.clone();
            let page_lists = Rc::clone(&page_lists);
            list.connect_row_selected(move |current_list, row| {
                let Some(row) = row else {
                    return;
                };
                // Single selection across every card on the page.
                for other in page_lists.borrow().iter() {
                    if other != current_list {
                        other.unselect_all();
                    }
                }
                let Some(row_id) = row_ids.get(row.index() as usize) else {
                    return;
                };
                render_detail(&model, row_id, &detail_content, &config_selection_state);
                if let Some(bounds) = row.compute_bounds(&sections_box) {
                    detail_popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(
                        bounds.x() as i32,
                        bounds.y() as i32,
                        bounds.width() as i32,
                        bounds.height() as i32,
                    )));
                }
                detail_popover.popup();
            });
        }
        page_lists.borrow_mut().push(list.clone());
        sections_box.append(&list);
    }

    // Record groups follow the scalar sections on the Animations page.
    if page.id == "animations" {
        append_animation_record_groups(model, sections_box);
    }
}

/// Animations page: the Bezier Curve Editor entry row (icon, title,
/// description, chevron) rendered above the scalar sections. Activating it
/// opens the in-window editor dialog.
fn append_animations_bezier_row(model: &Rc<UiProjection>, parent: &gtk::Box) {
    let discovery = model.config_discovery.clone();
    let editor_list = gtk::ListBox::new();
    // Single selection (not None): clicking, keyboard focus, and assistive
    // tech all open the editor through the same selection path.
    editor_list.set_selection_mode(gtk::SelectionMode::Single);
    editor_list.add_css_class("boxed-list");
    editor_list.set_margin_top(8);
    let editor_row = gtk::ListBoxRow::new();
    editor_row.set_activatable(true);
    editor_row.set_widget_name("hyprland-settings-bezier-editor-row");
    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row_box.set_margin_top(10);
    row_box.set_margin_bottom(10);
    row_box.set_margin_start(12);
    row_box.set_margin_end(12);
    // Themed icon with fallbacks: bezier/curve glyphs where the icon theme
    // has them, otherwise a generic drawing icon that exists everywhere.
    let icon = gtk::Image::from_gicon(&gtk::gio::ThemedIcon::from_names(&[
        "path-mode-bezier-symbolic",
        "draw-bezier-curves-symbolic",
        "draw-bezier-curves",
        "draw-arc-symbolic",
        "image-x-generic-symbolic",
    ]));
    icon.set_valign(gtk::Align::Center);
    row_box.append(&icon);
    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    let title = body_label("Bezier Curve Editor");
    title.set_halign(gtk::Align::Start);
    text_box.append(&title);
    let subtitle = small_label("Create and manage animation curves");
    subtitle.set_halign(gtk::Align::Start);
    text_box.append(&subtitle);
    row_box.append(&text_box);
    let chevron = gtk::Image::from_icon_name("go-next-symbolic");
    chevron.set_valign(gtk::Align::Center);
    row_box.append(&chevron);
    editor_row.set_child(Some(&row_box));
    editor_list.append(&editor_row);
    {
        let discovery = discovery.clone();
        editor_list.connect_row_selected(move |list, row| {
            if row.is_none() {
                return;
            }
            list.unselect_all();
            open_bezier_editor_dialog(list, &discovery);
        });
    }
    parent.append(&editor_list);
}

/// Animation record groups in the reference section order (Global,
/// Windows & Layers, Fading, Workspaces, Other). Each record renders as a
/// switch row with a friendly "speed · curve" subtitle; the switch stages
/// the enabled flag for the row menu's supervised preview and gated Save
/// controls. Records with an explicit override that are not curated into a
/// named group land at the end of Other so nothing editable is hidden. Raw
/// record text and preview/save controls never render on the page itself.
fn append_animation_record_groups(model: &Rc<UiProjection>, parent: &gtk::Box) {
    let discovery = model.config_discovery.clone();
    let entries = match list_animation_records_live() {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    };
    if entries.is_empty() {
        return;
    }
    let curated = |name: &str| {
        crate::ux_presentation::ANIMATION_RECORD_GROUPS
            .iter()
            .any(|(_, names)| names.contains(&name))
    };
    for (heading_text, names) in crate::ux_presentation::ANIMATION_RECORD_GROUPS {
        let mut group_entries: Vec<&AnimationRecordEntry> = names
            .iter()
            .filter_map(|name| entries.iter().find(|entry| entry.record.name == *name))
            .collect();
        if *heading_text == "Other" {
            for entry in &entries {
                if entry.record.overridden
                    && !entry.record.name.starts_with("__")
                    && !curated(&entry.record.name)
                {
                    group_entries.push(entry);
                }
            }
        }
        if group_entries.is_empty() {
            continue;
        }
        let heading = body_label(heading_text);
        heading.set_halign(gtk::Align::Start);
        heading.set_margin_top(14);
        heading.set_margin_start(6);
        heading.add_css_class("heading");
        heading.set_widget_name(&format!(
            "hyprland-settings-animation-group-{}",
            safe_widget_name(heading_text)
        ));
        parent.append(&heading);
        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::None);
        list.add_css_class("boxed-list");
        list.set_widget_name(&format!(
            "hyprland-settings-animation-card-{}",
            safe_widget_name(heading_text)
        ));
        for entry in group_entries {
            list.append(&animation_record_row(entry, &discovery));
        }
        parent.append(&list);
    }
}

/// One animation record as a switch row: switch (staged enabled flag),
/// friendly title and subtitle, and — for save-supported records — the
/// menu button holding the supervised preview and gated Save controls.
fn animation_record_row(
    entry: &AnimationRecordEntry,
    discovery: &ConfigDiscovery,
) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_activatable(false);
    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row_box.set_margin_top(8);
    row_box.set_margin_bottom(8);
    row_box.set_margin_start(12);
    row_box.set_margin_end(12);

    let enabled_switch = gtk::Switch::new();
    enabled_switch.set_valign(gtk::Align::Center);
    enabled_switch.set_active(entry.record.enabled == "1");
    enabled_switch.set_sensitive(entry.save_supported);
    enabled_switch.set_widget_name(&format!(
        "hyprland-settings-animation-enabled-{}",
        safe_widget_name(&entry.record.name)
    ));
    enabled_switch.update_property(&[gtk::accessible::Property::Label(&format!(
        "{} enabled (stage, then preview or save from the row menu)",
        crate::ux_presentation::animation_record_display_name(&entry.record.name)
    ))]);
    row_box.append(&enabled_switch);

    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    let title = body_label(&crate::ux_presentation::animation_record_display_name(
        &entry.record.name,
    ));
    title.set_halign(gtk::Align::Start);
    text_box.append(&title);
    let subtitle = small_label(&crate::ux_presentation::animation_record_subtitle(
        &entry.record.enabled,
        &entry.record.speed,
        &entry.record.bezier,
        &entry.record.style,
        entry.record.overridden,
    ));
    subtitle.set_halign(gtk::Align::Start);
    text_box.append(&subtitle);
    row_box.append(&text_box);

    if entry.save_supported {
        let menu_button = gtk::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");
        menu_button.set_valign(gtk::Align::Center);
        menu_button.add_css_class("flat");
        menu_button.set_widget_name(&format!(
            "hyprland-settings-animation-menu-{}",
            safe_widget_name(&entry.record.name)
        ));
        let menu_popover = gtk::Popover::new();
        menu_popover.set_child(Some(&animation_record_menu_box(
            entry,
            discovery,
            &enabled_switch,
        )));
        menu_button.set_popover(Some(&menu_popover));
        row_box.append(&menu_button);
    }

    row.set_child(Some(&row_box));
    row
}

/// Compact per-record controls: speed, curve selector (existing curves
/// only), preview with recovery, and the gated Save. The enabled flag is
/// staged by the row's switch, passed in here so the menu reads the same
/// state the row shows. The style is preserved and never editable. Same
/// controller and save wiring as the proven picker path.
fn animation_record_menu_box(
    entry: &AnimationRecordEntry,
    discovery: &ConfigDiscovery,
    enabled_switch: &gtk::Switch,
) -> gtk::Box {
    let record_name = entry.record.name.clone();
    let enabled_switch = enabled_switch.clone();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(10);
    content.set_margin_end(10);

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.append(&small_label("Speed"));
    let speed_spin = gtk::SpinButton::with_range(0.1, 20.0, 0.1);
    speed_spin.set_digits(2);
    if let Ok(speed) = entry.record.speed.parse::<f64>() {
        speed_spin.set_value(speed);
    }
    controls.append(&speed_spin);
    controls.append(&small_label("Curve"));
    let curve_combo = gtk::ComboBoxText::new();
    let curves: Vec<String> = list_curve_records_live()
        .map(|entries| entries.into_iter().map(|curve| curve.record.name).collect())
        .unwrap_or_default();
    for curve in &curves {
        curve_combo.append(Some(curve), curve);
    }
    if entry.record.bezier.is_empty() {
        curve_combo.set_active_id(Some("default"));
    } else {
        curve_combo.set_active_id(Some(&entry.record.bezier));
    }
    controls.append(&curve_combo);
    content.append(&controls);

    let status = small_label("");
    status.set_halign(gtk::Align::Start);
    content.append(&status);

    let values_for_save: Rc<dyn Fn() -> PickedRecordValues> = {
        let enabled_switch = enabled_switch.clone();
        let speed_spin = speed_spin.clone();
        let curve_combo = curve_combo.clone();
        Rc::new(move || PickedRecordValues::AnimationRecord {
            enabled: enabled_switch.is_active(),
            speed: speed_spin.value(),
            bezier: curve_combo
                .active_id()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "default".to_string()),
        })
    };

    let controller_slot: Rc<RefCell<Option<Rc<RefCell<FamilyRecordPreviewController>>>>> =
        Rc::new(RefCell::new(None));
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let preview_button = gtk::Button::with_label("Preview with recovery");
    preview_button.set_sensitive(entry.preview_supported);
    let keep_button = gtk::Button::with_label("Keep changes");
    let revert_button = gtk::Button::with_label("Revert now");
    let save_button = gtk::Button::with_label("Save previewed value");

    {
        let record_name = record_name.clone();
        let controller_slot = Rc::clone(&controller_slot);
        let values_for_save = Rc::clone(&values_for_save);
        let status = status.clone();
        preview_button.connect_clicked(move |_| {
            if controller_slot.borrow().is_none() {
                match FamilyRecordPreviewController::new_live(PickedFamily::Animation, &record_name)
                {
                    Ok(controller) => {
                        let controller = Rc::new(RefCell::new(controller));
                        register_record_picker_controller(&controller);
                        *controller_slot.borrow_mut() = Some(controller);
                    }
                    Err(error) => {
                        status.set_label(&error.user_text());
                        return;
                    }
                }
            }
            let Some(controller) = controller_slot.borrow().as_ref().cloned() else {
                return;
            };
            let outcome = controller.borrow_mut().preview(values_for_save());
            match outcome {
                Ok(receipt) => status.set_label(&receipt.status_text),
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }
    {
        let controller_slot = Rc::clone(&controller_slot);
        let status = status.clone();
        keep_button.connect_clicked(move |_| {
            let Some(controller) = controller_slot.borrow().as_ref().cloned() else {
                return;
            };
            let outcome = controller.borrow_mut().keep();
            match outcome {
                Ok(receipt) => status.set_label(&receipt.status_text),
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }
    {
        let controller_slot = Rc::clone(&controller_slot);
        let status = status.clone();
        revert_button.connect_clicked(move |_| {
            let Some(controller) = controller_slot.borrow().as_ref().cloned() else {
                return;
            };
            let outcome = controller.borrow_mut().revert_now();
            match outcome {
                Ok(receipt) => status.set_label(&receipt.status_text),
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }
    {
        let record_name = record_name.clone();
        let discovery = discovery.clone();
        let values_for_save = Rc::clone(&values_for_save);
        let status = status.clone();
        save_button.connect_clicked(move |_| {
            match save_picked_record_live(
                &discovery,
                PickedFamily::Animation,
                &record_name,
                values_for_save(),
            ) {
                Ok(receipt) => status.set_label(&receipt.status_text),
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }
    buttons.append(&preview_button);
    buttons.append(&keep_button);
    buttons.append(&revert_button);
    buttons.append(&save_button);
    content.append(&buttons);
    content
}

/// The Bezier Curve Editor: a read-only graph of the existing curves plus
/// the proven curve picker controls (existing curves only, gated Save).
/// Presented as an in-window `adw::Dialog` overlay on the Animations page —
/// never a separate toplevel, so a tiling compositor cannot tile it as an
/// independent client.
fn open_bezier_editor_dialog(parent: &impl IsA<gtk::Widget>, discovery: &ConfigDiscovery) {
    let dialog = adw::Dialog::new();
    dialog.set_title("Bezier Curve Editor");
    dialog.set_content_width(760);
    dialog.set_content_height(640);
    dialog.set_widget_name("hyprland-settings-bezier-editor-dialog");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 10);
    content.set_margin_top(14);
    content.set_margin_bottom(14);
    content.set_margin_start(14);
    content.set_margin_end(14);
    content.append(&small_label(
        "Existing curves from your session. Edit control points below; Save is gated as always.",
    ));
    content.append(&bezier_graph_area());
    curve_record_picker_group(discovery, &content);
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&content)
        .build();

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());
    toolbar.set_content(Some(&scroll));
    dialog.set_child(Some(&toolbar));
    dialog.present(Some(parent));
}

/// Read-only graph of the existing curves (cubic bezier from each curve's
/// control points), drawn once when the editor opens.
fn bezier_graph_area() -> gtk::DrawingArea {
    let area = gtk::DrawingArea::new();
    area.set_content_width(480);
    area.set_content_height(220);
    area.set_widget_name("hyprland-settings-bezier-graph");
    let curves: Vec<(String, f64, f64, f64, f64)> = list_curve_records_live()
        .map(|entries| {
            entries
                .into_iter()
                .map(|entry| {
                    (
                        entry.record.name.clone(),
                        entry.record.x0,
                        entry.record.y0,
                        entry.record.x1,
                        entry.record.y1,
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    area.set_draw_func(move |_, context, width, height| {
        let width = width as f64;
        let height = height as f64;
        // Frame.
        context.set_source_rgba(0.5, 0.5, 0.5, 0.4);
        context.rectangle(0.5, 0.5, width - 1.0, height - 1.0);
        let _ = context.stroke();
        let palette = [
            (0.40, 0.65, 0.95),
            (0.55, 0.85, 0.55),
            (0.95, 0.65, 0.35),
            (0.85, 0.50, 0.85),
        ];
        for (index, (_name, x0, y0, x1, y1)) in curves.iter().enumerate() {
            let (red, green, blue) = palette[index % palette.len()];
            context.set_source_rgba(red, green, blue, 0.95);
            context.set_line_width(2.0);
            // Map unit space to the widget (y up, small margin).
            let map_x = |x: f64| 12.0 + x * (width - 24.0);
            let map_y = |y: f64| height - 12.0 - y * (height - 24.0);
            context.move_to(map_x(0.0), map_y(0.0));
            context.curve_to(
                map_x(*x0),
                map_y(*y0),
                map_x(*x1),
                map_y(*y1),
                map_x(1.0),
                map_y(1.0),
            );
            let _ = context.stroke();
        }
    });
    area
}

fn page_structured_family(page_id: &str) -> Option<&'static str> {
    match page_id {
        "keybinds" => Some("hl.bind"),
        "monitors" => Some("hl.monitor"),
        "gestures" => Some("hl.gesture"),
        "devices" => Some("hl.device"),
        "ecosystem" => Some("hl.permission"),
        _ => None,
    }
}

/// A compact read-only card of preserved config entries for one family,
/// grouped by source file. Locked rows only — no edit affordance.
fn append_structured_entries_card(model: &UiProjection, family_id: &str, parent: &gtk::Box) {
    let entries: Vec<crate::ui::model::UiStructuredEntry> = model
        .structured_families
        .iter()
        .filter(|family| family.family_id == family_id)
        .flat_map(|family| family.entries.iter().cloned())
        .collect();
    if entries.is_empty() {
        return;
    }
    let heading = body_label("From your config (read-only)");
    heading.set_halign(gtk::Align::Start);
    heading.set_margin_top(14);
    heading.set_margin_start(6);
    heading.add_css_class("heading");
    parent.append(&heading);
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    list.add_css_class("boxed-list");
    list.set_widget_name(&format!(
        "hyprland-settings-config-entries-{}",
        safe_widget_name(family_id)
    ));
    for entry in entries.iter().take(40) {
        let row = gtk::ListBoxRow::new();
        row.set_activatable(false);
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        row_box.set_margin_top(6);
        row_box.set_margin_bottom(6);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);
        let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        text_box.set_hexpand(true);
        let line = body_label(entry.raw_line.trim());
        line.set_halign(gtk::Align::Start);
        text_box.append(&line);
        let origin = small_label(&format!(
            "{} · line {}",
            entry.source_path, entry.line_number
        ));
        origin.set_halign(gtk::Align::Start);
        text_box.append(&origin);
        row_box.append(&text_box);
        let lock = gtk::Image::from_icon_name("system-lock-screen-symbolic");
        lock.set_valign(gtk::Align::Center);
        row_box.append(&lock);
        row.set_child(Some(&row_box));
        list.append(&row);
    }
    parent.append(&list);
}

fn build_setting_row(result: &SearchResult, include_context: bool) -> gtk::ListBoxRow {
    let setting = &result.setting;
    let row = gtk::ListBoxRow::new();
    row.set_widget_name(&format!(
        "hyprland-settings-setting-row-{}",
        safe_widget_name(&setting.row_id)
    ));
    row.update_property(&[gtk::accessible::Property::Label(
        &setting_row_accessibility_text(setting),
    )]);
    row.set_activatable(true);
    row.set_selectable(true);

    // Compact card row: label + one-line subtitle on the left, the value
    // and control on the right. Everything else lives behind the row's
    // on-demand detail surface.
    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row_box.set_widget_name("hyprland-settings-setting-row-content");
    row_box.set_margin_top(8);
    row_box.set_margin_bottom(8);
    row_box.set_margin_start(12);
    row_box.set_margin_end(12);

    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    text_box.set_halign(gtk::Align::Start);
    text_box.set_valign(gtk::Align::Center);
    let title_text = match crate::presentation_labels::display_label_for_row(&setting.row_id) {
        Some(matched) => matched.to_string(),
        None => crate::ux_presentation::row_display_title(
            &setting.label,
            &setting.tab_label,
            &setting.official_setting,
        ),
    };
    let title = body_label(&title_text);
    title.set_halign(gtk::Align::Start);
    text_box.append(&title);
    if include_context {
        let context = small_label(&format!(
            "In {} / {} · {}",
            setting.tab_label,
            setting.subsection,
            search_rank_label(result.rank)
        ));
        context.set_halign(gtk::Align::Start);
        text_box.append(&context);
    }
    if !setting.description.is_empty() {
        // One-line subtitle; the full description stays in the detail view.
        let subtitle = wrapped_small_label(&crate::ux_presentation::short_description(
            &setting.description,
        ));
        subtitle.set_halign(gtk::Align::Start);
        text_box.append(&subtitle);
    }
    if let Some(status) = friendly_row_attention_status(setting) {
        let attention = small_label(&status);
        attention.set_halign(gtk::Align::Start);
        text_box.append(&attention);
    }
    row_box.append(&text_box);

    let end_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    end_box.set_halign(gtk::Align::End);
    end_box.set_valign(gtk::Align::Center);
    // Routine status stays out of normal rows: only a compact badge for
    // states that genuinely need attention. Full detail lives in the
    // row's on-demand detail surface and Safety Details.
    if let Some(badge_text) = crate::ux_presentation::row_badge(
        crate::ux_presentation::status_chip_for_row(&setting.row_id),
        setting.current_value.status == CurrentValueSourceStatus::DuplicateConflict,
    ) {
        let badge = small_label(badge_text);
        badge.set_halign(gtk::Align::End);
        badge.add_css_class("dim-label");
        end_box.append(&badge);
    }
    attach_inline_row_control(&end_box, setting);
    row_box.append(&end_box);

    row.set_child(Some(&row_box));
    // Pending-change accent: an amber left edge while this row has a live
    // previewed value that is not saved, kept current by the ledger.
    register_pending_row_widget(&setting.row_id, &row);
    if pending_change_snapshots()
        .iter()
        .any(|snapshot| snapshot.row_id == setting.row_id)
    {
        row.add_css_class("hyprland-settings-row-pending");
        row.update_property(&[gtk::accessible::Property::Description("Pending change")]);
    }
    row
}

/// Build a lazily-connected apply closure for inline row controls. The
/// preview controller is created on first interaction only (creating one
/// per visible row would hammer the readback), registered for session-drop
/// revert, and drives the same offer/throttle/drain path as the detail
/// controls. Errors surface by styling the control (no tooltips); nothing
/// here can write config or reload — it is the existing preview executor.
/// Inline preview apply with pending-changes tracking: the controller is
/// registered in the pending ledger under its row identity, and every
/// state-changing operation notifies the pending-changes surfaces.
fn inline_preview_apply(
    row_id: String,
    official_setting: String,
    page_id: Option<&'static str>,
    throttle_ms: Option<u64>,
    feedback: gtk::Widget,
) -> Rc<dyn Fn(String)> {
    let controller_slot: Rc<RefCell<Option<Rc<RefCell<RuntimePreviewUiController>>>>> =
        Rc::new(RefCell::new(None));
    let drain_scheduled = Rc::new(std::cell::Cell::new(false));
    fn mark(feedback: &gtk::Widget, failed: bool) {
        if failed {
            feedback.add_css_class("error");
        } else {
            feedback.remove_css_class("error");
        }
    }
    Rc::new(move |value: String| {
        if controller_slot.borrow().is_none() {
            // Reuse the ledger's controller when the row already has one
            // (page re-renders rebuild controls but must keep the one
            // preview session and its true original).
            if let Some(existing) = pending_controller_for_row(&row_id) {
                *controller_slot.borrow_mut() = Some(existing);
            } else {
                match RuntimePreviewUiController::new_live(&row_id) {
                    Ok(controller) => {
                        let controller = Rc::new(RefCell::new(controller));
                        register_preview_controller(&controller);
                        if !official_setting.is_empty() {
                            register_pending_controller(
                                &row_id,
                                &official_setting,
                                page_id,
                                &controller,
                            );
                        }
                        *controller_slot.borrow_mut() = Some(controller);
                    }
                    Err(_) => {
                        mark(&feedback, true);
                        return;
                    }
                }
            }
        }
        let Some(controller) = controller_slot.borrow().as_ref().cloned() else {
            return;
        };
        let outcome = controller
            .borrow_mut()
            .offer_value(&value, preview_now_ms());
        match outcome {
            Ok(Some(_receipt)) => {
                mark(&feedback, false);
                notify_pending_changed();
            }
            Ok(None) => {
                if !drain_scheduled.get() {
                    drain_scheduled.set(true);
                    let controller = controller.clone();
                    let feedback = feedback.clone();
                    let drain_scheduled = drain_scheduled.clone();
                    let delay = throttle_ms.unwrap_or(150) + 10;
                    gtk::glib::timeout_add_local_once(
                        std::time::Duration::from_millis(delay),
                        move || {
                            drain_scheduled.set(false);
                            match controller.borrow_mut().drain_pending(preview_now_ms()) {
                                Ok(_) => mark(&feedback, false),
                                Err(_) => mark(&feedback, true),
                            }
                            notify_pending_changed();
                        },
                    );
                }
            }
            Err(_) => mark(&feedback, true),
        }
    })
}

/// Right-aligned inline control for a setting row, matching the row's
/// proven preview capability. Only rows the matrix classifies as
/// default-previewable get a live control (the same reversible preview
/// path as the detail surface — Save stays a separate gated step);
/// everything else keeps its quiet chip and opens details on demand.
/// The value an inline control displays before any interaction: live
/// runtime truth first (so the first user change is always a real change
/// relative to the session original the preview executor captures), then
/// the config line, then the official default, then the edit projection's
/// suggestion. Uniform css-gap readbacks collapse to the single-number
/// shorthand for compact display.
fn runtime_seed_initial_value(
    official_setting: &str,
    setting: &crate::ui::model::UiSetting,
) -> String {
    let value = crate::runtime_preview_ui_projection::read_runtime_option_live(official_setting)
        .or_else(|| setting.current_value.raw_value.clone())
        .or_else(|| {
            crate::official_defaults::official_default_value(official_setting).map(str::to_string)
        })
        .or_else(|| setting.edit.proposed_value.clone())
        .unwrap_or_default();
    collapse_uniform_gap(&value)
}

/// "5 5 5 5" -> "5" (css-gap shorthand) when every component is the same
/// number; anything else passes through unchanged.
fn collapse_uniform_gap(value: &str) -> String {
    let parts: Vec<&str> = value.split_whitespace().collect();
    if (2..=4).contains(&parts.len())
        && parts.iter().all(|part| part.parse::<f64>().is_ok())
        && parts.windows(2).all(|window| window[0] == window[1])
    {
        parts[0].to_string()
    } else {
        value.to_string()
    }
}

fn attach_inline_row_control(end_box: &gtk::Box, setting: &crate::ui::model::UiSetting) {
    let Some(row_state) = runtime_preview_ui_row_state(&setting.row_id) else {
        return;
    };
    let inline_page_id =
        crate::ux_presentation::page_for_row(&setting.tab_id, &setting.official_setting)
            .map(|page| page.id);
    if !row_state.preview_enabled {
        return;
    }
    let initial_value = runtime_seed_initial_value(&setting.official_setting, setting);
    let control_name = format!(
        "hyprland-settings-inline-control-{}",
        safe_widget_name(&setting.row_id)
    );
    match row_state.control_kind {
        RuntimePreviewUiControlKind::Switch => {
            let switch = gtk::Switch::new();
            switch.set_widget_name(&control_name);
            switch.set_valign(gtk::Align::Center);
            switch.set_active(matches!(
                initial_value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            ));
            let apply = inline_preview_apply(
                setting.row_id.clone(),
                setting.official_setting.clone(),
                inline_page_id,
                row_state.throttle_ms,
                switch.clone().upcast(),
            );
            switch.connect_state_set(move |_, active| {
                apply(if active { "true" } else { "false" }.to_string());
                gtk::glib::Propagation::Proceed
            });
            end_box.append(&switch);
        }
        RuntimePreviewUiControlKind::Slider | RuntimePreviewUiControlKind::SpinRow => {
            let (minimum, maximum, step, digits) = match row_state.control_kind {
                RuntimePreviewUiControlKind::Slider => {
                    let (minimum, maximum) = row_state.slider_bounds.unwrap_or((0.0, 100.0));
                    // Integer-valued options over a wide range read as
                    // integers ("1", not "1.00"); narrow ranges stay
                    // fractional (opacity-style 0..1 sliders).
                    let integral_value = initial_value
                        .trim()
                        .parse::<f64>()
                        .map(|value| value.fract() == 0.0)
                        .unwrap_or(false);
                    if maximum - minimum > 3.0 && integral_value {
                        (minimum, maximum, 1.0, 0)
                    } else {
                        (minimum, maximum, 0.05, 2)
                    }
                }
                _ => {
                    let (minimum, maximum) = row_state.slider_bounds.unwrap_or((0.0, 1000.0));
                    (minimum, maximum, 1.0, 0)
                }
            };
            let spin = gtk::SpinButton::with_range(minimum, maximum, step);
            spin.set_widget_name(&control_name);
            spin.set_digits(digits);
            spin.set_valign(gtk::Align::Center);
            if let Ok(value) = initial_value.trim().parse::<f64>() {
                spin.set_value(value);
            }
            let apply = inline_preview_apply(
                setting.row_id.clone(),
                setting.official_setting.clone(),
                inline_page_id,
                row_state.throttle_ms,
                spin.clone().upcast(),
            );
            spin.connect_value_changed(move |spin| {
                let value = spin.value();
                let rendered = if digits == 0 {
                    format!("{}", value as i64)
                } else {
                    format!("{value}")
                };
                apply(rendered);
            });
            end_box.append(&spin);
        }
        RuntimePreviewUiControlKind::Dropdown => {
            let combo = gtk::ComboBoxText::new();
            combo.set_widget_name(&control_name);
            combo.set_valign(gtk::Align::Center);
            for (raw_value, label) in &row_state.dropdown_choices {
                combo.append(Some(raw_value), label);
            }
            combo.set_active_id(Some(initial_value.trim()));
            let apply = inline_preview_apply(
                setting.row_id.clone(),
                setting.official_setting.clone(),
                inline_page_id,
                row_state.throttle_ms,
                combo.clone().upcast(),
            );
            combo.connect_changed(move |combo| {
                if let Some(active) = combo.active_id() {
                    apply(active.to_string());
                }
            });
            end_box.append(&combo);
        }
        RuntimePreviewUiControlKind::ColorEntry => {
            attach_inline_color_control(
                end_box,
                setting,
                &initial_value,
                &control_name,
                row_state.throttle_ms,
            );
        }
        RuntimePreviewUiControlKind::NoControl => {}
        RuntimePreviewUiControlKind::ValueEntry => {
            // Scalar numeric values get a compact −/+ spinner like the
            // reference rows; genuinely non-scalar text (multi-value
            // syntax, keywords) keeps the plain entry.
            if let Ok(value) = initial_value.trim().parse::<f64>() {
                let (step, digits) = if value.fract() == 0.0 {
                    (1.0, 0)
                } else {
                    (0.05, 2)
                };
                let spin = gtk::SpinButton::with_range(-100_000.0, 100_000.0, step);
                spin.set_widget_name(&control_name);
                spin.set_digits(digits);
                spin.set_valign(gtk::Align::Center);
                spin.set_value(value);
                let apply = inline_preview_apply(
                    setting.row_id.clone(),
                    setting.official_setting.clone(),
                    inline_page_id,
                    row_state.throttle_ms,
                    spin.clone().upcast(),
                );
                spin.connect_value_changed(move |spin| {
                    let value = spin.value();
                    let rendered = if digits == 0 {
                        format!("{}", value as i64)
                    } else {
                        format!("{value}")
                    };
                    apply(rendered);
                });
                end_box.append(&spin);
                return;
            }
            let entry = gtk::Entry::new();
            entry.set_widget_name(&control_name);
            entry.set_valign(gtk::Align::Center);
            entry.set_width_chars(10);
            entry.set_text(initial_value.trim());
            let apply = inline_preview_apply(
                setting.row_id.clone(),
                setting.official_setting.clone(),
                inline_page_id,
                row_state.throttle_ms,
                entry.clone().upcast(),
            );
            entry.connect_activate(move |entry| {
                apply(entry.text().to_string());
            });
            end_box.append(&entry);
        }
    }
}

/// Paint a swatch (single color) or preview strip (gradient) for a raw
/// color value. Unparseable values paint nothing — fail closed.
fn color_swatch_area(raw_value: &str, width: i32, height: i32) -> gtk::DrawingArea {
    live_swatch_area(
        Rc::new(RefCell::new(raw_value.trim().to_string())),
        width,
        height,
    )
}

/// A swatch that repaints from shared text state — the picker entry
/// updates the state and queues a redraw for its live preview.
fn live_swatch_area(shared: Rc<RefCell<String>>, width: i32, height: i32) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::new();
    area.set_content_width(width);
    area.set_content_height(height);
    area.set_draw_func(move |_, context, width, height| {
        let raw = shared.borrow().clone();
        let width = width as f64;
        let height = height as f64;
        // Rounded tile with a checkerboard behind the color so alpha is
        // visible.
        let radius = (height / 4.0).min(7.0);
        rounded_rect_path(
            context, 0.0, 0.0, width, height, radius, radius, radius, radius,
        );
        context.clip();
        draw_checkerboard(context, width, height);
        if let Some(color) = crate::ux_presentation::parse_hyprland_color(&raw) {
            context.set_source_rgba(
                color.red as f64 / 255.0,
                color.green as f64 / 255.0,
                color.blue as f64 / 255.0,
                color.alpha as f64 / 255.0,
            );
            context.rectangle(0.0, 0.0, width, height);
            let _ = context.fill();
            return;
        }
        if let Some((colors, _angle)) = crate::ux_presentation::parse_hyprland_gradient(&raw) {
            let gradient = gtk::cairo::LinearGradient::new(0.0, 0.0, width, 0.0);
            let last = (colors.len() - 1).max(1) as f64;
            for (index, color) in colors.iter().enumerate() {
                gradient.add_color_stop_rgba(
                    index as f64 / last,
                    color.red as f64 / 255.0,
                    color.green as f64 / 255.0,
                    color.blue as f64 / 255.0,
                    color.alpha as f64 / 255.0,
                );
            }
            let _ = context.set_source(&gradient);
            context.rectangle(0.0, 0.0, width, height);
            let _ = context.fill();
        }
    });
    area
}

/// Inline color row control in the stops style: one checkered swatch
/// button per color stop (clicking a swatch opens that stop's picker),
/// a small remove button beside each stop when more than one exists, an
/// add-stop button, an angle stepper for gradients, and a back-arrow
/// discard button that reappears while the row differs from its original
/// value. Every change routes through the same reversible preview path;
/// the exact raw token text is preserved and invalid input fails closed.
/// Unparseable current values fall back to a raw text editor.
fn attach_inline_color_control(
    end_box: &gtk::Box,
    setting: &crate::ui::model::UiSetting,
    initial_value: &str,
    control_name: &str,
    throttle_ms: Option<u64>,
) {
    let inline_page_id =
        crate::ux_presentation::page_for_row(&setting.tab_id, &setting.official_setting)
            .map(|page| page.id);
    // Int-typed single-color readbacks arrive as decimal u32 (AARRGGBB
    // bits); render them as bare hex so the stop-based control parses
    // them instead of falling back to the raw editor.
    let decimal_as_hex = initial_value
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|_| initial_value.trim().len() >= 8)
        .map(|bits| format!("{bits:08x}"));
    let initial_value: &str = decimal_as_hex.as_deref().unwrap_or(initial_value);
    #[derive(Clone)]
    struct ColorRowState {
        tokens: Vec<String>,
        angle: Option<u16>,
    }
    fn raw_from_state(state: &ColorRowState) -> String {
        let mut parts = state.tokens.clone();
        if state.tokens.len() > 1 {
            if let Some(angle) = state.angle {
                parts.push(format!("{angle}deg"));
            }
        }
        parts.join(" ")
    }
    fn state_from_raw(raw: &str) -> Option<ColorRowState> {
        let trimmed = raw.trim();
        if crate::ux_presentation::parse_hyprland_color(trimmed).is_some() {
            return Some(ColorRowState {
                tokens: vec![trimmed.to_string()],
                angle: None,
            });
        }
        if crate::ux_presentation::parse_hyprland_gradient(trimmed).is_some() {
            let mut tokens = Vec::new();
            let mut angle = None;
            for part in trimmed.split_whitespace() {
                if let Some(value) = part.strip_suffix("deg") {
                    angle = value.parse::<u16>().ok();
                } else {
                    tokens.push(part.to_string());
                }
            }
            return Some(ColorRowState { tokens, angle });
        }
        None
    }

    let Some(initial_state) = state_from_raw(initial_value) else {
        // Fail closed: not a recognized color form. Raw text editing only.
        attach_raw_color_entry(end_box, setting, initial_value, control_name, throttle_ms);
        return;
    };

    let original_raw = initial_value.trim().to_string();
    let state = Rc::new(RefCell::new(initial_state));
    let apply = inline_preview_apply(
        setting.row_id.clone(),
        setting.official_setting.clone(),
        inline_page_id,
        throttle_ms,
        end_box.clone().upcast(),
    );

    let container = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    container.set_widget_name(control_name);
    container.set_valign(gtk::Align::Center);
    end_box.append(&container);

    // Rebuild renders the whole control from state: discard arrow, stop
    // swatches with remove buttons, add button, angle stepper.
    let rebuild: Rc<RefCell<Box<dyn Fn()>>> = Rc::new(RefCell::new(Box::new(|| {})));
    let apply_state = {
        let state = Rc::clone(&state);
        let apply = apply.clone();
        let rebuild = Rc::clone(&rebuild);
        Rc::new(move || {
            let raw = raw_from_state(&state.borrow());
            let valid = crate::ux_presentation::parse_hyprland_color(&raw).is_some()
                || crate::ux_presentation::parse_hyprland_gradient(&raw).is_some();
            if valid {
                apply(raw);
            }
            (rebuild.borrow())();
        })
    };

    {
        let container = container.clone();
        let state = Rc::clone(&state);
        let original_raw = original_raw.clone();
        let apply_state_outer = Rc::clone(&apply_state);
        let control_name = control_name.to_string();
        let rebuild_slot = Rc::clone(&rebuild);
        *rebuild.borrow_mut() = Box::new(move || {
            while let Some(child) = container.first_child() {
                container.remove(&child);
            }
            let current_raw = raw_from_state(&state.borrow());
            let changed = current_raw != original_raw;

            // Discard: back-arrow that restores the original value through
            // the same preview path.
            if changed {
                let discard = gtk::Button::from_icon_name("edit-undo-symbolic");
                discard.add_css_class("flat");
                discard.set_widget_name(&format!("{control_name}-discard"));
                discard
                    .update_property(&[gtk::accessible::Property::Label("Discard color changes")]);
                let state = Rc::clone(&state);
                let original_raw = original_raw.clone();
                let apply_state = Rc::clone(&apply_state_outer);
                discard.connect_clicked(move |_| {
                    if let Some(reset) = state_from_raw(&original_raw) {
                        *state.borrow_mut() = reset;
                        apply_state();
                    }
                });
                container.append(&discard);
            }

            let token_count = state.borrow().tokens.len();
            for (index, token) in state.borrow().tokens.iter().enumerate() {
                let stop_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                let swatch = gtk::Button::new();
                swatch.add_css_class("flat");
                swatch.add_css_class("hyprland-settings-swatch-button");
                swatch.set_widget_name(&format!("{control_name}-stop-{index}"));
                swatch.update_property(&[gtk::accessible::Property::Label(&format!(
                    "Color stop {}",
                    index + 1
                ))]);
                swatch.set_child(Some(&color_swatch_area(token, 44, 26)));
                {
                    let state = Rc::clone(&state);
                    let apply_state = Rc::clone(&apply_state_outer);
                    let token = token.clone();
                    let swatch_ref = swatch.clone();
                    swatch.connect_clicked(move |_| {
                        let state = Rc::clone(&state);
                        let apply_state = Rc::clone(&apply_state);
                        open_color_stop_picker(
                            &swatch_ref,
                            &token,
                            Rc::new(move |new_token: String| {
                                if let Some(slot) = state.borrow_mut().tokens.get_mut(index) {
                                    *slot = new_token.clone();
                                }
                                apply_state();
                            }),
                        );
                    });
                }
                stop_box.append(&swatch);
                if token_count > 1 {
                    let remove = gtk::Button::from_icon_name("edit-clear-symbolic");
                    remove.add_css_class("flat");
                    remove.add_css_class("circular");
                    remove.set_widget_name(&format!("{control_name}-remove-{index}"));
                    remove
                        .update_property(&[gtk::accessible::Property::Label("Remove color stop")]);
                    let state = Rc::clone(&state);
                    let apply_state = Rc::clone(&apply_state_outer);
                    remove.connect_clicked(move |_| {
                        if state.borrow().tokens.len() > 1 {
                            state.borrow_mut().tokens.remove(index);
                            apply_state();
                        }
                    });
                    stop_box.append(&remove);
                }
                container.append(&stop_box);
            }

            // Gradient angle stepper (before the trailing add button, so
            // the row reads swatch/remove/…/angle/plus like the reference).
            if token_count > 1 {
                let angle_spin = gtk::SpinButton::with_range(0.0, 360.0, 1.0);
                angle_spin.set_valign(gtk::Align::Center);
                angle_spin.set_widget_name(&format!("{control_name}-angle"));
                angle_spin.set_value(f64::from(state.borrow().angle.unwrap_or(0)));
                let state = Rc::clone(&state);
                let apply_state = Rc::clone(&apply_state_outer);
                angle_spin.connect_value_changed(move |spin| {
                    state.borrow_mut().angle = Some(spin.value() as u16);
                    apply_state();
                });
                container.append(&angle_spin);
            }

            // Add a stop, at the row end (duplicates the last color; max 10
            // like the upstream gradient grammar comfortably allows).
            if token_count < 10 {
                let add = gtk::Button::from_icon_name("list-add-symbolic");
                add.add_css_class("flat");
                add.set_widget_name(&format!("{control_name}-add-stop"));
                add.update_property(&[gtk::accessible::Property::Label("Add color stop")]);
                let state = Rc::clone(&state);
                let apply_state = Rc::clone(&apply_state_outer);
                add.connect_clicked(move |_| {
                    let last = state.borrow().tokens.last().cloned();
                    if let Some(last) = last {
                        state.borrow_mut().tokens.push(last);
                        if state.borrow().angle.is_none() {
                            state.borrow_mut().angle = Some(0);
                        }
                        apply_state();
                    }
                });
                container.append(&add);
            }
            let _ = &rebuild_slot;
        });
    }
    (rebuild.borrow())();
}

thread_local! {
    /// Custom colors picked this session (raw rgba tokens, newest first).
    /// In-memory only: remembered swatches for the picker's Custom row.
    static CUSTOM_PICKER_COLORS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Canonical rgba(hex8) token for a parsed color.
fn rgba_token(color: crate::ux_presentation::ParsedColor) -> String {
    format!(
        "rgba({:02x}{:02x}{:02x}{:02x})",
        color.red, color.green, color.blue, color.alpha
    )
}

/// Bare hex8 text for the custom view's entry.
fn hex8_token(color: crate::ux_presentation::ParsedColor) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        color.red, color.green, color.blue, color.alpha
    )
}

/// Trace a rounded rectangle path with per-corner radii.
fn rounded_rect_path(
    context: &gtk::cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    top_left: f64,
    top_right: f64,
    bottom_right: f64,
    bottom_left: f64,
) {
    use std::f64::consts::FRAC_PI_2;
    context.new_sub_path();
    context.arc(
        x + width - top_right,
        y + top_right,
        top_right,
        -FRAC_PI_2,
        0.0,
    );
    context.arc(
        x + width - bottom_right,
        y + height - bottom_right,
        bottom_right,
        0.0,
        FRAC_PI_2,
    );
    context.arc(
        x + bottom_left,
        y + height - bottom_left,
        bottom_left,
        FRAC_PI_2,
        2.0 * FRAC_PI_2,
    );
    context.arc(
        x + top_left,
        y + top_left,
        top_left,
        2.0 * FRAC_PI_2,
        3.0 * FRAC_PI_2,
    );
    context.close_path();
}

/// Checkerboard backdrop so alpha stays readable under any color.
fn draw_checkerboard(context: &gtk::cairo::Context, width: f64, height: f64) {
    let square = 6.0;
    context.set_source_rgba(0.45, 0.45, 0.45, 1.0);
    context.rectangle(0.0, 0.0, width, height);
    let _ = context.fill();
    context.set_source_rgba(0.66, 0.66, 0.66, 1.0);
    let mut y = 0.0;
    let mut odd_row = false;
    while y < height {
        let mut x = if odd_row { square } else { 0.0 };
        while x < width {
            context.rectangle(x, y, square, square);
            let _ = context.fill();
            x += square * 2.0;
        }
        y += square;
        odd_row = !odd_row;
    }
}

/// Per-stop color picker: an opaque in-window dialog (`adw::Dialog`, never
/// a translucent popover) with a Cancel / "Pick a Color" / Select header
/// over two views. The palette view shows nine contiguous shade columns
/// plus a Custom row remembering this session's custom colors (selected
/// swatch carries a checkmark). The Custom view lays out an eyedropper
/// placeholder, live preview swatch, and hex entry over a vertical rainbow
/// hue bar, a continuous saturation/value area with crosshair, and a
/// checkerboard alpha slider. Select renders the choice in the original
/// token's format family and routes through the caller (preview path
/// only). The eyedropper stays disabled: screen color picking needs XDG
/// portal integration, deferred and documented in the implementation
/// report.
fn open_color_stop_picker(parent: &gtk::Button, token: &str, on_apply: Rc<dyn Fn(String)>) {
    let dialog = adw::Dialog::new();
    dialog.set_title("Pick a Color");
    dialog.set_content_width(470);
    dialog.set_widget_name("hyprland-settings-color-picker-dialog");

    let initial = crate::ux_presentation::parse_hyprland_color(token).unwrap_or(
        crate::ux_presentation::ParsedColor {
            red: 0xff,
            green: 0xff,
            blue: 0xff,
            alpha: 0xff,
        },
    );
    let chosen = Rc::new(RefCell::new(initial));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(12);
    content.set_margin_bottom(14);
    content.set_margin_start(14);
    content.set_margin_end(14);

    // Header: Cancel | Pick a Color | Select — both ends real buttons.
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let cancel = gtk::Button::with_label("Cancel");
    let heading = gtk::Label::new(Some("Pick a Color"));
    heading.set_hexpand(true);
    heading.add_css_class("heading");
    let select = gtk::Button::with_label("Select");
    select.add_css_class("suggested-action");
    header.append(&cancel);
    header.append(&heading);
    header.append(&select);
    content.append(&header);

    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    stack.set_hhomogeneous(false);
    stack.set_vhomogeneous(false);

    // Shared chosen-color state: preview token plus the drawing areas that
    // repaint whenever the choice changes.
    let preview_state = Rc::new(RefCell::new(rgba_token(initial)));
    let redraw_targets: Rc<RefCell<Vec<gtk::DrawingArea>>> = Rc::new(RefCell::new(Vec::new()));
    let set_chosen: Rc<dyn Fn(crate::ux_presentation::ParsedColor)> = {
        let chosen = Rc::clone(&chosen);
        let preview_state = Rc::clone(&preview_state);
        let redraw_targets = Rc::clone(&redraw_targets);
        Rc::new(move |color: crate::ux_presentation::ParsedColor| {
            *chosen.borrow_mut() = color;
            *preview_state.borrow_mut() = rgba_token(color);
            for area in redraw_targets.borrow().iter() {
                area.queue_draw();
            }
        })
    };

    // ── Palette view: nine contiguous shade columns, then Custom row. ──
    let palette = crate::ux_presentation::picker_palette_columns();
    let palette_tokens: std::collections::HashSet<String> = palette
        .iter()
        .flatten()
        .map(|color| rgba_token(*color))
        .collect();
    let palette_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let columns_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    columns_box.set_halign(gtk::Align::Center);
    columns_box.set_widget_name("hyprland-settings-color-palette-grid");
    let shade_count = palette.first().map(|column| column.len()).unwrap_or(0);
    for column in &palette {
        let column_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        for (index, color) in column.iter().enumerate() {
            let top = index == 0;
            let bottom = index + 1 == shade_count;
            let tile = gtk::Button::new();
            tile.add_css_class("hyprland-settings-palette-tile");
            tile.update_property(&[gtk::accessible::Property::Label(&format!(
                "Palette color {}",
                rgba_token(*color)
            ))]);
            let area = gtk::DrawingArea::new();
            area.set_content_width(42);
            area.set_content_height(32);
            let color_for_draw = *color;
            area.set_draw_func(move |_, context, width, height| {
                let radius_top = if top { 8.0 } else { 0.0 };
                let radius_bottom = if bottom { 8.0 } else { 0.0 };
                rounded_rect_path(
                    context,
                    0.0,
                    0.0,
                    width as f64,
                    height as f64,
                    radius_top,
                    radius_top,
                    radius_bottom,
                    radius_bottom,
                );
                context.set_source_rgb(
                    color_for_draw.red as f64 / 255.0,
                    color_for_draw.green as f64 / 255.0,
                    color_for_draw.blue as f64 / 255.0,
                );
                let _ = context.fill();
            });
            tile.set_child(Some(&area));
            let set_chosen = Rc::clone(&set_chosen);
            let color_for_click = *color;
            tile.connect_clicked(move |_| set_chosen(color_for_click));
            column_box.append(&tile);
        }
        columns_box.append(&column_box);
    }
    palette_box.append(&columns_box);

    let custom_caption = small_label("Custom");
    custom_caption.set_halign(gtk::Align::Start);
    palette_box.append(&custom_caption);
    let custom_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    custom_row.set_widget_name("hyprland-settings-color-custom-swatch-row");
    let open_custom = gtk::Button::from_icon_name("list-add-symbolic");
    open_custom.set_widget_name("hyprland-settings-color-open-custom");
    open_custom.update_property(&[gtk::accessible::Property::Label("Custom color")]);
    custom_row.append(&open_custom);
    let mut swatch_tokens: Vec<String> = CUSTOM_PICKER_COLORS.with(|store| store.borrow().clone());
    let current_token = rgba_token(initial);
    if !swatch_tokens.contains(&current_token) {
        swatch_tokens.push(current_token.clone());
    }
    for swatch_token in swatch_tokens {
        let Some(color) = crate::ux_presentation::parse_hyprland_color(&swatch_token) else {
            continue;
        };
        let button = gtk::Button::new();
        button.add_css_class("hyprland-settings-palette-tile");
        button.update_property(&[gtk::accessible::Property::Label(&format!(
            "Custom color {swatch_token}"
        ))]);
        let area = gtk::DrawingArea::new();
        area.set_content_width(46);
        area.set_content_height(32);
        let preview_state_for_draw = Rc::clone(&preview_state);
        let token_for_draw = swatch_token.clone();
        area.set_draw_func(move |_, context, width, height| {
            let width = width as f64;
            let height = height as f64;
            rounded_rect_path(context, 0.0, 0.0, width, height, 8.0, 8.0, 8.0, 8.0);
            context.clip();
            draw_checkerboard(context, width, height);
            context.set_source_rgba(
                color.red as f64 / 255.0,
                color.green as f64 / 255.0,
                color.blue as f64 / 255.0,
                color.alpha as f64 / 255.0,
            );
            context.rectangle(0.0, 0.0, width, height);
            let _ = context.fill();
            if *preview_state_for_draw.borrow() == token_for_draw {
                // Selected checkmark: white halo under a dark stroke.
                let points = [
                    (width * 0.32, height * 0.52),
                    (width * 0.45, height * 0.68),
                    (width * 0.70, height * 0.34),
                ];
                for (stroke_width, level) in [(4.5, 0.95), (2.0, 0.12)] {
                    context.set_source_rgba(level, level, level, 0.95);
                    context.set_line_width(stroke_width);
                    context.move_to(points[0].0, points[0].1);
                    context.line_to(points[1].0, points[1].1);
                    context.line_to(points[2].0, points[2].1);
                    let _ = context.stroke();
                }
            }
        });
        redraw_targets.borrow_mut().push(area.clone());
        button.set_child(Some(&area));
        let set_chosen = Rc::clone(&set_chosen);
        button.connect_clicked(move |_| set_chosen(color));
        custom_row.append(&button);
    }
    palette_box.append(&custom_row);
    stack.add_named(&palette_box, Some("palette"));

    // ── Custom view: eyedropper | preview | hex, hue bar + SV, alpha. ──
    let custom_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    custom_box.set_widget_name("hyprland-settings-color-custom-view");

    let top_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let eyedropper = gtk::Button::from_icon_name("color-select-symbolic");
    eyedropper.add_css_class("circular");
    eyedropper.set_sensitive(false);
    eyedropper.set_widget_name("hyprland-settings-color-eyedropper");
    eyedropper.update_property(&[gtk::accessible::Property::Label(
        "Pick color from screen (not available yet)",
    )]);
    top_row.append(&eyedropper);
    let live_preview = live_swatch_area(Rc::clone(&preview_state), 220, 34);
    live_preview.set_hexpand(true);
    redraw_targets.borrow_mut().push(live_preview.clone());
    top_row.append(&live_preview);
    let hex_entry = gtk::Entry::new();
    hex_entry.set_width_chars(9);
    hex_entry.set_max_length(8);
    hex_entry.set_text(&hex8_token(initial));
    hex_entry.update_property(&[gtk::accessible::Property::Label("Hex color")]);
    top_row.append(&hex_entry);
    custom_box.append(&top_row);

    let initial_hsv = crate::ux_presentation::rgb_to_hsv(initial.red, initial.green, initial.blue);
    let hue_state = Rc::new(std::cell::Cell::new(initial_hsv.0));
    let sat_state = Rc::new(std::cell::Cell::new(initial_hsv.1));
    let val_state = Rc::new(std::cell::Cell::new(initial_hsv.2));
    let alpha_state = Rc::new(std::cell::Cell::new(initial.alpha));

    let main_area = gtk::Box::new(gtk::Orientation::Horizontal, 10);

    // Vertical rainbow hue bar with a knob at the current hue.
    let hue_bar = gtk::DrawingArea::new();
    hue_bar.set_content_width(18);
    hue_bar.set_content_height(250);
    hue_bar.set_widget_name("hyprland-settings-color-hue");
    hue_bar.update_property(&[gtk::accessible::Property::Label("Hue")]);
    {
        let hue_state = Rc::clone(&hue_state);
        hue_bar.set_draw_func(move |_, context, width, height| {
            let width = width as f64;
            let height = height as f64;
            rounded_rect_path(context, 0.0, 0.0, width, height, 8.0, 8.0, 8.0, 8.0);
            context.clip();
            let gradient = gtk::cairo::LinearGradient::new(0.0, 0.0, 0.0, height);
            for step in 0..=6 {
                let hue = step as f64 * 60.0;
                let (red, green, blue) = crate::ux_presentation::hsv_to_rgb(hue, 1.0, 1.0);
                gradient.add_color_stop_rgb(
                    step as f64 / 6.0,
                    red as f64 / 255.0,
                    green as f64 / 255.0,
                    blue as f64 / 255.0,
                );
            }
            let _ = context.set_source(&gradient);
            context.rectangle(0.0, 0.0, width, height);
            let _ = context.fill();
            // Knob.
            let knob_y = (hue_state.get() / 360.0 * height).clamp(6.0, height - 6.0);
            context.set_source_rgba(1.0, 1.0, 1.0, 0.95);
            context.set_line_width(2.5);
            context.arc(width / 2.0, knob_y, 5.5, 0.0, std::f64::consts::TAU);
            let _ = context.stroke();
            context.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            context.set_line_width(1.0);
            context.arc(width / 2.0, knob_y, 7.0, 0.0, std::f64::consts::TAU);
            let _ = context.stroke();
        });
    }
    redraw_targets.borrow_mut().push(hue_bar.clone());
    main_area.append(&hue_bar);

    // Continuous saturation/value area with full-length crosshair lines.
    let sv_area = gtk::DrawingArea::new();
    sv_area.set_content_width(340);
    sv_area.set_content_height(250);
    sv_area.set_hexpand(true);
    sv_area.set_widget_name("hyprland-settings-color-sv-area");
    {
        let hue_state = Rc::clone(&hue_state);
        let sat_state = Rc::clone(&sat_state);
        let val_state = Rc::clone(&val_state);
        sv_area.set_draw_func(move |_, context, width, height| {
            let width = width as f64;
            let height = height as f64;
            rounded_rect_path(context, 0.0, 0.0, width, height, 6.0, 6.0, 6.0, 6.0);
            context.clip();
            // Continuous SV plane: white -> pure hue horizontally, then a
            // transparent -> black vertical overlay.
            let (red, green, blue) = crate::ux_presentation::hsv_to_rgb(hue_state.get(), 1.0, 1.0);
            let saturation_gradient = gtk::cairo::LinearGradient::new(0.0, 0.0, width, 0.0);
            saturation_gradient.add_color_stop_rgb(0.0, 1.0, 1.0, 1.0);
            saturation_gradient.add_color_stop_rgb(
                1.0,
                red as f64 / 255.0,
                green as f64 / 255.0,
                blue as f64 / 255.0,
            );
            let _ = context.set_source(&saturation_gradient);
            context.rectangle(0.0, 0.0, width, height);
            let _ = context.fill();
            let value_gradient = gtk::cairo::LinearGradient::new(0.0, 0.0, 0.0, height);
            value_gradient.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 0.0);
            value_gradient.add_color_stop_rgba(1.0, 0.0, 0.0, 0.0, 1.0);
            let _ = context.set_source(&value_gradient);
            context.rectangle(0.0, 0.0, width, height);
            let _ = context.fill();
            // Crosshair.
            let marker_x = sat_state.get() * width;
            let marker_y = (1.0 - val_state.get()) * height;
            context.set_source_rgba(1.0, 1.0, 1.0, 0.85);
            context.set_line_width(1.0);
            context.move_to(marker_x, 0.0);
            context.line_to(marker_x, height);
            let _ = context.stroke();
            context.move_to(0.0, marker_y);
            context.line_to(width, marker_y);
            let _ = context.stroke();
            context.set_line_width(2.0);
            context.arc(marker_x, marker_y, 6.0, 0.0, std::f64::consts::TAU);
            let _ = context.stroke();
        });
    }
    redraw_targets.borrow_mut().push(sv_area.clone());
    main_area.append(&sv_area);
    custom_box.append(&main_area);

    // Checkerboard alpha slider: transparent -> opaque current color.
    let alpha_area = gtk::DrawingArea::new();
    alpha_area.set_content_height(22);
    alpha_area.set_hexpand(true);
    alpha_area.set_widget_name("hyprland-settings-color-alpha");
    alpha_area.update_property(&[gtk::accessible::Property::Label("Alpha")]);
    {
        let hue_state = Rc::clone(&hue_state);
        let sat_state = Rc::clone(&sat_state);
        let val_state = Rc::clone(&val_state);
        let alpha_state = Rc::clone(&alpha_state);
        alpha_area.set_draw_func(move |_, context, width, height| {
            let width = width as f64;
            let height = height as f64;
            rounded_rect_path(context, 0.0, 0.0, width, height, 10.0, 10.0, 10.0, 10.0);
            context.clip();
            draw_checkerboard(context, width, height);
            let (red, green, blue) = crate::ux_presentation::hsv_to_rgb(
                hue_state.get(),
                sat_state.get(),
                val_state.get(),
            );
            let (red, green, blue) = (
                red as f64 / 255.0,
                green as f64 / 255.0,
                blue as f64 / 255.0,
            );
            let gradient = gtk::cairo::LinearGradient::new(0.0, 0.0, width, 0.0);
            gradient.add_color_stop_rgba(0.0, red, green, blue, 0.0);
            gradient.add_color_stop_rgba(1.0, red, green, blue, 1.0);
            let _ = context.set_source(&gradient);
            context.rectangle(0.0, 0.0, width, height);
            let _ = context.fill();
            // Knob.
            let knob_x = (f64::from(alpha_state.get()) / 255.0 * width).clamp(9.0, width - 9.0);
            context.set_source_rgba(red, green, blue, 1.0);
            context.arc(knob_x, height / 2.0, 8.0, 0.0, std::f64::consts::TAU);
            let _ = context.fill();
            context.set_source_rgba(1.0, 1.0, 1.0, 0.95);
            context.set_line_width(2.0);
            context.arc(knob_x, height / 2.0, 8.0, 0.0, std::f64::consts::TAU);
            let _ = context.stroke();
        });
    }
    redraw_targets.borrow_mut().push(alpha_area.clone());
    custom_box.append(&alpha_area);

    stack.add_named(&custom_box, Some("custom"));
    content.append(&stack);

    // ── Wiring. ──
    let sync_from_hsv: Rc<dyn Fn()> = {
        let hue_state = Rc::clone(&hue_state);
        let sat_state = Rc::clone(&sat_state);
        let val_state = Rc::clone(&val_state);
        let alpha_state = Rc::clone(&alpha_state);
        let set_chosen = Rc::clone(&set_chosen);
        let hex_entry = hex_entry.clone();
        Rc::new(move || {
            let (red, green, blue) = crate::ux_presentation::hsv_to_rgb(
                hue_state.get(),
                sat_state.get(),
                val_state.get(),
            );
            let color = crate::ux_presentation::ParsedColor {
                red,
                green,
                blue,
                alpha: alpha_state.get(),
            };
            set_chosen(color);
            hex_entry.set_text(&hex8_token(color));
        })
    };
    {
        // Hue bar: click and drag set the hue from the y position.
        let hue_state = Rc::clone(&hue_state);
        let sync = Rc::clone(&sync_from_hsv);
        let bar = hue_bar.clone();
        let update_hue = move |y: f64| {
            let height = f64::from(bar.content_height()).max(1.0);
            hue_state.set((y / height * 360.0).clamp(0.0, 360.0));
            sync();
        };
        let click = gtk::GestureClick::new();
        {
            let update_hue = update_hue.clone();
            click.connect_pressed(move |_, _, _, y| update_hue(y));
        }
        hue_bar.add_controller(click);
        let drag = gtk::GestureDrag::new();
        {
            let update_hue = update_hue.clone();
            drag.connect_drag_update(move |gesture, _, dy| {
                if let Some((_, start_y)) = gesture.start_point() {
                    update_hue(start_y + dy);
                }
            });
        }
        hue_bar.add_controller(drag);
    }
    {
        // SV area: click and drag set saturation/value.
        let sat_state = Rc::clone(&sat_state);
        let val_state = Rc::clone(&val_state);
        let sync = Rc::clone(&sync_from_hsv);
        let area = sv_area.clone();
        let update_sv = move |x: f64, y: f64| {
            let width = f64::from(area.width()).max(1.0);
            let height = f64::from(area.height()).max(1.0);
            sat_state.set((x / width).clamp(0.0, 1.0));
            val_state.set((1.0 - y / height).clamp(0.0, 1.0));
            sync();
        };
        let click = gtk::GestureClick::new();
        {
            let update_sv = update_sv.clone();
            click.connect_pressed(move |_, _, x, y| update_sv(x, y));
        }
        sv_area.add_controller(click);
        let drag = gtk::GestureDrag::new();
        {
            let update_sv = update_sv.clone();
            drag.connect_drag_update(move |gesture, dx, dy| {
                if let Some((start_x, start_y)) = gesture.start_point() {
                    update_sv(start_x + dx, start_y + dy);
                }
            });
        }
        sv_area.add_controller(drag);
    }
    {
        // Alpha slider: click and drag set the alpha from the x position.
        let alpha_state = Rc::clone(&alpha_state);
        let sync = Rc::clone(&sync_from_hsv);
        let area = alpha_area.clone();
        let update_alpha = move |x: f64| {
            let width = f64::from(area.width()).max(1.0);
            alpha_state.set(((x / width) * 255.0).clamp(0.0, 255.0) as u8);
            sync();
        };
        let click = gtk::GestureClick::new();
        {
            let update_alpha = update_alpha.clone();
            click.connect_pressed(move |_, _, x, _| update_alpha(x));
        }
        alpha_area.add_controller(click);
        let drag = gtk::GestureDrag::new();
        {
            let update_alpha = update_alpha.clone();
            drag.connect_drag_update(move |gesture, dx, _| {
                if let Some((start_x, _)) = gesture.start_point() {
                    update_alpha(start_x + dx);
                }
            });
        }
        alpha_area.add_controller(drag);
    }
    {
        // Manual hex entry: 6 or 8 hex digits, updates the HSV state too.
        let chosen = Rc::clone(&chosen);
        let set_chosen = Rc::clone(&set_chosen);
        let hue_state = Rc::clone(&hue_state);
        let sat_state = Rc::clone(&sat_state);
        let val_state = Rc::clone(&val_state);
        let alpha_state = Rc::clone(&alpha_state);
        hex_entry.connect_changed(move |entry| {
            let text = entry.text().to_string();
            let candidate = if text.len() == 6 {
                crate::ux_presentation::parse_hyprland_color(&format!("rgb({text})"))
            } else if text.len() == 8 {
                crate::ux_presentation::parse_hyprland_color(&format!("rgba({text})"))
            } else {
                None
            };
            if let Some(color) = candidate {
                if color == *chosen.borrow() {
                    return;
                }
                let (hue, saturation, value) =
                    crate::ux_presentation::rgb_to_hsv(color.red, color.green, color.blue);
                hue_state.set(hue);
                sat_state.set(saturation);
                val_state.set(value);
                alpha_state.set(color.alpha);
                set_chosen(color);
            }
        });
    }
    {
        let stack = stack.clone();
        open_custom.connect_clicked(move |_| {
            stack.set_visible_child_name("custom");
        });
    }
    {
        let dialog = dialog.clone();
        cancel.connect_clicked(move |_| {
            dialog.close();
        });
    }
    {
        let dialog = dialog.clone();
        let chosen = Rc::clone(&chosen);
        let token = token.to_string();
        select.connect_clicked(move |_| {
            let color = *chosen.borrow();
            let chosen_token = rgba_token(color);
            // Remember non-palette picks as this session's custom swatches.
            if !palette_tokens.contains(&chosen_token) {
                CUSTOM_PICKER_COLORS.with(|store| {
                    let mut store = store.borrow_mut();
                    store.retain(|existing| existing != &chosen_token);
                    store.insert(0, chosen_token.clone());
                    store.truncate(8);
                });
            }
            // Preserve the original token's format family; the caller
            // routes the result through the reversible preview path.
            let rendered = crate::ux_presentation::render_color_like(&token, color);
            on_apply(rendered);
            dialog.close();
        });
    }

    // The dialog sizes to whichever view is visible (the stack is not
    // homogeneous); no scroll container, so switching to the taller custom
    // view grows the dialog instead of clipping the alpha slider.
    dialog.set_child(Some(&content));
    dialog.present(Some(parent));
}

/// Raw-only editor for color rows whose current value is not in a
/// recognized form: a text button opening entry + Apply/Cancel. Fail
/// closed — Apply stays insensitive until the text parses.
fn attach_raw_color_entry(
    end_box: &gtk::Box,
    setting: &crate::ui::model::UiSetting,
    initial_value: &str,
    control_name: &str,
    throttle_ms: Option<u64>,
) {
    let inline_page_id =
        crate::ux_presentation::page_for_row(&setting.tab_id, &setting.official_setting)
            .map(|page| page.id);
    let button = gtk::Button::new();
    button.set_widget_name(control_name);
    button.set_valign(gtk::Align::Center);
    button.add_css_class("hyprland-settings-swatch-button");
    // Checkered empty swatch: the value is not in a recognized color form
    // yet, but the affordance still looks like a color control.
    button.set_child(Some(&color_swatch_area("", 44, 26)));
    button.update_property(&[gtk::accessible::Property::Label("Edit color")]);
    let picker = gtk::Popover::new();
    picker.set_parent(&button);
    let picker_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    picker_box.set_margin_top(10);
    picker_box.set_margin_bottom(10);
    picker_box.set_margin_start(10);
    picker_box.set_margin_end(10);
    picker_box.append(&small_label(
        "rgba(RRGGBBAA), rgb(RRGGBB), 0xAARRGGBB, or a gradient of those",
    ));
    let preview_state = Rc::new(RefCell::new(initial_value.trim().to_string()));
    let preview = live_swatch_area(preview_state.clone(), 180, 22);
    picker_box.append(&preview);
    let entry = gtk::Entry::new();
    entry.set_text(initial_value.trim());
    entry.set_width_chars(24);
    picker_box.append(&entry);
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    buttons.set_halign(gtk::Align::End);
    let cancel = gtk::Button::with_label("Cancel");
    let apply_button = gtk::Button::with_label("Apply preview");
    apply_button.add_css_class("suggested-action");
    apply_button.set_sensitive(false);
    buttons.append(&cancel);
    buttons.append(&apply_button);
    picker_box.append(&buttons);
    picker.set_child(Some(&picker_box));
    {
        let preview_state = preview_state.clone();
        let preview = preview.clone();
        let apply_button = apply_button.clone();
        entry.connect_changed(move |entry| {
            let text = entry.text().to_string();
            let valid = crate::ux_presentation::parse_hyprland_color(&text).is_some()
                || crate::ux_presentation::parse_hyprland_gradient(&text).is_some();
            apply_button.set_sensitive(valid);
            *preview_state.borrow_mut() = text;
            preview.queue_draw();
        });
    }
    let apply = inline_preview_apply(
        setting.row_id.clone(),
        setting.official_setting.clone(),
        inline_page_id,
        throttle_ms,
        button.clone().upcast(),
    );
    {
        let entry = entry.clone();
        let picker = picker.clone();
        apply_button.connect_clicked(move |_| {
            let text = entry.text().to_string();
            if crate::ux_presentation::parse_hyprland_color(&text).is_some()
                || crate::ux_presentation::parse_hyprland_gradient(&text).is_some()
            {
                apply(text);
            }
            picker.popdown();
        });
    }
    {
        let picker = picker.clone();
        cancel.connect_clicked(move |_| picker.popdown());
    }
    {
        let picker = picker.clone();
        button.connect_clicked(move |_| picker.popup());
    }
    end_box.append(&button);
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
        format!("Official key: {}", setting.official_setting),
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
    frame.update_property(&[gtk::accessible::Property::Label("Setting detail pane")]);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_widget_name("hyprland-settings-detail-pane-content");
    content.update_property(&[gtk::accessible::Property::Label(
        "Setting detail pane content",
    )]);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    render_empty_detail(&content);
    frame.set_child(Some(&content));

    let scroll = gtk::ScrolledWindow::builder()
        .min_content_width(460)
        .min_content_height(320)
        .max_content_height(560)
        .propagate_natural_height(true)
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
    // Session-drop recovery: a preview left active on a previously rendered
    // detail pane is reverted before its controller is dropped. Unconfirmed
    // supervised previews revert too; explicitly Kept ones stay.
    revert_all_active_previews();
    revert_all_unconfirmed_dead_man_previews();

    let Some(detail) = model.detail_for_row(row_id) else {
        render_empty_detail(detail_content);
        return;
    };

    detail_content.set_widget_name(&format!(
        "hyprland-settings-detail-pane-{}",
        safe_widget_name(row_id)
    ));
    detail_content.update_property(&[gtk::accessible::Property::Label(
        &detail_pane_accessibility_text(&detail),
    )]);
    clear_box(detail_content);
    append_detail_section(detail_content, "Setting", |section| {
        let heading = match crate::presentation_labels::display_label_for_row(&detail.row_id) {
            Some(matched) => matched.to_string(),
            None => {
                crate::ux_presentation::fallback_display_label(&detail.label, &detail.tab_label)
            }
        };
        section.append(&title_label(&heading));
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

    append_detail_section(detail_content, "Live preview", |section| {
        append_runtime_preview_controls(model, &detail, section);
    });

    append_detail_section(detail_content, "Edit", |section| {
        append_user_facing_write_reason(&detail, section);
        append_source_include_insertion_target_review(model, &detail, section);
        append_runtime_approval_review_surface(&detail, section);
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

fn append_runtime_approval_review_surface(detail: &RowDetailProjection, section: &gtk::Box) {
    if detail.row_id != "appearance.gaps_in" && detail.official_setting != "general.gaps_in" {
        return;
    }

    let evidence = proven_runtime_approval_evidence_summary();
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("hyprland-settings-runtime-approval-review-disabled");
    frame.set_tooltip_text(Some(
        "Disabled runtime approval review. This displays live-restore proof but does not enable runtime Apply.",
    ));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_widget_name("hyprland-settings-runtime-live-restore-evidence");
    content.set_tooltip_text(Some(
        "Runtime live-restore evidence for general:gaps_in. Review-only; no hyprctl command runs.",
    ));
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&body_label("Runtime approval review"));
    content.append(&small_label("Runtime changes are not enabled yet."));
    content.append(&small_label("This setting has a proven live-restore test."));
    content.append(&small_label("Production runtime/reload remains disabled."));
    append_detail_line(&content, "Setting", &evidence.setting);
    append_detail_line(&content, "Prior value", &evidence.prior_value);
    append_detail_line(&content, "Temporary test value", &evidence.temporary_value);
    append_detail_line(&content, "Mutation command", &evidence.mutation_command);
    append_detail_line(&content, "Restore command", &evidence.restore_command);
    append_detail_line(
        &content,
        "Post-mutation readback",
        &evidence.post_mutation_readback,
    );
    append_detail_line(
        &content,
        "Post-restore readback",
        &evidence.post_restore_readback,
    );
    append_detail_line(&content, "Approval status", &evidence.approval_status);
    append_detail_line(
        &content,
        "Production runtime/reload",
        &evidence.production_runtime_status,
    );
    content.append(&small_label(
        "This review surface is proof display only and does not call hyprctl.",
    ));

    let enable = gtk::Button::with_label("Enable runtime apply (planned)");
    enable.set_widget_name("hyprland-settings-runtime-approval-enable-disabled");
    enable.set_tooltip_text(Some(
        "Disabled future action. This does not enable runtime/reload Apply or run hyprctl.",
    ));
    enable.set_sensitive(false);
    content.append(&enable);

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

/// Supervised dead-man preview panel for dead-man-gated rows. Only rows
/// classified DeadManPreviewCandidate get an armed "Preview with recovery"
/// button; needs-live-proof and model-only rows render the panel with the arm
/// control disabled and the reason shown; blocked rows show only the reason.
/// Every action routes through RuntimePreviewDeadManController — UI code
/// builds no commands and cannot bypass the countdown.
fn append_dead_man_preview_panel(detail: &RowDetailProjection, section: &gtk::Box) {
    let Some(ui_state) = dead_man_ui_state(&detail.row_id) else {
        section.append(&small_label(
            "Dead-man preview state unavailable for this row.",
        ));
        return;
    };

    let dead_man_badge = small_label(&format!("{}: {}", ui_state.badge, ui_state.why_supervised));
    dead_man_badge.set_widget_name(&format!(
        "hyprland-settings-live-preview-dead-man-{}",
        safe_widget_name(&detail.row_id)
    ));
    section.append(&dead_man_badge);

    if !ui_state.shows_panel {
        let blocked = small_label(&format!(
            "Supervised preview blocked ({}): {}",
            ui_state.classification.as_str(),
            ui_state.disabled_reason.unwrap_or("blocked"),
        ));
        blocked.set_widget_name(&format!(
            "hyprland-settings-dead-man-blocked-{}",
            safe_widget_name(&detail.row_id)
        ));
        section.append(&blocked);
        return;
    }

    section.append(&small_label(&ui_state.warning_text));
    let recovery = small_label(ui_state.recovery_instruction);
    recovery.set_widget_name(&format!(
        "hyprland-settings-dead-man-recovery-{}",
        safe_widget_name(&detail.row_id)
    ));
    section.append(&recovery);

    let status = small_label(&format!(
        "Supervised preview status: disarmed ({})",
        ui_state.classification.as_str()
    ));
    status.set_widget_name(&format!(
        "hyprland-settings-dead-man-status-{}",
        safe_widget_name(&detail.row_id)
    ));

    let value_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    value_row.append(&body_label("Preview value"));
    let value_entry = gtk::Entry::new();
    value_entry.set_text(
        detail
            .current_value
            .raw_value
            .as_deref()
            .or(detail.edit.proposed_value.as_deref())
            .unwrap_or_default(),
    );
    value_entry.set_hexpand(true);
    value_entry.set_sensitive(ui_state.arm_enabled);
    value_row.append(&value_entry);
    section.append(&value_row);

    let arm_button = gtk::Button::with_label("Preview with recovery");
    arm_button.set_widget_name(&format!(
        "hyprland-settings-dead-man-arm-{}",
        safe_widget_name(&detail.row_id)
    ));
    arm_button.set_sensitive(ui_state.arm_enabled);

    let keep_button = gtk::Button::with_label("Keep changes");
    keep_button.set_widget_name(&format!(
        "hyprland-settings-dead-man-keep-{}",
        safe_widget_name(&detail.row_id)
    ));
    keep_button.set_sensitive(false);
    let revert_button = gtk::Button::with_label("Revert now");
    revert_button.set_widget_name(&format!(
        "hyprland-settings-dead-man-revert-{}",
        safe_widget_name(&detail.row_id)
    ));
    revert_button.set_sensitive(false);
    let cancel_button = gtk::Button::with_label("Cancel");
    cancel_button.set_widget_name(&format!(
        "hyprland-settings-dead-man-cancel-{}",
        safe_widget_name(&detail.row_id)
    ));
    cancel_button.set_sensitive(false);

    if !ui_state.arm_enabled {
        if let Some(reason) = ui_state.disabled_reason {
            let disabled = small_label(&format!(
                "Supervised preview disabled ({}): {reason}",
                ui_state.classification.as_str()
            ));
            disabled.set_widget_name(&format!(
                "hyprland-settings-dead-man-disabled-{}",
                safe_widget_name(&detail.row_id)
            ));
            section.append(&disabled);
        }
        // Per-row proof-aware status for input/cursor rows: what subsystem
        // is at risk, what fallback must hold, and how the row could be
        // proven. Display only; nothing here can trigger a mutation.
        if let Some(plan) = crate::runtime_preview_input_proof::input_proof_plan(&detail.row_id) {
            let proof_status = small_label(&format!(
                "Needs per-row live proof ({} risk). Fallback requirement: {}. Proof status: {}.",
                plan.category.as_str(),
                plan.fallback.summary,
                plan.proof_classification.as_str(),
            ));
            proof_status.set_widget_name(&format!(
                "hyprland-settings-input-proof-status-{}",
                safe_widget_name(&detail.row_id)
            ));
            section.append(&proof_status);
            let proof_how = small_label(&if plan.live_proof_exists {
                format!(
                    "An env-gated live proof exists for this row: {}",
                    plan.live_proof_env
                )
            } else {
                format!(
                    "To arm this row, a supervised live proof must verify apply and revert while a fallback input path stays usable. {}",
                    plan.what_could_go_wrong
                )
            });
            proof_how.set_widget_name(&format!(
                "hyprland-settings-input-proof-how-{}",
                safe_widget_name(&detail.row_id)
            ));
            section.append(&proof_how);
        }
    } else if let Some(plan) = crate::runtime_preview_input_proof::input_proof_plan(&detail.row_id)
    {
        if plan.proof_classification.armable() {
            let proven = small_label(
                "This row passed its per-row live proof: apply and revert were verified against the running compositor with all input paths usable.",
            );
            proven.set_widget_name(&format!(
                "hyprland-settings-input-proof-passed-{}",
                safe_widget_name(&detail.row_id)
            ));
            section.append(&proven);
        }
    }

    let controller = match RuntimePreviewDeadManController::new_live(&detail.row_id) {
        Ok(controller) => Rc::new(RefCell::new(controller)),
        Err(error) => {
            section.append(&small_label(&error.user_text()));
            return;
        }
    };
    register_dead_man_controller(&controller);

    {
        let controller = controller.clone();
        let status = status.clone();
        let value_entry = value_entry.clone();
        let keep_button = keep_button.clone();
        let revert_button = revert_button.clone();
        let cancel_button = cancel_button.clone();
        arm_button.connect_clicked(move |arm_button| {
            let armed = { controller.borrow_mut().arm() };
            match armed {
                Ok(receipt) => {
                    status.set_label(&receipt.status_text);
                    revert_button.set_sensitive(true);
                    cancel_button.set_sensitive(true);
                    arm_button.set_sensitive(false);
                    let applied = {
                        let value = value_entry.text().to_string();
                        controller.borrow_mut().apply(&value)
                    };
                    match applied {
                        Ok(receipt) => {
                            status.set_label(&receipt.status_text);
                            keep_button.set_sensitive(true);
                            // Countdown driver: ticks once per second and
                            // auto-reverts on timeout.
                            let controller = controller.clone();
                            let status = status.clone();
                            let keep_button = keep_button.clone();
                            let revert_button = revert_button.clone();
                            let cancel_button = cancel_button.clone();
                            let arm_button = arm_button.clone();
                            gtk::glib::timeout_add_local(
                                std::time::Duration::from_millis(1000),
                                move || {
                                    let outcome = {
                                        match controller.try_borrow_mut() {
                                            Ok(mut controller) => controller.tick(1000),
                                            Err(_) => return gtk::glib::ControlFlow::Continue,
                                        }
                                    };
                                    match outcome {
                                        Ok(Some(receipt)) => {
                                            status.set_label(&receipt.status_text);
                                            keep_button.set_sensitive(false);
                                            revert_button.set_sensitive(false);
                                            cancel_button.set_sensitive(false);
                                            arm_button.set_sensitive(true);
                                            gtk::glib::ControlFlow::Break
                                        }
                                        Ok(None) => {
                                            let phase = controller.borrow().phase();
                                            if phase
                                                == RuntimePreviewDeadManUiPhase::CountingDown
                                            {
                                                status.set_label(&format!(
                                                    "Previewing with recovery: auto-revert in {} seconds unless you Keep changes.",
                                                    controller.borrow().remaining_seconds()
                                                ));
                                                gtk::glib::ControlFlow::Continue
                                            } else {
                                                gtk::glib::ControlFlow::Break
                                            }
                                        }
                                        Err(error) => {
                                            status.set_label(&error.user_text());
                                            gtk::glib::ControlFlow::Break
                                        }
                                    }
                                },
                            );
                        }
                        Err(error) => status.set_label(&error.user_text()),
                    }
                }
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }
    {
        let controller = controller.clone();
        let status = status.clone();
        let revert_button = revert_button.clone();
        let cancel_button = cancel_button.clone();
        let arm_button = arm_button.clone();
        keep_button.connect_clicked(move |keep_button| {
            match controller.borrow_mut().confirm_keep() {
                Ok(receipt) => {
                    status.set_label(&receipt.status_text);
                    keep_button.set_sensitive(false);
                    revert_button.set_sensitive(true);
                    cancel_button.set_sensitive(false);
                    arm_button.set_sensitive(true);
                }
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }
    {
        let controller = controller.clone();
        let status = status.clone();
        let keep_button = keep_button.clone();
        let cancel_button = cancel_button.clone();
        let arm_button = arm_button.clone();
        revert_button.connect_clicked(move |revert_button| {
            match controller.borrow_mut().revert_now() {
                Ok(receipt) => {
                    status.set_label(&receipt.status_text);
                    keep_button.set_sensitive(false);
                    revert_button.set_sensitive(false);
                    cancel_button.set_sensitive(false);
                    arm_button.set_sensitive(true);
                }
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }
    {
        let controller = controller.clone();
        let status = status.clone();
        let keep_button = keep_button.clone();
        let revert_button = revert_button.clone();
        let arm_button = arm_button.clone();
        cancel_button.connect_clicked(move |cancel_button| {
            match controller.borrow_mut().cancel() {
                Ok(receipt) => {
                    status.set_label(&receipt.status_text);
                    keep_button.set_sensitive(false);
                    revert_button.set_sensitive(false);
                    cancel_button.set_sensitive(false);
                    arm_button.set_sensitive(true);
                }
                Err(error) => status.set_label(&error.user_text()),
            }
        });
    }

    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    buttons.append(&arm_button);
    buttons.append(&keep_button);
    buttons.append(&revert_button);
    buttons.append(&cancel_button);
    section.append(&buttons);
    section.append(&status);
}

/// Per-setting live runtime preview controls. Enabled only for rows the
/// capability matrix classifies as default-previewable; every action routes
/// through the RuntimePreviewUiController — no hyprctl strings, no config
/// writes, and no reload can originate here. Save persists once through the
/// app's existing safe scalar write flow.
fn append_runtime_preview_controls(
    model: &UiProjection,
    detail: &RowDetailProjection,
    section: &gtk::Box,
) {
    let Some(row_state) = runtime_preview_ui_row_state(&detail.row_id) else {
        section.append(&small_label(
            "Live preview does not apply to this row (not a scalar setting row).",
        ));
        return;
    };

    let badge = body_label(&row_state.capability_badge);
    badge.set_widget_name(&format!(
        "hyprland-settings-live-preview-badge-{}",
        safe_widget_name(&detail.row_id)
    ));
    section.append(&badge);

    if !row_state.preview_enabled {
        // Only rows the matrix classifies as dead-man-supervisable get the
        // supervised panel; blocked rows show their reason only.
        if row_state.capability
            == crate::runtime_preview::RuntimePreviewCapability::LivePreviewSupportedWithDeadMan
        {
            append_dead_man_preview_panel(detail, section);
            return;
        }
        if let Some(reason) = &row_state.unavailable_reason {
            section.append(&small_label(&format!("Why: {reason}")));
        }
        return;
    }

    let controller = match pending_controller_for_row(&detail.row_id) {
        // Reuse the row's existing preview session (see the inline path).
        Some(existing) => existing,
        None => match RuntimePreviewUiController::new_live(&detail.row_id) {
            Ok(controller) => Rc::new(RefCell::new(controller)),
            Err(error) => {
                section.append(&small_label(&error.user_text()));
                return;
            }
        },
    };
    register_preview_controller(&controller);
    {
        // The detail-pane preview participates in the pending-changes
        // surfaces exactly like inline controls do.
        let page_id = model
            .settings
            .iter()
            .find(|setting| setting.row_id == detail.row_id)
            .and_then(|setting| {
                crate::ux_presentation::page_for_row(&setting.tab_id, &setting.official_setting)
            })
            .map(|page| page.id);
        register_pending_controller(
            &detail.row_id,
            &detail.official_setting,
            page_id,
            &controller,
        );
    }

    section.append(&small_label(
        "Preview applies the value to the running Hyprland session only. Nothing is written to your config until you choose Save.",
    ));

    let status = small_label(&row_state.status_text);
    status.set_widget_name(&format!(
        "hyprland-settings-live-preview-status-{}",
        safe_widget_name(&detail.row_id)
    ));

    // Runtime truth first, like the inline controls: the displayed value
    // must match the session original the executor will capture, or the
    // first change can be a runtime no-op that never registers as pending.
    let initial_value =
        crate::runtime_preview_ui_projection::read_runtime_option_live(&detail.official_setting)
            .or_else(|| detail.current_value.raw_value.clone())
            .or_else(|| {
                crate::official_defaults::official_default_value(&detail.official_setting)
                    .map(str::to_string)
            })
            .or_else(|| detail.edit.proposed_value.clone())
            .map(|value| collapse_uniform_gap(&value))
            .unwrap_or_default();

    let apply_from_control = {
        let controller = controller.clone();
        let status = status.clone();
        let drain_scheduled = Rc::new(std::cell::Cell::new(false));
        move |value: String| {
            let outcome = controller
                .borrow_mut()
                .offer_value(&value, preview_now_ms());
            match outcome {
                Ok(Some(receipt)) => {
                    status.set_label(&receipt.status_text);
                    notify_pending_changed();
                }
                Ok(None) => {
                    // Value is pending in the throttle; schedule one trailing
                    // drain if none is queued yet.
                    if !drain_scheduled.get() {
                        drain_scheduled.set(true);
                        let controller = controller.clone();
                        let status = status.clone();
                        let drain_scheduled = drain_scheduled.clone();
                        let delay = row_state.throttle_ms.unwrap_or(150) + 10;
                        gtk::glib::timeout_add_local_once(
                            std::time::Duration::from_millis(delay),
                            move || {
                                drain_scheduled.set(false);
                                match controller.borrow_mut().drain_pending(preview_now_ms()) {
                                    Ok(Some(receipt)) => status.set_label(&receipt.status_text),
                                    Ok(None) => {}
                                    Err(error) => status.set_label(&error.user_text()),
                                }
                                notify_pending_changed();
                            },
                        );
                    }
                }
                Err(error) => status.set_label(&error.user_text()),
            }
        }
    };

    match row_state.control_kind {
        RuntimePreviewUiControlKind::Switch => {
            let control_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            control_row.append(&body_label("Preview value"));
            let switch = gtk::Switch::new();
            switch.set_widget_name(&format!(
                "hyprland-settings-live-preview-switch-{}",
                safe_widget_name(&detail.row_id)
            ));
            switch.set_active(matches!(initial_value.trim(), "true" | "1" | "yes" | "on"));
            switch.set_valign(gtk::Align::Center);
            let apply = apply_from_control.clone();
            switch.connect_active_notify(move |switch| {
                apply(if switch.is_active() { "true" } else { "false" }.to_string());
            });
            control_row.append(&switch);
            section.append(&control_row);
        }
        RuntimePreviewUiControlKind::Slider => {
            let (min, max) = row_state.slider_bounds.unwrap_or((0.0, 1.0));
            let step = ((max - min) / 100.0).max(0.001);
            let slider = gtk::Scale::with_range(gtk::Orientation::Horizontal, min, max, step);
            slider.set_widget_name(&format!(
                "hyprland-settings-live-preview-slider-{}",
                safe_widget_name(&detail.row_id)
            ));
            slider.set_hexpand(true);
            slider.set_draw_value(true);
            if let Ok(value) = initial_value.trim().parse::<f64>() {
                slider.set_value(value.clamp(min, max));
            }
            let integer_like = step >= 1.0;
            let apply = apply_from_control.clone();
            slider.connect_value_changed(move |slider| {
                let value = if integer_like {
                    format!("{}", slider.value().round() as i64)
                } else {
                    format!("{:.3}", slider.value())
                };
                apply(value);
            });
            section.append(&slider);
        }
        RuntimePreviewUiControlKind::SpinRow => {
            let spin = gtk::SpinButton::with_range(-100_000.0, 100_000.0, 1.0);
            spin.set_widget_name(&format!(
                "hyprland-settings-live-preview-spin-{}",
                safe_widget_name(&detail.row_id)
            ));
            if let Ok(value) = initial_value.trim().parse::<f64>() {
                spin.set_value(value);
            }
            let apply = apply_from_control.clone();
            spin.connect_value_changed(move |spin| {
                let value = spin.value();
                let rendered = if (value - value.round()).abs() < f64::EPSILON {
                    format!("{}", value.round() as i64)
                } else {
                    format!("{value}")
                };
                apply(rendered);
            });
            section.append(&spin);
        }
        RuntimePreviewUiControlKind::Dropdown => {
            let combo = gtk::ComboBoxText::new();
            combo.set_widget_name(&format!(
                "hyprland-settings-live-preview-dropdown-{}",
                safe_widget_name(&detail.row_id)
            ));
            for (raw_value, label) in &row_state.dropdown_choices {
                combo.append(Some(raw_value), label);
            }
            combo.set_active_id(Some(initial_value.trim()));
            let apply = apply_from_control.clone();
            combo.connect_changed(move |combo| {
                if let Some(active) = combo.active_id() {
                    apply(active.to_string());
                }
            });
            section.append(&combo);
        }
        RuntimePreviewUiControlKind::ColorEntry | RuntimePreviewUiControlKind::ValueEntry => {
            let control_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            control_row.append(&body_label(
                if row_state.control_kind == RuntimePreviewUiControlKind::ColorEntry {
                    "Preview color"
                } else {
                    "Preview value"
                },
            ));
            let entry = gtk::Entry::new();
            entry.set_widget_name(&format!(
                "hyprland-settings-live-preview-entry-{}",
                safe_widget_name(&detail.row_id)
            ));
            entry.set_text(&initial_value);
            entry.set_hexpand(true);
            let apply = apply_from_control.clone();
            entry.connect_activate(move |entry| {
                apply(entry.text().to_string());
            });
            control_row.append(&entry);
            section.append(&control_row);
        }
        RuntimePreviewUiControlKind::NoControl => {}
    }

    section.append(&status);

    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    let save_button = gtk::Button::with_label("Save previewed value");
    save_button.set_widget_name(&format!(
        "hyprland-settings-live-preview-save-{}",
        safe_widget_name(&detail.row_id)
    ));
    let can_persist = row_state.save_state.available()
        && detail.edit.editable
        && detail
            .edit
            .pending
            .as_ref()
            .map(|pending| pending.can_review)
            .unwrap_or(false);
    save_button.set_sensitive(can_persist);
    if !can_persist {
        buttons.append(&save_button);
        let reason = small_label(&format!("Save disabled: {}", row_state.save_state.reason()));
        reason.set_widget_name(&format!(
            "hyprland-settings-live-preview-save-reason-{}",
            safe_widget_name(&detail.row_id)
        ));
        section.append(&reason);
    } else {
        buttons.append(&save_button);
    }
    {
        let controller = controller.clone();
        let status = status.clone();
        let known_setting_ids = model.known_setting_ids.clone();
        let config_discovery = model.config_discovery.clone();
        let current_config = model.current_config.clone();
        let setting_id = detail.row_id.clone();
        save_button.connect_clicked(move |button| {
            let last_value = controller.borrow().last_applied_value();
            let Some(value) = last_value else {
                status.set_label("Nothing to save: no preview value has been applied yet.");
                return;
            };
            match controller.borrow_mut().mark_saved() {
                Ok(_) => {}
                Err(error) => {
                    status.set_label(&error.user_text());
                    return;
                }
            }
            match crate::production_save::gated_scalar_save_live(
                known_setting_ids.clone(),
                &config_discovery,
                &current_config,
                &setting_id,
                &value,
            ) {
                Ok(outcome) => {
                    button.set_sensitive(false);
                    status.set_label(&format!(
                        "Saved: {} = {} persisted once with backup {} (Safe Live Save Mode verified; no reload).",
                        outcome.setting_id,
                        outcome
                            .verified_value
                            .unwrap_or_else(|| "unknown".to_string()),
                        outcome.backup_path.display(),
                    ));
                }
                Err(reason) => status.set_label(&reason),
            }
            notify_pending_changed();
        });
    }

    let revert_button = gtk::Button::with_label("Revert preview");
    revert_button.set_widget_name(&format!(
        "hyprland-settings-live-preview-revert-{}",
        safe_widget_name(&detail.row_id)
    ));
    {
        let controller = controller.clone();
        let status = status.clone();
        revert_button.connect_clicked(move |_| {
            match controller.borrow_mut().revert() {
                Ok(receipt) => status.set_label(&receipt.status_text),
                Err(error) => status.set_label(&error.user_text()),
            }
            notify_pending_changed();
        });
    }
    buttons.append(&revert_button);

    let cancel_button = gtk::Button::with_label("Cancel preview");
    cancel_button.set_widget_name(&format!(
        "hyprland-settings-live-preview-cancel-{}",
        safe_widget_name(&detail.row_id)
    ));
    {
        let controller = controller.clone();
        let status = status.clone();
        cancel_button.connect_clicked(move |_| {
            match controller.borrow_mut().cancel() {
                Ok(receipt) => status.set_label(&receipt.status_text),
                Err(error) => status.set_label(&error.user_text()),
            }
            notify_pending_changed();
        });
    }
    buttons.append(&cancel_button);

    section.append(&buttons);
    let _ = RuntimePreviewUiSessionState::Idle;
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
        match crate::production_save::gated_scalar_save_live(
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
            Err(reason) => {
                result_label.set_label(&format!("Apply blocked: {reason}"));
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
    {}
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
