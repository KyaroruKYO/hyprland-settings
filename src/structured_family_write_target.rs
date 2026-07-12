//! Controlled write-target classification for structured families.
//!
//! Every structured-family write in this phase must go to a controlled target:
//! a test-owned fixture file, a copied config tree, or a temporary config
//! file. The active real Hyprland config is never writable here. Targets are
//! classified against a declared controlled root, and any path that resolves
//! to the active config directory, escapes the root, or cannot be proven
//! controlled is rejected.

use std::path::{Component, Path, PathBuf};

use crate::structured_family::structured_family_render_target_allowed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyControlledWriteTargetKind {
    TestOwnedFixtureTarget,
    CopiedConfigTreeTarget,
    TemporaryConfigTarget,
    ActiveRealConfigTarget,
    UnknownTarget,
}

impl StructuredFamilyControlledWriteTargetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TestOwnedFixtureTarget => "TestOwnedFixtureTarget",
            Self::CopiedConfigTreeTarget => "CopiedConfigTreeTarget",
            Self::TemporaryConfigTarget => "TemporaryConfigTarget",
            Self::ActiveRealConfigTarget => "ActiveRealConfigTarget",
            Self::UnknownTarget => "UnknownTarget",
        }
    }

    pub fn writable_in_controlled_phase(self) -> bool {
        matches!(
            self,
            Self::TestOwnedFixtureTarget
                | Self::CopiedConfigTreeTarget
                | Self::TemporaryConfigTarget
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyControlledWriteTargetRejection {
    ActiveRealConfigTargetRejected,
    UnknownTargetRejected,
    PathEscapeRejected,
    SymlinkEscapeRejected,
    TargetOutsideControlledRoot,
    ControlledRootNotAllowed,
    DeclaredKindNotWritable,
    DeclaredKindInconsistentWithPath,
}

impl StructuredFamilyControlledWriteTargetRejection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ActiveRealConfigTargetRejected => "ActiveRealConfigTargetRejected",
            Self::UnknownTargetRejected => "UnknownTargetRejected",
            Self::PathEscapeRejected => "PathEscapeRejected",
            Self::SymlinkEscapeRejected => "SymlinkEscapeRejected",
            Self::TargetOutsideControlledRoot => "TargetOutsideControlledRoot",
            Self::ControlledRootNotAllowed => "ControlledRootNotAllowed",
            Self::DeclaredKindNotWritable => "DeclaredKindNotWritable",
            Self::DeclaredKindInconsistentWithPath => "DeclaredKindInconsistentWithPath",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWriteTarget {
    pub declared_kind: StructuredFamilyControlledWriteTargetKind,
    pub path: PathBuf,
    pub controlled_root: PathBuf,
}

impl StructuredFamilyControlledWriteTarget {
    pub fn new(
        declared_kind: StructuredFamilyControlledWriteTargetKind,
        path: impl Into<PathBuf>,
        controlled_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            declared_kind,
            path: path.into(),
            controlled_root: controlled_root.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWriteTargetPolicy {
    pub declared_kind: StructuredFamilyControlledWriteTargetKind,
    pub resolved_kind: StructuredFamilyControlledWriteTargetKind,
    pub writable: bool,
    pub active_real_config_writable: bool,
    pub rejection_reasons: Vec<StructuredFamilyControlledWriteTargetRejection>,
}

fn active_real_config_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        let home = PathBuf::from(home);
        roots.push(home.join(".config").join("hypr"));
        roots.push(home.join(".config").join("hyprland"));
    }
    roots
}

/// True when the path is (or resolves to) the user's active real Hyprland
/// config area. Public so the active-config pilot can prove target identity
/// with the same rule the controlled policy uses to refuse it.
pub fn structured_family_path_is_active_real_config(path: &Path) -> bool {
    path_is_active_real_config(path)
}

fn path_is_active_real_config(path: &Path) -> bool {
    let candidates = active_real_config_roots();
    if candidates.iter().any(|root| path.starts_with(root)) {
        return true;
    }
    if let Ok(canonical) = path.canonicalize() {
        if candidates.iter().any(|root| canonical.starts_with(root)) {
            return true;
        }
    }
    false
}

fn path_has_parent_traversal(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

fn canonical_parent_escapes_root(path: &Path, controlled_root: &Path) -> Option<bool> {
    let canonical_root = controlled_root.canonicalize().ok()?;
    let parent = path.parent()?;
    let canonical_parent = parent.canonicalize().ok()?;
    // A symlinked parent that resolves outside the controlled root is an
    // escape even when the literal path text stays inside the root.
    Some(!canonical_parent.starts_with(&canonical_root))
}

fn canonical_target_file_escapes_root(path: &Path, controlled_root: &Path) -> Option<bool> {
    // The target file itself may be a symlink pointing outside the root even
    // when its parent directory is inside; writing through it would mutate a
    // foreign file. Only checkable when the target already exists.
    let canonical_root = controlled_root.canonicalize().ok()?;
    let canonical_path = path.canonicalize().ok()?;
    Some(!canonical_path.starts_with(&canonical_root))
}

fn canonical_target_is_active_real_config(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    let Ok(canonical_parent) = parent.canonicalize() else {
        return false;
    };
    let file_name = path.file_name().unwrap_or_default();
    path_is_active_real_config(&canonical_parent.join(file_name))
}

fn declared_kind_consistent_with_path(
    declared_kind: StructuredFamilyControlledWriteTargetKind,
    controlled_root: &Path,
) -> bool {
    let temp_dir = std::env::temp_dir();
    match declared_kind {
        StructuredFamilyControlledWriteTargetKind::TemporaryConfigTarget
        | StructuredFamilyControlledWriteTargetKind::CopiedConfigTreeTarget => {
            controlled_root.starts_with(&temp_dir)
        }
        StructuredFamilyControlledWriteTargetKind::TestOwnedFixtureTarget => {
            structured_family_render_target_allowed(controlled_root)
        }
        StructuredFamilyControlledWriteTargetKind::ActiveRealConfigTarget
        | StructuredFamilyControlledWriteTargetKind::UnknownTarget => false,
    }
}

/// Classify a controlled write target. Fails closed: any path that resolves
/// to the active real Hyprland config, escapes the declared controlled root
/// (textually or through symlinks), sits under a disallowed root, or carries
/// an unwritable/unproven declared kind is rejected.
pub fn classify_structured_family_write_target(
    target: &StructuredFamilyControlledWriteTarget,
) -> StructuredFamilyControlledWriteTargetPolicy {
    let mut rejection_reasons = Vec::new();
    let mut resolved_kind = target.declared_kind;

    if path_is_active_real_config(&target.path)
        || path_is_active_real_config(&target.controlled_root)
        || canonical_target_is_active_real_config(&target.path)
    {
        resolved_kind = StructuredFamilyControlledWriteTargetKind::ActiveRealConfigTarget;
        rejection_reasons
            .push(StructuredFamilyControlledWriteTargetRejection::ActiveRealConfigTargetRejected);
    } else {
        if path_has_parent_traversal(&target.path)
            || path_has_parent_traversal(&target.controlled_root)
        {
            resolved_kind = StructuredFamilyControlledWriteTargetKind::UnknownTarget;
            rejection_reasons
                .push(StructuredFamilyControlledWriteTargetRejection::PathEscapeRejected);
        }
        if !target.path.starts_with(&target.controlled_root) {
            resolved_kind = StructuredFamilyControlledWriteTargetKind::UnknownTarget;
            rejection_reasons
                .push(StructuredFamilyControlledWriteTargetRejection::TargetOutsideControlledRoot);
        }
        if (canonical_parent_escapes_root(&target.path, &target.controlled_root).unwrap_or(false)
            || canonical_target_file_escapes_root(&target.path, &target.controlled_root)
                .unwrap_or(false))
            && !rejection_reasons
                .contains(&StructuredFamilyControlledWriteTargetRejection::SymlinkEscapeRejected)
        {
            resolved_kind = StructuredFamilyControlledWriteTargetKind::UnknownTarget;
            rejection_reasons
                .push(StructuredFamilyControlledWriteTargetRejection::SymlinkEscapeRejected);
        }
        if !structured_family_render_target_allowed(&target.controlled_root) {
            resolved_kind = StructuredFamilyControlledWriteTargetKind::UnknownTarget;
            rejection_reasons
                .push(StructuredFamilyControlledWriteTargetRejection::ControlledRootNotAllowed);
        }
        if !target.declared_kind.writable_in_controlled_phase() {
            resolved_kind = target.declared_kind;
            rejection_reasons
                .push(StructuredFamilyControlledWriteTargetRejection::DeclaredKindNotWritable);
            if target.declared_kind == StructuredFamilyControlledWriteTargetKind::UnknownTarget {
                rejection_reasons
                    .push(StructuredFamilyControlledWriteTargetRejection::UnknownTargetRejected);
            }
        } else if rejection_reasons.is_empty()
            && !declared_kind_consistent_with_path(target.declared_kind, &target.controlled_root)
        {
            resolved_kind = StructuredFamilyControlledWriteTargetKind::UnknownTarget;
            rejection_reasons.push(
                StructuredFamilyControlledWriteTargetRejection::DeclaredKindInconsistentWithPath,
            );
        }
        if resolved_kind == StructuredFamilyControlledWriteTargetKind::UnknownTarget
            && !rejection_reasons
                .contains(&StructuredFamilyControlledWriteTargetRejection::UnknownTargetRejected)
        {
            rejection_reasons
                .push(StructuredFamilyControlledWriteTargetRejection::UnknownTargetRejected);
        }
    }

    let writable = rejection_reasons.is_empty() && resolved_kind.writable_in_controlled_phase();

    StructuredFamilyControlledWriteTargetPolicy {
        declared_kind: target.declared_kind,
        resolved_kind,
        writable,
        // The active real config is never writable through this policy. This
        // field is a constant statement of that boundary, not a switch.
        active_real_config_writable: false,
        rejection_reasons,
    }
}
