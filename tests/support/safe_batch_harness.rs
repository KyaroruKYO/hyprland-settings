use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::config_discovery::{discover_hyprland_config, ConfigDiscoveryStatus};
use hyprland_settings::config_graph::{
    inspect_config_graph, ConfigDetectionConfidence, ConfigGraphFile, ConfigGraphSummary,
    ConfigManagementHint, ConfigManagementHintKind,
};
use hyprland_settings::config_parser::parse_hyprland_config_file;
use hyprland_settings::current_config::{
    CurrentConfigLoadStatus, CurrentConfigSnapshot, CurrentValue, CurrentValueProjection,
    CurrentValueSourceStatus, CurrentValueStatus,
};
use hyprland_settings::pending_change::{
    stage_pending_change_with_sources, PendingChangeValidation, PendingChangeValueSources,
};
use hyprland_settings::safe_batch_write::{
    build_safe_batch_write_plan, SafeBatchChangeRequest, SafeBatchEligibility, SafeBatchWritePlan,
    SafeBatchWriteReport,
};
use hyprland_settings::source_aware_current_config::{
    current_config_from_graph, source_aware_mapping_report,
};
use hyprland_settings::source_values::read_system_xkb_rules;
use hyprland_settings::write_classification::{
    config_key_from_official_setting, finite_choice_options, high_risk_write_policy,
    source_backed_numeric_bounds, ScalarWriteValueKind, SourceBackedNumericType,
    SAFE_WRITABLE_ROWS,
};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessValuePair {
    pub old_value: String,
    pub proposed_value: String,
    pub invalid_value: Option<String>,
    pub coverage_status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessRowCase {
    pub row_id: String,
    pub official_setting: String,
    pub category: String,
    pub value_kind: ScalarWriteValueKind,
    pub expected_classification: SafeBatchEligibility,
    pub value_pair: Option<HarnessValuePair>,
}

pub fn report_path(name: &str) -> PathBuf {
    PathBuf::from("data/reports").join(name)
}

pub fn write_report(name: &str, value: &Value) {
    let path = report_path(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("report parent should be created");
    }
    fs::write(
        &path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(value).expect("report should serialize")
        ),
    )
    .expect("report should be written");
}

pub fn temp_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should work")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-safe-batch-deep-{label}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp root should be created");
    root
}

pub fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture parent should be created");
    }
    fs::write(path, contents).expect("fixture should be written");
}

pub fn known_settings() -> BTreeSet<String> {
    SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id.to_string())
        .collect()
}

pub fn graph_file(path: &Path, hints: Vec<ConfigManagementHintKind>) -> ConfigGraphFile {
    ConfigGraphFile {
        path: path.to_path_buf(),
        resolved_path: fs::canonicalize(path).ok(),
        source_depth: 0,
        readable: true,
        is_symlink: hints.contains(&ConfigManagementHintKind::SymlinkManaged),
        symlink_target: None,
        hints: hints
            .into_iter()
            .map(|kind| ConfigManagementHint {
                kind,
                confidence: ConfigDetectionConfidence::Confirmed,
                evidence: "safe-batch deep harness fixture metadata".to_string(),
            })
            .collect(),
    }
}

pub fn graph_for(files: Vec<ConfigGraphFile>, root_path: PathBuf) -> ConfigGraphSummary {
    ConfigGraphSummary {
        root_path,
        connected_file_count: files.len(),
        unreadable_file_count: 0,
        multi_file: files.len() > 1,
        has_profile_hints: false,
        has_mode_hints: false,
        has_theme_hints: false,
        has_generated_hints: files.iter().any(|file| {
            file.hints
                .iter()
                .any(|hint| hint.kind == ConfigManagementHintKind::GeneratedFile)
        }),
        has_script_managed_hints: files.iter().any(|file| {
            file.hints.iter().any(|hint| {
                matches!(
                    hint.kind,
                    ConfigManagementHintKind::ScriptManaged
                        | ConfigManagementHintKind::ScriptReferenced
                        | ConfigManagementHintKind::SymlinkManaged
                )
            })
        }),
        files,
        source_references: Vec::new(),
        unreadable_files: Vec::new(),
        cycles: Vec::new(),
        unsupported_sources: Vec::new(),
    }
}

