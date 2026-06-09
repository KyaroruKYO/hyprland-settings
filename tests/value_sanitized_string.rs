use anyhow::Result;
use hyprland_settings::value::sanitized_string::SanitizedStringValue;

#[test]
fn sanitized_string_accepts_single_line_values() -> Result<()> {
    let font = SanitizedStringValue::parse("JetBrains Mono")?;
    let enum_like = SanitizedStringValue::parse("flat")?;

    assert_eq!(font.serialize(), "JetBrains Mono");
    assert_eq!(enum_like.serialize(), "flat");

    Ok(())
}

#[test]
fn sanitized_string_rejects_config_breaking_values() {
    for raw in ["", "value\nnext", "value # comment", "`cmd`", "$(cmd)"] {
        assert!(
            SanitizedStringValue::parse(raw).is_err(),
            "{raw:?} should be invalid"
        );
    }
}
