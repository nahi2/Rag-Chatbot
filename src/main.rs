mod confluence;

use qdrant_client::prelude::*;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::error::Error;
use qdrant_client::qdrant::{UpsertPoints, VectorParams, VectorsConfig};
use qdrant_client::qdrant::vectors_config::Config;
use serde_json::json;
use crate::confluence::{ConfluenceConfig, ConfluenceMeta};


#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(create_collection).service(get_pages).service(search_pages)
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
        vectors_config: Some(VectorsConfig {
            config: Some(Config::Params(VectorParams {
                size: 1536,
                distance: Distance::Cosine.into(),
                ..Default::default()
            })),
        }),
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
    let client = QdrantClient::from_url("http://localhost:6334").build()?;

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

#[post("/search_pages")]
async fn search_pages(req_body: String) -> Result<HttpResponse, Box<dyn Error>> {
    let client = QdrantClient::from_url("http://localhost:6334").build()?;

    let db_response = client.search_points(&SearchPoints{
        collection_name: "memory".to_string(),
        vector: ConfluenceMeta::get_embeddings(req_body.to_string()).await,
        filter: None,
        limit: 5,
        with_payload: None,
        params: None,
        score_threshold: None,
        offset: None,
        vector_name: None,
        with_vectors: None,
        read_consistency: None,
        timeout: None,
        shard_key_selector: None,
        sparse_indices: None,
    }).await;

    match db_response {
        Ok(response) => {
            // Here, you can format 'response' as a string.
            // This example uses a placeholder format; adjust it based on 'response' structure.
            let response_body = format!("Search Response: {:?}", response);
            Ok(HttpResponse::Ok().body(response_body))
        },
        Err(e) => {
            Ok(HttpResponse::InternalServerError().body(format!("Search Error: {}", e)))
        }
    }
}