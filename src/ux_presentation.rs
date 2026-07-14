//! UX presentation layer: the grouped sidebar category model and the quiet
//! per-row status chips used by the settings-first UI pass.
//!
//! This module is presentational only. It layers short, honest status
//! vocabulary and task-oriented grouping over the existing classification
//! systems — it never reclassifies a row, never touches a save path, and
//! never weakens a gate. Full reasons stay available in the detail pane
//! and on the Safety Details page.

use serde::Serialize;

use crate::runtime_preview::{runtime_preview_row_capability, RuntimePreviewCapability};
use crate::runtime_preview_dead_man::{classify_dead_man_row, RuntimePreviewDeadManClassification};

/// One sidebar page. A page either shows rows from a model tab (optionally
/// filtered to a set of official-setting prefixes, with exactly one
/// "rest" page per tab catching everything unclaimed so no row is ever
/// lost) or is a standalone view. Presentation only: rows keep their tab,
/// ids, classifications, and save paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PageSpec {
    pub id: &'static str,
    pub label: &'static str,
    /// Model tab the rows come from; None = standalone view.
    pub source_tab: Option<&'static str>,
    /// Official-setting prefixes this page claims; None with a source tab
    /// means "every row of the tab not claimed by a sibling page".
    pub include_prefixes: Option<&'static [&'static str]>,
    /// Cross-tab claims: (model tab, official prefixes) pulled onto this
    /// page from other tabs (e.g. the layout/snap keys shown on General).
    pub extra_sources: &'static [(&'static str, &'static [&'static str])],
}

/// The sidebar page layout: category headers with their pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SidebarCategoryPages {
    pub label: &'static str,
    pub pages: &'static [PageSpec],
}

pub const SIDEBAR_PAGE_LAYOUT: &[SidebarCategoryPages] = &[
    SidebarCategoryPages {
        label: "Look & Feel",
        pages: &[
            PageSpec {
                id: "general",
                label: "General",
                source_tab: Some("appearance"),
                include_prefixes: None,
                extra_sources: &[(
                    "windows-layout",
                    &["general.layout", "general.allow_tearing", "general.snap."],
                )],
            },
            PageSpec {
                id: "decoration",
                label: "Decoration",
                source_tab: Some("appearance"),
                include_prefixes: Some(&["decoration."]),
                extra_sources: &[],
            },
            PageSpec {
                id: "animations",
                label: "Animations",
                source_tab: Some("animations"),
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "cursor",
                label: "Cursor",
                source_tab: Some("cursor"),
                include_prefixes: None,
                extra_sources: &[],
            },
        ],
    },
    SidebarCategoryPages {
        label: "Input",
        pages: &[
            PageSpec {
                id: "keybinds",
                label: "Keybinds",
                source_tab: Some("keybinds"),
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "devices",
                label: "Devices",
                source_tab: Some("input"),
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "gestures",
                label: "Gestures",
                source_tab: Some("input"),
                include_prefixes: Some(&["gestures.", "gesture."]),
                extra_sources: &[],
            },
        ],
    },
    SidebarCategoryPages {
        label: "Display",
        pages: &[
            PageSpec {
                id: "monitors",
                label: "Monitors",
                source_tab: Some("display"),
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "workspaces",
                label: "Workspaces",
                source_tab: None,
                include_prefixes: None,
                extra_sources: &[],
            },
        ],
    },
    SidebarCategoryPages {
        label: "Window Management",
        pages: &[
            PageSpec {
                id: "layouts",
                label: "Layouts",
                source_tab: None,
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "window-rules",
                label: "Window Rules",
                source_tab: None,
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "layer-rules",
                label: "Layer Rules",
                source_tab: None,
                include_prefixes: None,
                extra_sources: &[],
            },
        ],
    },
    SidebarCategoryPages {
        label: "Startup",
        pages: &[
            PageSpec {
                id: "autostart",
                label: "Autostart",
                source_tab: None,
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "env-variables",
                label: "Env Variables",
                source_tab: None,
                include_prefixes: None,
                extra_sources: &[],
            },
        ],
    },
    SidebarCategoryPages {
        label: "Advanced",
        pages: &[
            // The xwayland.* rows live in the model's "display" tab and the
            // ecosystem.* rows in "permissions" — claiming them from the
            // wrong tab silently hid both pages (zero claimed rows) while
            // the rows fell to the Monitors/Permissions rest pages.
            PageSpec {
                id: "xwayland",
                label: "XWayland",
                source_tab: Some("display"),
                include_prefixes: Some(&["xwayland."]),
                extra_sources: &[],
            },
            PageSpec {
                id: "ecosystem",
                label: "Ecosystem",
                source_tab: Some("permissions"),
                include_prefixes: Some(&["ecosystem."]),
                extra_sources: &[],
            },
            PageSpec {
                id: "system",
                label: "System",
                source_tab: Some("system"),
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "permissions",
                label: "Permissions",
                source_tab: Some("permissions"),
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "windows-layout",
                label: "Windows & Layout",
                source_tab: Some("windows-layout"),
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "profiles",
                label: "Profiles",
                source_tab: None,
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "config",
                label: "Settings",
                source_tab: None,
                include_prefixes: None,
                extra_sources: &[],
            },
            PageSpec {
                id: "safety-details",
                label: "Safety Details",
                source_tab: None,
                include_prefixes: None,
                extra_sources: &[],
            },
        ],
    },
];

