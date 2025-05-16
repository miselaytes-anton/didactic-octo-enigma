use crate::services::db;
use crate::services::epub_parser;
use crate::services::tts::TtsError;
use crate::services::tts::TtsService;
use actix_multipart::Multipart;
use actix_web::http::header::{ContentDisposition, DispositionType, ACCEPT_LANGUAGE};
use actix_web::web::Bytes;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct HtmlRequest {
    pub html_content: String,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<TtsError> for ApiError {
    fn from(err: TtsError) -> Self {
        ApiError {
            message: err.to_string(),
        }
    }
}

pub struct ApiState {
    pub tts_service: Arc<TtsService>,
}

/// Parse the Accept-Language header and return the preferred language
fn get_language_from_header(req: &HttpRequest) -> String {
    if let Some(lang_header) = req.headers().get(ACCEPT_LANGUAGE) {
        if let Ok(lang_str) = lang_header.to_str() {
            // Parse the Accept-Language header
            // Format is typically: en-US,en;q=0.9,ru;q=0.8
            let langs: Vec<&str> = lang_str.split(',').collect();

            if !langs.is_empty() {
                // Get the first (most preferred) language
                let primary_lang = langs[0].split(';').next().unwrap_or("en-US").trim();
                return primary_lang.to_string();
            }
        }
    }

    // Default to English if no language is specified
    "en-US".to_string()
}

#[post("/upload")]
async fn upload_epub(mut payload: Multipart) -> impl Responder {
    while let Some(field) = payload.next().await {
        let field = match field {
            Ok(field) => field,
            Err(e) => {
                return HttpResponse::BadRequest().body(format!("Error processing form: {}", e))
            }
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
                    Err(e) => {
                        return HttpResponse::BadRequest()
                            .body(format!("Error reading file: {}", e))
                    }
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
                        }
                        Err(e) => {
                            return HttpResponse::InternalServerError()
                                .body(format!("Error saving to database: {}", e));
                        }
                    }
                }
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .body(format!("Error parsing EPUB: {}", e))
                }
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
            let metadata: Value = serde_json::from_str(&doc.metadata).unwrap_or_else(|_| json!({}));

            let chapters_html: Value =
                serde_json::from_str(&doc.chapters_html).unwrap_or_else(|_| json!({}));

            // Add document_id and chapters_html to response
            let mut response = metadata;
            if let Value::Object(ref mut obj) = response {
                obj.insert("document_id".to_string(), json!(doc.id));
                obj.insert("chapters_html".to_string(), chapters_html);
            }

            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::NotFound().body(format!("Document not found: {}", e)),
    }
}

#[get("/document/{id}/chapter/{index}")]
async fn get_chapter_by_index(path_params: web::Path<(i64, usize)>) -> impl Responder {
    let (id, index) = path_params.into_inner();

    println!("Trying to access chapter with index: {}", index); // Debug log

    match db::get_chapter_html_by_index(id, index) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(rusqlite::Error::QueryReturnedNoRows) => HttpResponse::NotFound().body(format!(
            "Chapter not found with index {} in document {}",
            index, id
        )),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Error retrieving chapter: {}", e))
        }
    }
}

#[get("/document/{id}/chapter/{index}/audio")]
async fn get_audio(
    path_params: web::Path<(i64, usize)>,
    req: HttpRequest,
    data: web::Data<ApiState>,
) -> impl Responder {
    let (id, index) = path_params.into_inner();

    println!("Received request to audio");

    // Get language from header
    let language = get_language_from_header(&req);
    println!("Using language: {}", language);

    // Get TTS service with the appropriate language
    let tts_service = match data.tts_service.with_language(&language) {
        Ok(service) => service,
        Err(e) => {
            error!(
                "Failed to create TTS service for language {}: {}",
                language, e
            );
            return Err(actix_web::error::ErrorInternalServerError(ApiError::from(
                e,
            )));
        }
    };

    match db::get_chapter_html_by_index(id, index) {
        Ok(html) => {
            let audio_stream = tts_service.html_to_audio(&html).map_err(|e| {
                error!("Failed to convert HTML to audio: {}", e);
                actix_web::error::ErrorInternalServerError(ApiError::from(e))
            })?;

            // Create WAV header
            let wav_header = tts_service.create_wav_header(audio_stream.total_len);

            // Map the stream to actix-compatible chunks
            let stream = futures::stream::unfold(
                (audio_stream, Some(wav_header), true),
                |(mut stream, wav_header, is_first)| async move {
                    if is_first && wav_header.is_some() {
                        println!("sending wav header");
                        // First chunk is the WAV header
                        return Some((Ok(Bytes::from(wav_header.unwrap())), (stream, None, false)));
                    }

                    match stream.next().await {
                        Some(Ok(bytes)) => {
                            println!("Streaming audio chunk: {} bytes", bytes.len());
                            Some((Ok(bytes), (stream, None, false)))
                        }
                        Some(Err(e)) => {
                            error!("Error streaming audio: {}", e);
                            Some((
                                Err(actix_web::error::ErrorInternalServerError(e)),
                                (stream, None, false),
                            ))
                        }
                        None => None,
                    }
                },
            );

            // Return streaming response
            Ok(HttpResponse::Ok()
                .content_type("audio/wav")
                .append_header(ContentDisposition {
                    disposition: DispositionType::Attachment,
                    parameters: vec![],
                })
                .streaming(stream))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(HttpResponse::NotFound().body(format!(
            "Chapter not found with index {} in document {}",
            index, id
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().body(format!("Error retrieving chapter: {}", e))
        ),
    }
}

// Configure the API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_epub)
        .service(get_document)
        .service(get_audio)
        .service(get_chapter_by_index);
}
