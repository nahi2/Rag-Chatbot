use openai_rust;
use dotenv;
use std::{env, result};

pub fn get_openai_key() -> String {
    dotenv::dotenv().ok();
    env::var("")
}

pub async fn gen_embeddings(text: String) -> Result<(),Box <dyn Error>>{
    openai_rust::
}
