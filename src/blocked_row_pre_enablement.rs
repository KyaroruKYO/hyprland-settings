use std::fs;
use std::path::{Path, PathBuf};

use crate::config_parser::parse_hyprland_config_file;
use crate::write_classification::{config_key_from_official_setting, is_safe_writable_setting};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockedRowBucket {
    DisplayRender,
    CursorInput,
    DebugCrash,
}

impl BlockedRowBucket {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DisplayRender => "display/render",
            Self::CursorInput => "cursor/input",
            Self::DebugCrash => "debug/crash",
        }
    }

    pub fn gate_family(self) -> &'static str {
        match self {
            Self::DisplayRender => "display-render-pre-enablement-gate-model",
            Self::CursorInput => "cursor-input-pre-enablement-gate-model",
            Self::DebugCrash => "debug-crash-pre-enablement-gate-model",
        }
    }

    pub fn warning(self) -> &'static str {
        match self {
            Self::DisplayRender => {
                "Display/render high-risk setting. Future writes require an explicit advanced gate, backup, watchdog recovery, and non-live proof before enablement."
            }
            Self::CursorInput => {
                "Cursor/input high-risk setting. Future writes require an explicit advanced gate and recovery path that does not depend on normal pointer behavior."
            }
            Self::DebugCrash => {
                "Debug/crash high-risk setting. Future writes require an explicit advanced gate and proof that debug or crash behavior cannot strand the session."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreEnablementValueFamily {
    Boolean,
    FiniteChoice(&'static [&'static str]),
    IntegerRange { min: i64, max: i64 },
    FloatRange { min: f64, max: f64 },
    TransferFunction,
    DynamicMonitorName,
}

impl PreEnablementValueFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::FiniteChoice(_) => "finite-choice",
            Self::IntegerRange { .. } => "bounded-integer",
            Self::FloatRange { .. } => "bounded-float",
            Self::TransferFunction => "transfer-function",
            Self::DynamicMonitorName => "dynamic-monitor-name",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockedPreEnablementRow {
    pub row_id: &'static str,
    pub official_setting: &'static str,
    pub bucket: BlockedRowBucket,
    pub value_family: PreEnablementValueFamily,
    pub official_source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreEnablementValidation {
    Accepted,
    Rejected { reason: String },
}

impl PreEnablementValidation {
    pub fn is_accepted(&self) -> bool {
        matches!(self, Self::Accepted)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureWriteRereadProof {
    pub config_path: PathBuf,
    pub official_setting: String,
    pub written_value: String,
    pub reread_value: Option<String>,
    pub rollback_restored: bool,
    pub production_allowlist_used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreEnablementGateProjection {
    pub row_id: String,
    pub gate_family: String,
    pub ungated_write_rejected_by_current_allowlist: bool,
    pub production_gate_added: bool,
    pub remaining_gate_blocker: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiWarningProjection {
    pub row_id: String,
    pub placement: String,
    pub warning: String,
    pub production_ui_wiring_added: bool,
}

const AUTO_012: &[&str] = &["0", "disable", "1", "enable", "2", "auto"];
const HDR_AUTO: &[&str] = &["0", "disable", "1", "hdr", "2", "hdredid"];
const NON_SHADER_CM: &[&str] = &[
    "0", "disable", "1", "always", "2", "ondemand", "3", "ignore",
];
const FP16_SDR_TF: &[&str] = &["0", "monitor", "1", "linear"];
const HARDWARE_CURSOR: &[&str] = &["0", "Disabled", "1", "Enabled", "2", "Auto"];
const FORCE_012: &[&str] = &["0", "disable", "1", "enable", "2", "force"];
const DAMAGE_TRACKING: &[&str] = &["0", "disable", "1", "monitor", "2", "full"];
const ERROR_POSITION: &[&str] = &["0", "top", "1", "bottom"];
const PREFER_HDR: &[&str] = &["0", "disable", "1", "enable", "2", "gamescope_only"];
const TRANSFER_FUNCTION: &[&str] = &[
    "default",
    "0",
    "auto",
    "srgb",
    "3",
    "gamma22",
    "1",
    "gamma22force",
    "2",
];

pub const BLOCKED_PRE_ENABLEMENT_ROWS: &[BlockedPreEnablementRow] = &[
    display_render(
        "xwayland.enabled",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:509",
    ),
    display_render(
        "xwayland.create_abstract_socket",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:512",
    ),
    display_render(
        "opengl.nvidia_anti_flicker",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:518",
    ),
    display_render(
        "render.direct_scanout",
        PreEnablementValueFamily::FiniteChoice(AUTO_012),
        "ConfigValues.cpp:524",
    ),
    display_render(
        "render.expand_undersized_textures",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:525",
    ),
    display_render(
        "render.xp_mode",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:526",
    ),
    display_render(
        "render.ctm_animation",
        PreEnablementValueFamily::FiniteChoice(AUTO_012),
        "ConfigValues.cpp:527",
    ),
    display_render(
        "render.cm_enabled",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:529",
    ),
    display_render(
        "render.send_content_type",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:530",
    ),
    display_render(
        "render.cm_auto_hdr",
        PreEnablementValueFamily::FiniteChoice(HDR_AUTO),
        "ConfigValues.cpp:531",
    ),
    display_render(
        "render.new_render_scheduling",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:533",
    ),
    display_render(
        "render.non_shader_cm",
        PreEnablementValueFamily::FiniteChoice(NON_SHADER_CM),
        "ConfigValues.cpp:534",
    ),
    display_render(
        "render.cm_sdr_eotf",
        PreEnablementValueFamily::TransferFunction,
        "ConfigValues.cpp:535; TransferFunction.cpp:10-17",
    ),
    display_render(
        "render.commit_timing_enabled",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:536",
    ),
    display_render(
        "render.icc_vcgt_enabled",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:537",
    ),
    display_render(
        "render.use_shader_blur_blend",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:538",
    ),
    display_render(
        "render.use_fp16",
        PreEnablementValueFamily::FiniteChoice(AUTO_012),
        "ConfigValues.cpp:539",
    ),
    display_render(
        "render.keep_unmodified_copy",
        PreEnablementValueFamily::FiniteChoice(AUTO_012),
        "ConfigValues.cpp:540",
    ),
    display_render(
        "render.non_shader_cm_interop",
        PreEnablementValueFamily::FiniteChoice(AUTO_012),
        "ConfigValues.cpp:542",
    ),
    display_render(
        "render.fp16_sdr_tf",
        PreEnablementValueFamily::FiniteChoice(FP16_SDR_TF),
        "ConfigValues.cpp:544",
    ),
    cursor_input(
        "cursor.invisible",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:550",
    ),
    cursor_input(
        "cursor.no_hardware_cursors",
        PreEnablementValueFamily::FiniteChoice(HARDWARE_CURSOR),
        "ConfigValues.cpp:551",
    ),
    cursor_input(
        "cursor.no_break_fs_vrr",
        PreEnablementValueFamily::FiniteChoice(AUTO_012),
        "ConfigValues.cpp:552",
    ),
    cursor_input(
        "cursor.min_refresh_rate",
        PreEnablementValueFamily::IntegerRange { min: 10, max: 500 },
        "ConfigValues.cpp:554",
    ),
    cursor_input(
        "cursor.hotspot_padding",
        PreEnablementValueFamily::IntegerRange { min: 0, max: 20 },
        "ConfigValues.cpp:555",
    ),
    cursor_input(
        "cursor.inactive_timeout",
        PreEnablementValueFamily::FloatRange {
            min: 0.0,
            max: 20.0,
        },
        "ConfigValues.cpp:556",
    ),
    cursor_input(
        "cursor.no_warps",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:557",
    ),
    cursor_input(
        "cursor.persistent_warps",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:558",
    ),
    cursor_input(
        "cursor.warp_on_change_workspace",
        PreEnablementValueFamily::FiniteChoice(FORCE_012),
        "ConfigValues.cpp:559",
    ),
    cursor_input(
        "cursor.warp_on_toggle_special",
        PreEnablementValueFamily::FiniteChoice(FORCE_012),
        "ConfigValues.cpp:561",
    ),
    cursor_input(
        "cursor.default_monitor",
        PreEnablementValueFamily::DynamicMonitorName,
        "ConfigValues.cpp:563; Compositor.cpp:2992-3014",
    ),
    cursor_input(
        "cursor.zoom_factor",
        PreEnablementValueFamily::FloatRange {
            min: 1.0,
            max: 10.0,
        },
        "ConfigValues.cpp:564",
    ),
    cursor_input(
        "cursor.zoom_rigid",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:565",
    ),
    cursor_input(
        "cursor.zoom_disable_aa",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:566",
    ),
    cursor_input(
        "cursor.zoom_detached_camera",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:567",
    ),
    cursor_input(
        "cursor.enable_hyprcursor",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:568",
    ),
    cursor_input(
        "cursor.use_cpu_buffer",
        PreEnablementValueFamily::FiniteChoice(AUTO_012),
        "ConfigValues.cpp:572",
    ),
    cursor_input(
        "cursor.warp_back_after_non_mouse_input",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:574",
    ),
    debug_crash(
        "debug.overlay",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:586",
    ),
    debug_crash(
        "debug.damage_blink",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:587",
    ),
    debug_crash(
        "debug.gl_debugging",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:588",
    ),
    debug_crash(
        "debug.disable_logs",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:589",
    ),
    debug_crash(
        "debug.disable_time",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:590",
    ),
    debug_crash(
        "debug.damage_tracking",
        PreEnablementValueFamily::FiniteChoice(DAMAGE_TRACKING),
        "ConfigValues.cpp:591",
    ),
    debug_crash(
        "debug.enable_stdout_logs",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:592",
    ),
    debug_crash(
        "debug.manual_crash",
        PreEnablementValueFamily::IntegerRange { min: 0, max: 1 },
        "ConfigValues.cpp:593",
    ),
    debug_crash(
        "debug.suppress_errors",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:594",
    ),
    debug_crash(
        "debug.disable_scale_checks",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:595",
    ),
    debug_crash(
        "debug.error_limit",
        PreEnablementValueFamily::IntegerRange { min: 0, max: 20 },
        "ConfigValues.cpp:596",
    ),
    debug_crash(
        "debug.error_position",
        PreEnablementValueFamily::FiniteChoice(ERROR_POSITION),
        "ConfigValues.cpp:597",
    ),
    debug_crash(
        "debug.colored_stdout_logs",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:598",
    ),
    debug_crash(
        "debug.log_damage",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:599",
    ),
    debug_crash(
        "debug.pass",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:600",
    ),
    debug_crash(
        "debug.full_cm_proto",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:601",
    ),
    debug_crash(
        "debug.ds_handle_same_buffer",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:602",
    ),
    debug_crash(
        "debug.ds_handle_same_buffer_fifo",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:603",
    ),
    debug_crash(
        "debug.fifo_pending_workaround",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:604",
    ),
    debug_crash(
        "debug.render_solitary_wo_damage",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:605",
    ),
    debug_crash(
        "debug.vfr",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:606",
    ),
    debug_crash(
        "debug.invalidate_fp16",
        PreEnablementValueFamily::FiniteChoice(AUTO_012),
        "ConfigValues.cpp:607",
    ),
    display_render(
        "experimental.wp_cm_1_2",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:681",
    ),
    display_render(
        "quirks.prefer_hdr",
        PreEnablementValueFamily::FiniteChoice(PREFER_HDR),
        "ConfigValues.cpp:687",
    ),
    display_render(
        "quirks.skip_non_kms_dmabuf_formats",
        PreEnablementValueFamily::Boolean,
        "ConfigValues.cpp:688",
    ),
];

const fn display_render(
    row_id: &'static str,
    value_family: PreEnablementValueFamily,
    official_source: &'static str,
) -> BlockedPreEnablementRow {
    row(
        row_id,
        BlockedRowBucket::DisplayRender,
        value_family,
        official_source,
    )
}

const fn cursor_input(
    row_id: &'static str,
    value_family: PreEnablementValueFamily,
    official_source: &'static str,
) -> BlockedPreEnablementRow {
    row(
        row_id,
        BlockedRowBucket::CursorInput,
        value_family,
        official_source,
    )
}

const fn debug_crash(
    row_id: &'static str,
    value_family: PreEnablementValueFamily,
    official_source: &'static str,
) -> BlockedPreEnablementRow {
    row(
        row_id,
        BlockedRowBucket::DebugCrash,
        value_family,
        official_source,
    )
}

const fn row(
    row_id: &'static str,
    bucket: BlockedRowBucket,
    value_family: PreEnablementValueFamily,
    official_source: &'static str,
) -> BlockedPreEnablementRow {
    BlockedPreEnablementRow {
        row_id,
        official_setting: row_id,
        bucket,
        value_family,
        official_source,
    }
}

pub fn blocked_pre_enablement_rows() -> &'static [BlockedPreEnablementRow] {
    BLOCKED_PRE_ENABLEMENT_ROWS
}

pub fn blocked_pre_enablement_row(row_id: &str) -> Option<&'static BlockedPreEnablementRow> {
    BLOCKED_PRE_ENABLEMENT_ROWS
        .iter()
        .find(|row| row.row_id == row_id)
}

pub fn valid_pre_enablement_example(row: &BlockedPreEnablementRow) -> String {
    match row.value_family {
        PreEnablementValueFamily::Boolean => "true".to_string(),
        PreEnablementValueFamily::FiniteChoice(values) => values
            .first()
            .expect("finite-choice pre-enablement rows require values")
            .to_string(),
        PreEnablementValueFamily::IntegerRange { min, .. } => min.to_string(),
        PreEnablementValueFamily::FloatRange { min, .. } => format_float(min),
        PreEnablementValueFamily::TransferFunction => "default".to_string(),
        PreEnablementValueFamily::DynamicMonitorName => "".to_string(),
    }
}

pub fn invalid_pre_enablement_example(row: &BlockedPreEnablementRow) -> String {
    match row.value_family {
        PreEnablementValueFamily::Boolean => "maybe".to_string(),
        PreEnablementValueFamily::FiniteChoice(_) => "not-a-source-backed-option".to_string(),
        PreEnablementValueFamily::IntegerRange { max, .. } => (max + 1).to_string(),
        PreEnablementValueFamily::FloatRange { max, .. } => format_float(max + 1.0),
        PreEnablementValueFamily::TransferFunction => "not-a-transfer-function".to_string(),
        PreEnablementValueFamily::DynamicMonitorName => "monitor;hyprctl reload".to_string(),
    }
}

pub fn validate_pre_enablement_value(
    row: &BlockedPreEnablementRow,
    value: &str,
) -> PreEnablementValidation {
    let trimmed = value.trim();
    match row.value_family {
        PreEnablementValueFamily::Boolean => {
            if matches!(trimmed, "true" | "false") {
                PreEnablementValidation::Accepted
            } else {
                reject("boolean pre-enablement proof accepts only true or false")
            }
        }
        PreEnablementValueFamily::FiniteChoice(values) => {
            if values.iter().any(|candidate| *candidate == trimmed) {
                PreEnablementValidation::Accepted
            } else {
                reject("finite-choice value is not in the source-backed option map")
            }
        }
        PreEnablementValueFamily::IntegerRange { min, max } => {
            if trimmed.is_empty() || trimmed.contains(char::is_whitespace) {
                return reject("integer value must be a single token");
            }
            match trimmed.parse::<i64>() {
                Ok(number) if number >= min && number <= max => PreEnablementValidation::Accepted,
                Ok(_) => reject("integer value is outside the source-backed bounds"),
                Err(_) => reject("integer value is not numeric"),
            }
        }
        PreEnablementValueFamily::FloatRange { min, max } => {
            if trimmed.is_empty() || trimmed.contains(char::is_whitespace) {
                return reject("float value must be a single token");
            }
            match trimmed.parse::<f64>() {
                Ok(number) if number.is_finite() && number >= min && number <= max => {
                    PreEnablementValidation::Accepted
                }
                Ok(number) if !number.is_finite() => reject("float value must be finite"),
                Ok(_) => reject("float value is outside the source-backed bounds"),
                Err(_) => reject("float value is not numeric"),
            }
        }
        PreEnablementValueFamily::TransferFunction => {
            if TRANSFER_FUNCTION
                .iter()
                .any(|candidate| *candidate == trimmed)
            {
                PreEnablementValidation::Accepted
            } else {
                reject("transfer function is not in the official source table")
            }
        }
        PreEnablementValueFamily::DynamicMonitorName => {
            if trimmed.contains('\n')
                || trimmed.contains('\r')
                || trimmed.contains(';')
                || trimmed.contains('`')
                || trimmed.contains('$')
            {
                reject("monitor name must stay on one config line and avoid command syntax")
            } else {
                PreEnablementValidation::Accepted
            }
        }
    }
}

pub fn prove_fixture_write_reread(
    row: &BlockedPreEnablementRow,
    value: &str,
    root: impl AsRef<Path>,
) -> Result<FixtureWriteRereadProof, String> {
    if !validate_pre_enablement_value(row, value).is_accepted() {
        return Err("fixture proof requires a value accepted by the proof validator".to_string());
    }

    let root = root.as_ref();
    ensure_temp_path(root)?;
    fs::create_dir_all(root)
        .map_err(|error| format!("failed to create fixture root {}: {error}", root.display()))?;
    let config_path = root.join("hyprland-pre-enablement.conf");
    let previous_contents = "# pre-enablement rollback baseline\n";
    fs::write(&config_path, previous_contents)
        .map_err(|error| format!("failed to seed fixture config: {error}"))?;

    let config_key = config_key_from_official_setting(row.official_setting);
    fs::write(&config_path, format!("{config_key} = {value}\n"))
        .map_err(|error| format!("failed to write fixture config: {error}"))?;
    let parsed = parse_hyprland_config_file(&config_path)
        .map_err(|error| format!("failed to parse fixture config: {error:#}"))?;
    let reread_value = parsed
        .scalar_records()
        .find(|record| record.normalized_setting_id.as_deref() == Some(row.official_setting))
        .and_then(|record| record.raw_value.clone());

    fs::write(&config_path, previous_contents)
        .map_err(|error| format!("failed to restore fixture baseline: {error}"))?;
    let rollback_restored = fs::read_to_string(&config_path)
        .map(|contents| contents == previous_contents)
        .unwrap_or(false);

    Ok(FixtureWriteRereadProof {
        config_path,
        official_setting: row.official_setting.to_string(),
        written_value: value.to_string(),
        reread_value,
        rollback_restored,
        production_allowlist_used: false,
    })
}

pub fn pre_enablement_gate_projection(
    row: &BlockedPreEnablementRow,
) -> PreEnablementGateProjection {
    PreEnablementGateProjection {
        row_id: row.row_id.to_string(),
        gate_family: row.bucket.gate_family().to_string(),
        ungated_write_rejected_by_current_allowlist: !is_safe_writable_setting(row.row_id),
        production_gate_added: false,
        remaining_gate_blocker:
            "production-capable high-risk gate and live-independent recovery proof remain required before enablement"
                .to_string(),
    }
}

pub fn ui_warning_projection(row: &BlockedPreEnablementRow) -> UiWarningProjection {
    UiWarningProjection {
        row_id: row.row_id.to_string(),
        placement: "advanced/high-risk pre-enablement projection".to_string(),
        warning: row.bucket.warning().to_string(),
        production_ui_wiring_added: false,
    }
}

fn ensure_temp_path(path: &Path) -> Result<(), String> {
    if path.starts_with(std::env::temp_dir()) {
        Ok(())
    } else {
        Err(format!(
            "pre-enablement fixture writes require a temp path, got {}",
            path.display()
        ))
    }
}

fn reject(reason: &str) -> PreEnablementValidation {
    PreEnablementValidation::Rejected {
        reason: reason.to_string(),
    }
}

fn format_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}
