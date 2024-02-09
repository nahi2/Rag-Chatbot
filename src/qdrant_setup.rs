use std::env;
use std::error::Error;
use actix_web::HttpResponse;
use qdrant_client::qdrant::{SearchResponse, UpsertPoints, VectorParams, VectorsConfig};
use qdrant_client::qdrant::vectors_config::Config;
use dotenv::{dotenv};
use qdrant_client::client::QdrantClient;
use qdrant_client::qdrant::point_id::PointIdOptions;
use qdrant_client::prelude::{CreateCollection, Distance, SearchPoints};
use serde_json::Value;
use crate::confluence::ConfluenceMeta;

pub struct QdrantConfig {
    qdrant_url: String,
    collection_name: String
}

impl Default for QdrantConfig{
    fn default() -> QdrantConfig {
        dotenv().ok();
        Self{
            qdrant_url: env::var("QDRANT_URL").unwrap(),
            collection_name: env::var("QDRANT_COLLECTION_NAME").unwrap(),
        }
        }

}

impl QdrantConfig {
    pub fn get_url(&self) -> String {
        self.qdrant_url.to_string()
    }

    pub async fn setup_collection(&self) -> Result<HttpResponse, Box<dyn Error>> {
        let client = QdrantClient::from_url(&self.qdrant_url).build()?;

        match client.create_collection(&CreateCollection {
            collection_name: self.collection_name.to_string().into(),
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

    pub async fn search_collection(&self, prompt: String) -> Result<Vec<u64>, Box<dyn Error>> {
        let client = QdrantClient::from_url(&self.qdrant_url).build()?;
        let embedded_prompt = ConfluenceMeta::get_embeddings(prompt).await;

        let mut result_ids = Vec::new();

        let response = client.search_points(&SearchPoints {
            collection_name: self.collection_name.to_string(),
            vector: embedded_prompt,
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
        }).await?;

        for scored_point in &response.result {
            let point_id_options = scored_point.id.clone().unwrap().point_id_options.unwrap();

            if let PointIdOptions::Num(num) = point_id_options {
                result_ids.push(num)
            }
        };

        Ok(result_ids)
    }
}

