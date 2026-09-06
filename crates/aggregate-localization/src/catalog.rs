use crate::Language;
use fluent_bundle::{FluentArgs, FluentResource, concurrent::FluentBundle};

const KOREAN: &str = concat!(
    include_str!("../../../locales/ko-KR/interface.ftl"),
    "\n",
    include_str!("../../../locales/ko-KR/countries.ftl")
);
const ENGLISH: &str = concat!(
    include_str!("../../../locales/en-US/interface.ftl"),
    "\n",
    include_str!("../../../locales/en-US/countries.ftl")
);

pub struct Localization {
    language: Language,
    korean: FluentBundle<FluentResource>,
    english: FluentBundle<FluentResource>,
}

impl Localization {
    pub fn new(language: Language) -> Result<Self, String> {
        Ok(Self {
            language,
            korean: bundle(Language::Korean, KOREAN)?,
            english: bundle(Language::English, ENGLISH)?,
        })
    }

    pub fn language(&self) -> Language {
        self.language
    }
    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    pub fn text(&self, id: &str) -> Result<String, String> {
        self.format(id, None)
    }

    pub fn format(&self, id: &str, arguments: Option<&FluentArgs<'_>>) -> Result<String, String> {
        let selected = match self.language {
            Language::Korean => &self.korean,
            Language::English => &self.english,
        };
        let catalog = if selected.has_message(id) {
            selected
        } else {
            &self.english
        };
        let pattern = catalog
            .get_message(id)
            .and_then(|message| message.value())
            .ok_or_else(|| format!("missing translation: {id}"))?;
        let mut errors = vec![];
        let text = catalog
            .format_pattern(pattern, arguments, &mut errors)
            .into_owned();
        if errors.is_empty() {
            Ok(text)
        } else {
            Err(format!("translation {id}: {errors:?}"))
        }
    }
}

fn bundle(language: Language, source: &str) -> Result<FluentBundle<FluentResource>, String> {
    let resource = FluentResource::try_new(source.into())
        .map_err(|(_, errors)| format!("{}: {errors:?}", language.locale()))?;
    let mut bundle = FluentBundle::new_concurrent(vec![
        language
            .locale()
            .parse()
            .map_err(|error| format!("invalid locale: {error}"))?,
    ]);
    bundle
        .add_resource(resource)
        .map_err(|errors| format!("duplicate translations: {errors:?}"))?;
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    // All messages are top-level. Exercise formatting with the management variable contract
    // in both catalogs: missing or misspelled variables must fail, not render diagnostic text.
    fn keys(source: &str) -> BTreeSet<&str> {
        source
            .lines()
            .filter(|line| !line.starts_with(char::is_whitespace) && !line.starts_with('#'))
            .filter_map(|line| line.split_once('=').map(|(key, _)| key.trim()))
            .collect()
    }

    #[test]
    fn both_languages_have_matching_complete_messages() {
        assert_eq!(keys(KOREAN), keys(ENGLISH));
        for language in [Language::Korean, Language::English] {
            let catalog = Localization::new(language).unwrap();
            let mut arguments = FluentArgs::new();
            for name in [
                "day",
                "cost",
                "workers",
                "work",
                "inputs",
                "outputs",
                "facility",
                "building",
                "error",
                "province",
                "good",
                "amount",
                "production",
                "construction",
                "idle",
                "required",
                "available",
                "active",
                "requested",
                "consumed",
                "shortfall",
                "level",
                "provinces",
                "regions",
            ] {
                arguments.set(name, "7");
            }
            for key in keys(ENGLISH) {
                assert!(
                    !catalog.format(key, Some(&arguments)).unwrap().is_empty(),
                    "{key}"
                );
            }
        }
    }

    #[test]
    fn management_event_variables_are_required_and_rendered_in_both_languages() {
        for language in [Language::Korean, Language::English] {
            let catalog = Localization::new(language).unwrap();
            assert!(catalog.text("management-news-shortage").is_err());
            let mut arguments = FluentArgs::new();
            arguments.set("province", "North Valley");
            arguments.set("good", "Grain");
            arguments.set("amount", "13");
            let text = catalog
                .format("management-news-shortage", Some(&arguments))
                .unwrap();
            for value in ["North Valley", "Grain", "13"] {
                assert!(text.contains(value));
            }
        }
    }

    #[test]
    fn language_switch_and_missing_key_are_explicit() {
        let mut catalog = Localization::new(Language::Korean).unwrap();
        assert_eq!(catalog.text("menu-exit").unwrap(), "나가기");
        catalog.set_language(Language::English);
        assert_eq!(catalog.text("menu-exit").unwrap(), "Exit");
        assert!(catalog.text("nonexistent-key").is_err());
    }
}
