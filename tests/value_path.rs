use anyhow::Result;
use hyprland_settings::value::path_value::PathValue;

#[test]
fn path_value_accepts_absolute_relative_and_tilde_paths() -> Result<()> {
    let absolute = PathValue::parse("/home/kyo/.config/hypr/shader.frag")?;
    let relative = PathValue::parse("./shader.frag")?;
    let tilde = PathValue::parse("~/.config/hypr/keymap.xkb")?;

    assert_eq!(absolute.serialize(), "/home/kyo/.config/hypr/shader.frag");
    assert_eq!(relative.serialize(), "./shader.frag");
    assert_eq!(tilde.serialize(), "~/.config/hypr/keymap.xkb");

    Ok(())
}

#[test]
fn path_value_rejects_command_like_or_multiline_values() {
    for raw in [
        "",
        "shader.frag\nnext",
        "shader.frag # comment",
        "`cmd`",
        "$(cmd)",
        "a;b",
        "a|b",
        "a && b",
        "a || b",
    ] {
        assert!(PathValue::parse(raw).is_err(), "{raw:?} should be invalid");
    }
}