pub fn redact_path(path: &Path) -> String {
    let text = path.display().to_string();
    redact_text(&text)
}

pub fn redact_text(text: &str) -> String {
    text.replace("/home/kyo/.config/hypr", "<hypr-config>")
        .replace("/home/kyo", "~")
}

pub fn snapshot(
    values: Vec<(&str, &str, &Path, usize, &str, CurrentValueStatus)>,
) -> CurrentConfigSnapshot {
    let mut map = BTreeMap::new();
    for (official_setting, raw_value, source_path, line_number, raw_line, status) in values {
        map.insert(
            official_setting.to_string(),
            CurrentValue {
                setting_id: official_setting.to_string(),
                raw_value: raw_value.to_string(),
                source_path: source_path.to_path_buf(),
                line_number,
                raw_line: raw_line.to_string(),
                duplicate_lines: matches!(status, CurrentValueStatus::DuplicateConflict)
                    .then_some(vec![line_number, line_number + 1])
                    .unwrap_or_default(),
                status,
                warning: None,
            },
        );
    }
    CurrentConfigSnapshot {
        status: CurrentConfigLoadStatus::Loaded {
            path: PathBuf::from("/tmp/safe-batch-deep-fixture.conf"),
            scalar_count: map.len(),
            structured_count: 0,
            unsupported_count: 0,
        },
        values: map,
        structured_records: Vec::new(),
        unsupported_records: Vec::new(),
    }
}

pub fn plan_for(
    batch_id: &str,
    current_config: &CurrentConfigSnapshot,
    graph: &ConfigGraphSummary,
    changes: Vec<SafeBatchChangeRequest>,
) -> SafeBatchWritePlan {
    build_safe_batch_write_plan(
        batch_id,
        &known_settings(),
        current_config,
        graph,
        changes,
        "deep-harness",
    )
}

pub fn setting_category(row_id: &str) -> String {
    row_id.split('.').next().unwrap_or("unknown").to_string()
}

pub fn display_render_risky_for_harness(row_id: &str) -> bool {
    high_risk_write_policy(row_id)
        .map(|policy| policy.recovery_bucket.contains("display-render"))
        .unwrap_or(false)
        || matches!(
            row_id.split('.').next(),
            Some("render" | "xwayland" | "opengl" | "experimental" | "quirks")
        )
        || row_id == "decoration.screen_shader"
}

pub fn expected_classification_for_row(row_id: &str) -> SafeBatchEligibility {
    if display_render_risky_for_harness(row_id) {
        return SafeBatchEligibility::BlockedDisplayRenderRisk;
    }
    if high_risk_write_policy(row_id).is_some() {
        return SafeBatchEligibility::BlockedHighRisk;
    }
    if row_id.contains("profile") || row_id.contains("mode_switch") {
        return SafeBatchEligibility::BlockedProfileModeSwitch;
    }
    if row_id.starts_with("runtime.") {
        return SafeBatchEligibility::BlockedRuntimeOnly;
    }
    SafeBatchEligibility::EligibleSafeBatchScalar
}

