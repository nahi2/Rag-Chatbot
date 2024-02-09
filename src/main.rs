mod confluence;
mod qdrant_setup;

use qdrant_client::prelude::*;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::error::Error;
use qdrant_client::qdrant::{UpsertPoints, VectorParams, VectorsConfig};
use qdrant_client::qdrant::vectors_config::Config;
use dotenv::{dotenv};
use serde_json::json;
use crate::confluence::{ConfluenceConfig, ConfluenceMeta};
use crate::qdrant_setup::QdrantConfig;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(create_collection).service(get_pages).service(chat)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[get("/create_collection")]
async fn create_collection() -> Result<HttpResponse, Box<dyn Error>> {
    let config = QdrantConfig::default();
    QdrantConfig::setup_collection(&config).await
}

#[get("/get_pages")]
async fn get_pages() -> Result<HttpResponse, Box<dyn Error>> {
    let qdrant_config = QdrantConfig::default();
    let client = QdrantClient::from_url(&qdrant_config.get_url()).build()?;

    let confluence_config = ConfluenceConfig::new();

    let pages_content = match confluence_config?.get_pages().await {
        Ok(content) => content,
        Err(e) => return Err(Box::new(e)),
    };

    for v in ConfluenceMeta::create_store(pages_content).await.iter(){
        let embeddings = ConfluenceMeta::get_embeddings(v.page_body.to_string()).await;

        client.upsert_points("memory".to_string(), None ,vec![PointStruct::new((&v.page_id).parse::<u64>().unwrap(), embeddings, json!({"test":"test"}).try_into().expect("json"))], None).await?;
    }

    Ok(HttpResponse::from(HttpResponse::Ok()))
}

#[post("/chat")]
pub async fn chat(req_body: String) -> Result<HttpResponse, Box<dyn Error>> {
    let qdrant_config = QdrantConfig::default();
    match qdrant_config.search_collection(req_body).await {
        Ok(response) => {
            let confluence_config = ConfluenceConfig::new();

            let pages_content = match confluence_config?.get_pages().await {
                Ok(content) => content,
                Err(e) => return Err(Box::new(e)),
            };

            for v in ConfluenceMeta::create_store(pages_content).await.iter(){
                for id in response{
                    if (v.page_id.parse::<u64>()) == Ok(id) {

                    }
                }
            }
        },
        Err(e) => {
            // Handle the error, possibly log it, and return a server error response
            eprintln!("Error in search_collection: {:?}", e); // Logging the error
            Ok(HttpResponse::InternalServerError().body("Internal server error"))
        }
    }
}