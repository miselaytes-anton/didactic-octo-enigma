use crate::api::ApiState;
use crate::services::tts::{TtsConfig, TtsService};
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use tracing::{error, info};

mod api;
mod models;
mod services;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize the database
    match services::db::init_db() {
        Ok(_) => println!("Database initialized successfully"),
        Err(e) => eprintln!("Failed to initialize database: {}", e),
    }

    // Configure TTS service with default language (English)
    // The actual language used will be determined from the Accept-Language header in the request
    let config = TtsConfig::default();
    info!("Default language: {}", config.language);

    // Create TTS service
    let tts_service = match TtsService::new(config) {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to initialize TTS service: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to initialize TTS service: {}", e),
            ));
        }
    };

    println!("Starting server at http://127.0.0.1:8081");
    start_server(tts_service).await
}

/// Start the API server
async fn start_server(tts_service: TtsService) -> std::io::Result<()> {
    let bind_addr = "127.0.0.1:8081";
    info!("Starting server on {}", bind_addr);

    let state = web::Data::new(ApiState {
        tts_service: Arc::new(tts_service),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .configure(api::configure_routes)
    })
    .bind(bind_addr)?
    .run()
    .await
}