/// Look up a page spec by id.
pub fn page_spec(page_id: &str) -> Option<&'static PageSpec> {
    SIDEBAR_PAGE_LAYOUT
        .iter()
        .flat_map(|category| category.pages.iter())
        .find(|page| page.id == page_id)
}

/// Whether a row (from `row_tab`) belongs on a page: prefix pages claim
/// their prefixes, cross-tab extra sources claim theirs, and the tab's
/// rest page takes everything no other page claimed.
pub fn page_claims_row_in_tab(page: &PageSpec, row_tab: &str, official_setting: &str) -> bool {
    // Cross-tab claims win first.
    if page.extra_sources.iter().any(|(tab, prefixes)| {
        *tab == row_tab
            && prefixes
                .iter()
                .any(|prefix| official_setting.starts_with(prefix))
    }) {
        return true;
    }
    if page.source_tab != Some(row_tab) {
        return false;
    }
    match page.include_prefixes {
        Some(prefixes) => prefixes
            .iter()
            .any(|prefix| official_setting.starts_with(prefix)),
        None => {
            // Rest page: not claimed by any sibling prefix page of the
            // same tab, nor by any page's cross-tab claim on this tab.
            let claimed_by_sibling = SIDEBAR_PAGE_LAYOUT
                .iter()
                .flat_map(|category| category.pages.iter())
                .filter(|sibling| sibling.source_tab == Some(row_tab))
                .filter_map(|sibling| sibling.include_prefixes)
                .flatten()
                .any(|prefix| official_setting.starts_with(prefix));
            let claimed_cross_tab = SIDEBAR_PAGE_LAYOUT
                .iter()
                .flat_map(|category| category.pages.iter())
                .flat_map(|sibling| sibling.extra_sources.iter())
                .filter(|(tab, _)| *tab == row_tab)
                .any(|(_, prefixes)| {
                    prefixes
                        .iter()
                        .any(|prefix| official_setting.starts_with(prefix))
                });
            !claimed_by_sibling && !claimed_cross_tab
        }
    }
}

/// Every model tab a page draws rows from.
pub fn page_source_tabs(page: &PageSpec) -> Vec<&'static str> {
    let mut tabs = Vec::new();
    if let Some(tab) = page.source_tab {
        tabs.push(tab);
    }
    for (tab, _) in page.extra_sources {
        if !tabs.contains(tab) {
            tabs.push(tab);
        }
    }
    tabs
}

/// Back-compat single-tab form used where the row's tab equals the page's
/// source tab.
pub fn page_claims_row(page: &PageSpec, official_setting: &str) -> bool {
    match page.source_tab {
        Some(tab) => page_claims_row_in_tab(page, tab, official_setting),
        None => false,
    }
}