pub fn value_pair_for(row_id: &str, value_kind: ScalarWriteValueKind) -> Option<HarnessValuePair> {
    let pair = match value_kind {
        ScalarWriteValueKind::Boolean => ("true", "false", Some("maybe")),
        ScalarWriteValueKind::FiniteChoice => {
            let options = finite_choice_options(row_id)?;
            let first = options.first()?.raw_value;
            let second = options
                .iter()
                .find(|option| option.raw_value != first)
                .map(|option| option.raw_value)
                .unwrap_or(first);
            return Some(HarnessValuePair {
                old_value: first.to_string(),
                proposed_value: second.to_string(),
                invalid_value: Some("__invalid_choice__".to_string()),
                coverage_status: "tested",
            });
        }
        ScalarWriteValueKind::SourceBacked => {
            let rules = read_system_xkb_rules().ok()?;
            let values = match row_id {
                "input.kb_model" => rules.models,
                "input.kb_layout" => rules.layouts,
                "input.kb_variant" => rules.variants,
                "input.kb_options" => rules.options,
                "input.kb_rules" => {
                    return Some(HarnessValuePair {
                        old_value: "evdev".to_string(),
                        proposed_value: "base".to_string(),
                        invalid_value: Some("__invalid_rules__".to_string()),
                        coverage_status: "tested",
                    });
                }
                _ => return None,
            };
            let first = values.first()?.raw_value.clone();
            let second = values
                .iter()
                .find(|value| value.raw_value != first)
                .map(|value| value.raw_value.clone())
                .unwrap_or_else(|| first.clone());
            return Some(HarnessValuePair {
                old_value: first,
                proposed_value: second,
                invalid_value: Some("__invalid_xkb__".to_string()),
                coverage_status: "tested",
            });
        }
        ScalarWriteValueKind::MonitorName if row_id == "cursor.default_monitor" => {
            ("HDMI-A-1", "DP-1", Some("bad;monitor"))
        }
        ScalarWriteValueKind::MonitorName => ("DP-1", "", Some("bad;monitor")),
        ScalarWriteValueKind::Number => return numeric_pair(row_id, false),
        ScalarWriteValueKind::Percent => return numeric_pair(row_id, true),
        ScalarWriteValueKind::Color => ("rgba(ffffffff)", "rgba(000000ff)", Some("not-a-color")),
        ScalarWriteValueKind::Gradient => (
            "rgba(ffffffff) rgba(000000ff) 45deg",
            "rgba(000000ff) rgba(ffffffff) 90deg",
            Some("not-a-gradient"),
        ),
        ScalarWriteValueKind::Vector2 => ("0 0", "1 1", Some("1 nope")),
        ScalarWriteValueKind::NumericList => {
            ("0.2 0.0 0.5 1 1.2 1.5", "0.1 0.2 0.3", Some("bad list"))
        }
        ScalarWriteValueKind::CssGap => ("5", "6", Some("-1")),
        ScalarWriteValueKind::AccelProfile => ("flat", "adaptive", Some("bad#profile")),
        ScalarWriteValueKind::CommaSeparatedFloatList => {
            ("0.333,0.5", "0.25,0.75", Some("0.0, 2.0"))
        }
        ScalarWriteValueKind::LineSafeString => ("Sans", "Serif", Some("bad\nstring")),
        ScalarWriteValueKind::Path => (
            "~/.config/hypr/a.conf",
            "~/.config/hypr/b.conf",
            Some("bad\npath"),
        ),
        ScalarWriteValueKind::RegexString => ("^(kitty)$", "^(Alacritty)$", Some("bad\nregex")),
        ScalarWriteValueKind::StringLike
        | ScalarWriteValueKind::ComplexRaw
        | ScalarWriteValueKind::Unknown => return None,
    };

    Some(HarnessValuePair {
        old_value: pair.0.to_string(),
        proposed_value: pair.1.to_string(),
        invalid_value: pair.2.map(ToString::to_string),
        coverage_status: "tested",
    })
}

fn numeric_pair(row_id: &str, percent: bool) -> Option<HarnessValuePair> {
    if let Some(bounds) = source_backed_numeric_bounds(row_id) {
        let (old_value, proposed_value) = match bounds.value_type {
            SourceBackedNumericType::Integer => {
                let old = bounds.min.ceil().max(bounds.min) as i64;
                let proposed = ((old + 1) as f64).min(bounds.max).max(bounds.min) as i64;
                (old.to_string(), proposed.to_string())
            }
            SourceBackedNumericType::Float => {
                let old = bounds.min;
                let proposed = if bounds.max > bounds.min {
                    (bounds.min + ((bounds.max - bounds.min) / 2.0)).min(bounds.max)
                } else {
                    bounds.min
                };
                (format!("{old:.2}"), format!("{proposed:.2}"))
            }
        };
        return Some(HarnessValuePair {
            old_value,
            proposed_value,
            invalid_value: Some("not-a-number".to_string()),
            coverage_status: "tested",
        });
    }

    let (old_value, proposed_value, invalid_value) = if percent {
        ("0.25", "0.75", "2.0")
    } else {
        ("1", "2", "-1")
    };
    Some(HarnessValuePair {
        old_value: old_value.to_string(),
        proposed_value: proposed_value.to_string(),
        invalid_value: Some(invalid_value.to_string()),
        coverage_status: "tested",
    })
}

