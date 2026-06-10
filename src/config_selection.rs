use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSelectionState {
    pub auto_detected_config: Option<PathBuf>,
    pub manual_preview: Option<ManualConfigChoice>,
    pub session_read_only_config: Option<PathBuf>,
    pub source_follow_choice: SourceFollowChoice,
    pub lifecycle: ConfigSelectionLifecycle,
}

impl ConfigSelectionState {
    pub fn auto_detected(path: impl Into<PathBuf>) -> Self {
        Self {
            auto_detected_config: Some(path.into()),
            manual_preview: None,
            session_read_only_config: None,
            source_follow_choice: SourceFollowChoice::ReviewAllConnectedFiles,
            lifecycle: ConfigSelectionLifecycle::AutoDetected,
        }
    }

    pub fn no_detected_config() -> Self {
        Self {
            auto_detected_config: None,
            manual_preview: None,
            session_read_only_config: None,
            source_follow_choice: SourceFollowChoice::ReviewAllConnectedFiles,
            lifecycle: ConfigSelectionLifecycle::NoConfigDetected,
        }
    }

    pub fn preview_manual_config(
        mut self,
        path: impl Into<PathBuf>,
        source_follow_choice: SourceFollowChoice,
    ) -> Self {
        self.manual_preview = Some(ManualConfigChoice::preview(path));
        self.source_follow_choice = source_follow_choice;
        self.lifecycle = ConfigSelectionLifecycle::ManualPreview;
        self
    }

    pub fn confirm_preview(mut self) -> Self {
        if let Some(preview) = &mut self.manual_preview {
            preview.confirmed = true;
        }
        self.lifecycle = ConfigSelectionLifecycle::ConfirmedForFutureReview;
        self
    }

    pub fn use_preview_for_session_read_only(mut self) -> Self {
        if let Some(preview) = &self.manual_preview {
            self.session_read_only_config = Some(preview.path.clone());
            self.lifecycle = ConfigSelectionLifecycle::SessionReadOnly;
        }
        self
    }

    pub fn cancel_preview(mut self) -> Self {
        self.manual_preview = None;
        self.session_read_only_config = None;
        self.lifecycle = ConfigSelectionLifecycle::Cancelled;
        self
    }

    pub fn preview(&self) -> ConfigSelectionPreview {
        ConfigSelectionPreview {
            detected_config: self.auto_detected_config.clone(),
            selected_for_review: self
                .manual_preview
                .as_ref()
                .map(|choice| choice.path.clone()),
            session_read_only_config: self.session_read_only_config.clone(),
            selection_source: if self.manual_preview.is_some() {
                ConfigSelectionSource::ManualPreview
            } else if self.auto_detected_config.is_some() {
                ConfigSelectionSource::AutoDetected
            } else {
                ConfigSelectionSource::None
            },
            source_follow_choice: self.source_follow_choice,
            preview_only: self
                .manual_preview
                .as_ref()
                .is_some_and(|choice| choice.preview_only),
            confirmed: self
                .manual_preview
                .as_ref()
                .is_some_and(|choice| choice.confirmed),
            session_only: self.lifecycle == ConfigSelectionLifecycle::SessionReadOnly,
            cancelled: self.lifecycle == ConfigSelectionLifecycle::Cancelled,
        }
    }

    pub fn write_target_path(&self) -> Option<&PathBuf> {
        self.auto_detected_config.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualConfigChoice {
    pub path: PathBuf,
    pub preview_only: bool,
    pub confirmed: bool,
}

impl ManualConfigChoice {
    pub fn preview(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            preview_only: true,
            confirmed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSelectionPreview {
    pub detected_config: Option<PathBuf>,
    pub selected_for_review: Option<PathBuf>,
    pub session_read_only_config: Option<PathBuf>,
    pub selection_source: ConfigSelectionSource,
    pub source_follow_choice: SourceFollowChoice,
    pub preview_only: bool,
    pub confirmed: bool,
    pub session_only: bool,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSelectionSource {
    AutoDetected,
    ManualPreview,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFollowChoice {
    ReviewAllConnectedFiles,
    OnlySelectedFile,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSelectionLifecycle {
    AutoDetected,
    NoConfigDetected,
    ManualPreview,
    ConfirmedForFutureReview,
    SessionReadOnly,
    Cancelled,
}
