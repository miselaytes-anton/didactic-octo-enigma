use actix_web::{post, web, HttpResponse, Responder, get};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use serde_json::{json, Value};
use crate::services::epub_parser::parse_epub;
use crate::services::db;

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
                Ok(epub_content) => {
                    // Convert HTML content to a JSON object
                    let mut html_json = json!({});
                    for (path, html) in &epub_content.html_content {
                        html_json[path] = json!(html);
                    }
                    
                    // Save to database
                    match db::save_document(&epub_content.metadata, &html_json) {
                        Ok(document_id) => {
                            // Create response with metadata and document ID
                            let mut metadata_value = serde_json::to_value(&epub_content.metadata)
                                .unwrap_or_else(|_| json!({}));
                            
                            if let Value::Object(ref mut obj) = metadata_value {
                                obj.insert("document_id".to_string(), json!(document_id));
                            }
                            
                            return HttpResponse::Ok().json(metadata_value);
                        },
                        Err(e) => {
                            return HttpResponse::InternalServerError()
                                .body(format!("Error saving to database: {}", e));
                        }
                    }
                },
                Err(e) => return HttpResponse::InternalServerError().body(format!("Error parsing EPUB: {}", e)),
            }
        }
    }
    
    HttpResponse::BadRequest().body("No EPUB file found in the upload")
}

#[get("/document/{id}")]
async fn get_document(path: web::Path<i64>) -> impl Responder {
    let id = path.into_inner();
    
    match db::get_document(id) {
        Ok(doc) => {
            // Parse the JSON strings into Value objects
            let metadata: Value = serde_json::from_str(&doc.metadata)
                .unwrap_or_else(|_| json!({}));
            
            let _chapters_html: Value = serde_json::from_str(&doc.chapters_html)
                .unwrap_or_else(|_| json!({}));
            
            // Add document_id to metadata
            let mut response = metadata;
            if let Value::Object(ref mut obj) = response {
                obj.insert("document_id".to_string(), json!(doc.id));
            }
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            HttpResponse::NotFound().body(format!("Document not found: {}", e))
        }
    }
}

// Configure the API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_epub)
       .service(get_document);
}