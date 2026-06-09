use gtk4::glib;

fn main() -> glib::ExitCode {
    if let Err(error) = maybe_run_high_risk_watchdog_cli() {
        eprintln!("{error:#}");
        return glib::ExitCode::FAILURE;
    }
    if let Err(error) = maybe_run_live_validation_cli() {
        eprintln!("{error:#}");
        return glib::ExitCode::FAILURE;
    }
    if let Err(error) = maybe_run_config_persistence_cli() {
        eprintln!("{error:#}");
        return glib::ExitCode::FAILURE;
    }
    hyprland_settings::ui::app::run()
}

fn maybe_run_high_risk_watchdog_cli() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.get(1).map(String::as_str) != Some("high-risk-watchdog") {
        return Ok(());
    }

    let mut plan_path: Option<std::path::PathBuf> = None;
    let mut result_path: Option<std::path::PathBuf> = None;
    let mut action: Option<String> = None;
    let mut token: Option<String> = None;
    let mut backup_root: Option<std::path::PathBuf> = None;
    let mut now = 0_u64;
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--plan" => {
                index += 1;
                plan_path = Some(
                    args.get(index)
                        .ok_or_else(|| anyhow::anyhow!("--plan needs a path"))?
                        .into(),
                );
            }
            "--result" => {
                index += 1;
                result_path = Some(
                    args.get(index)
                        .ok_or_else(|| anyhow::anyhow!("--result needs a path"))?
                        .into(),
                );
            }
            "--backup-root" => {
                index += 1;
                backup_root = Some(
                    args.get(index)
                        .ok_or_else(|| anyhow::anyhow!("--backup-root needs a path"))?
                        .into(),
                );
            }
            "--action" => {
                index += 1;
                action = Some(
                    args.get(index)
                        .ok_or_else(|| anyhow::anyhow!("--action needs confirm or expire"))?
                        .to_string(),
                );
            }
            "--token" => {
                index += 1;
                token = Some(
                    args.get(index)
                        .ok_or_else(|| anyhow::anyhow!("--token needs a value"))?
                        .to_string(),
                );
            }
            "--now" => {
                index += 1;
                now = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("--now needs a unix timestamp"))?
                    .parse::<u64>()?;
            }
            other => anyhow::bail!("unknown high-risk-watchdog argument: {other}"),
        }
        index += 1;
    }

    let plan_path = plan_path.ok_or_else(|| anyhow::anyhow!("--plan is required"))?;
    let result_path = result_path.ok_or_else(|| anyhow::anyhow!("--result is required"))?;
    let backup_root = backup_root.ok_or_else(|| anyhow::anyhow!("--backup-root is required"))?;
    let action = action.ok_or_else(|| anyhow::anyhow!("--action is required"))?;
    let mut plan = hyprland_settings::high_risk_recovery::load_watchdog_plan(&plan_path)?;
    plan.result_log_path = result_path;
    let planner =
        hyprland_settings::high_risk_recovery::HighRiskRecoveryPlanner::new(backup_root, now);
    let result = match action.as_str() {
        "confirm" => {
            let token = token.ok_or_else(|| anyhow::anyhow!("--token is required for confirm"))?;
            planner.confirm_watchdog(&plan, &token)?
        }
        "expire" => planner.expire_watchdog_and_restore(&plan)?,
        other => anyhow::bail!("unsupported high-risk watchdog action: {other}"),
    };
    println!(
        "high-risk watchdog {:?} wrote {}",
        result.recovery.status,
        result.result_log_path.display()
    );
    std::process::exit(0);
}

fn maybe_run_config_persistence_cli() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.get(1).map(String::as_str) != Some("validate-config-persistence") {
        return Ok(());
    }
    let mut dry_run = false;
    let mut verify_hyprland = false;
    let mut row_id: Option<String> = None;
    let mut candidates_path =
        hyprland_settings::config_persistence_validation::default_candidates_path();
    let mut results_path = hyprland_settings::config_persistence_validation::default_results_path();
    let mut timeout_seconds = 10_u64;
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => dry_run = true,
            "--verify-hyprland" => verify_hyprland = true,
            "--candidates" => {
                index += 1;
                candidates_path = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("--candidates needs a path"))?
                    .into();
            }
            "--results" => {
                index += 1;
                results_path = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("--results needs a path"))?
                    .into();
            }
            "--row" => {
                index += 1;
                row_id = Some(
                    args.get(index)
                        .ok_or_else(|| anyhow::anyhow!("--row needs a row ID"))?
                        .to_string(),
                );
            }
            "--batch" => {
                index += 1;
                let batch = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("--batch needs a value"))?;
                if batch != "batch-a-likely-safe-booleans" {
                    anyhow::bail!("only batch-a-likely-safe-booleans is supported in this sprint");
                }
            }
            "--timeout-seconds" => {
                index += 1;
                timeout_seconds = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("--timeout-seconds needs a value"))?
                    .parse::<u64>()?;
                if timeout_seconds == 0 || timeout_seconds > 30 {
                    anyhow::bail!("timeout must be between 1 and 30 seconds");
                }
            }
            other => anyhow::bail!("unknown validate-config-persistence argument: {other}"),
        }
        index += 1;
    }
    if dry_run && verify_hyprland {
        anyhow::bail!("choose either --dry-run or --verify-hyprland");
    }
    let mut report =
        hyprland_settings::config_persistence_validation::load_candidates(&candidates_path)?;
    if let Some(row_id) = row_id {
        report.rows.retain(|row| row.row_id == row_id);
        if report.rows.is_empty() {
            anyhow::bail!("row was not found in config-persistence candidates");
        }
    }
    let mut verifier = hyprland_settings::config_persistence_validation::RealHyprlandConfigVerifier;
    let results =
        hyprland_settings::config_persistence_validation::run_config_persistence_validation(
            &report,
            verify_hyprland,
            &mut verifier,
            std::time::Duration::from_secs(timeout_seconds),
        );
    hyprland_settings::config_persistence_validation::save_results(&results_path, &results)?;
    println!(
        "config persistence validation {} wrote {} rows to {}",
        results.mode,
        results.counts.rows,
        results_path.display()
    );
    std::process::exit(0);
}

