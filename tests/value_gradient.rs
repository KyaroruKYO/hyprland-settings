use hyprland_settings::value::gradient::GradientValue;

#[test]
fn gradient_value_accepts_color_lists_and_optional_angle() {
    for value in [
        "rgba(ffffffff)",
        "rgba(ffffffff) rgba(000000ff)",
        "#fff #000",
        "rgb(ff00aa) 0xff00aaff 45deg",
        "rgba(255,0,170,0.5) #000000 90deg",
        "rgba(ffffffff) -90deg",
    ] {
        let parsed = GradientValue::parse(value).expect("gradient value should parse");

        assert_eq!(parsed.serialize(), value);
    }
}

#[test]
fn gradient_value_rejects_invalid_colors_or_angles() {
    for value in [
        "",
        "45deg",
        "red",
        "rgba(ffffffff) red",
        "rgba(ffffffff) 45turn",
        "rgba(ffffffff) 45.5deg",
        "rgba(ffffffff) 45deg rgba(000000ff)",
        "rgba(ffffffff) 45deg 90deg",
        "rgba(ffffffff)\nrgba(000000ff)",
        "rgba(ffffffff) # comment",
        "rgba(ffffffff) rgba(000000ff) rgba(111111ff) rgba(222222ff) rgba(333333ff) rgba(444444ff) rgba(555555ff) rgba(666666ff) rgba(777777ff) rgba(888888ff) rgba(999999ff)",
        "$(cmd)",
    ] {
        assert!(
            GradientValue::parse(value).is_err(),
            "gradient value should reject {value:?}"
        );
    }
}
