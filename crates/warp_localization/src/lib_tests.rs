use fluent_bundle::FluentArgs;
use unic_langid::langid;

use super::*;

#[test]
fn resolves_chinese_system_locales_to_simplified_chinese() {
    assert_eq!(resolve_system_locale(Some("zh-CN")), langid!("zh-CN"));
    assert_eq!(resolve_system_locale(Some("zh-Hans")), langid!("zh-CN"));
    assert_eq!(resolve_system_locale(Some("zh_CN")), langid!("zh-CN"));
}

#[test]
fn resolves_unknown_or_missing_system_locales_to_english() {
    assert_eq!(resolve_system_locale(Some("fr-FR")), langid!("en-US"));
    assert_eq!(resolve_system_locale(Some("zh-TW")), langid!("en-US"));
    assert_eq!(resolve_system_locale(Some("zh-Hant")), langid!("en-US"));
    assert_eq!(resolve_system_locale(Some("not a locale")), langid!("en-US"));
    assert_eq!(resolve_system_locale(None), langid!("en-US"));
}

#[test]
fn catalogs_have_matching_keys() {
    assert_eq!(validate_catalogs(), Ok(()));
}

#[test]
fn formats_arguments_in_both_languages() {
    let mut args = FluentArgs::new();
    args.set("name", "Warp");

    let english = Localization::new(UiLanguage::EnUs).unwrap();
    assert_eq!(
        english.text("welcome-user", Some(&args)),
        "Welcome to Warp"
    );

    let chinese = Localization::new(UiLanguage::ZhCn).unwrap();
    assert_eq!(chinese.text("welcome-user", Some(&args)), "欢迎使用 Warp");
}

#[test]
fn unknown_keys_are_visible_during_development() {
    let localization = Localization::new(UiLanguage::ZhCn).unwrap();
    assert_eq!(localization.text("missing-key", None), "missing-key");
}
