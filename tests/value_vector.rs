use anyhow::Result;
use hyprland_settings::value::vector::Vec2Value;

#[test]
fn vec2_parser_accepts_source_backed_space_forms() -> Result<()> {
    let space = Vec2Value::parse("10 20")?;
    let whitespace = Vec2Value::parse("  10   20  ")?;

    assert_eq!(space.serialize(), "10 20");
    assert_eq!(whitespace.serialize(), "10 20");

    Ok(())
}

#[test]
fn vec2_parser_accepts_finite_float_components() -> Result<()> {
    let value = Vec2Value::parse("-1.5 2.25")?;

    assert_eq!(value.serialize(), "-1.5 2.25");

    Ok(())
}

#[test]
fn vec2_parser_rejects_invalid_values() {
    for raw in [
        "", "10", "10 20 30", "10,20", "10,20,30", "10,", "nan 1", "inf 1",
    ] {
        assert!(Vec2Value::parse(raw).is_err(), "{raw:?} should be invalid");
    }
}
