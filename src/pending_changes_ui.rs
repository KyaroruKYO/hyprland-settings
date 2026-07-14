//! Pending-changes presentation logic: the pure text/diff pieces behind
//! the pending-changes review surface (summary titles, row subtitles,
//! change-kind badges, the read-only next-save rendering, and a unified
//! line diff).
//!
//! Presentation only. Nothing here writes a file or runs a command: the
//! next-save text is computed with the same read-only helpers the real
//! gated save uses, so the preview shows exactly what a save would write
//! without weakening or bypassing any gate.

use crate::scalar_write::preview_scalar_change_text;
use crate::write_classification::config_key_from_official_setting;

/// "1 unsaved change" / "4 unsaved changes".
pub fn pending_summary_title(count: usize) -> String {
    format!(
        "{count} unsaved change{}",
        if count == 1 { "" } else { "s" }
    )
}

/// "3 changes" group caption.
pub fn pending_group_caption(count: usize) -> String {
    format!("{count} change{}", if count == 1 { "" } else { "s" })
}

/// "general:allow_tearing · set to true" row subtitle: the colon-form
/// config key plus the staged value.
pub fn pending_change_subtitle(official_setting: &str, value: &str) -> String {
    format!(
        "{} · set to {}",
        config_key_from_official_setting(official_setting),
        value
    )
}

/// What the next save would do to the config file for this row: update an
/// existing line ("Modified") or append a new one ("Added").
pub fn pending_change_kind(config_has_line: bool) -> &'static str {
    if config_has_line {
        "Modified"
    } else {
        "Added"
    }
}

/// One staged change as the next-save renderer needs it.
pub struct NextSaveChange {
    pub setting_id: String,
    pub proposed_value: String,
    /// 1-based line in the target config when the key already has a
    /// scalar line there (the save replaces it); None appends.
    pub line_in_target: Option<usize>,
}

/// Read-only render of what the next gated save would write: every staged
/// change applied to the current text with the same helpers the real
/// writer uses. Replacements run before appends so original line numbers
/// stay valid throughout.
pub fn next_save_config_text(original: &str, changes: &[NextSaveChange]) -> Result<String, String> {
    let mut text = original.to_string();
    for change in changes.iter().filter(|c| c.line_in_target.is_some()) {
        text = preview_scalar_change_text(
            &text,
            &change.setting_id,
            change.line_in_target,
            &change.proposed_value,
        )
        .map_err(|error| error.to_string())?;
    }
    for change in changes.iter().filter(|c| c.line_in_target.is_none()) {
        text = preview_scalar_change_text(&text, &change.setting_id, None, &change.proposed_value)
            .map_err(|error| error.to_string())?;
    }
    Ok(text)
}

/// One rendered diff line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    /// `---` / `+++` file headers.
    Meta,
    /// `@@ -a,b +c,d @@` hunk headers.
    Hunk,
    Added,
    Removed,
    Context,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ConfigDiffPreview {
    pub lines: Vec<DiffLine>,
    pub added: usize,
    pub removed: usize,
}

impl ConfigDiffPreview {
    pub fn is_empty(&self) -> bool {
        self.added == 0 && self.removed == 0
    }
}