/// Symbolic icon for a sidebar page (standard icon-theme names).
pub fn page_icon(page_id: &str) -> &'static str {
    match page_id {
        "dashboard" => "go-home-symbolic",
        "general" => "preferences-system-symbolic",
        "decoration" => "applications-graphics-symbolic",
        "animations" => "media-playback-start-symbolic",
        "cursor" => "input-mouse-symbolic",
        "keybinds" => "input-keyboard-symbolic",
        "devices" => "input-touchpad-symbolic",
        "gestures" => "input-tablet-symbolic",
        "monitors" => "video-display-symbolic",
        "workspaces" => "view-grid-symbolic",
        "layouts" => "view-paged-symbolic",
        "window-rules" => "window-new-symbolic",
        "layer-rules" => "focus-windows-symbolic",
        "autostart" => "system-run-symbolic",
        "env-variables" => "utilities-terminal-symbolic",
        "xwayland" => "application-x-executable-symbolic",
        "ecosystem" => "network-workgroup-symbolic",
        "system" => "emblem-system-symbolic",
        "permissions" => "security-high-symbolic",
        "windows-layout" => "window-restore-symbolic",
        "profiles" => "folder-symbolic",
        "config" => "document-properties-symbolic",
        "safety-details" => "dialog-information-symbolic",
        _ => "application-x-executable-symbolic",
    }
}

/// The compact badge for a row, if it needs one. Routine states (live
/// preview, save only, configured/default values) show nothing — that
/// detail lives in the row's detail surface and Safety Details.
pub fn row_badge(chip: StatusChip, needs_attention: bool) -> Option<&'static str> {
    if needs_attention {
        return Some("Needs attention");
    }
    match chip {
        StatusChip::Blocked => Some("Blocked"),
        StatusChip::HardwareRequired => Some("Hardware required"),
        StatusChip::NotProvenYet => Some("Not proven yet"),
        StatusChip::LivePreview | StatusChip::SaveOnly => None,
    }
}

/// Curated per-row section placement for the pages where generated
/// subsections would combine things the reference layout separates.
/// Factual grouping by official key — nothing semantic is guessed; rows
/// without an entry fall back to their generated subsection.
pub fn section_for_row(official_setting: &str, subsection: &str, page_label: &str) -> String {
    const CURATED_PREFIXES: &[(&str, &str)] = &[
        // Animations scalar rows sit in a "General" section above the
        // record groups, matching the reference page order.
        ("animations.", "General"),
        ("general.gaps_", "Gaps"),
        ("general.float_gaps", "Gaps"),
        ("general.col.", "Border Colors"),
        ("general.border_size", "Borders"),
        ("general.no_border_on_floating", "Borders"),
        ("general.resize_on_border", "Borders"),
        ("general.extend_border_grab_area", "Borders"),
        ("general.hover_icon_on_border", "Borders"),
        ("general.resize_corner", "Borders"),
        ("general.snap.", "Snap"),
        ("general.layout", "Layout"),
        ("general.allow_tearing", "Layout"),
        ("general.no_focus_fallback", "Layout"),
        ("decoration.rounding", "Rounding and Opacity"),
        ("decoration.active_opacity", "Rounding and Opacity"),
        ("decoration.inactive_opacity", "Rounding and Opacity"),
        ("decoration.fullscreen_opacity", "Rounding and Opacity"),
        ("decoration.blur.", "Blur"),
        ("decoration.shadow.", "Shadow"),
        ("decoration.dim_", "Dim"),
    ];
    for (prefix, section) in CURATED_PREFIXES {
        if official_setting.starts_with(prefix) {
            return (*section).to_string();
        }
    }
    section_display_name(subsection, page_label)
}

/// Friendly section heading: strip redundant page words from generated
/// subsection names and map the known awkward ones to natural titles.
/// Mechanical plus a small curated table — nothing semantic is guessed.
pub fn section_display_name(subsection: &str, page_label: &str) -> String {
    const CURATED: &[(&str, &str)] = &[
        ("Decoration Blur", "Blur"),
        ("Decoration Shadow", "Shadow"),
        ("Decoration", "Rounding and Opacity"),
        ("General Col", "Border Colors"),
        ("General Snap", "Snap"),
        ("Group Groupbar", "Group Bar"),
        ("Input Touchpad", "Touchpad"),
        ("Input Touchdevice", "Touchscreen"),
        ("Input Tablet", "Tablet"),
        ("Gestures", "Workspace Swipe"),
        ("Misc", "Miscellaneous"),
    ];
    let trimmed = subsection.trim();
    if let Some((_, replacement)) = CURATED
        .iter()
        .find(|(pattern, _)| trimmed.eq_ignore_ascii_case(pattern))
    {
        return replacement.to_string();
    }
    let stripped = fallback_display_label(trimmed, page_label);
    if stripped.eq_ignore_ascii_case(page_label) || stripped.is_empty() {
        "General".to_string()
    } else {
        stripped
    }
}

