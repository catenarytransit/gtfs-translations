use std::collections::HashMap;
use std::collections::HashSet;
use language_tags::LanguageTag;
use serde::{Deserialize, Serialize};
use derivative::Derivative;
use std::error::Error;

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum RecordIdTypes {
    RecordSubId((String, String)),
    RecordId(String)
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum TranslatableField {
    Agency(AgencyFields),
    Areas(AreaFields),
    Calendar(CalendarFields),
    FareProducts(FareProductFields),
    FeedInfo(FeedInfoFields),
    Routes(RouteFields),
    StopTimes(StopTimeFields),
    Stops(StopFields),
    Trips(TripFields),
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum TranslationKey {
    Record(String),
    RecordSub((String, String)),
    Value(String),
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct TranslationLookup {
    pub language: LanguageTag,
    pub field: TranslatableField,
    pub key: TranslationKey,
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum StopTimeFields {
    Headsign,
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum RouteFields {
    Desc,
    LongName,
    ShortName,
    Url,
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum CalendarFields {
    ServiceId,
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum FeedInfoFields {
    PublisherName,
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum AreaFields {
    Name,
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum AgencyFields {
    Name,
    FareUrl,
    Url,
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum FareProductFields {
    ProductName,
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum TripFields {
    Headsign,
    ShortName
}

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
pub enum StopFields {
    Code,
    Name,
    TtsName,
    PlatformCode,
    Desc,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TranslationResult {
    pub avaliable_languages: Vec<LanguageTag>,
    pub translations: HashMap<TranslationLookup, String>,
    pub possible_translations: Vec<(TranslatableField, LanguageTag)>,
}

pub fn table_and_field_to_enum(table_name: &str, field_name: &str) -> Option<TranslatableField> {
    match table_name {
        "agency" => {
            match field_name {
                "agency_name" => Some(TranslatableField::Agency(AgencyFields::Name)),
                "agency_url" => Some(TranslatableField::Agency(AgencyFields::Url)),
                "agency_fare_url" => Some(TranslatableField::Agency(AgencyFields::FareUrl)),
                _ => None
              }
        },
        "areas" => {
            match field_name {
                "area_name" => Some(TranslatableField::Areas(AreaFields::Name)),
                _ => None
              }
        },
        "routes" => {
            match field_name {
                "route_long_name" => Some(TranslatableField::Routes(RouteFields::LongName)),
                "route_short_name" => Some(TranslatableField::Routes(RouteFields::ShortName)),
                "route_url" => Some(TranslatableField::Routes(RouteFields::Url)),
                _ => None
              }
        },
        "stop_times" => {
            match field_name {
                "stop_headsign" => Some(TranslatableField::StopTimes(StopTimeFields::Headsign)),
                _ => None
              }
        },
        "stops" => {
            match field_name {
                "stop_code" => Some(TranslatableField::Stops(StopFields::Code)),
                "stop_name" => Some(TranslatableField::Stops(StopFields::Name)),
                "tts_stop_name" => Some(TranslatableField::Stops(StopFields::TtsName)),
                "stop_desc" => Some(TranslatableField::Stops(StopFields::Desc)),
                "platform_code" => Some(TranslatableField::Stops(StopFields::PlatformCode)),
                _ => None
            }
        },
        "trips" => {
            match field_name {
                "trip_headsign" => Some(TranslatableField::Trips(TripFields::Headsign)),
                "trip_short_name" => Some(TranslatableField::Trips(TripFields::ShortName)),
                _ => None
            }
        },
        "calendar" => {
            match field_name {
                "service_id" => Some(TranslatableField::Calendar(CalendarFields::ServiceId)),
                _ => None
            }
        },
        "fare_products" => {
            match field_name {
                "fare_product_name" => Some(TranslatableField::FareProducts(FareProductFields::ProductName)),
                _ => None
            }
        },
        "feed_info" => {
            match field_name {
                "feed_publisher_name" => Some(TranslatableField::FeedInfo(FeedInfoFields::PublisherName)),
            _ => None
            }
        }
        _ => None
    }
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RawTranslation {
    pub table_name: String,
    pub field_name: String,
    pub language: String,
    pub translation: String,
    pub record_id: Option<String>,
    pub record_sub_id: Option<String>,
    pub field_value: Option<String>,
}

fn key_options_to_struct(record_id: Option<String>, record_sub_id: Option<String>, field_value: Option<String>) -> Option<TranslationKey> {
    //https://gtfs.org/schedule/reference/#translationstxt
    //If both referencing methods (record_id, record_sub_id) and field_value are used to translate the same value in 2 different rows, the translation provided with (record_id, record_sub_id) takes precedence.
    match (record_id, record_sub_id, field_value) {
        (Some(record_id), Some(record_sub_id), _) => Some(TranslationKey::RecordSub((record_id, record_sub_id))),
        (Some(record_id), _, _) => Some(TranslationKey::Record(record_id)),
        (_, _, Some(field_value)) => Some(TranslationKey::Value(field_value)),
        _ => None
    } 
}
pub fn translate_raw_translations(raw_translations: Vec<RawTranslation>) -> TranslationResult {
    let mut res:HashMap<TranslationLookup, String> = HashMap::new();
        let mut possible_translations:HashSet<(TranslatableField, LanguageTag)> = HashSet::new();

        for row in raw_translations {
            if let Ok(language_tag) = LanguageTag::parse(row.language.as_str()) {
            if let Some(field) = table_and_field_to_enum(row.table_name.as_str(), row.field_name.as_str()) {
                if let Some(key) = key_options_to_struct(row.record_id, row.record_sub_id, row.field_value) {
                    res.insert(TranslationLookup {
                        language: language_tag.clone(),
                        field: field.clone(),
                        key: key
                    }, row.translation);
                    possible_translations.insert((field, language_tag));
                }
            }

            }
        }

        let possible_translations = possible_translations.into_iter().collect::<Vec<(TranslatableField, LanguageTag)>>();
        let mut avaliable_languages: HashSet<LanguageTag> = HashSet::new();

        for summary_item in possible_translations.iter() {
           avaliable_languages.insert(summary_item.1.clone());
        }

        let avaliable_languages = avaliable_languages.into_iter().collect::<Vec<LanguageTag>>();

        TranslationResult {
            avaliable_languages: avaliable_languages,
            possible_translations: possible_translations,
            translations: res
        }
}

pub fn translation_csv_text_to_translations(data: &str) -> Result<TranslationResult, Box<dyn std::error::Error>> {
    let mut rdr = csv::Reader::from_reader(data.as_bytes());
    let mut iter = rdr.into_deserialize::<RawTranslation>();

    let mut pre_translations: Vec<RawTranslation> = vec![];

    for row in iter {
        if let Ok(row) = row {
            pre_translations.push(row);
        }
    }

    Ok(translate_raw_translations(pre_translations))
}

#[cfg(test)]
mod tests {
    use super::*;
}
