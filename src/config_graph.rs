use std::collections::{BTreeSet, VecDeque};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_MAX_DEPTH: usize = 16;
const SCRIPT_SCAN_MAX_FILES: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigGraphOptions {
    pub home_dir: Option<PathBuf>,
    pub script_dirs: Vec<PathBuf>,
    pub max_depth: usize,
}

impl ConfigGraphOptions {
    pub fn from_env() -> Self {
        let home_dir = env::var_os("HOME").map(PathBuf::from);
        let script_dirs = home_dir
            .as_ref()
            .map(|home| vec![home.join(".config/hypr/scripts")])
            .unwrap_or_default();

        Self {
            home_dir,
            script_dirs,
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigGraphSummary {
    pub root_path: PathBuf,
    pub files: Vec<ConfigGraphFile>,
    pub source_references: Vec<ConfigSourceReference>,
    pub unreadable_files: Vec<ConfigGraphIssue>,
    pub cycles: Vec<ConfigGraphIssue>,
    pub unsupported_sources: Vec<ConfigGraphIssue>,
    pub connected_file_count: usize,
    pub unreadable_file_count: usize,
    pub multi_file: bool,
    pub has_profile_hints: bool,
    pub has_mode_hints: bool,
    pub has_theme_hints: bool,
    pub has_generated_hints: bool,
    pub has_script_managed_hints: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigGraphFile {
    pub path: PathBuf,
    pub resolved_path: Option<PathBuf>,
    pub source_depth: usize,
    pub readable: bool,
    pub is_symlink: bool,
    pub symlink_target: Option<PathBuf>,
    pub hints: Vec<ConfigManagementHint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSourceReference {
    pub source_file: PathBuf,
    pub line_number: usize,
    pub raw_line: String,
    pub raw_target: String,
    pub resolved_target: Option<PathBuf>,
    pub kind: ConfigSourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSourceKind {
    Source,
    Include,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigManagementHint {
    pub kind: ConfigManagementHintKind,
    pub confidence: ConfigDetectionConfidence,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigManagementHintKind {
    CurrentProfile,
    DesktopProfile,
    GamingProfile,
    LaptopProfile,
    PerformanceProfile,
    ModeProfile,
    ThemeProfile,
    HostProfile,
    GeneratedFile,
    ScriptReferenced,
    ScriptManaged,
    SymlinkManaged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigDetectionConfidence {
    Confirmed,
    Likely,
    Possible,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigGraphIssue {
    pub path: PathBuf,
    pub line_number: Option<usize>,
    pub raw_line: Option<String>,
    pub message: String,
}

pub fn inspect_config_graph(root: impl AsRef<Path>) -> ConfigGraphSummary {
    inspect_config_graph_with_options(root, ConfigGraphOptions::from_env())
}

pub fn inspect_config_graph_with_options(
    root: impl AsRef<Path>,
    options: ConfigGraphOptions,
) -> ConfigGraphSummary {
    let root_path = root.as_ref().to_path_buf();
    let mut builder = ConfigGraphBuilder::new(root_path.clone(), options);
    builder.visit(root.as_ref(), 0);
    builder.apply_script_hints();
    builder.finish()
}

struct ConfigGraphBuilder {
    summary: ConfigGraphSummary,
    options: ConfigGraphOptions,
    visited: BTreeSet<PathBuf>,
    active: BTreeSet<PathBuf>,
}

impl ConfigGraphBuilder {
    fn new(root_path: PathBuf, options: ConfigGraphOptions) -> Self {
        Self {
            summary: ConfigGraphSummary {
                root_path,
                files: Vec::new(),
                source_references: Vec::new(),
                unreadable_files: Vec::new(),
                cycles: Vec::new(),
                unsupported_sources: Vec::new(),
                connected_file_count: 0,
                unreadable_file_count: 0,
                multi_file: false,
                has_profile_hints: false,
                has_mode_hints: false,
                has_theme_hints: false,
                has_generated_hints: false,
                has_script_managed_hints: false,
            },
            options,
            visited: BTreeSet::new(),
            active: BTreeSet::new(),
        }
    }

    fn visit(&mut self, path: &Path, depth: usize) {
        let path = normalize_without_fs(path);
        if depth > self.options.max_depth {
            self.summary.cycles.push(ConfigGraphIssue {
                path,
                line_number: None,
                raw_line: None,
                message: "source/include depth limit reached".to_string(),
            });
            return;
        }

        let key = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if self.active.contains(&key) {
            self.summary.cycles.push(ConfigGraphIssue {
                path,
                line_number: None,
                raw_line: None,
                message: "source/include cycle detected".to_string(),
            });
            return;
        }
        if self.visited.contains(&key) {
            return;
        }

        self.active.insert(key.clone());
        let symlink = symlink_target(&path);
        let is_symlink = symlink.is_some();
        let resolved_path = fs::canonicalize(&path).ok();

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) => {
                self.summary.files.push(ConfigGraphFile {
                    path: path.clone(),
                    resolved_path,
                    source_depth: depth,
                    readable: false,
                    is_symlink,
                    symlink_target: symlink,
                    hints: path_hints(&path, is_symlink),
                });
                self.summary.unreadable_files.push(ConfigGraphIssue {
                    path: path.clone(),
                    line_number: None,
                    raw_line: None,
                    message: format!("could not read connected config file: {error}"),
                });
                self.active.remove(&key);
                self.visited.insert(key);
                return;
            }
        };

        let mut hints = path_hints(&path, is_symlink);
        hints.extend(content_hints(&content));

        self.summary.files.push(ConfigGraphFile {
            path: path.clone(),
            resolved_path,
            source_depth: depth,
            readable: true,
            is_symlink,
            symlink_target: symlink,
            hints,
        });

        for source in parse_source_references(&path, &content, &self.options.home_dir) {
            match source {
                ParsedSource::Reference(reference) => {
                    let target = reference.resolved_target.clone();
                    self.summary.source_references.push(reference);
                    if let Some(target) = target {
                        self.visit(&target, depth + 1);
                    }
                }
                ParsedSource::Unsupported(issue) => {
                    self.summary.unsupported_sources.push(issue);
                }
            }
        }

        self.active.remove(&key);
        self.visited.insert(key);
    }

    fn apply_script_hints(&mut self) {
        let scripts = collect_script_text(&self.options.script_dirs);
        if scripts.is_empty() {
            return;
        }

        for file in &mut self.summary.files {
            let names = script_reference_needles(file);
            let mut referenced_by = Vec::new();
            let mut managed_by = Vec::new();

            for script in &scripts {
                if names.iter().any(|needle| script.content.contains(needle)) {
                    referenced_by.push(script.path.display().to_string());
                    if script_has_mutation_pattern(&script.content) {
                        managed_by.push(script.path.display().to_string());
                    }
                }
            }

            if !referenced_by.is_empty() {
                push_unique_hint(
                    &mut file.hints,
                    ConfigManagementHintKind::ScriptReferenced,
                    ConfigDetectionConfidence::Possible,
                    format!("referenced by {}", referenced_by.join(", ")),
                );
            }

            if !managed_by.is_empty() {
                push_unique_hint(
                    &mut file.hints,
                    ConfigManagementHintKind::ScriptManaged,
                    ConfigDetectionConfidence::Likely,
                    format!(
                        "referenced by script text with mutation patterns: {}",
                        managed_by.join(", ")
                    ),
                );
            }
        }
    }

    fn finish(mut self) -> ConfigGraphSummary {
        self.summary.connected_file_count = self.summary.files.len();
        self.summary.unreadable_file_count = self.summary.unreadable_files.len();
        self.summary.multi_file = self.summary.connected_file_count > 1;

        for file in &self.summary.files {
            for hint in &file.hints {
                match hint.kind {
                    ConfigManagementHintKind::CurrentProfile
                    | ConfigManagementHintKind::DesktopProfile
                    | ConfigManagementHintKind::GamingProfile
                    | ConfigManagementHintKind::LaptopProfile
                    | ConfigManagementHintKind::PerformanceProfile => {
                        self.summary.has_profile_hints = true;
                    }
                    ConfigManagementHintKind::ModeProfile => {
                        self.summary.has_mode_hints = true;
                    }
                    ConfigManagementHintKind::ThemeProfile => {
                        self.summary.has_theme_hints = true;
                    }
                    ConfigManagementHintKind::GeneratedFile => {
                        self.summary.has_generated_hints = true;
                    }
                    ConfigManagementHintKind::ScriptManaged
                    | ConfigManagementHintKind::ScriptReferenced
                    | ConfigManagementHintKind::SymlinkManaged => {
                        self.summary.has_script_managed_hints = true;
                    }
                    ConfigManagementHintKind::HostProfile => {}
                }
            }
        }

        self.summary
    }
}

enum ParsedSource {
    Reference(ConfigSourceReference),
    Unsupported(ConfigGraphIssue),
}

fn parse_source_references(
    file_path: &Path,
    content: &str,
    home_dir: &Option<PathBuf>,
) -> Vec<ParsedSource> {
    let mut references = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let Some((kind, raw_target)) = parse_source_line(line) else {
            continue;
        };
        let line_number = line_index + 1;
        if raw_target.contains('*') || raw_target.contains('?') || raw_target.contains('[') {
            references.push(ParsedSource::Unsupported(ConfigGraphIssue {
                path: file_path.to_path_buf(),
                line_number: Some(line_number),
                raw_line: Some(line.to_string()),
                message: "glob source/include paths are not expanded yet".to_string(),
            }));
            continue;
        }

        let resolved_target = resolve_source_target(file_path, &raw_target, home_dir);
        references.push(ParsedSource::Reference(ConfigSourceReference {
            source_file: file_path.to_path_buf(),
            line_number,
            raw_line: line.to_string(),
            raw_target,
            resolved_target,
            kind,
        }));
    }
    references
}

fn parse_source_line(line: &str) -> Option<(ConfigSourceKind, String)> {
    let trimmed = strip_inline_comment(line).trim();
    if trimmed.is_empty() {
        return None;
    }

    let (key, value) = trimmed.split_once('=')?;
    let key = key.trim();
    let kind = match key {
        "source" => ConfigSourceKind::Source,
        "include" => ConfigSourceKind::Include,
        _ => return None,
    };
    let value = value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();
    if value.is_empty() {
        return None;
    }
    Some((kind, value))
}

fn strip_inline_comment(line: &str) -> &str {
    line.split_once('#')
        .map(|(before, _)| before)
        .unwrap_or(line)
}

fn resolve_source_target(
    file_path: &Path,
    raw_target: &str,
    home_dir: &Option<PathBuf>,
) -> Option<PathBuf> {
    if raw_target.starts_with("$(") || raw_target.contains('`') {
        return None;
    }

    let expanded = if raw_target == "~" {
        home_dir.clone()?
    } else if let Some(rest) = raw_target.strip_prefix("~/") {
        home_dir.as_ref()?.join(rest)
    } else {
        PathBuf::from(raw_target)
    };

    if expanded.is_absolute() {
        Some(normalize_without_fs(&expanded))
    } else {
        let parent = file_path.parent().unwrap_or_else(|| Path::new("."));
        Some(normalize_without_fs(&parent.join(expanded)))
    }
}

fn symlink_target(path: &Path) -> Option<PathBuf> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.file_type().is_symlink() {
        return None;
    }
    let target = fs::read_link(path).ok()?;
    if target.is_absolute() {
        Some(target)
    } else {
        path.parent()
            .map(|parent| normalize_without_fs(&parent.join(target)))
    }
}

fn path_hints(path: &Path, is_symlink: bool) -> Vec<ConfigManagementHint> {
    let mut hints = Vec::new();
    let path_text = path.to_string_lossy().to_ascii_lowercase();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if is_symlink {
        push_unique_hint(
            &mut hints,
            ConfigManagementHintKind::SymlinkManaged,
            ConfigDetectionConfidence::Possible,
            "file is a symlink".to_string(),
        );
    }

    if path_text.contains("/modes/") || path_text.contains("/profiles/") {
        push_unique_hint(
            &mut hints,
            ConfigManagementHintKind::ModeProfile,
            ConfigDetectionConfidence::Likely,
            "path is under a mode/profile directory".to_string(),
        );
    }
    if path_text.contains("/themes/") {
        push_unique_hint(
            &mut hints,
            ConfigManagementHintKind::ThemeProfile,
            ConfigDetectionConfidence::Likely,
            "path is under a themes directory".to_string(),
        );
    }
    if path_text.contains("/hosts/") {
        push_unique_hint(
            &mut hints,
            ConfigManagementHintKind::HostProfile,
            ConfigDetectionConfidence::Possible,
            "path is under a hosts directory".to_string(),
        );
    }

    for (needle, kind) in [
        ("current", ConfigManagementHintKind::CurrentProfile),
        ("desktop", ConfigManagementHintKind::DesktopProfile),
        ("gaming", ConfigManagementHintKind::GamingProfile),
        ("laptop", ConfigManagementHintKind::LaptopProfile),
        ("performance", ConfigManagementHintKind::PerformanceProfile),
        ("theme", ConfigManagementHintKind::ThemeProfile),
    ] {
        if file_name.contains(needle) || path_text.contains(&format!("/{needle}/")) {
            push_unique_hint(
                &mut hints,
                kind,
                ConfigDetectionConfidence::Possible,
                format!("path or file name contains {needle}"),
            );
        }
    }

    hints
}

fn content_hints(content: &str) -> Vec<ConfigManagementHint> {
    let mut hints = Vec::new();
    let first_lines = content
        .lines()
        .take(12)
        .collect::<Vec<_>>()
        .join("\n")
        .to_ascii_lowercase();
    if first_lines.contains("generated by")
        || first_lines.contains("do not edit")
        || first_lines.contains("managed by")
    {
        push_unique_hint(
            &mut hints,
            ConfigManagementHintKind::GeneratedFile,
            ConfigDetectionConfidence::Likely,
            "file contains generated/do-not-edit/managed-by marker".to_string(),
        );
    }
    hints
}

fn push_unique_hint(
    hints: &mut Vec<ConfigManagementHint>,
    kind: ConfigManagementHintKind,
    confidence: ConfigDetectionConfidence,
    evidence: String,
) {
    if hints
        .iter()
        .any(|hint| hint.kind == kind && hint.evidence == evidence)
    {
        return;
    }
    hints.push(ConfigManagementHint {
        kind,
        confidence,
        evidence,
    });
}

#[derive(Debug, Clone)]
struct ScriptText {
    path: PathBuf,
    content: String,
}

fn collect_script_text(script_dirs: &[PathBuf]) -> Vec<ScriptText> {
    let mut scripts = Vec::new();
    let mut queue: VecDeque<PathBuf> = script_dirs.iter().cloned().collect();
    let mut scanned = 0;

    while let Some(path) = queue.pop_front() {
        if scanned >= SCRIPT_SCAN_MAX_FILES {
            break;
        }
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if metadata.is_dir() {
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            for entry in entries.flatten() {
                queue.push_back(entry.path());
            }
            continue;
        }
        if !metadata.is_file() {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        scripts.push(ScriptText { path, content });
        scanned += 1;
    }

    scripts
}

fn script_reference_needles(file: &ConfigGraphFile) -> Vec<String> {
    let mut needles = Vec::new();
    needles.push(file.path.display().to_string());
    if let Some(name) = file.path.file_name().and_then(|name| name.to_str()) {
        needles.push(name.to_string());
    }
    if let Some(resolved) = &file.resolved_path {
        needles.push(resolved.display().to_string());
    }
    if let Some(target) = &file.symlink_target {
        needles.push(target.display().to_string());
        if let Some(name) = target.file_name().and_then(|name| name.to_str()) {
            needles.push(name.to_string());
        }
    }
    needles.sort();
    needles.dedup();
    needles
}

fn script_has_mutation_pattern(content: &str) -> bool {
    let lowered = content.to_ascii_lowercase();
    [
        "ln -s",
        "ln -sf",
        "ln -sfn",
        "cp ",
        "mv ",
        "sed -i",
        "perl -pi",
        "tee ",
        "cat >",
        "hyprctl reload",
        "hyprctl keyword",
        "hyprctl dispatch",
    ]
    .iter()
    .any(|pattern| lowered.contains(pattern))
}

fn normalize_without_fs(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
