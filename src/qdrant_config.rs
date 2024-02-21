use qdrant_client::client::QdrantClient;
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{CreateCollection, Distance, PointsOperationResponse, PointStruct, SearchPoints, VectorParams, VectorsConfig};
use std::error::Error;
use qdrant_client::qdrant::point_id::PointIdOptions;
use crate::open_ai_config::gen_embeddings;

pub async fn create_collection() -> Result<String, Box<dyn Error>> {
    let client = QdrantClient::from_url("http://localhost:6334").build()?;
    match client
        .create_collection(&CreateCollection {
            collection_name: "memory".to_string(),
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
        })
        .await
    {
        Ok(_) => Ok("Collection Created Successfully".to_string()),
        Err(e) => Err(Box::from(e)),
    }
}

pub async fn upload_points_store(points: Vec<PointStruct>) -> Result<PointsOperationResponse, Box<dyn Error>> {
    let client = QdrantClient::from_url("http://localhost:6334").build()?;
    Ok(client.upsert_points_batch_blocking(
        "memory".to_string(),
        None,
        points,
        None,
        100
    ).await?)
}

pub async fn search_qdrant_collection(prompt: String) -> Result<Vec<u64>, Box<dyn Error>> {
    let client = QdrantClient::from_url("http://localhost:6334").build()?;
    let embedded_prompt = gen_embeddings(&prompt).await?;

    let mut result_ids = Vec::new();

    let response = client.search_points(&SearchPoints {
        collection_name: "memory".to_string(),
        vector: embedded_prompt,
        filter: None,
        limit: 3,
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
