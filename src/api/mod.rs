use actix_web::{post, web, HttpResponse, Responder, get};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use serde_json::{json, Value};
use crate::services::epub_parser;
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
            match epub_parser::parse_epub(&data) {
                Ok(epub_content) => {
                    // Convert HTML content to a JSON object
                    let html_json = json!({
                        "chapters": epub_content.chapters.iter().map(|chapter| {
                            json!({
                                "title": chapter.title,
                                "content": chapter.content
                            })
                        }).collect::<Vec<_>>()
                    });
                    
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
            
            let chapters_html: Value = serde_json::from_str(&doc.chapters_html)
                .unwrap_or_else(|_| json!({}));
            
            // Add document_id and chapters_html to response
            let mut response = metadata;
            if let Value::Object(ref mut obj) = response {
                obj.insert("document_id".to_string(), json!(doc.id));
                obj.insert("chapters_html".to_string(), chapters_html);
            }
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            HttpResponse::NotFound().body(format!("Document not found: {}", e))
        }
    }
}

#[get("/document/{id}/chapter/{index}")]
async fn get_chapter_by_index(path_params: web::Path<(i64, usize)>) -> impl Responder {
    let (id, index) = path_params.into_inner();
    
    println!("Trying to access chapter with index: {}", index); // Debug log
    
    match db::get_chapter_html_by_index(id, index) {
        Ok(html) => {
            HttpResponse::Ok()
                .content_type("text/html")
                .body(html)
        },
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            HttpResponse::NotFound().body(
                format!("Chapter not found with index {} in document {}", index, id)
            )
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(
                format!("Error retrieving chapter: {}", e)
            )
        }
    }
}

// Configure the API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_epub)
       .service(get_document)
       .service(get_chapter_by_index);
}