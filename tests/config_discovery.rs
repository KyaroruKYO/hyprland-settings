use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::config_discovery::{
    config_candidates, discover_hyprland_config_with_env, ConfigDiscoveryEnv,
    ConfigDiscoveryStatus, ConfigPathSource,
};

fn temp_root(name: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-{name}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn write_config(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("config path should have parent"))?;
    fs::write(&path, "general:gaps_in = 5\n")?;
    Ok(path)
}

#[test]
fn xdg_config_home_wins_when_file_exists() -> Result<()> {
    let root = temp_root("xdg-wins")?;
    let xdg = root.join("xdg");
    let home = root.join("home");
    let xdg_path = write_config(&xdg, "hypr/hyprland.conf")?;
    let _fallback_path = write_config(&home, ".config/hypr/hyprland.conf")?;

    let discovery = discover_hyprland_config_with_env(&ConfigDiscoveryEnv {
        xdg_config_home: Some(xdg),
        home: Some(home),
    });

    assert_eq!(
        discovery.status,
        ConfigDiscoveryStatus::Found {
            path: xdg_path,
            source: ConfigPathSource::XdgConfigHome,
        }
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn fallback_home_path_is_used_when_xdg_file_is_missing() -> Result<()> {
    let root = temp_root("fallback")?;
    let xdg = root.join("xdg");
    let home = root.join("home");
    let fallback_path = write_config(&home, ".config/hypr/hyprland.conf")?;

    let discovery = discover_hyprland_config_with_env(&ConfigDiscoveryEnv {
        xdg_config_home: Some(xdg),
        home: Some(home),
    });

    assert_eq!(
        discovery.status,
        ConfigDiscoveryStatus::Found {
            path: fallback_path,
            source: ConfigPathSource::HomeFallback,
        }
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn missing_config_reports_attempted_paths() -> Result<()> {
    let root = temp_root("missing")?;
    let xdg = root.join("xdg");
    let home = root.join("home");

    let discovery = discover_hyprland_config_with_env(&ConfigDiscoveryEnv {
        xdg_config_home: Some(xdg.clone()),
        home: Some(home.clone()),
    });

    assert_eq!(discovery.status, ConfigDiscoveryStatus::Missing);
    assert_eq!(
        discovery.attempted_paths,
        vec![
            xdg.join("hypr/hyprland.conf"),
            home.join(".config/hypr/hyprland.conf")
        ]
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn hyprland_lua_is_not_a_candidate() -> Result<()> {
    let root = temp_root("lua-ignored")?;
    let xdg = root.join("xdg");
    fs::create_dir_all(xdg.join("hypr"))?;
    fs::write(xdg.join("hypr/hyprland.lua"), "return {}\n")?;

    let env = ConfigDiscoveryEnv {
        xdg_config_home: Some(xdg.clone()),
        home: None,
    };
    let discovery = discover_hyprland_config_with_env(&env);
    let candidates = config_candidates(&env);

    assert_eq!(discovery.status, ConfigDiscoveryStatus::Missing);
    assert_eq!(
        candidates,
        vec![(
            ConfigPathSource::XdgConfigHome,
            xdg.join("hypr/hyprland.conf")
        )]
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn non_file_config_path_is_reported() -> Result<()> {
    let root = temp_root("not-file")?;
    let xdg = root.join("xdg");
    fs::create_dir_all(xdg.join("hypr/hyprland.conf"))?;

    let discovery = discover_hyprland_config_with_env(&ConfigDiscoveryEnv {
        xdg_config_home: Some(xdg.clone()),
        home: None,
    });

    assert_eq!(
        discovery.status,
        ConfigDiscoveryStatus::NotAFile {
            path: xdg.join("hypr/hyprland.conf"),
            source: ConfigPathSource::XdgConfigHome,
        }
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn unreadable_config_path_is_reported_when_permissions_block_open() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_root("unreadable")?;
    let xdg = root.join("xdg");
    let path = write_config(&xdg, "hypr/hyprland.conf")?;
    let original_permissions = fs::metadata(&path)?.permissions();
    let mut blocked_permissions = original_permissions.clone();
    blocked_permissions.set_mode(0o000);
    fs::set_permissions(&path, blocked_permissions)?;

    let discovery = discover_hyprland_config_with_env(&ConfigDiscoveryEnv {
        xdg_config_home: Some(xdg),
        home: None,
    });

    fs::set_permissions(&path, original_permissions)?;

    match discovery.status {
        ConfigDiscoveryStatus::Unreadable {
            path: found_path, ..
        } => {
            assert_eq!(found_path, path);
        }
        other => panic!("expected unreadable config, got {other:?}"),
    }

    fs::remove_dir_all(root)?;
    Ok(())
}
