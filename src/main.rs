mod confluence;

use qdrant_client::prelude::*;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::error::Error;
use reqwest::StatusCode;
use crate::confluence::{ConfluenceConfig, ConfluenceMeta};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(create_collection).service(get_pages)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[post("/create_collection")]
async fn create_collection(req_body: String) -> Result<HttpResponse, Box<dyn Error>> {
    let client = QdrantClient::from_url("http://localhost:6334").build()?;

    match client.create_collection(&CreateCollection {
        collection_name: req_body.into(),
        hnsw_config: None,
        wal_config: None,
        optimizers_config: None,
        shard_number: None,
        on_disk_payload: None,
        timeout: None,
        vectors_config: None,
        replication_factor: None,
        write_consistency_factor: None,
        init_from_collection: None,
        quantization_config: None,
        sharding_method: None,
        sparse_vectors_config: None,
    }).await {
        Ok(_) => Ok(HttpResponse::Ok().body("Collection Successfully Created")),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Error: {}", e))),
    }
}

#[get("/get_pages")]
async fn get_pages() -> Result<HttpResponse, Box<dyn Error>> {
    let confluence_config = ConfluenceConfig::new();

    let pages_content = match confluence_config?.get_pages().await {
        Ok(content) => content,
        Err(e) => return Err(Box::new(e)),
    };

    ConfluenceMeta::get_embeddings(pages_content).await?;

    Ok(HttpResponse::from(HttpResponse::Ok()))
}
