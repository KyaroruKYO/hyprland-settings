use hyprland_settings::value::color::ColorValue;

#[test]
fn color_value_accepts_supported_hyprland_color_literals() {
    for value in [
        "#f0a",
        "#ff00aa",
        "#ff00aaff",
        "rgb(ff00aa)",
        "rgb(255,0,170)",
        "rgba(ff00aaff)",
        "rgba(255,0,170,0.5)",
        "0xff00aaff",
        "16711935",
    ] {
        let parsed = ColorValue::parse(value).expect("color literal should parse");

        assert_eq!(parsed.serialize(), value);
    }
}

#[test]
fn color_value_rejects_invalid_color_literals() {
    for value in [
        "",
        "red",
        "#ff",
        "#ff00aazz",
        "rgb(ffff)",
        "rgb(256,0,0)",
        "rgba(ff00aaff00)",
        "rgba(255,0,0,2)",
        "0xzz00aaff",
        "4294967296",
    ] {
        assert!(
            ColorValue::parse(value).is_err(),
            "color value should reject {value:?}"
        );
    }
}