pub fn harness_rows() -> Vec<HarnessRowCase> {
    SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| HarnessRowCase {
            row_id: row.row_id.to_string(),
            official_setting: row.official_setting.to_string(),
            category: setting_category(row.row_id),
            value_kind: row.value_kind,
            expected_classification: expected_classification_for_row(row.row_id),
            value_pair: value_pair_for(row.row_id, row.value_kind),
        })
        .collect()
}

pub fn validate_generated_pair(row: &HarnessRowCase) -> Option<(bool, bool)> {
    let pair = row.value_pair.as_ref()?;
    let current = CurrentValueProjection {
        status: CurrentValueSourceStatus::Configured,
        raw_value: Some(pair.old_value.clone()),
        source_path: Some(PathBuf::from("/tmp/deep-harness.conf")),
        line_number: Some(1),
        raw_line: Some(format!(
            "{} = {}",
            config_key_from_official_setting(&row.official_setting),
            pair.old_value
        )),
        duplicate_lines: Vec::new(),
        warning: None,
    };
    let sources = PendingChangeValueSources {
        monitor_names: vec!["DP-1".to_string()],
    };
    let valid = stage_pending_change_with_sources(
        &row.row_id,
        &current,
        pair.proposed_value.clone(),
        &sources,
    )
    .validation
        == PendingChangeValidation::Valid;
    let invalid_rejected = pair.invalid_value.as_ref().is_none_or(|invalid| {
        stage_pending_change_with_sources(&row.row_id, &current, invalid.clone(), &sources)
            .validation
            != PendingChangeValidation::Valid
    });
    Some((valid, invalid_rejected))
}

pub fn build_fixture_for_rows(
    rows: &[HarnessRowCase],
    label: &str,
) -> (
    PathBuf,
    CurrentConfigSnapshot,
    ConfigGraphSummary,
    Vec<SafeBatchChangeRequest>,
    Vec<HarnessRowCase>,
) {
    let root = temp_root(label);
    let mut files = (0..4)
        .map(|index| root.join(format!("safe-batch-{index}.conf")))
        .collect::<Vec<_>>();
    if files.is_empty() {
        files.push(root.join("safe-batch.conf"));
    }

    let mut contents = vec![String::new(); files.len()];
    let mut snapshot_rows = Vec::new();
    let mut changes = Vec::new();
    let mut executed_rows = Vec::new();

    for row in rows.iter().filter(|row| {
        row.expected_classification == SafeBatchEligibility::EligibleSafeBatchScalar
            && row.value_pair.is_some()
            && validate_generated_pair(row)
                .map(|(valid, _)| valid)
                .unwrap_or(false)
    }) {
        let pair = row.value_pair.as_ref().expect("pair checked");
        let file_index = executed_rows.len() % files.len();
        let line_number = contents[file_index].lines().count() + 1;
        let key = config_key_from_official_setting(&row.official_setting);
        let raw_line = format!("{key} = {}", pair.old_value);
        contents[file_index].push_str(&raw_line);
        contents[file_index].push('\n');
        snapshot_rows.push((
            row.official_setting.clone(),
            pair.old_value.clone(),
            files[file_index].clone(),
            line_number,
            raw_line,
        ));
        changes.push(SafeBatchChangeRequest::new(
            row.row_id.clone(),
            pair.proposed_value.clone(),
        ));
        executed_rows.push(row.clone());
    }

    for (path, content) in files.iter().zip(contents.iter()) {
        write_file(path, content);
    }

    let current = snapshot(
        snapshot_rows
            .iter()
            .map(|(setting, value, path, line, raw)| {
                (
                    setting.as_str(),
                    value.as_str(),
                    path.as_path(),
                    *line,
                    raw.as_str(),
                    CurrentValueStatus::Configured,
                )
            })
            .collect(),
    );
    let graph = graph_for(
        files
            .iter()
            .map(|path| graph_file(path, Vec::new()))
            .collect(),
        files[0].clone(),
    );
    (root, current, graph, changes, executed_rows)
}

