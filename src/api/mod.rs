use actix_web::{post, web, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use crate::services::epub_parser::parse_epub;

#[post("/upload")]
async fn upload_epub(mut payload: Multipart) -> impl Responder {
    while let Some(field) = payload.next().await {
        let field = match field {
            Ok(field) => field,
            Err(e) => return HttpResponse::BadRequest().body(format!("Error processing form: {}", e)),
        };
        
        // Only process files
        let content_disposition = field.content_disposition();
        if let Some(filename) = content_disposition.get_filename() {
            if !filename.ends_with(".epub") {
                return HttpResponse::BadRequest().body("Only EPUB files are supported");
            }
            
            // Read file contents
            let mut data = Vec::new();
            let mut field_stream = field;
            
            while let Some(chunk) = field_stream.next().await {
                match chunk {
                    Ok(bytes) => data.extend_from_slice(&bytes),
                    Err(e) => return HttpResponse::BadRequest().body(format!("Error reading file: {}", e)),
                }
            }
            
            // Parse the EPUB file
            match parse_epub(&data) {
                Ok(metadata) => return HttpResponse::Ok().json(metadata),
                Err(e) => return HttpResponse::InternalServerError().body(format!("Error parsing EPUB: {}", e)),
            }
        }
    }
    
    HttpResponse::BadRequest().body("No EPUB file found in the upload")
}

// Configure the API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_epub);
}