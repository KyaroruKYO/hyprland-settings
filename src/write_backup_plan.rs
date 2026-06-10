use std::path::PathBuf;

use crate::write_target_candidate::WriteTargetCandidate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteBackupPlan {
    pub target_file_path: PathBuf,
    pub resolved_target_path: Option<PathBuf>,
    pub backup_path: PathBuf,
    pub backup_naming_policy: String,
    pub backup_required: bool,
    pub fixture_only: bool,
    pub production_backup_disabled: bool,
}

impl WriteBackupPlan {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            "The app will back up this exact file before saving changes.".to_string(),
            format!("Backup file: {}", self.backup_path.display()),
            "Backup proof is fixture-only right now.".to_string(),
        ]
    }
}

pub fn build_exact_backup_plan(candidate: &WriteTargetCandidate) -> WriteBackupPlan {
    WriteBackupPlan {
        target_file_path: candidate.file_path.clone(),
        resolved_target_path: candidate.resolved_path.clone(),
        backup_path: candidate.file_path.with_extension("review-plan.bak"),
        backup_naming_policy: "target file plus review-plan.bak suffix".to_string(),
        backup_required: candidate.backup_required,
        fixture_only: true,
        production_backup_disabled: true,
    }
}