fn maybe_run_live_validation_cli() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.get(1).map(String::as_str) != Some("live-validate") {
        return Ok(());
    }
    let mut live = false;
    let mut dry_run = false;
    let mut diagnose = false;
    let mut row_id: Option<String> = None;
    let mut timeout_override: Option<u64> = None;
    let mut plan_path = hyprland_settings::live_validation::default_plan_path();
    let mut results_path = hyprland_settings::live_validation::default_results_path();
    let mut diagnostics_path = hyprland_settings::live_validation::default_diagnostics_path();
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--live" => live = true,
            "--dry-run" => dry_run = true,
            "--diagnose" => diagnose = true,
            "--plan" => {
                index += 1;
                plan_path = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("--plan needs a path"))?
                    .into();
            }
            "--results" => {
                index += 1;
                results_path = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("--results needs a path"))?
                    .into();
            }
            "--diagnostics" => {
                index += 1;
                diagnostics_path = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("--diagnostics needs a path"))?
                    .into();
            }
            "--row" => {
                index += 1;
                row_id = Some(
                    args.get(index)
                        .ok_or_else(|| anyhow::anyhow!("--row needs a row ID"))?
                        .to_string(),
                );
            }
            "--batch" => {
                index += 1;
                let batch = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("--batch needs a value"))?;
                if batch != "batch-a-likely-safe-booleans" {
                    anyhow::bail!("only batch-a-likely-safe-booleans is supported in this sprint");
                }
            }
            "--timeout-seconds" => {
                index += 1;
                let timeout = args
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("--timeout-seconds needs a value"))?
                    .parse::<u64>()?;
                if timeout == 0 || timeout > 10 {
                    anyhow::bail!("timeout must be between 1 and 10 seconds");
                }
                timeout_override = Some(timeout);
            }
            other => anyhow::bail!("unknown live-validate argument: {other}"),
        }
        index += 1;
    }
    if live && dry_run {
        anyhow::bail!("choose either --live or --dry-run");
    }
    let plan = hyprland_settings::live_validation::load_plan(&plan_path)?;
    if diagnose {
        if dry_run {
            anyhow::bail!("--diagnose is a live diagnostic mode; do not combine with --dry-run");
        }
        let row_id = row_id.ok_or_else(|| anyhow::anyhow!("--diagnose requires --row <rowId>"))?;
        let selected = std::iter::once(row_id).collect();
        let mut runner = hyprland_settings::live_validation::RealHyprctlRunner;
        let mut watchdog = hyprland_settings::live_validation::ProcessRollbackWatchdog::default();
        let diagnostics = hyprland_settings::live_validation::run_live_diagnostics(
            &plan,
            &selected,
            timeout_override,
            &mut runner,
            &mut watchdog,
        );
        hyprland_settings::live_validation::save_diagnostics(&diagnostics_path, &diagnostics)?;
        println!(
            "live validation diagnostics wrote {} rows to {}",
            diagnostics.counts.rows,
            diagnostics_path.display()
        );
        std::process::exit(0);
    }
    let results = if live {
        let mut runner = hyprland_settings::live_validation::RealHyprctlRunner;
        let mut watchdog = hyprland_settings::live_validation::ProcessRollbackWatchdog::default();
        hyprland_settings::live_validation::run_live_validation(&plan, &mut runner, &mut watchdog)
    } else {
        hyprland_settings::live_validation::run_dry_validation(&plan)
    };
    hyprland_settings::live_validation::save_results(&results_path, &results)?;
    println!(
        "live validation {} wrote {} rows to {}",
        results.mode,
        results.counts.rows,
        results_path.display()
    );
    std::process::exit(0);
}
