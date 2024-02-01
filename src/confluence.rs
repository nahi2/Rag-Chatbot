use std::env;
use std::error::Error;
use dotenv::{dotenv};
use reqwest::header::ACCEPT;
use scraper::Html;
use serde_json::{json, Result as SerdeResult, Value};

pub struct ConfluenceConfig {
    url: String,
    username: String,
    password: String,
    open_ai_key: String
}

impl ConfluenceConfig {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        Ok(Self{
            url: env::var("CONFLUENCE_URL")?,
            username: env::var("CONFLUENCE_USERNAME")?,
            password: env::var("CONFLUENCE_API_KEY")?,
            open_ai_key: env::var("OPEN_AI_KEY")?,
        })
    }

    pub async fn get_pages(&self) -> Result<String, reqwest::Error> {
        let client = reqwest::Client::new();

        let res = client
            .get(&self.url)
            .basic_auth(&self.username, Some(&self.password))
            .header(ACCEPT, "application/json")
            .send()
            .await?;

        let body = res.text().await?;

        Ok(body)
    }
}

#[derive(Debug)]
pub struct ConfluenceMeta{
    page_id: String,
    page_title: String,
    page_body: String
}

impl ConfluenceMeta{
    pub async fn create_store(message: String) -> Vec<ConfluenceMeta> {
        let v: Value = serde_json::from_str(&message).expect("message");
        let mut store: Vec<ConfluenceMeta> = Vec::new();

        for page in v["results"].as_array().expect("results"){
            let mut metadata: ConfluenceMeta = ConfluenceMeta{
                page_id: "id".to_string(),
                page_title: "title".to_string(),
                page_body: "body".to_string(),
            };

            match page["id"].as_str() {
                Some(v) => {
                    metadata.page_id = v.to_string();
                },
                None => {}
            };

            match page["title"].as_str() {
                Some(v) => {
                    metadata.page_title = v.to_string();
                },
                None => {}
            };

            match page["body"]["storage"]["value"].as_str() {
                Some(v) => {
                    let document = Html::parse_document(v);
                    let body = document.root_element();
                    metadata.page_body=body.text().collect::<Vec<_>>().join(" ").trim().to_string();;
                },
                None => {}
            };
            store.push(metadata)
        };
        store
    }
    pub async fn get_embeddings(pages_content: String) -> Result<Vec<f64>, reqwest::Error>{
        let confluence_config = ConfluenceConfig::new().expect("keys set");
        let client = reqwest::Client::new();

        let client = reqwest::Client::new();
        let response = client.post("https://api.openai.com/v1/embeddings")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", confluence_config.open_ai_key))
            .json(&json!({
            "input": pages_content,
            "model": "text-embedding-3-small"
        }))
            .send()
            .await?;

        let mut embedding: Vec<f64> = Vec::new();
        let v: Value = serde_json::from_str(&response.text().await?).expect("message");
        for &item in v["data"].as_array().expect("array").get(0).iter(){
             embedding = if let Value::Array(items) = &item["embedding"] {
                items.iter().filter_map(|item| {
                    match item {
                        Value::Number(num) => num.as_f64(),
                        _ => None,
                    }
                }).collect::<Vec<f64>>()
            } else {
                vec![]
            };
        }
        println!("{:?}", embedding);
        Ok(embedding)
    }
}

