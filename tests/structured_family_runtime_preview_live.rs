use hyprland_settings::runtime_preview_executor::{
    HyprctlRuntimePreviewRunner, RuntimePreviewRunner,
};
use hyprland_settings::structured_family_runtime_preview::{
    parse_animation_leaf, parse_bezier_points,
};

fn read_animations(runner: &mut HyprctlRuntimePreviewRunner) -> String {
    runner
        .run("hyprctl", &["animations".to_string()])
        .expect("read-only animations listing should succeed")
}

/// Env-gated live proof for structured-family modify-existing round trips.
///
/// `HYPRLAND_SETTINGS_STRUCTURED_FAMILY` selects `hl.animation`, `hl.curve`,
/// or `all`. Each proof only modifies a record that already exists in the
/// runtime (the explicitly overridden `global` animation node, the built-in
/// `default` bezier), captures the original via read-only readback first,
/// applies a minimal delta, verifies, restores the exact original values,
/// and verifies again — zero residue. Never writes config, never reloads.
#[test]
#[ignore = "mutates live animation/curve records; run only with HYPRLAND_SETTINGS_RUN_STRUCTURED_RUNTIME_PREVIEW_LIVE=1 and HYPRLAND_SETTINGS_STRUCTURED_FAMILY=<hl.animation|hl.curve|all>"]
fn structured_family_modify_existing_round_trips() {
    if std::env::var("HYPRLAND_SETTINGS_RUN_STRUCTURED_RUNTIME_PREVIEW_LIVE").as_deref() != Ok("1")
    {
        eprintln!("skipping: HYPRLAND_SETTINGS_RUN_STRUCTURED_RUNTIME_PREVIEW_LIVE is not set");
        return;
    }
    let Ok(selector) = std::env::var("HYPRLAND_SETTINGS_STRUCTURED_FAMILY") else {
        panic!("HYPRLAND_SETTINGS_STRUCTURED_FAMILY must be hl.animation, hl.curve, or all");
    };
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
        return;
    }
    let mut runner = HyprctlRuntimePreviewRunner;

    if selector == "hl.animation" || selector == "all" {
        // Capture the explicitly overridden global node.
        let listing = read_animations(&mut runner);
        let (enabled, speed, bezier) =
            parse_animation_leaf(&listing, "global").expect("global animation leaf should parse");
        assert!(
            listing.contains("name: global"),
            "global node must exist for a modify-existing proof"
        );
        let original_speed: f64 = speed.parse().expect("speed parses");
        let preview_speed = original_speed + 0.5;
        let enabled_lua = if enabled == "1" { "true" } else { "false" };
        let bezier_name = if bezier.is_empty() {
            "default"
        } else {
            &bezier
        };

        // Apply the minimal delta.
        runner
            .run(
                "hyprctl",
                &[
                    "eval".to_string(),
                    format!(
                        "hl.animation({{ leaf = \"global\", enabled = {enabled_lua}, speed = {preview_speed}, bezier = \"{bezier_name}\" }})"
                    ),
                ],
            )
            .expect("animation apply should succeed");
        let during = read_animations(&mut runner);
        let (_, during_speed, _) =
            parse_animation_leaf(&during, "global").expect("global parses after apply");
        assert!(
            (during_speed.parse::<f64>().expect("parses") - preview_speed).abs() < 1e-3,
            "apply must be visible in readback: expected {preview_speed}, got {during_speed}"
        );

        // Restore the exact original values.
        runner
            .run(
                "hyprctl",
                &[
                    "eval".to_string(),
                    format!(
                        "hl.animation({{ leaf = \"global\", enabled = {enabled_lua}, speed = {original_speed}, bezier = \"{bezier_name}\" }})"
                    ),
                ],
            )
            .expect("animation revert should succeed");
        let after = read_animations(&mut runner);
        let (after_enabled, after_speed, after_bezier) =
            parse_animation_leaf(&after, "global").expect("global parses after revert");
        assert_eq!(after_enabled, enabled, "enabled must restore exactly");
        assert_eq!(after_speed, speed, "speed must restore exactly");
        assert_eq!(after_bezier, bezier, "bezier must restore exactly");
        println!(
            "FAMILY PROOF PASSED hl.animation | record global | original enabled={enabled} speed={speed} bezier={bezier} | preview speed={preview_speed} | reverted-and-verified exactly | zero residue"
        );
    }

    if selector == "hl.curve" || selector == "all" {
        // Capture the built-in default curve.
        let listing = read_animations(&mut runner);
        let (x0, y0, x1, y1) =
            parse_bezier_points(&listing, "default").expect("default bezier should parse");

        // Apply a minimal control-point delta.
        let preview_y0 = y0 + 0.01;
        runner
            .run(
                "hyprctl",
                &[
                    "eval".to_string(),
                    format!("hl.curve(\"default\", {{ type = \"bezier\", points = {{ {{{x0}, {preview_y0}}}, {{{x1}, {y1}}} }} }})"),
                ],
            )
            .expect("curve apply should succeed");
        let during = read_animations(&mut runner);
        let (_, during_y0, _, _) =
            parse_bezier_points(&during, "default").expect("default parses after apply");
        assert!(
            (during_y0 - preview_y0).abs() < 1e-3,
            "curve apply must be visible in readback: expected {preview_y0}, got {during_y0}"
        );

        // Restore the exact original points.
        runner
            .run(
                "hyprctl",
                &[
                    "eval".to_string(),
                    format!("hl.curve(\"default\", {{ type = \"bezier\", points = {{ {{{x0}, {y0}}}, {{{x1}, {y1}}} }} }})"),
                ],
            )
            .expect("curve revert should succeed");
        let after = read_animations(&mut runner);
        let (after_x0, after_y0, after_x1, after_y1) =
            parse_bezier_points(&after, "default").expect("default parses after revert");
        assert!(
            (after_x0 - x0).abs() < 1e-6
                && (after_y0 - y0).abs() < 1e-6
                && (after_x1 - x1).abs() < 1e-6
                && (after_y1 - y1).abs() < 1e-6,
            "curve must restore exactly: expected ({x0},{y0},{x1},{y1}), got ({after_x0},{after_y0},{after_x1},{after_y1})"
        );
        println!(
            "FAMILY PROOF PASSED hl.curve | record default | original ({x0}, {y0}, {x1}, {y1}) | preview y0={preview_y0} | reverted-and-verified exactly | zero residue"
        );
    }
}
