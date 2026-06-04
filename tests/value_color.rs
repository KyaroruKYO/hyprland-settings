use hyprland_settings::value::color::ColorValue;

#[test]
fn color_value_accepts_supported_hyprland_color_literals() {
    for value in ["rgb(ff00aa)", "rgba(ff00aaff)", "0xff00aaff"] {
        let parsed = ColorValue::parse(value).expect("color literal should parse");

        assert_eq!(parsed.serialize(), value);
    }
}

#[test]
fn color_value_rejects_invalid_color_literals() {
    for value in ["", "red", "rgb(ffff)", "rgba(ff00aaff00)", "0xzz00aaff"] {
        assert!(
            ColorValue::parse(value).is_err(),
            "color value should reject {value:?}"
        );
    }
}
