use std::borrow::Cow;
use std::collections::HashSet;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use fluent_syntax::ast;
use serde::{Deserialize, Serialize};
use unic_langid::{LanguageIdentifier, langid};

const EN_US_SOURCE: &str = include_str!("../locales/en-US/core.ftl");
const ZH_CN_SOURCE: &str = include_str!("../locales/zh-CN/core.ftl");

const EN_US: LanguageIdentifier = langid!("en-US");
const ZH_CN: LanguageIdentifier = langid!("zh-CN");

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[serde(rename_all = "kebab-case")]
#[schemars(
    description = "The language used by the Warp user interface.",
    rename_all = "kebab-case"
)]
pub enum UiLanguage {
    #[default]
    System,
    EnUs,
    ZhCn,
}

impl UiLanguage {
    pub fn resolve(self) -> LanguageIdentifier {
        match self {
            Self::System => resolve_system_locale(sys_locale::get_locale().as_deref()),
            Self::EnUs => EN_US,
            Self::ZhCn => ZH_CN,
        }
    }
}

pub struct Localization {
    preference: UiLanguage,
    locale: LanguageIdentifier,
    primary: FluentBundle<FluentResource>,
    fallback: FluentBundle<FluentResource>,
}

impl Localization {
    pub fn new(preference: UiLanguage) -> Result<Self, Vec<String>> {
        let locale = preference.resolve();
        let fallback = bundle(EN_US, EN_US_SOURCE)?;
        let primary = if locale == ZH_CN {
            bundle(ZH_CN, ZH_CN_SOURCE)?
        } else {
            bundle(EN_US, EN_US_SOURCE)?
        };

        Ok(Self {
            preference,
            locale,
            primary,
            fallback,
        })
    }

    pub fn preference(&self) -> UiLanguage {
        self.preference
    }

    pub fn locale(&self) -> &LanguageIdentifier {
        &self.locale
    }

    pub fn text(&self, id: &str, args: Option<&FluentArgs<'_>>) -> String {
        format_message(&self.primary, id, args)
            .or_else(|| format_message(&self.fallback, id, args))
            .unwrap_or_else(|| id.to_owned())
    }
}

pub fn resolve_system_locale(locale: Option<&str>) -> LanguageIdentifier {
    let Some(locale) = locale else {
        return EN_US;
    };
    let normalized = locale.replace('_', "-").to_ascii_lowercase();
    if normalized == "zh"
        || normalized.starts_with("zh-cn")
        || normalized.starts_with("zh-sg")
        || normalized.starts_with("zh-hans")
    {
        ZH_CN
    } else {
        EN_US
    }
}

pub fn validate_catalogs() -> Result<(), Vec<String>> {
    let english = message_ids(EN_US_SOURCE)?;
    let chinese = message_ids(ZH_CN_SOURCE)?;
    let mut errors = Vec::new();

    for missing in english.difference(&chinese) {
        errors.push(format!("zh-CN is missing message `{missing}`"));
    }
    for extra in chinese.difference(&english) {
        errors.push(format!("zh-CN contains unknown message `{extra}`"));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn bundle(
    locale: LanguageIdentifier,
    source: &str,
) -> Result<FluentBundle<FluentResource>, Vec<String>> {
    let resource = FluentResource::try_new(source.to_owned())
        .map_err(|(_, errors)| errors.into_iter().map(|error| error.to_string()).collect())?;
    let mut bundle = FluentBundle::new_concurrent(vec![locale]);
    bundle
        .add_resource(resource)
        .map_err(|errors| errors.into_iter().map(|error| error.to_string()).collect())?;
    Ok(bundle)
}

fn format_message(
    bundle: &FluentBundle<FluentResource>,
    id: &str,
    args: Option<&FluentArgs<'_>>,
) -> Option<String> {
    let pattern = bundle.get_message(id)?.value()?;
    let mut errors = Vec::new();
    let value: Cow<'_, str> = bundle.format_pattern(pattern, args, &mut errors);
    Some(value.into_owned())
}

fn message_ids(source: &str) -> Result<HashSet<String>, Vec<String>> {
    let resource = fluent_syntax::parser::parse(source)
        .map_err(|(_, errors)| errors.into_iter().map(|error| error.to_string()).collect())?;
    Ok(resource
        .body
        .into_iter()
        .filter_map(|entry| match entry {
            ast::Entry::Message(message) => Some(message.id.name.to_owned()),
            _ => None,
        })
        .collect())
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
