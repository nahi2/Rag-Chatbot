use dotenv;
use std::{env, result};
use reqwest::header::ACCEPT;
use std::error::Error;
use std::sync::MutexGuard;
use scraper::Html;
use serde_json::{json, Result as SerdeResult, Value};

pub struct ConfluenceConfig {
    confluence_url: String,
    confluence_username:String,
    confluence_api_key:String,
}

impl ConfluenceConfig {
    pub fn new() -> Result<Self, ()> {
        dotenv::dotenv().ok();
        Ok(Self {
            confluence_url: env::var("CONFLUENCE_URL").map_err(|err|{
                eprintln!("Could not find confluence url")
            })?,
            confluence_username: env::var("CONFLUENCE_USERNAME").map_err(|err|{
                eprintln!("Could not find confluence url")
            })?,
            confluence_api_key: env::var("CONFLUENCE_API_KEY").map_err(|err|{
                eprintln!("Could not find confluence url")
            })?,
        })
    }

    pub async fn get_conf_pages(&self) -> Result<String, Box<dyn Error>> {
        let client = reqwest::Client::new();

        let res = client
            .get(&self.confluence_url)
            .basic_auth(&self.confluence_username, Some(&self.confluence_api_key))
            .header(ACCEPT, "application/json")
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            Ok(body)
        } else {
            Err(format!("Request failed with status: {}", res.status()).into())
        }
}
}

#[derive(Debug)]
pub struct ConfluenceMeta{
    pub page_id: String,
    pub page_title: String,
    pub page_body: String
}

impl ConfluenceMeta {
    pub fn convert_to_json(raw_pages: &str) -> Result<Value, serde_json::error::Error> {
        match serde_json::from_str(raw_pages) {
            Ok(pages) => Ok(pages),
            Err(e) => {
                eprintln!("Error converting pages: {}", e);
                Err(e)
            }
        }
    }
    pub fn build_store(json_pages: Value) -> Option<Vec<ConfluenceMeta>> {
        let mut store = Vec::new();
        let pages_array = json_pages["results"].as_array()?;

        for page in pages_array {
            let mut metadata = ConfluenceMeta {
                page_id: page["id"].as_str()?.to_string(),
                page_title: page["title"].as_str()?.to_string(),
                page_body: page["body"]["storage"]["value"].as_str()?.to_string(),
            };

            let document = Html::parse_document(&metadata.page_body);
            let body = document.root_element(); // This may not be correct, depending on your HTML parsing library
            metadata.page_body = body.text().collect::<Vec<_>>().join(" ").trim().to_string();

            store.push(metadata);
        }
        Some(store)
    }
}
