use actix_web::{App, HttpServer};

mod api;
mod models;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the database
    match services::db::init_db() {
        Ok(_) => println!("Database initialized successfully"),
        Err(e) => eprintln!("Failed to initialize database: {}", e),
    }
    
    println!("Starting server at http://127.0.0.1:8081");
    HttpServer::new(|| {
        App::new()
            .configure(api::configure_routes)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}