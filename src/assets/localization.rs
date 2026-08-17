use super::*;

use enum_iterator::Sequence;

type Id = String;

pub struct Localization {
    records: HashMap<Id, Item>,
}

#[derive(Sequence, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum Language {
    #[default]
    English,
    Spanish,
    // Finnish,
    Russian,
}

impl Language {
    pub fn native(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
            // Language::Finnish => "Finnish",
            Language::Russian => "Русский",
        }
    }
}

#[derive(Debug, Deserialize)]
struct Item {
    en: String,
    es: Option<String>,
    // fi: Option<String>,
    ru: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Record {
    id: Id,
    #[serde(flatten)]
    item: Item,
}

impl Localization {
    pub fn get(&self, id: &str, language: Language) -> &str {
        self.records
            .get(id)
            .map(|record| {
                fn fallback<'a>(t: Option<&'a str>, fallback: &'a str) -> &'a str {
                    t.filter(|t| !t.is_empty()).unwrap_or(fallback)
                }
                match language {
                    Language::English => record.en.as_str(),
                    Language::Spanish => fallback(record.es.as_deref(), &record.en),
                    // Language::Finnish => fallback(record.fi.as_deref(), &record.en),
                    Language::Russian => fallback(record.ru.as_deref(), &record.en),
                }
            })
            .unwrap_or_else(|| {
                log::error!("missing localization text for key: {:?}", id);
                "<OOPS missing text>"
            })
    }
}

impl geng::asset::Load for Localization {
    type Options = ();

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        _options: &(),
    ) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        async move {
            let bytes = manager.load_bytes(path).await?;
            let reader = csv::ReaderBuilder::new().from_reader(&bytes[..]);
            let mut records = HashMap::new();
            for record in reader.into_deserialize::<Record>() {
                match record {
                    Err(err) => log::error!("CSV record failed to parse: {:?}", err),
                    Ok(record) => match records.entry(record.id) {
                        std::collections::hash_map::Entry::Occupied(entry) => {
                            log::error!("Duplicate ID found in localization: {:?}", entry.key());
                        }
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            log::trace!("loaded {:?}: {:?}", entry.key(), record.item);
                            entry.insert(record.item);
                        }
                    },
                }
            }
            Ok(Localization { records })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("csv");
}
