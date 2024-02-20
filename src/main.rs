mod confluence;
mod qdrant_config;

use crate::confluence::ConfluenceMeta;
use crate::qdrant_config::create_collection;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use std::fmt;
use std::sync::Mutex;

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
            .service(test_collection)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[get("/set_doc_store")]
async fn set_doc_store(data: web::Data<AppState>) -> HttpResponse {
    println!("{:?}", data.store.lock().iter());
    HttpResponse::Ok().into()
}

#[get("/test_collection")]
async fn test_collection() -> HttpResponse {
    let Ok(_) = create_collection().await else {
        return HttpResponse::BadRequest().body("Failed to create collection, check qdrant server is running or if the collection already exists");
    };
    HttpResponse::Ok().into()
}
