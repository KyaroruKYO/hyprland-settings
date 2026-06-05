use anyhow::Result;
use hyprland_settings::source_values::{parse_xkb_rules_lst, read_system_xkb_rules};

#[test]
fn xkb_rules_lst_parser_extracts_source_backed_values() {
    let rules = parse_xkb_rules_lst(
        r#"
! model
  pc105           Generic 105-key PC

! layout
  us              English (US)
  de              German

! variant
  intl            us: English (US, intl., with dead keys)
  nodeadkeys      de: German (no dead keys)

! option
  grp:alt_shift_toggle Alt+Shift
  ctrl:nocaps     Caps Lock as Ctrl
"#,
    );

    assert!(rules.has_model("pc105"));
    assert!(rules.has_layout("us"));
    assert!(rules.has_layout("de"));
    assert!(rules.has_variant("intl"));
    assert!(rules.has_option("grp:alt_shift_toggle"));
    assert!(!rules.has_layout("definitely-not-a-layout"));
    assert!(rules.validates_setting_value("input.kb_model", "pc105"));
    assert!(rules.validates_setting_value("input.kb_layout", "us,de"));
    assert!(rules.validates_setting_value("input.kb_variant", "intl"));
    assert!(rules.validates_setting_value("input.kb_options", "grp:alt_shift_toggle,ctrl:nocaps"));
    assert!(rules.validates_setting_value("input.kb_rules", "evdev"));
    assert!(!rules.validates_setting_value("input.kb_layout", "__not_a_layout__"));
    assert!(!rules.validates_setting_value("input.kb_options", "grp:alt_shift_toggle\nexec bad"));

    let intl = rules
        .variants
        .iter()
        .find(|variant| variant.raw_value == "intl")
        .expect("intl variant should be parsed");
    assert_eq!(intl.parent_layout.as_deref(), Some("us"));
    assert_eq!(intl.label, "English (US, intl., with dead keys)");
}

#[test]
fn system_xkb_rules_are_read_without_mutation() -> Result<()> {
    let rules = read_system_xkb_rules()?;

    assert!(rules
        .source_path
        .as_ref()
        .is_some_and(|path| path.is_file()));
    assert!(rules.models.len() > 10);
    assert!(rules.layouts.len() > 10);
    assert!(rules.variants.len() > 10);
    assert!(rules.options.len() > 10);

    assert!(rules.has_layout("us"));
    assert!(rules.has_model("pc105"));
    assert!(rules.has_option("grp:alt_shift_toggle"));

    Ok(())
}