/// The category header shown above a page's sidebar row, if grouped.
pub fn category_for_tab(page_id: &str) -> Option<&'static str> {
    SIDEBAR_PAGE_LAYOUT
        .iter()
        .find(|category| category.pages.iter().any(|page| page.id == page_id))
        .map(|category| category.label)
}

/// Quiet, honest per-row status. One short chip instead of a paragraph;
/// the detailed reason is one tap away and unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StatusChip {
    /// Previews instantly with revert/cancel (directly or under the
    /// supervised countdown).
    LivePreview,
    /// Persists through the gated Save; no live preview.
    SaveOnly,
    /// Proof needs hardware that is not present.
    HardwareRequired,
    /// No passed proof receipt yet; control stays disabled.
    NotProvenYet,
    /// High risk or no safe mechanism; editing stays blocked.
    Blocked,
}

impl StatusChip {
    pub fn label(self) -> &'static str {
        match self {
            Self::LivePreview => "Live Preview",
            Self::SaveOnly => "Save Only",
            Self::HardwareRequired => "Hardware Required",
            Self::NotProvenYet => "Not Proven Yet",
            Self::Blocked => "Blocked",
        }
    }
}

/// The save-gate chip: every production Save requires Safe Live Save
/// Mode, surfaced as one short line instead of the full explanation.
pub const SAVE_GATE_CHIP: &str = "Requires Safe Live Save Mode";

/// Collapse a row's runtime-preview and dead-man classifications into one
/// quiet chip. Fail-closed: anything unknown reads as Not Proven Yet.
pub fn status_chip_for_row(row_id: &str) -> StatusChip {
    let Some(capability) = runtime_preview_row_capability(row_id) else {
        return StatusChip::NotProvenYet;
    };
    match capability.capability {
        RuntimePreviewCapability::LivePreviewSupported
        | RuntimePreviewCapability::LivePreviewSupportedWithThrottle => StatusChip::LivePreview,
        RuntimePreviewCapability::LivePreviewSupportedWithDeadMan => {
            match classify_dead_man_row(row_id) {
                Some(row) => match row.classification {
                    RuntimePreviewDeadManClassification::DeadManPreviewCandidate => {
                        StatusChip::LivePreview
                    }
                    RuntimePreviewDeadManClassification::DeadManPreviewCandidateNeedsLiveProof => {
                        if row_needs_hardware(row.reason) {
                            StatusChip::HardwareRequired
                        } else {
                            StatusChip::NotProvenYet
                        }
                    }
                    RuntimePreviewDeadManClassification::DeadManPreviewModelOnly => {
                        StatusChip::SaveOnly
                    }
                    _ => StatusChip::Blocked,
                },
                None => StatusChip::NotProvenYet,
            }
        }
        RuntimePreviewCapability::LivePreviewReadOnlyOnly
        | RuntimePreviewCapability::RequiresConfigWrite
        | RuntimePreviewCapability::RequiresReload
        | RuntimePreviewCapability::RequiresRelog
        | RuntimePreviewCapability::RequiresRestart => StatusChip::SaveOnly,
        RuntimePreviewCapability::BlockedHighRisk
        | RuntimePreviewCapability::BlockedUnsupportedGrammar
        | RuntimePreviewCapability::BlockedStructuredFamilySemantics => StatusChip::Blocked,
        RuntimePreviewCapability::NotProvenYet => StatusChip::NotProvenYet,
    }
}

/// Hardware-gated proof reasons mention the missing device class.
fn row_needs_hardware(reason: &str) -> bool {
    let lowered = reason.to_ascii_lowercase();
    ["touch", "tablet", "hardware", "device"]
        .iter()
        .any(|term| lowered.contains(term))
}

/// The friendly display label for a row: the adopted matched label where
/// one exists, otherwise the official metadata label unchanged (unmatched
/// rows are reported, never guessed).
pub fn resolved_row_label<'a>(row_id: &str, official_label: &'a str) -> &'a str
where
    'static: 'a,
{
    crate::presentation_labels::display_label_for_row(row_id).unwrap_or(official_label)
}

