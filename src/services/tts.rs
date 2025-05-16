use bytes::{Bytes, BytesMut};
use futures::Stream;
use piper_rs::synth::PiperSpeechSynthesizer;
use scraper::{Html, Selector};
use std::io::{self, Cursor};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::{error, info};

#[derive(Error, Debug)]
pub enum TtsError {
    #[error("Failed to process HTML: {0}")]
    HtmlProcessingError(String),

    #[error("Failed to run Piper TTS: {0}")]
    PiperError(String),

    #[error("IO error: {0}")]
    IoError(io::Error),

    #[error("Model error: {0}")]
    ModelError(String),
}

impl From<io::Error> for TtsError {
    fn from(err: io::Error) -> Self {
        TtsError::IoError(err)
    }
}

pub struct TtsConfig {
    pub voice_name: String,
    pub model_path: String,
    pub voice_path: String,
    pub sample_rate: u32,
    pub language: String,
}

impl Default for TtsConfig {
    fn default() -> Self {
        let voice_name = "en_US-ryan-high".to_string();
        Self {
            voice_name: voice_name.clone(),
            model_path: format!("./assets/voices/{}.onnx", voice_name),
            voice_path: format!("./assets/voices/{}.onnx.config", voice_name),
            sample_rate: 22050,
            language: "en-US".to_string(),
        }
    }
}

impl TtsConfig {
    pub fn new(voice_name: String, sample_rate: u32) -> Self {
        Self {
            voice_name: voice_name.clone(),
            model_path: format!("./assets/voices/{}.onnx", voice_name),
            voice_path: format!("./assets/voices/{}.onnx.config", voice_name),
            sample_rate,
            language: voice_name.split('-').next().unwrap_or("en_US").to_string(),
        }
    }

    /// Create a new TtsConfig based on the specified language
    pub fn from_language(language: &str) -> Self {
        let (voice_name, sample_rate) = match language {
            "ru" | "ru-RU" => ("ru_RU-ruslan-medium".to_string(), 22050),
            _ => ("en_US-ryan-high".to_string(), 22050), // Default to English
        };

        Self {
            voice_name: voice_name.clone(),
            model_path: format!("./assets/voices/{}.onnx", voice_name),
            voice_path: format!("./assets/voices/{}.onnx.config", voice_name),
            sample_rate,
            language: language.to_string(),
        }
    }
}

/// A stream of audio bytes from Piper TTS
pub struct AudioStream {
    audio_data: Cursor<Vec<u8>>,
    buffer: BytesMut,
    position: usize,
    pub total_len: usize,
}

impl Stream for AudioStream {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // If we've reached the end of the data, we're done
        if this.position >= this.total_len {
            return Poll::Ready(None);
        }

        // Ensure we have space
        if this.buffer.len() == this.buffer.capacity() {
            this.buffer.reserve(4096);
        }

        let mut buf = [0u8; 4096];