pub fn blocked_plan_for(reason: SafeBatchEligibility) -> (PathBuf, SafeBatchWritePlan) {
    let root = temp_root(reason.label());
    let config = root.join("hyprland.conf");
    let managed = root.join("managed.conf");
    let raw = "decoration:blur:enabled = true";
    write_file(&config, &format!("{raw}\n"));
    write_file(&managed, "decoration:shadow:enabled = false\n");

    let mut values = vec![(
        "decoration.blur.enabled",
        "true",
        config.as_path(),
        1usize,
        raw,
        CurrentValueStatus::Configured,
    )];
    let mut files = vec![graph_file(&config, Vec::new())];
    let request = match reason {
        SafeBatchEligibility::BlockedHighRisk => {
            SafeBatchChangeRequest::new("cursor.invisible", "1")
        }
        SafeBatchEligibility::BlockedDisplayRenderRisk => {
            SafeBatchChangeRequest::new("render.direct_scanout", "1")
        }
        SafeBatchEligibility::BlockedGeneratedFile => {
            values.push((
                "decoration.shadow.enabled",
                "false",
                managed.as_path(),
                1,
                "decoration:shadow:enabled = false",
                CurrentValueStatus::Configured,
            ));
            files.push(graph_file(
                &managed,
                vec![ConfigManagementHintKind::GeneratedFile],
            ));
            SafeBatchChangeRequest::new("appearance.shadow.enabled", "true")
        }
        SafeBatchEligibility::BlockedScriptManaged => {
            values.push((
                "decoration.shadow.enabled",
                "false",
                managed.as_path(),
                1,
                "decoration:shadow:enabled = false",
                CurrentValueStatus::Configured,
            ));
            files.push(graph_file(
                &managed,
                vec![ConfigManagementHintKind::ScriptManaged],
            ));
            SafeBatchChangeRequest::new("appearance.shadow.enabled", "true")
        }
        SafeBatchEligibility::BlockedSymlinkManaged => {
            values.push((
                "decoration.shadow.enabled",
                "false",
                managed.as_path(),
                1,
                "decoration:shadow:enabled = false",
                CurrentValueStatus::Configured,
            ));
            files.push(graph_file(
                &managed,
                vec![ConfigManagementHintKind::SymlinkManaged],
            ));
            SafeBatchChangeRequest::new("appearance.shadow.enabled", "true")
        }
        SafeBatchEligibility::BlockedAmbiguousFile => {
            files.clear();
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false")
        }
        SafeBatchEligibility::BlockedDuplicateConflict => {
            values[0].5 = CurrentValueStatus::DuplicateConflict;
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false")
        }
        SafeBatchEligibility::BlockedMissingLine => {
            let current = CurrentConfigSnapshot {
                status: CurrentConfigLoadStatus::Loaded {
                    path: config.clone(),
                    scalar_count: 0,
                    structured_count: 0,
                    unsupported_count: 0,
                },
                values: BTreeMap::new(),
                structured_records: Vec::new(),
                unsupported_records: Vec::new(),
            };
            let graph = graph_for(files, config.clone());
            let plan = plan_for(
                "blocked-missing-line",
                &current,
                &graph,
                vec![SafeBatchChangeRequest::new(
                    "appearance.blur.enabled",
                    "false",
                )],
            );
            return (root, plan);
        }
        SafeBatchEligibility::BlockedStructuredFamily => {
            SafeBatchChangeRequest::new("hl.monitor", "DP-1,preferred,auto,1")
        }
        SafeBatchEligibility::BlockedUnknownTarget => {
            SafeBatchChangeRequest::new("unknown.safe_batch.setting", "true")
        }
        SafeBatchEligibility::BlockedRuntimeOnly => {
            SafeBatchChangeRequest::new("runtime.reload", "true")
        }
        SafeBatchEligibility::BlockedProfileModeSwitch => {
            SafeBatchChangeRequest::new("profile.mode_switch", "true")
        }
        SafeBatchEligibility::EligibleSafeBatchScalar => {
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false")
        }
    };

    let current = snapshot(values);
    let graph = graph_for(files, config.clone());
    let plan = plan_for(
        &format!("blocked-{}", reason.label()),
        &current,
        &graph,
        vec![request],
    );
    (root, plan)
}

