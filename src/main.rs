use gtk4::glib;

fn main() -> glib::ExitCode {
    if let Err(error) = maybe_run_live_validation_cli() {
        eprintln!("{error:#}");
        return glib::ExitCode::FAILURE;
    }
    hyprland_settings::ui::app::run()
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
