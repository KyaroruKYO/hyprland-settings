use std::path::PathBuf;

use adw::prelude::*;
use gtk4::glib;

use crate::config_discovery::discover_hyprland_config;
use crate::current_config::CurrentConfigSnapshot;
use crate::export::ExportBundle;
use crate::metadata::{resolve_metadata_path, MetadataPathResolution};
use crate::ui::window::{show_error_window, show_main_window};
use crate::validation::validate_bundle;

const APP_ID: &str = "io.github.kyarorukyo.hyprlandsettings";

pub fn run() -> glib::ExitCode {
    let cli_override = std::env::args_os().nth(1).map(PathBuf::from);

    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        let cli_override = cli_override.clone();
        let config_discovery = discover_hyprland_config();
        let current_config = CurrentConfigSnapshot::from_discovery(&config_discovery);
        match resolve_metadata_path(cli_override) {
            Ok(resolution) => match load_validated_bundle(&resolution) {
                Ok((bundle, summary)) => {
                    show_main_window(app, bundle, summary, config_discovery, current_config)
                }
                Err(error) => show_error_window(
                    app,
                    &format!(
                        "{} ({})",
                        resolution.export_dir.display(),
                        resolution.source
                    ),
                    &error.to_string(),
                ),
            },
            Err(error) => show_error_window(app, "metadata lookup", &error.to_string()),
        }
    });

    app.run_with_args(&["hyprland-settings"])
}

fn load_validated_bundle(
    resolution: &MetadataPathResolution,
) -> anyhow::Result<(ExportBundle, crate::validation::ValidationSummary)> {
    let bundle = ExportBundle::load(&resolution.export_dir)?;
    let summary = validate_bundle(&bundle)?;
    Ok((bundle, summary))
}

#[cfg(test)]
mod tests {
    use super::APP_ID;

    #[test]
    fn app_id_uses_final_public_identity() {
        assert_eq!(APP_ID, "io.github.kyarorukyo.hyprlandsettings");
    }
}