fn graph_scalar_occurrences(graph: &ConfigGraphSummary) -> BTreeMap<String, Vec<Value>> {
    let mut occurrences = BTreeMap::<String, Vec<Value>>::new();
    for file in &graph.files {
        if !file.readable {
            continue;
        }
        let Ok(parsed) = parse_hyprland_config_file(&file.path) else {
            continue;
        };
        for record in parsed.scalar_records() {
            let Some(setting_id) = &record.normalized_setting_id else {
                continue;
            };
            occurrences
                .entry(setting_id.clone())
                .or_default()
                .push(json!({
                    "path": redact_path(&record.path),
                    "lineNumber": record.line_number,
                    "rawValue": record.raw_value,
                    "rawLine": "<redacted; raw source line omitted from committed report>"
                }));
        }
    }
    occurrences
}

pub fn real_config_readonly_audit() -> Value {
    let discovery = discover_hyprland_config();
    let ConfigDiscoveryStatus::Found { path, .. } = &discovery.status else {
        return json!({
            "performed": false,
            "reason": discovery.summary(),
            "realUserConfigEdited": false,
            "realBackupsCreated": false
        });
    };
    let graph = inspect_config_graph(path);
    let current = current_config_from_graph(&graph);
    let source_aware_report = source_aware_mapping_report(&graph, &current);
    let graph_occurrences = graph_scalar_occurrences(&graph);
    let rows = harness_rows();
    let mut eligible = 0usize;
    let mut blocked = BTreeMap::<String, usize>::new();
    let mut duplicate_conflicts = Vec::<String>::new();
    let mut duplicate_conflict_details = Vec::new();
    let mut blocker_details = Vec::new();
    let mut missing_line_subtypes = BTreeMap::<String, usize>::new();
    let mut configured_but_unmapped = Vec::new();
    let mut truly_not_configured = Vec::new();
    let mut newly_eligible_rows = Vec::<Value>::new();
    let mut managed_hints = Vec::new();

    for row in rows.iter().filter(|row| row.value_pair.is_some()) {
        let Some(pair) = &row.value_pair else {
            continue;
        };
        let plan = plan_for(
            "real-config-readonly-audit",
            &current,
            &graph,
            vec![SafeBatchChangeRequest::new(
                row.row_id.clone(),
                pair.proposed_value.clone(),
            )],
        );
        if plan.can_execute {
            eligible += 1;
            if let Some(change) = plan.eligible_changes.first() {
                newly_eligible_rows.push(json!({
                    "rowId": row.row_id,
                    "officialSetting": row.official_setting,
                    "targetPath": redact_path(&change.target_path),
                    "lineNumber": change.line_number,
                    "oldValueMatched": true,
                    "rawLine": "<redacted; raw source line omitted from committed report>",
                    "normalRisk": true,
                    "generated": false,
                    "scriptManaged": false,
                    "symlinkManaged": false,
                    "duplicateConflicted": false
                }));
            }
        } else if let Some(blocked_change) = plan.blocked_changes.first() {
            *blocked
                .entry(blocked_change.reason.label().to_string())
                .or_default() += 1;
            let current_value = current.value_for(&row.official_setting);
            let graph_hits = graph_occurrences
                .get(&row.official_setting)
                .cloned()
                .unwrap_or_default();
            let mut subtype = "not_applicable";
            if blocked_change.reason == SafeBatchEligibility::BlockedDuplicateConflict {
                duplicate_conflicts.push(row.row_id.clone());
                duplicate_conflict_details.push(json!({
                    "rowId": row.row_id,
                    "officialSetting": row.official_setting,
                    "occurrences": graph_hits.clone(),
                    "currentActiveValue": current_value.raw_value,
                    "whyApplyIsBlocked": "The same setting appears more than once, so safe-batch writing will not silently choose one target.",
                    "manualResolution": "Remove or consolidate the duplicate entries manually, then rerun the read-only audit before applying."
                }));
                subtype = "duplicate_conflict";
            } else if blocked_change.reason == SafeBatchEligibility::BlockedMissingLine {
                if graph_hits.is_empty() {
                    subtype = "not_configured_default_value";
                    truly_not_configured.push(row.row_id.clone());
                } else {
                    subtype = "configured_in_graph_but_not_mapped_to_safe_current_source";
                    configured_but_unmapped.push(json!({
                        "rowId": row.row_id,
                        "officialSetting": row.official_setting,
                        "occurrences": graph_hits.clone(),
                        "whyStillBlocked": "The connected graph has a scalar occurrence, but the current safe write source snapshot did not expose an exact eligible target for this row.",
                        "safety": "This remains blocked until source/include-aware current value mapping is proven for Apply."
                    }));
                }
                *missing_line_subtypes
                    .entry(subtype.to_string())
                    .or_default() += 1;
            }
            blocker_details.push(json!({
                "rowId": row.row_id,
                "officialSetting": row.official_setting,
                "blockedReason": blocked_change.reason.label(),
                "blockerSubtype": subtype,
                "evidence": redact_text(&blocked_change.evidence),
                "userFacingCopy": blocked_change.user_facing_copy,
                "currentStatus": current_value.status_label(),
                "sourcePath": current_value.source_path.as_deref().map(redact_path),
                "lineNumber": current_value.line_number,
                "graphOccurrenceCount": graph_hits.len()
            }));
        } else {
            *blocked.entry("not_executable".to_string()).or_default() += 1;
        }
    }

    for file in &graph.files {
        for hint in &file.hints {
            if matches!(
                hint.kind,
                ConfigManagementHintKind::GeneratedFile
                    | ConfigManagementHintKind::ScriptManaged
                    | ConfigManagementHintKind::ScriptReferenced
                    | ConfigManagementHintKind::SymlinkManaged
            ) {
                managed_hints.push(json!({
                    "path": redact_path(&file.path),
                    "hint": format!("{:?}", hint.kind),
                    "evidence": "<redacted; local script/profile evidence omitted from committed report>",
                    "redactionReason": "local script paths and personal config layout are not needed for public proof"
                }));
            }
        }
    }

    json!({
        "performed": true,
        "rootPath": redact_path(path),
        "sourceAwareMapping": {
            "rootFileMapped": source_aware_report.root_file_mapped,
            "readableFilesMapped": source_aware_report.readable_files_mapped,
            "unreadableFiles": source_aware_report.unreadable_files,
            "scalarCount": source_aware_report.scalar_count,
            "structuredCount": source_aware_report.structured_count,
            "unsupportedCount": source_aware_report.unsupported_count,
            "duplicateSettingCount": source_aware_report.duplicate_setting_count
        },
        "settingsWithGeneratedProposedValuesConsidered": eligible + blocked.values().sum::<usize>(),
        "eligibleSafeBatchWrites": eligible,
        "eligibleRowsWithExactTargets": newly_eligible_rows,
        "blocked": blocked,
        "blockerDetails": blocker_details,
        "missingLineSubtypes": missing_line_subtypes,
        "configuredButNotMappedToSafeCurrentSource": configured_but_unmapped,
        "trulyNotConfiguredDefaultRows": truly_not_configured,
        "duplicateConflicts": duplicate_conflicts,
        "duplicateConflictDetails": duplicate_conflict_details,
        "managedHints": managed_hints,
        "appearanceBlurEnabledDuplicateBlocked": duplicate_conflicts.iter().any(|row| row == "appearance.blur.enabled"),
        "privacy": {
            "pathsRedacted": true,
            "rawSourceLinesRedacted": true,
            "localScriptEvidenceRedacted": true
        },
        "realUserConfigEdited": false,
        "realBackupsCreated": false,
        "productionVerificationRun": false,
        "productionRecoveryRun": false
    })
}

