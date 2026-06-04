use anyhow::Result;
use hyprland_settings::value::vector::Vec2Value;

#[test]
fn vec2_parser_accepts_space_and_comma_forms() -> Result<()> {
    let space = Vec2Value::parse("10 20")?;
    let comma = Vec2Value::parse("10,20")?;
    let whitespace = Vec2Value::parse("  10   20  ")?;

    assert_eq!(space.serialize(), "10 20");
    assert_eq!(comma.serialize(), "10,20");
    assert_eq!(whitespace.serialize(), "10 20");

    Ok(())
}

#[test]
fn vec2_parser_accepts_finite_float_components() -> Result<()> {
    let value = Vec2Value::parse("-1.5,2.25")?;

    assert_eq!(value.serialize(), "-1.5,2.25");

    Ok(())
}

#[test]
fn vec2_parser_rejects_invalid_values() {
    for raw in ["", "10", "10 20 30", "10,20,30", "10,", "nan 1", "inf 1"] {
        assert!(Vec2Value::parse(raw).is_err(), "{raw:?} should be invalid");
    }
}
