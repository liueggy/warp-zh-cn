use fluent_bundle::FluentArgs;
use warp_localization::{Localization, UiLanguage};
use warpui::{AppContext, Entity, SingletonEntity};

use crate::terminal::general_settings::GeneralSettings;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LocalizationEvent {
    LanguageChanged,
}

pub struct LocalizationModel {
    localization: Localization,
}

impl LocalizationModel {
    fn new(language: UiLanguage) -> Self {
        Self {
            localization: Localization::new(language)
                .expect("embedded localization catalogs must be valid"),
        }
    }

    pub fn language(&self) -> UiLanguage {
        self.localization.preference()
    }

    pub fn locale(&self) -> &unic_langid::LanguageIdentifier {
        self.localization.locale()
    }

    pub fn text(&self, id: &str, args: Option<&FluentArgs<'_>>) -> String {
        self.localization.text(id, args)
    }
}

impl Entity for LocalizationModel {
    type Event = LocalizationEvent;
}

impl SingletonEntity for LocalizationModel {}

pub fn init(ctx: &mut AppContext) {
    let language = *GeneralSettings::as_ref(ctx).language.value();
    let localization = ctx.add_singleton_model(move |_| LocalizationModel::new(language));

    ctx.subscribe_to_model(&GeneralSettings::handle(ctx), move |settings, _, _, ctx| {
        let language = *settings.as_ref(ctx).language.value();
        let changed = localization.read(ctx, |model, _| model.language() != language);
        if !changed {
            return;
        }

        localization.update(ctx, |model, ctx| {
            *model = LocalizationModel::new(language);
            ctx.emit(LocalizationEvent::LanguageChanged);
        });
        ctx.invalidate_all_views();
    });
}

pub fn text(id: &str, ctx: &AppContext) -> String {
    LocalizationModel::as_ref(ctx).text(id, None)
}

pub fn text_with_args(id: &str, args: &FluentArgs<'_>, ctx: &AppContext) -> String {
    LocalizationModel::as_ref(ctx).text(id, Some(args))
}
