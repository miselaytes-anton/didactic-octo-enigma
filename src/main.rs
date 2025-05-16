use actix_web::{App, HttpServer};

mod api;
mod models;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://127.0.0.1:8081");
    HttpServer::new(|| {
        App::new()
            .configure(api::configure_routes)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}