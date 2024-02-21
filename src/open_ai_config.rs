use openai_rust;
use dotenv;
use std::{env, result};
use std::error::Error;
use openai_rust::embeddings::EmbeddingsData;

pub fn get_openai_key() -> Result<String,()> {
    dotenv::dotenv().ok();
    env::var("OPEN_AI_KEY").map_err(|err|{
        eprintln!("Could not find confluence url")
    })
}

pub async fn gen_embeddings(text: &str) -> Result<Vec<f32>, Box<dyn Error>>{
    let Ok(open_ai_key) = get_openai_key() else {
        return Err(Box::from("failed to get key"))
    };
    let open_ai_client = openai_rust::Client::new(&open_ai_key);
    let args = openai_rust::embeddings::EmbeddingsArguments::new("text-embedding-ada-002", text.to_string());

    let embeddings_result = open_ai_client.create_embeddings(args).await?;

    let result = embeddings_result.data.get(0).ok_or("failed to parse open ai response")?;

    Ok(result.embedding.clone())
}
