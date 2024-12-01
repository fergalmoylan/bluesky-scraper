use std::sync::Arc;
use atrium_api::app::bsky::feed::post::Record;
use locale_codes::language::lookup;
use log::error;
use regex::Regex;
use rust_bert::pipelines::ner::NERModel;
use rust_bert::pipelines::ner::Entity;
use rust_bert::pipelines::sentiment::{SentimentModel, SentimentPolarity};
use serde::{Deserialize, Serialize};

pub(crate) struct RustBertModels {
    sentiment_model: SentimentModel,
    ner_model: NERModel,
}

impl RustBertModels {
    pub fn new() -> Self {
        let sentiment_model = SentimentModel::new(Default::default())
            .expect("Failed to create SentimentModel");
        let ner_model = NERModel::new(Default::default())
            .expect("Failed to create NERModel");
        Self {
            sentiment_model,
            ner_model,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordNER {
    pub name_entity: String,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordSentiment {
    pub score: f64,
    pub sentiment: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TransformedRecord {
    created_at: String,
    text: String,
    languages: Vec<String>,
    hashtags: Vec<String>,
    urls: Vec<String>,
    hostnames: Vec<String>,
    sentiment: Option<RecordSentiment>,
}

impl TransformedRecord {

    fn calculate_ner(text: &str, model: &NERModel) -> Option<Vec<String>> {
        let input = [text];
        let output = model.predict(&input);

        if let Some(entities) = output.first() {
            Some(
                entities
                    .iter()
                    .map(|entity| entity.word.clone())
                    .collect(),
            )
        } else {
            error!("No NER data available.");
            None
        }
    }

     fn calculate_sentiment(text: &str, model: &SentimentModel) -> Option<RecordSentiment> {
         let input = [text];
         let output = model.predict(input);

         if let Some(sentiment) = output.first() {
             Some(RecordSentiment {
                 score: sentiment.score,
                 sentiment: match sentiment.polarity {
                     SentimentPolarity::Positive => "Positive".to_string(),
                     SentimentPolarity::Negative => "Negative".to_string(),
                 },
             })
         } else {
             error!("No sentiment data available.");
             None
         }
    }

    fn convert_lang_codes(lang_codes: &[String]) -> Vec<String> {
        let re = Regex::new(r"^([a-z]{2,3})").unwrap();
        lang_codes
            .iter()
            .filter_map(|lang_code| {
                let short_code = re
                    .captures(lang_code)
                    .and_then(|cap| cap.get(1).map(|m| m.as_str()))
                    .unwrap_or(lang_code);
                let truncated_code = &short_code[..short_code.len().min(3)];
                lookup(truncated_code).map(|lang| lang.reference_name.clone())
            })
            .collect()
    }

    fn extract_hashtags(text: &str) -> Vec<String> {
        let hashtag_regex = Regex::new(r"#\w+").unwrap();
        hashtag_regex
            .find_iter(text)
            .map(|mat| mat.as_str().to_string())
            .collect()
    }

    fn extract_urls(text: &str) -> Vec<String> {
        let url_regex =
            Regex::new(r"(?i)\b(?:[a-z][\w+.-]*://[^\s/$.?#].[^\s]*|localhost(?::\d{1,5})?/?)\b")
                .unwrap();
        url_regex
            .find_iter(text)
            .map(|mat| mat.as_str().to_string())
            .collect()
    }

    fn extract_hosts(urls: Vec<String>) -> Vec<String> {
        let host_regex =
            Regex::new(r"(?i)^(?:[a-z][\w+.-]*://)?(?:[^@/\n]+@)?([^:/?\n]+)").unwrap();
        urls.iter()
            .filter_map(|url| {
                host_regex
                    .captures(url)
                    .and_then(|caps| caps.get(1).map(|host| host.as_str().to_string()))
            })
            .collect()
    }

    pub fn from_original(record: Record, models: &RustBertModels) -> Self {
        let sentiment_model = &models.sentiment_model;
        let ner_model = &models.ner_model;
        let created_at = record.created_at.as_str().to_string();
        let text = record.text.replace("\n", "");
        let lang_codes: Vec<String> = record.langs.as_ref().map_or(Vec::new(), |langs| {
            langs
                .iter()
                .map(|lang| lang.as_ref().as_str().to_string())
                .collect()
        });
        let hashtags = Self::extract_hashtags(&text);
        let urls = Self::extract_urls(&text);
        let hostnames = Self::extract_hosts(urls.clone());
        let languages = Self::convert_lang_codes(&lang_codes);
        let sentiment = Self::calculate_sentiment(text.as_str(), sentiment_model);
        let ner_entities = Self::calculate_ner(text.as_str(), ner_model).unwrap();
        //println!("{:#?}\n", ner_entities);
        TransformedRecord {
            created_at,
            text,
            languages,
            hashtags,
            urls,
            hostnames,
            sentiment,
        }
    }
}