/// A formatting-only fallback for unmatched rows: strip the page name the
/// official label repeats (the page title already says it). Mechanical —
/// no meaning is guessed and the remaining official words are unchanged;
/// rows whose label is only the page name keep it as-is.
pub fn fallback_display_label(official_label: &str, tab_label: &str) -> String {
    let trimmed = official_label.trim();
    let prefix = format!("{} ", tab_label.trim());
    match trimmed.strip_prefix(&prefix) {
        Some(rest) if !rest.trim().is_empty() => rest.trim().to_string(),
        _ => trimmed.to_string(),
    }
}

/// Stronger mechanical fallback used by row titles: also strips the
/// official section word (the config-section name the page/section
/// heading already shows) and the redundant "Col " color-group marker.
/// Purely formatting — the remaining official words are unchanged.
pub fn row_display_title(official_label: &str, tab_label: &str, official_setting: &str) -> String {
    // Curated titles for the few rows whose mechanical fallback reads
    // awkwardly ("Nogroup Border"). Presentation only.
    const CURATED_TITLES: &[(&str, &str)] = &[
        ("general.col.nogroup_border", "No-group border color"),
        (
            "general.col.nogroup_border_active",
            "Active no-group border color",
        ),
    ];
    if let Some((_, curated)) = CURATED_TITLES
        .iter()
        .find(|(key, _)| *key == official_setting)
    {
        return (*curated).to_string();
    }
    let mut title = fallback_display_label(official_label, tab_label);
    if let Some(section) = official_setting.split('.').next() {
        let mut section_word = section.to_string();
        if let Some(first) = section_word.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        if let Some(rest) = title.strip_prefix(&format!("{section_word} ")) {
            if !rest.trim().is_empty() {
                title = rest.trim().to_string();
            }
        }
    }
    if let Some(rest) = title.strip_prefix("Col ") {
        if !rest.trim().is_empty() {
            title = rest.trim().to_string();
        }
    }
    title
}

/// A friendly display form for a finite-choice raw value: presentation
/// only — the raw value is what gets validated and saved, unchanged.
/// Generic humanization (separators to spaces, first letter capitalized);
/// numeric and empty values pass through untouched.
pub fn choice_display_label(raw_value: &str) -> String {
    let trimmed = raw_value.trim();
    if trimmed.is_empty() || trimmed.parse::<f64>().is_ok() {
        return trimmed.to_string();
    }
    let spaced = trimmed.replace(['_', '-'], " ");
    let mut characters = spaced.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => spaced,
    }
}

/// HyprMod-style Animations page grouping: section heading -> the runtime
/// animation record names it hosts, in display order. Records present in
/// the runtime listing but not named here render at the end of "Other"
/// when they carry an explicit override, so nothing editable is hidden.
pub const ANIMATION_RECORD_GROUPS: &[(&str, &[&str])] = &[
    ("Global", &["global"]),
    ("Windows & Layers", &["windows", "layers"]),
    ("Fading", &["fade"]),
    ("Workspaces", &["workspaces"]),
    (
        "Other",
        &["border", "borderangle", "zoomFactor", "monitorAdded"],
    ),
];

/// Friendly display name for a runtime animation record.
pub fn animation_record_display_name(name: &str) -> String {
    match name {
        "global" => "Global".to_string(),
        "windows" => "Windows".to_string(),
        "layers" => "Layers".to_string(),
        "fade" => "Fade".to_string(),
        "workspaces" => "Workspaces".to_string(),
        "border" => "Border".to_string(),
        "borderangle" => "Border Angle".to_string(),
        "zoomFactor" => "Zoom Factor".to_string(),
        "monitorAdded" => "Monitor Added".to_string(),
        "specialWorkspace" => "Special Workspace".to_string(),
        other => choice_display_label(other),
    }
}

/// Compact friendly subtitle for an animation record row, in the
/// "4.0ds · easeOutQuint [· style]" shape; disabled and inherited records
/// lead with that word instead of raw flag digits.
pub fn animation_record_subtitle(
    enabled: &str,
    speed: &str,
    bezier: &str,
    style: &str,
    overridden: bool,
) -> String {
    let speed_text = match speed.trim().parse::<f64>() {
        Ok(value) => format!("{value:.1}ds"),
        Err(_) => format!("{}ds", speed.trim()),
    };
    let curve = if bezier.trim().is_empty() {
        "default"
    } else {
        bezier.trim()
    };
    let mut parts: Vec<String> = Vec::new();
    if !overridden {
        parts.push("inherited".to_string());
    } else if enabled.trim() == "0" {
        parts.push("disabled".to_string());
    }
    parts.push(speed_text);
    parts.push(curve.to_string());
    if overridden && !style.trim().is_empty() {
        parts.push(style.trim().to_string());
    }
    parts.join(" · ")
}