pub fn summarize_rows(rows: &[HarnessRowCase]) -> Value {
    let mut classifications = BTreeMap::<String, usize>::new();
    let mut value_coverage = BTreeMap::<String, usize>::new();
    for row in rows {
        *classifications
            .entry(row.expected_classification.label().to_string())
            .or_default() += 1;
        let key = if row.value_pair.is_some() {
            "tested"
        } else {
            "not_tested_value_generation_missing_fixture"
        };
        *value_coverage.entry(key.to_string()).or_default() += 1;
    }
    json!({
        "total": rows.len(),
        "classifications": classifications,
        "valueGeneration": value_coverage
    })
}

pub fn reportable_row(row: &HarnessRowCase) -> Value {
    let (valid_generated, invalid_rejected) =
        validate_generated_pair(row).unwrap_or((false, false));
    json!({
        "rowId": row.row_id,
        "officialSetting": row.official_setting,
        "category": row.category,
        "valueFamily": format!("{:?}", row.value_kind),
        "riskStatus": if high_risk_write_policy(&row.row_id).is_some() { "high-risk policy present" } else { "normal policy" },
        "highRiskPolicyStatus": high_risk_write_policy(&row.row_id).map(|policy| policy.approval_gate).unwrap_or("none"),
        "safeBatchExpectedClassification": row.expected_classification.label(),
        "fixtureTargetCanBeGenerated": row.value_pair.is_some(),
        "validProposedValueCanBeGenerated": row.value_pair.is_some() && valid_generated,
        "invalidProposedValueCanBeGenerated": row.value_pair.as_ref().and_then(|pair| pair.invalid_value.as_ref()).is_some(),
        "invalidProposedValueRejected": invalid_rejected,
        "valueGenerationStatus": if row.value_pair.is_some() { "tested" } else { "not_tested_value_generation_missing_fixture" }
    })
}

