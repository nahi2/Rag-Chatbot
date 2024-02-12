mod confluence;

use std::sync::Mutex;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, ResponseError};
use serde_json::{Error, Value};
use crate::confluence::ConfluenceMeta;

struct  AppState {
    store: Mutex<Vec<ConfluenceMeta>>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let shared_data = web::Data::new(AppState {
        store: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(shared_data.clone()).service(set_doc_store)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

#[get("/set_doc_store")]
async fn set_doc_store(data: web::Data<AppState>) -> HttpResponse {
    let confluence_config = match confluence::ConfluenceConfig::new() {
        Ok(config) => config,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let raw_pages = match confluence_config.get_conf_pages().await {
        Ok(pages) => pages,
        Err(_) => return HttpResponse::BadRequest().finish()
    };
    let json_pages = match ConfluenceMeta::convert_to_json(&raw_pages) {
        Ok(value) => value,
        Err(_) => return HttpResponse::BadRequest().finish()
    };
    let store = match ConfluenceMeta::build_store(json_pages) {
        None => {
            return HttpResponse::BadRequest().finish()
        }
        Some(store) => store
    };
    println!("{:?}", store);
    HttpResponse::Ok().into()
}