/// The picker palette in the reference layout: nine columns (six hue
/// families, brown, then light and dark neutrals), five shades each,
/// lighter at the top. Rendered as contiguous vertical shade stacks.
pub fn picker_palette_columns() -> Vec<Vec<ParsedColor>> {
    fn hsv(hue: f64, saturation: f64, value: f64) -> ParsedColor {
        let (red, green, blue) = hsv_to_rgb(hue, saturation, value);
        ParsedColor {
            red,
            green,
            blue,
            alpha: 0xff,
        }
    }
    let mut columns = Vec::new();
    for &hue in &[217.0, 145.0, 50.0, 28.0, 2.0, 282.0] {
        columns.push(vec![
            hsv(hue, 0.45, 0.95),
            hsv(hue, 0.62, 0.92),
            hsv(hue, 0.80, 0.88),
            hsv(hue, 0.92, 0.78),
            hsv(hue, 0.95, 0.62),
        ]);
    }
    // Brown: muted orange family.
    columns.push(vec![
        hsv(26.0, 0.45, 0.80),
        hsv(26.0, 0.55, 0.70),
        hsv(26.0, 0.62, 0.58),
        hsv(26.0, 0.68, 0.46),
        hsv(26.0, 0.72, 0.35),
    ]);
    // Light neutrals.
    columns.push(vec![
        hsv(0.0, 0.0, 0.97),
        hsv(0.0, 0.0, 0.90),
        hsv(0.0, 0.0, 0.82),
        hsv(0.0, 0.0, 0.72),
        hsv(0.0, 0.0, 0.62),
    ]);
    // Dark neutrals down to black.
    columns.push(vec![
        hsv(0.0, 0.0, 0.42),
        hsv(0.0, 0.0, 0.33),
        hsv(0.0, 0.0, 0.24),
        hsv(0.0, 0.0, 0.14),
        hsv(0.0, 0.0, 0.0),
    ]);
    columns
}

/// A parsed color for swatch/preview rendering. Presentation only: the
/// raw config text stays the value that gets validated and saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

fn hex_byte(text: &str) -> Option<u8> {
    u8::from_str_radix(text, 16).ok()
}

/// Parse the Hyprland color forms: `rgba(RRGGBBAA)`, `rgb(RRGGBB)`, and
/// legacy `0xAARRGGBB`. Anything else fails closed (no swatch, raw text
/// stays read-only in the picker).
pub fn parse_hyprland_color(raw: &str) -> Option<ParsedColor> {
    let trimmed = raw.trim();
    if let Some(inner) = trimmed
        .strip_prefix("rgba(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let inner = inner.trim();
        if inner.len() == 8 && inner.chars().all(|character| character.is_ascii_hexdigit()) {
            return Some(ParsedColor {
                red: hex_byte(&inner[0..2])?,
                green: hex_byte(&inner[2..4])?,
                blue: hex_byte(&inner[4..6])?,
                alpha: hex_byte(&inner[6..8])?,
            });
        }
        return None;
    }
    if let Some(inner) = trimmed
        .strip_prefix("rgb(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let inner = inner.trim();
        if inner.len() == 6 && inner.chars().all(|character| character.is_ascii_hexdigit()) {
            return Some(ParsedColor {
                red: hex_byte(&inner[0..2])?,
                green: hex_byte(&inner[2..4])?,
                blue: hex_byte(&inner[4..6])?,
                alpha: 0xff,
            });
        }
        return None;
    }
    if let Some(inner) = trimmed.strip_prefix("0x") {
        if inner.len() == 8 && inner.chars().all(|character| character.is_ascii_hexdigit()) {
            return Some(ParsedColor {
                alpha: hex_byte(&inner[0..2])?,
                red: hex_byte(&inner[2..4])?,
                green: hex_byte(&inner[4..6])?,
                blue: hex_byte(&inner[6..8])?,
            });
        }
        return None;
    }
    None
}

