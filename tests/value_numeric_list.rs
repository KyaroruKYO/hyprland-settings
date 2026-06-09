use hyprland_settings::value::numeric_list::NumericListValue;

#[test]
fn numeric_list_accepts_scroll_point_shape() {
    let parsed = NumericListValue::parse("0.2 0.0 0.5 1 1.2 1.5")
        .expect("scroll points numeric list should parse");

    assert_eq!(parsed.serialize(), "0.2 0.0 0.5 1 1.2 1.5");
    assert_eq!(parsed.numbers().len(), 6);
}

#[test]
fn numeric_list_rejects_invalid_scroll_point_shape() {
    for value in [
        "",
        "0.2",
        "0 0.5 1",
        "-0.2 0.5 1",
        "0.2 nope",
        "0.2 NaN",
        "0.2 inf",
        "0.2 0.5 # comment",
        "0.2\n0.5",
        "$(cmd)",
    ] {
        assert!(
            NumericListValue::parse(value).is_err(),
            "numeric list should reject {value:?}"
        );
    }
}
