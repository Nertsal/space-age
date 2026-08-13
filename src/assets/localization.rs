use super::*;

type Id = String;

pub struct Localization {
    records: HashMap<Id, Item>,
}

#[derive(Debug, Clone, Copy)]
pub enum Language {
    English,
    Spanish,
}

#[derive(Debug, Deserialize)]
struct Item {
    en: String,
    es: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Record {
    id: Id,
    #[serde(flatten)]
    item: Item,
}

impl Localization {
    pub fn get(&self, id: &str, language: Language) -> &str {
        let Some(record) = self.records.get(id) else {
            log::error!("missing localization text for key: {:?}", id);
            return "<OOPS missing text>";
        };
        match language {
            Language::English => &record.en,
            Language::Spanish => record.es.as_ref().unwrap_or(&record.en), // English fallback
        }
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
