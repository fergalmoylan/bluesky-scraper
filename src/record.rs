use atrium_api::app::bsky::feed::post::Record;
use locale_codes::language::lookup;
use regex::Regex;
use sentiment::analyze;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordSentiment {
    pub score: f32,
    pub comparative_score: f32,
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
    sentiment: RecordSentiment,
}

impl TransformedRecord {
    fn calculate_sentiment(text: String) -> RecordSentiment {
        let analysis = analyze(text.clone());
        let score = analysis.score;
        let comparative_score = analysis.comparative;
        let sentiment = if score > 0.0 {
            String::from("positive")
        } else if score < 0.0 {
            String::from("negative")
        } else {
            String::from("neutral")
        };
        RecordSentiment {
            score,
            comparative_score,
            sentiment,
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

    pub fn from_original(record: Record) -> Self {
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
        let sentiment = Self::calculate_sentiment(text.clone());
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
