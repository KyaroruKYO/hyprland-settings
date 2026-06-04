use hyprland_settings::value::regex_value::RegexValue;

#[test]
fn regex_value_accepts_line_safe_patterns_without_executing_them() {
    for value in ["^(Alacritty|kitty)$", "firefox", "class:.*term.*"] {
        let parsed = RegexValue::parse(value).expect("regex-like text should parse");

        assert_eq!(parsed.serialize(), value);
    }
}

#[test]
fn regex_value_rejects_config_breaking_or_command_like_text() {
    for value in ["", "a\nb", "a # b", "`cmd`", "$(cmd)"] {
        assert!(
            RegexValue::parse(value).is_err(),
            "regex value should reject {value:?}"
        );
    }
}