/// Parse a gradient-form value: two or more colors, optionally followed by
/// `<angle>deg`. Single colors are not gradients; any unrecognized token
/// fails the whole parse (fail closed — the strip simply does not render).
pub fn parse_hyprland_gradient(raw: &str) -> Option<(Vec<ParsedColor>, Option<u16>)> {
    let mut colors = Vec::new();
    let mut angle = None;
    let tokens: Vec<&str> = raw.split_whitespace().collect();
    for (index, token) in tokens.iter().enumerate() {
        if let Some(color) = parse_hyprland_color(token) {
            if angle.is_some() {
                return None; // colors after the angle are not valid syntax
            }
            colors.push(color);
            continue;
        }
        if let Some(value) = token.strip_suffix("deg") {
            if index != tokens.len() - 1 {
                return None;
            }
            angle = Some(value.parse::<u16>().ok()?);
            continue;
        }
        return None;
    }
    if colors.len() < 2 {
        return None;
    }
    Some((colors, angle))
}

/// HSV -> RGB for the picker (h in degrees 0..360, s/v in 0..=1).
pub fn hsv_to_rgb(hue: f64, saturation: f64, value: f64) -> (u8, u8, u8) {
    let hue = hue.rem_euclid(360.0);
    let chroma = value * saturation;
    let x = chroma * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = value - chroma;
    let (red, green, blue) = match hue as u32 {
        0..=59 => (chroma, x, 0.0),
        60..=119 => (x, chroma, 0.0),
        120..=179 => (0.0, chroma, x),
        180..=239 => (0.0, x, chroma),
        240..=299 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    (
        ((red + m) * 255.0).round() as u8,
        ((green + m) * 255.0).round() as u8,
        ((blue + m) * 255.0).round() as u8,
    )
}

/// RGB -> HSV for the picker (returns h in degrees, s/v in 0..=1).
pub fn rgb_to_hsv(red: u8, green: u8, blue: u8) -> (f64, f64, f64) {
    let red = red as f64 / 255.0;
    let green = green as f64 / 255.0;
    let blue = blue as f64 / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let hue = if delta == 0.0 {
        0.0
    } else if max == red {
        60.0 * (((green - blue) / delta).rem_euclid(6.0))
    } else if max == green {
        60.0 * ((blue - red) / delta + 2.0)
    } else {
        60.0 * ((red - green) / delta + 4.0)
    };
    let saturation = if max == 0.0 { 0.0 } else { delta / max };
    (hue, saturation, max)
}

/// Render a picked color in the same format family as the original token
/// (0xAARRGGBB stays 0x-form; rgb(hex6) stays rgb when fully opaque;
/// everything else renders the canonical rgba(RRGGBBAA)).
pub fn render_color_like(original_token: &str, color: ParsedColor) -> String {
    let trimmed = original_token.trim();
    if trimmed.starts_with("0x") {
        return format!(
            "0x{:02x}{:02x}{:02x}{:02x}",
            color.alpha, color.red, color.green, color.blue
        );
    }
    if trimmed.starts_with("rgb(") && color.alpha == 0xff {
        return format!(
            "rgb({:02x}{:02x}{:02x})",
            color.red, color.green, color.blue
        );
    }
    format!(
        "rgba({:02x}{:02x}{:02x}{:02x})",
        color.red, color.green, color.blue, color.alpha
    )
}

/// Shorten an official description to its first sentence, capped for a
/// one-line row subtitle. The full description stays in the detail pane.
pub fn short_description(description: &str) -> String {
    let trimmed = description.trim();
    let first_sentence = trimmed
        .split_inclusive(['.', '!', '?'])
        .next()
        .unwrap_or(trimmed)
        .trim();
    const MAX: usize = 110;
    let sentence = if first_sentence.chars().count() <= MAX {
        first_sentence.to_string()
    } else {
        let cut: String = first_sentence.chars().take(MAX - 1).collect();
        format!("{}…", cut.trim_end())
    };
    // Sentence-case: official descriptions often start lowercase.
    let mut characters = sentence.chars();
    match characters.next() {
        Some(first) if first.is_ascii_lowercase() => {
            first.to_ascii_uppercase().to_string() + characters.as_str()
        }
        _ => sentence,
    }
}
