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
    let mut plan_path = hyprland_settings::live_validation::default_plan_path();
    let mut results_path = hyprland_settings::live_validation::default_results_path();
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--live" => live = true,
            "--dry-run" => dry_run = true,
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
            }
            other => anyhow::bail!("unknown live-validate argument: {other}"),
        }
        index += 1;
    }
    if live && dry_run {
        anyhow::bail!("choose either --live or --dry-run");
    }
    let plan = hyprland_settings::live_validation::load_plan(&plan_path)?;
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
