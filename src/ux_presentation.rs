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
            },
            PageSpec {
                id: "decoration",
                label: "Decoration",
                source_tab: Some("appearance"),
                include_prefixes: Some(&["decoration."]),
            },
            PageSpec {
                id: "animations",
                label: "Animations",
                source_tab: Some("animations"),
                include_prefixes: None,
            },
            PageSpec {
                id: "cursor",
                label: "Cursor",
                source_tab: Some("cursor"),
                include_prefixes: None,
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
            },
            PageSpec {
                id: "devices",
                label: "Devices",
                source_tab: Some("input"),
                include_prefixes: None,
            },
            PageSpec {
                id: "gestures",
                label: "Gestures",
                source_tab: Some("input"),
                include_prefixes: Some(&["gestures.", "gesture."]),
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
            },
            PageSpec {
                id: "workspaces",
                label: "Workspaces",
                source_tab: None,
                include_prefixes: None,
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
            },
            PageSpec {
                id: "window-rules",
                label: "Window Rules",
                source_tab: None,
                include_prefixes: None,
            },
            PageSpec {
                id: "layer-rules",
                label: "Layer Rules",
                source_tab: None,
                include_prefixes: None,
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
            },
            PageSpec {
                id: "env-variables",
                label: "Env Variables",
                source_tab: None,
                include_prefixes: None,
            },
        ],
    },
    SidebarCategoryPages {
        label: "Advanced",
        pages: &[
            PageSpec {
                id: "xwayland",
                label: "XWayland",
                source_tab: Some("system"),
                include_prefixes: Some(&["xwayland."]),
            },
            PageSpec {
                id: "ecosystem",
                label: "Ecosystem",
                source_tab: Some("system"),
                include_prefixes: Some(&["ecosystem."]),
            },
            PageSpec {
                id: "system",
                label: "System",
                source_tab: Some("system"),
                include_prefixes: None,
            },
            PageSpec {
                id: "permissions",
                label: "Permissions",
                source_tab: Some("permissions"),
                include_prefixes: None,
            },
            PageSpec {
                id: "windows-layout",
                label: "Windows & Layout",
                source_tab: Some("windows-layout"),
                include_prefixes: None,
            },
            PageSpec {
                id: "profiles",
                label: "Profiles",
                source_tab: None,
                include_prefixes: None,
            },
            PageSpec {
                id: "config",
                label: "Settings",
                source_tab: None,
                include_prefixes: None,
            },
            PageSpec {
                id: "safety-details",
                label: "Safety Details",
                source_tab: None,
                include_prefixes: None,
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

/// Whether a row belongs on a page: prefix pages claim their prefixes; the
/// tab's rest page takes everything no sibling prefix page claimed.
pub fn page_claims_row(page: &PageSpec, official_setting: &str) -> bool {
    let Some(source_tab) = page.source_tab else {
        return false;
    };
    match page.include_prefixes {
        Some(prefixes) => prefixes
            .iter()
            .any(|prefix| official_setting.starts_with(prefix)),
        None => {
            // Rest page: not claimed by any sibling prefix page of the
            // same tab.
            !SIDEBAR_PAGE_LAYOUT
                .iter()
                .flat_map(|category| category.pages.iter())
                .filter(|sibling| sibling.source_tab == Some(source_tab))
                .filter_map(|sibling| sibling.include_prefixes)
                .flatten()
                .any(|prefix| official_setting.starts_with(prefix))
        }
    }
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
        ("General", "Gaps & Borders"),
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
    if first_sentence.chars().count() <= MAX {
        return first_sentence.to_string();
    }
    let cut: String = first_sentence.chars().take(MAX - 1).collect();
    format!("{}…", cut.trim_end())
}
