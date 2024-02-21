mod confluence;
mod qdrant_config;
mod open_ai_config;

use crate::confluence::ConfluenceMeta;
use crate::qdrant_config::{create_collection, search_qdrant_collection, upload_points_store};
use actix_web::{get, web, App, HttpResponse, HttpServer};
use std::fmt;
use std::sync::Mutex;
use actix_web::cookie::time::format_description::parse;
use qdrant_client::qdrant::PointStruct;
use serde_json::json;
use crate::open_ai_config::gen_embeddings;

#[derive(Debug)]
struct AppState {
    store: Mutex<Vec<ConfluenceMeta>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let shared_data = if let Ok(store) = ConfluenceMeta::encapsulate_store().await {
        web::Data::new(AppState {
            store: Mutex::new(store),
        })
    } else {
        eprintln!("failed to get data from confluence");
        web::Data::new(AppState {
            store: Mutex::new(Vec::new()),
        })
    };

    HttpServer::new(move || {
        App::new()
            .app_data(shared_data.clone())
            .service(set_doc_store)
            .service(search_collection)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}



#[get("/set_doc_store")]
async fn set_doc_store(data: web::Data<AppState>) -> HttpResponse {
    let Ok(_) = create_collection().await else {
        return HttpResponse::BadRequest().body("Failed to create collection, check qdrant server is running or if the collection already exists");
    };

    let store = match data.store.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Mutex is poisoned: {:?}", poisoned);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut point_structs: Vec<PointStruct> = Vec::new();

    for page in store.iter() {
        let Ok(embedding) = gen_embeddings(&page.page_body).await else{
            return HttpResponse::BadRequest().body("failed to generate embeddings")
        };

        point_structs.push(PointStruct::new(
            page.page_id,
            embedding,
            json!({"page_title": format!("{:?}", page.page_title)}).try_into().expect("json")
        ))
    };

    let Ok(_) = upload_points_store(point_structs).await else {
        return HttpResponse::BadRequest().body("failed to upload to vector database")
    };

    HttpResponse::Ok().into()
}

#[get("/search_collection")]
async fn search_collection(req_body: String) -> HttpResponse {
    println!("{:?}", search_qdrant_collection(req_body).await);
    HttpResponse::Ok().into()
}