pub fn source_contains_safe_batch_ui_copy() -> bool {
    let window = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let safe_batch =
        fs::read_to_string("src/safe_batch_write.rs").expect("safe batch source should read");
    let readiness = fs::read_to_string("src/write_enablement_readiness.rs")
        .expect("readiness source should read");
    let combined = format!("{window}\n{safe_batch}\n{readiness}");
    [
        "Safe batch write is available for normal settings.",
        "The app will back up files before writing.",
        "The app will check the result after writing.",
        "If something fails, the app will restore the backup.",
        "Blocked: this setting needs a family-specific recovery path before the app can write it.",
        "Blocked: this setting appears in more than one place.",
    ]
    .iter()
    .all(|copy| combined.contains(copy))
}

pub fn assert_no_real_config_path(path: &Path) {
    assert!(
        !path.starts_with("/home/kyo/.config/hypr"),
        "fixture path must not be real user config: {}",
        path.display()
    );
}

pub fn execution_report_json(report: &SafeBatchWriteReport) -> Value {
    json!({
        "status": format!("{:?}", report.status),
        "backupCount": report.backups.len(),
        "verifiedChangeCount": report.verified_changes.len(),
        "failures": report.failures,
        "recoveryAttempted": report.recovery_attempted,
        "recoverySucceeded": report.recovery_succeeded,
        "restoreVerificationSucceeded": report.restore_verification_succeeded,
        "hyprlandReloadAttempted": report.hyprland_reload_attempted,
        "mutatingHyprctlUsed": report.mutating_hyprctl_used,
        "runtimeMutated": report.runtime_mutated
    })
}