/// Unified line diff (three context lines per hunk) between the saved
/// config text and the next-save text. Small-file LCS; config files are
/// tiny by diff standards, and oversized inputs fall back to a plain
/// full-replacement rendering rather than an expensive computation.
pub fn unified_diff(
    old_text: &str,
    new_text: &str,
    old_label: &str,
    new_label: &str,
) -> ConfigDiffPreview {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let mut lines = Vec::new();
    let mut added = 0;
    let mut removed = 0;

    // Ops over the two line arrays: true = keep (context), else add/remove.
    #[derive(Clone, Copy, PartialEq)]
    enum Op {
        Context,
        Removed,
        Added,
    }
    let ops: Vec<(Op, usize)> = if old_lines.len().saturating_mul(new_lines.len()) > 4_000_000 {
        // Fallback for pathological sizes: whole-file replacement.
        let mut ops: Vec<(Op, usize)> = (0..old_lines.len()).map(|i| (Op::Removed, i)).collect();
        ops.extend((0..new_lines.len()).map(|i| (Op::Added, i)));
        ops
    } else {
        // LCS table over lines.
        let rows = old_lines.len();
        let columns = new_lines.len();
        let mut table = vec![0usize; (rows + 1) * (columns + 1)];
        for row in (0..rows).rev() {
            for column in (0..columns).rev() {
                table[row * (columns + 1) + column] = if old_lines[row] == new_lines[column] {
                    table[(row + 1) * (columns + 1) + column + 1] + 1
                } else {
                    table[(row + 1) * (columns + 1) + column]
                        .max(table[row * (columns + 1) + column + 1])
                };
            }
        }
        let mut ops = Vec::new();
        let (mut row, mut column) = (0usize, 0usize);
        while row < rows && column < columns {
            if old_lines[row] == new_lines[column] {
                ops.push((Op::Context, row));
                row += 1;
                column += 1;
            } else if table[(row + 1) * (columns + 1) + column]
                >= table[row * (columns + 1) + column + 1]
            {
                ops.push((Op::Removed, row));
                row += 1;
            } else {
                ops.push((Op::Added, column));
                column += 1;
            }
        }
        ops.extend((row..rows).map(|i| (Op::Removed, i)));
        ops.extend((column..columns).map(|i| (Op::Added, i)));
        ops
    };

    if !ops.iter().any(|(op, _)| *op != Op::Context) {
        return ConfigDiffPreview {
            lines,
            added,
            removed,
        };
    }

    lines.push(DiffLine {
        kind: DiffLineKind::Meta,
        text: format!("--- {old_label}"),
    });
    lines.push(DiffLine {
        kind: DiffLineKind::Meta,
        text: format!("+++ {new_label}"),
    });

    // Group ops into hunks with up to three context lines around changes.
    const CONTEXT: usize = 3;
    let change_positions: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, (op, _))| *op != Op::Context)
        .map(|(index, _)| index)
        .collect();
    let mut hunks: Vec<(usize, usize)> = Vec::new();
    for &position in &change_positions {
        let start = position.saturating_sub(CONTEXT);
        let end = (position + CONTEXT + 1).min(ops.len());
        match hunks.last_mut() {
            Some((_, last_end)) if start <= *last_end => *last_end = end,
            _ => hunks.push((start, end)),
        }
    }

    // Line counters for hunk headers (1-based, difflib style).
    for (start, end) in hunks {
        let old_start = ops[..start]
            .iter()
            .filter(|(op, _)| matches!(op, Op::Context | Op::Removed))
            .count();
        let new_start = ops[..start]
            .iter()
            .filter(|(op, _)| matches!(op, Op::Context | Op::Added))
            .count();
        let old_count = ops[start..end]
            .iter()
            .filter(|(op, _)| matches!(op, Op::Context | Op::Removed))
            .count();
        let new_count = ops[start..end]
            .iter()
            .filter(|(op, _)| matches!(op, Op::Context | Op::Added))
            .count();
        lines.push(DiffLine {
            kind: DiffLineKind::Hunk,
            text: format!(
                "@@ -{},{} +{},{} @@",
                if old_count == 0 {
                    old_start
                } else {
                    old_start + 1
                },
                old_count,
                if new_count == 0 {
                    new_start
                } else {
                    new_start + 1
                },
                new_count
            ),
        });
        for &(op, index) in &ops[start..end] {
            match op {
                Op::Context => lines.push(DiffLine {
                    kind: DiffLineKind::Context,
                    text: format!(" {}", old_lines[index]),
                }),
                Op::Removed => {
                    removed += 1;
                    lines.push(DiffLine {
                        kind: DiffLineKind::Removed,
                        text: format!("-{}", old_lines[index]),
                    });
                }
                Op::Added => {
                    added += 1;
                    lines.push(DiffLine {
                        kind: DiffLineKind::Added,
                        text: format!("+{}", new_lines[index]),
                    });
                }
            }
        }
    }

    ConfigDiffPreview {
        lines,
        added,
        removed,
    }
}