        // Try to read some data
        match io::Read::read(&mut this.audio_data, &mut buf) {
            Ok(0) => {
                // End of stream
                this.position = this.total_len;

                // Return any remaining data
                if !this.buffer.is_empty() {
                    let bytes = this.buffer.split().freeze();
                    return Poll::Ready(Some(Ok(bytes)));
                }

                Poll::Ready(None)
            }
            Ok(n) => {
                // Append data to our buffer
                this.buffer.extend_from_slice(&buf[..n]);
                this.position += n;

                // Return a chunk if we have enough data
                if this.buffer.len() >= 4096 {
                    let bytes = this.buffer.split().freeze();
                    Poll::Ready(Some(Ok(bytes)))
                } else if this.position >= this.total_len {
                    // Return whatever we have if we've read everything
                    let bytes = this.buffer.split().freeze();
                    Poll::Ready(Some(Ok(bytes)))
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}

pub struct TtsService {
    model_path: String,
    config_path: String,
    config: TtsConfig,
}

impl TtsService {
    pub fn new(config: TtsConfig) -> Result<Self, TtsError> {
        info!("Setting up Piper TTS with model: {}", config.model_path);

        Ok(Self {
            model_path: config.model_path.clone(),
            config_path: config.voice_path.clone(),
            config,
        })
    }

    /// Change the TTS model based on the specified language
    pub fn with_language(&self, language: &str) -> Result<Self, TtsError> {
        let config = TtsConfig::from_language(language);
        info!(
            "Switching TTS model to language: {} (using voice: {})",
            language, config.voice_name
        );

        Self::new(config)
    }

    /// Extract plain text from HTML content
    fn extract_text_from_html(&self, html_content: &str) -> Result<String, TtsError> {
        let document = Html::parse_document(html_content);

        // Select text nodes
        let body_selector = Selector::parse("body").map_err(|e| {
            TtsError::HtmlProcessingError(format!("Failed to create body selector: {}", e))
        })?;

        // Get body text, or use the entire document if no body found
        let text = if let Some(body) = document.select(&body_selector).next() {
            // Extract text content, ignoring script and style tags
            let script_selector = Selector::parse("script, style").map_err(|e| {
                TtsError::HtmlProcessingError(format!(
                    "Failed to create script/style selector: {}",
                    e
                ))
            })?;

            let mut text = body.text().collect::<Vec<_>>().join(" ");

            // Remove unwanted elements
            for element in document.select(&script_selector) {
                for text_node in element.text() {
                    text = text.replace(text_node, "");
                }
            }

            text
        } else {
            // Fallback to getting all text from the document
            document.root_element().text().collect::<Vec<_>>().join(" ")
        };

        // Clean up the text: normalize whitespace and trim
        let cleaned_text = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        Ok(cleaned_text)
    }

    /// Convert HTML content to an audio stream
    pub fn html_to_audio(&self, html_content: &str) -> Result<AudioStream, TtsError> {
        // Extract text from HTML
        let text = self.extract_text_from_html(html_content)?;

        info!(
            "Synthesizing speech for text: {:?}",
            text.chars().take(40).collect::<String>()
        );

        // Load the model from config
        let model = piper_rs::from_config_path(Path::new(&self.config_path))
            .map_err(|e| TtsError::ModelError(e.to_string()))?;

        // Create a synthesizer
        let synth = PiperSpeechSynthesizer::new(model)
            .map_err(|e| TtsError::PiperError(format!("Failed to create synthesizer: {}", e)))?;

        // Create a temporary file for the audio
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join(format!("epub_audio_{}.wav", uuid::Uuid::new_v4()));

        // Synthesize the text to a file
        synth
            .synthesize_to_file(&output_path, text, None)
            .map_err(|e| TtsError::PiperError(format!("Failed to synthesize text: {}", e)))?;

        // Read the file into memory
        let raw_audio = std::fs::read(&output_path).map_err(|e| {
            TtsError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read audio file: {}", e),
            ))
        })?;

        // Delete the temporary file
        let _ = std::fs::remove_file(&output_path);

        let total_len = raw_audio.len();

        info!(
            "Speech synthesis completed, audio length: {} bytes",
            total_len
        );

        // Create audio stream
        let audio_cursor = Cursor::new(raw_audio);

        Ok(AudioStream {
            audio_data: audio_cursor,
            buffer: BytesMut::with_capacity(8192),
            position: 0,
            total_len,
        })
    }

    /// Convert a file containing HTML to an audio stream
    pub async fn html_file_to_audio<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<AudioStream, TtsError> {
        // Read the HTML file
        let mut file = File::open(file_path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        // Process the HTML
        self.html_to_audio(&contents)
    }

    /// Create a WAV file header
    pub fn create_wav_header(&self, _audio_data_len: usize) -> Vec<u8> {
        // Since the synthesizer now creates a complete WAV file,
        // we don't need to create a header anymore. Return empty vector.
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_from_html() {
        let config = TtsConfig::default();
        let service = TtsService::new(config).unwrap();

        let html = r#"<html><body>
            <h1>Chapter 1</h1>
            <p>This is a test paragraph.</p>
            <script>var x = 10;</script>
            <p>Another paragraph.</p>
        </body></html>"#;

        let result = service.extract_text_from_html(html).unwrap();
        assert_eq!(
            result,
            "Chapter 1 This is a test paragraph. Another paragraph."
        );
    }
}
