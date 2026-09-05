use crate::Language;
use fluent_bundle::{FluentArgs, FluentResource, concurrent::FluentBundle};

const KOREAN: &str = include_str!("../../../locales/ko-KR/interface.ftl");
const ENGLISH: &str = include_str!("../../../locales/en-US/interface.ftl");

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

    // This catalog uses only top-level, argument-free messages. The formatter below verifies
    // every message, not just parsing. Extend this contract when parameterized messages land.
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
            for key in keys(ENGLISH) {
                assert!(!catalog.text(key).unwrap().is_empty(), "{key}");
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
